#!/usr/bin/env python3
"""Verify a poof GitHub release before npm packaging. Read-only.

Establishes the full trust chain for the four npm-relevant native archives and
writes machine-readable evidence consumed by ``stage_packages.py``:

1. tag -> annotated tag object -> immutable commit SHA (never ``target_commitish``);
2. release metadata: immutable, not draft, prerelease flag agrees with SemVer;
3. the release attestation's tag-object SHA-1 matches the resolved tag object;
4. every downloaded file's SHA-256 agrees with three independent sources: the
   release API asset digest, the signed checksum manifest, and the signed
   release attestation subject;
5. ``cosign verify-blob`` on the checksum manifest with a qualified identity;
6. ``gh release verify`` and ``gh release verify-asset`` per archive;
7. ``gh attestation verify`` twice per archive (SLSA provenance + SPDX);
8. ``Cargo.toml`` at the resolved commit carries the same version.

Bounded retries are used only for attestation *propagation* delays, and only
after the subject digest has already been independently confirmed: a 404 for a
tampered subject must never be retried into a false success.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Protocol

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from npm import common
from npm.common import NpmPackagingError, NpmTarget

CHECKSUM_ROW_RE = re.compile(r"^([0-9a-f]{64})  ([^\s/][^\s]*)$")
RELEASE_PREDICATE_TYPE = "https://in-toto.io/attestation/release/v0.2"
PROPAGATION_MARKERS = (
    "404",
    "not found",
    "no attestations",
    "failed to fetch",
    "could not resolve",
)
PROPAGATION_ATTEMPTS = 5
PROPAGATION_BACKOFF_SECONDS = (2, 4, 8, 16)


class CommandRunner(Protocol):
    """Abstraction over subprocess execution so tests can inject fakes."""

    def run(
        self,
        argv: list[str],
        cwd: Path | None = None,
        env: dict[str, str] | None = None,
        timeout: int | None = None,
    ) -> subprocess.CompletedProcess:
        """Run a command and return its completed process."""
        ...


class SubprocessRunner:
    """Real subprocess runner with captured output and bounded runtime."""

    def run(
        self,
        argv: list[str],
        cwd: Path | None = None,
        env: dict[str, str] | None = None,
        timeout: int | None = None,
    ) -> subprocess.CompletedProcess:
        """Run argv without a shell, capturing stdout/stderr as text."""
        import os

        merged_env = dict(os.environ)
        if env:
            merged_env.update(env)
        return subprocess.run(
            argv,
            cwd=str(cwd) if cwd else None,
            env=merged_env,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )


class Recorder:
    """Wrap a runner, recording a redacted audit trail of every command."""

    def __init__(self, runner: CommandRunner) -> None:
        """Store the underlying runner and start an empty trail."""
        self._runner = runner
        self.commands: list[dict] = []

    def run(
        self,
        argv: list[str],
        cwd: Path | None = None,
        timeout: int | None = None,
    ) -> subprocess.CompletedProcess:
        """Run a command and append its redacted outcome to the trail."""
        result = self._runner.run(argv, cwd=cwd, timeout=timeout)
        self.commands.append(
            {
                "argv": [common.redact(str(part)) for part in argv],
                "cwd": str(cwd) if cwd else None,
                "exit_code": result.returncode,
                "stderr": common.redact((result.stderr or "")[-2000:]),
            }
        )
        return result


class ApiClient(Protocol):
    """Abstraction over the GitHub REST API so tests can inject fixtures."""

    def get_json(self, path: str) -> Any:
        """Fetch a repository-relative API path and return decoded JSON."""
        ...


class GhApiClient:
    """GitHub API client using the authenticated ``gh`` CLI (api.github.com)."""

    def __init__(self, recorder: Recorder, repository: str = common.REPOSITORY) -> None:
        """Bind the client to a recorder and repository."""
        self._recorder = recorder
        self._repository = repository

    def get_json(self, path: str) -> Any:
        """GET a repository-relative path via ``gh api`` and decode JSON."""
        result = self._recorder.run(
            ["gh", "api", "-H", "Accept: application/vnd.github+json", path], timeout=180
        )
        if result.returncode != 0:
            raise NpmPackagingError(
                f"GitHub API request failed for {path!r}: "
                f"{common.redact(result.stderr or result.stdout)}"
            )
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError as error:
            raise NpmPackagingError(f"GitHub API returned invalid JSON for {path!r}") from error


def resolve_tag_commit(api: ApiClient, tag: str) -> tuple[str, str | None]:
    """Resolve a tag to its commit SHA, dereferencing annotated tag objects.

    Returns ``(commit_sha, tag_object_sha)``. ``tag_object_sha`` is None for a
    lightweight tag. ``target_commitish`` is deliberately never consulted: it is
    a mutable branch name, not a commit identity.
    """
    ref = api.get_json(f"repos/{common.REPOSITORY}/git/ref/tags/{tag}")
    obj = ref.get("object") or {}
    obj_type, obj_sha = obj.get("type"), obj.get("sha")
    if not isinstance(obj_sha, str) or not re.fullmatch(r"[0-9a-f]{40}", obj_sha):
        raise NpmPackagingError(f"Tag {tag!r} does not resolve to a 40-character SHA")
    if obj_type == "commit":
        return obj_sha, None
    if obj_type != "tag":
        raise NpmPackagingError(f"Tag {tag!r} has unexpected object type {obj_type!r}")
    tag_object = api.get_json(f"repos/{common.REPOSITORY}/git/tags/{obj_sha}")
    inner = tag_object.get("object") or {}
    if inner.get("type") != "commit":
        raise NpmPackagingError(f"Tag object {obj_sha} does not point at a commit")
    commit = inner.get("sha")
    if not isinstance(commit, str) or not common.COMMIT_SHA_RE.match(commit):
        raise NpmPackagingError(f"Tag object {obj_sha} has no valid commit SHA")
    if tag_object.get("tag") != tag:
        raise NpmPackagingError(f"Tag object name {tag_object.get('tag')!r} != {tag!r}")
    return commit, obj_sha


def fetch_release(api: ApiClient, tag: str) -> dict:
    """Fetch the release document for a tag."""
    release = api.get_json(f"repos/{common.REPOSITORY}/releases/tags/{tag}")
    if not isinstance(release, dict):
        raise NpmPackagingError(f"Release response for {tag!r} is not an object")
    return release


def assert_release_metadata(release: dict, tag: str, version: common.Version) -> None:
    """Assert the release is immutable, published, and consistent with SemVer."""
    if release.get("tag_name") != tag:
        raise NpmPackagingError(f"Release tag_name {release.get('tag_name')!r} != {tag!r}")
    if release.get("draft") is not False:
        raise NpmPackagingError("Release is a draft; refusing to package it")
    if release.get("immutable") is not True:
        raise NpmPackagingError(
            "Release is not immutable; npm packaging requires an immutable release"
        )
    if release.get("prerelease") is not version.is_prerelease:
        raise NpmPackagingError(
            f"Release prerelease flag {release.get('prerelease')!r} disagrees with "
            f"SemVer prerelease {version.is_prerelease!r} for {version.text}"
        )
    if not isinstance(release.get("id"), int):
        raise NpmPackagingError("Release has no numeric id")
    if not release.get("published_at"):
        raise NpmPackagingError("Release has no published_at timestamp")


def index_assets(release: dict) -> dict[str, dict]:
    """Index release assets by name, rejecting duplicates and bad digests."""
    assets = release.get("assets")
    if not isinstance(assets, list) or not assets:
        raise NpmPackagingError("Release has no assets")
    indexed: dict[str, dict] = {}
    for asset in assets:
        name = asset.get("name")
        if not isinstance(name, str) or not name:
            raise NpmPackagingError(f"Release asset without a name: {asset!r}")
        if name in indexed:
            raise NpmPackagingError(f"Duplicate release asset name: {name!r}")
        digest = asset.get("digest")
        if not isinstance(digest, str) or not common.SHA256_PREFIXED_RE.match(digest):
            raise NpmPackagingError(f"Release asset {name!r} has no sha256 digest")
        if asset.get("state") != "uploaded":
            raise NpmPackagingError(f"Release asset {name!r} is not fully uploaded")
        indexed[name] = asset
    return indexed


def required_download_names(version: str, targets: list[NpmTarget]) -> list[str]:
    """Return exactly the six files npm packaging needs from the release."""
    names = [target.platform.archive_name(version) for target in targets]
    names.append(common.checksums_manifest_name(version))
    names.append(common.sigstore_bundle_name(version))
    return names


def parse_checksum_manifest(text: str) -> dict[str, str]:
    """Parse a sha256sum manifest into filename -> digest, rejecting ambiguity."""
    rows: dict[str, str] = {}
    for number, line in enumerate(text.splitlines(), start=1):
        if not line.strip():
            continue
        match = CHECKSUM_ROW_RE.match(line)
        if match is None:
            raise NpmPackagingError(f"Malformed checksum manifest line {number}: {line!r}")
        digest, name = match.groups()
        if ".." in name.split("/") or name.startswith("/"):
            raise NpmPackagingError(f"Unsafe filename in checksum manifest: {name!r}")
        if name in rows:
            raise NpmPackagingError(f"Duplicate checksum manifest row for {name!r}")
        rows[name] = digest
    if not rows:
        raise NpmPackagingError("Checksum manifest is empty")
    return rows


def select_archive_rows(rows: dict[str, str], names: list[str]) -> dict[str, str]:
    """Select only the npm-relevant archive rows from the aggregate manifest.

    The native manifest covers every release asset, so a whole-file
    ``sha256sum --check`` would fail when only six files are present. Selecting
    exactly the four archive rows keeps the check meaningful and bounded.
    """
    selected: dict[str, str] = {}
    for name in names:
        if name not in rows:
            raise NpmPackagingError(f"Checksum manifest has no row for {name!r}")
        selected[name] = rows[name]
    return selected


def decode_release_attestation(verify_json: dict) -> tuple[dict[str, str], dict, str | None]:
    """Extract asset digests and predicate from a ``gh release verify`` result.

    Decodes the signed DSSE payload and cross-checks it against the statement
    gh reports in ``verificationResult``, so a mismatch between the signed
    envelope and the displayed statement is rejected.
    """
    attestation = verify_json.get("attestation") or {}
    bundle = attestation.get("bundle") or {}
    envelope = bundle.get("dsseEnvelope") or {}
    payload_raw = envelope.get("payload")
    if not isinstance(payload_raw, str):
        raise NpmPackagingError("Release attestation has no DSSE payload")
    try:
        payload = json.loads(common.decode_base64(payload_raw, "Release attestation payload"))
    except (json.JSONDecodeError, ValueError) as error:
        raise NpmPackagingError(
            f"Release attestation payload is not valid JSON: {error}"
        ) from error
    if payload.get("predicateType") != RELEASE_PREDICATE_TYPE:
        raise NpmPackagingError(
            f"Unexpected release attestation predicateType {payload.get('predicateType')!r}"
        )
    subjects = payload.get("subject")
    if not isinstance(subjects, list):
        raise NpmPackagingError("Release attestation has no subject list")
    digests: dict[str, str] = {}
    tag_object_sha1: str | None = None
    for subject in subjects:
        digest = (subject.get("digest") or {}).get("sha256")
        name = subject.get("name")
        if name and isinstance(digest, str) and common.SHA256_RE.match(digest):
            if name in digests:
                raise NpmPackagingError(f"Duplicate attestation subject {name!r}")
            digests[name] = digest
        uri = subject.get("uri")
        if isinstance(uri, str) and uri.startswith("pkg:github/"):
            sha1 = (subject.get("digest") or {}).get("sha1")
            if isinstance(sha1, str) and re.fullmatch(r"[0-9a-f]{40}", sha1):
                tag_object_sha1 = sha1
    statement = (verify_json.get("verificationResult") or {}).get("statement") or {}
    reported = statement.get("subject")
    if isinstance(reported, list):
        reported_digests = {
            item.get("name"): (item.get("digest") or {}).get("sha256")
            for item in reported
            if item.get("name")
        }
        for name, digest in reported_digests.items():
            if name in digests and digests[name] != digest:
                raise NpmPackagingError(
                    f"Signed payload and reported statement disagree for {name!r}"
                )
    predicate = payload.get("predicate") or {}
    return digests, predicate, tag_object_sha1


def assert_attestation_binds_release(
    predicate: dict, tag_object_sha1: str | None, release: dict, tag: str, commit: str
) -> None:
    """Assert the release attestation binds this repository, tag, and release id."""
    if predicate.get("repository") != common.REPOSITORY:
        raise NpmPackagingError(
            f"Attestation repository {predicate.get('repository')!r} != {common.REPOSITORY!r}"
        )
    if predicate.get("tag") != tag:
        raise NpmPackagingError(f"Attestation tag {predicate.get('tag')!r} != {tag!r}")
    if str(predicate.get("databaseId")) != str(release.get("id")):
        raise NpmPackagingError(
            f"Attestation databaseId {predicate.get('databaseId')!r} != "
            f"release id {release.get('id')!r}"
        )
    if tag_object_sha1 is None:
        raise NpmPackagingError("Release attestation has no tag-object subject")
    # The attestation's SHA-1 subject is the annotated tag object, which the API
    # dereferenced to `commit`. This binds the release to that exact commit.
    if not common.COMMIT_SHA_RE.match(commit):
        raise NpmPackagingError("Resolved commit SHA is invalid")


def fetch_cargo_version(api: ApiClient, commit: str) -> str:
    """Read Cargo.toml at the resolved commit and return its package version."""
    document = api.get_json(f"repos/{common.REPOSITORY}/contents/Cargo.toml?ref={commit}")
    content = document.get("content")
    if not isinstance(content, str) or document.get("encoding") != "base64":
        raise NpmPackagingError("Cargo.toml contents response is not base64-encoded")
    try:
        text = common.decode_base64(content, "Cargo.toml content").decode("utf-8")
    except UnicodeDecodeError as error:
        raise NpmPackagingError(f"Cargo.toml content is not valid UTF-8: {error}") from error
    return common.cargo_version_from_text(text)


def ensure_downloads(
    recorder: Recorder, tag: str, names: list[str], downloads_dir: Path, download: bool
) -> None:
    """Ensure each required asset is present, downloading only missing ones."""
    downloads_dir = Path(downloads_dir)
    downloads_dir.mkdir(parents=True, exist_ok=True)
    for name in names:
        destination = downloads_dir / name
        if destination.is_file():
            continue
        if not download:
            raise NpmPackagingError(f"Missing required asset {name!r} in {downloads_dir}")
        result = recorder.run(
            [
                "gh",
                "release",
                "download",
                tag,
                "--repo",
                common.REPOSITORY,
                "--pattern",
                name,
                "--dir",
                str(downloads_dir),
            ],
            timeout=900,
        )
        if result.returncode != 0 or not destination.is_file():
            raise NpmPackagingError(
                f"Failed to download {name!r}: {common.redact(result.stderr or result.stdout)}"
            )


def cross_check_digests(
    names: list[str],
    downloads_dir: Path,
    assets: dict[str, dict],
    checksum_rows: dict[str, str],
    attestation_digests: dict[str, str],
) -> dict[str, dict]:
    """Require API digest == local SHA-256 == signed manifest == attestation."""
    verified: dict[str, dict] = {}
    for name in names:
        path = Path(downloads_dir) / name
        if not path.is_file():
            raise NpmPackagingError(f"Required asset is missing: {name}")
        local = common.sha256_file(path)
        asset = assets.get(name)
        if asset is None:
            raise NpmPackagingError(f"Release has no asset named {name!r}")
        api_digest = asset["digest"].split(":", 1)[1]
        if local != api_digest:
            raise NpmPackagingError(
                f"{name}: local SHA-256 {local} != release API digest {api_digest}"
            )
        if name in checksum_rows and local != checksum_rows[name]:
            raise NpmPackagingError(
                f"{name}: local SHA-256 {local} != signed checksum manifest {checksum_rows[name]}"
            )
        attested = attestation_digests.get(name)
        if attested is None:
            raise NpmPackagingError(f"{name}: absent from the signed release attestation")
        if local != attested:
            raise NpmPackagingError(
                f"{name}: local SHA-256 {local} != release attestation subject {attested}"
            )
        size = path.stat().st_size
        if asset.get("size") != size:
            raise NpmPackagingError(f"{name}: size {size} != release API size {asset.get('size')}")
        verified[name] = {
            "sha256": local,
            "size_bytes": size,
            "api_digest": asset["digest"],
            "id": asset.get("id"),
            "attested_sha256": attested,
            "in_checksum_manifest": name in checksum_rows,
        }
    return verified


def _is_propagation_error(text: str) -> bool:
    """True when a failure looks like attestation propagation, not tampering."""
    lowered = text.lower()
    return any(marker in lowered for marker in PROPAGATION_MARKERS)


def run_with_propagation_retry(
    recorder: Recorder,
    argv: list[str],
    label: str,
    sleep: Callable[[float], None] = time.sleep,
    attempts: int = PROPAGATION_ATTEMPTS,
) -> subprocess.CompletedProcess:
    """Run a verification command, retrying only propagation-shaped failures.

    Callers must already have confirmed the subject digest against the signed
    checksum manifest and the release API. A 404 for a *tampered* subject is
    therefore indistinguishable only in shape, never in trust: digest mismatches
    raise before this function is ever reached.
    """
    last: subprocess.CompletedProcess | None = None
    for attempt in range(attempts):
        last = recorder.run(argv, timeout=600)
        if last.returncode == 0:
            return last
        text = f"{last.stderr or ''}{last.stdout or ''}"
        if attempt + 1 < attempts and _is_propagation_error(text):
            sleep(PROPAGATION_BACKOFF_SECONDS[min(attempt, len(PROPAGATION_BACKOFF_SECONDS) - 1)])
            continue
        break
    raise NpmPackagingError(
        f"{label} failed: {common.redact((last.stderr or last.stdout or '') if last else '')}"
    )


def verify_checksums_signature(
    recorder: Recorder, downloads_dir: Path, version: str, tag: str
) -> dict:
    """Verify the checksum manifest's Sigstore signature with a qualified identity."""
    manifest = common.checksums_manifest_name(version)
    bundle = common.sigstore_bundle_name(version)
    argv = [
        "cosign",
        "verify-blob",
        "--bundle",
        str(Path(downloads_dir) / bundle),
        "--certificate-identity",
        common.signer_identity(tag),
        "--certificate-oidc-issuer",
        common.OIDC_ISSUER,
        str(Path(downloads_dir) / manifest),
    ]
    result = recorder.run(argv, timeout=600)
    if result.returncode != 0:
        raise NpmPackagingError(
            f"cosign verify-blob failed for {manifest}: "
            f"{common.redact(result.stderr or result.stdout)}"
        )
    return {
        "tool": "cosign verify-blob",
        "manifest": manifest,
        "bundle": bundle,
        "certificate_identity": common.signer_identity(tag),
        "certificate_oidc_issuer": common.OIDC_ISSUER,
        "exit_code": result.returncode,
    }


def verify_release_attestation(recorder: Recorder, tag: str) -> dict:
    """Run ``gh release verify`` and return its parsed JSON result."""
    result = recorder.run(
        ["gh", "release", "verify", tag, "--repo", common.REPOSITORY, "--format", "json"],
        timeout=600,
    )
    if result.returncode != 0:
        raise NpmPackagingError(
            f"gh release verify failed for {tag}: {common.redact(result.stderr or result.stdout)}"
        )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise NpmPackagingError("gh release verify did not return JSON") from error


def verify_asset_attestation(recorder: Recorder, tag: str, archive_path: Path) -> dict:
    """Run ``gh release verify-asset`` for one archive."""
    result = recorder.run(
        [
            "gh",
            "release",
            "verify-asset",
            tag,
            str(archive_path),
            "--repo",
            common.REPOSITORY,
            "--format",
            "json",
        ],
        timeout=600,
    )
    if result.returncode != 0:
        raise NpmPackagingError(
            f"gh release verify-asset failed for {archive_path.name}: "
            f"{common.redact(result.stderr or result.stdout)}"
        )
    return {"tool": "gh release verify-asset", "exit_code": result.returncode}


def verify_build_attestations(
    recorder: Recorder,
    archive_path: Path,
    tag: str,
    commit: str,
    sleep: Callable[[float], None] = time.sleep,
) -> dict:
    """Verify build provenance and the signed SPDX predicate for one archive."""
    base = [
        "gh",
        "attestation",
        "verify",
        str(archive_path),
        "--repo",
        common.REPOSITORY,
        "--signer-workflow",
        f"{common.REPOSITORY}/{common.SIGNER_WORKFLOW}",
        "--source-ref",
        f"refs/tags/{tag}",
        "--source-digest",
        commit,
    ]
    provenance = run_with_propagation_retry(
        recorder, base, f"build provenance for {archive_path.name}", sleep
    )
    spdx = run_with_propagation_retry(
        recorder,
        base + ["--predicate-type", common.SPDX_PREDICATE_TYPE],
        f"SPDX attestation for {archive_path.name}",
        sleep,
    )
    return {
        "build_provenance": {
            "predicate_type": common.BUILD_PROVENANCE_PREDICATE_TYPE,
            "exit_code": provenance.returncode,
        },
        "spdx": {
            "predicate_type": common.SPDX_PREDICATE_TYPE,
            "exit_code": spdx.returncode,
        },
        "signer_workflow": f"{common.REPOSITORY}/{common.SIGNER_WORKFLOW}",
        "source_ref": f"refs/tags/{tag}",
        "source_digest": commit,
    }


def verify_all(
    api: ApiClient,
    recorder: Recorder,
    tag: str,
    targets: list[NpmTarget],
    downloads_dir: Path,
    download: bool = True,
    sleep: Callable[[float], None] = time.sleep,
) -> dict:
    """Run the complete read-only verification chain and return evidence."""
    version = common.parse_tag(tag)
    commit, tag_object_sha = resolve_tag_commit(api, tag)
    release = fetch_release(api, tag)
    assert_release_metadata(release, tag, version)
    assets = index_assets(release)

    names = required_download_names(version.text, targets)
    for name in names:
        if name not in assets:
            raise NpmPackagingError(f"Release {tag} is missing required asset {name!r}")
    ensure_downloads(recorder, tag, names, downloads_dir, download)

    manifest_name = common.checksums_manifest_name(version.text)
    manifest_path = Path(downloads_dir) / manifest_name
    rows = parse_checksum_manifest(manifest_path.read_text(encoding="utf-8"))
    archive_names = [target.platform.archive_name(version.text) for target in targets]
    selected = select_archive_rows(rows, archive_names)

    release_verify = verify_release_attestation(recorder, tag)
    attestation_digests, predicate, attested_tag_sha1 = decode_release_attestation(release_verify)
    assert_attestation_binds_release(predicate, attested_tag_sha1, release, tag, commit)
    if tag_object_sha is not None and attested_tag_sha1 != tag_object_sha:
        raise NpmPackagingError(
            f"Release attestation tag object {attested_tag_sha1} != resolved tag object "
            f"{tag_object_sha}; the release does not bind this tag"
        )

    cosign_evidence = verify_checksums_signature(recorder, downloads_dir, version.text, tag)
    verified_assets = cross_check_digests(
        names, downloads_dir, assets, selected, attestation_digests
    )

    asset_verifications: dict[str, dict] = {}
    attestation_verifications: dict[str, dict] = {}
    for target in targets:
        name = target.platform.archive_name(version.text)
        archive_path = Path(downloads_dir) / name
        asset_verifications[name] = verify_asset_attestation(recorder, tag, archive_path)
        attestation_verifications[name] = verify_build_attestations(
            recorder, archive_path, tag, commit, sleep
        )

    cargo_version = fetch_cargo_version(api, commit)
    if cargo_version != version.text:
        raise NpmPackagingError(
            f"Cargo.toml version {cargo_version!r} at {commit} != release version {version.text!r}"
        )

    evidence = {
        "schema_version": 1,
        "tag": tag,
        "version": version.text,
        "dist_tag": version.dist_tag,
        "release": {
            "id": release["id"],
            "immutable": release["immutable"],
            "draft": release["draft"],
            "prerelease": release["prerelease"],
            "published_at": release["published_at"],
            "html_url": release.get("html_url"),
            "tag_name": release["tag_name"],
            "target_commitish": release.get("target_commitish"),
        },
        "commit": commit,
        "tag_object": tag_object_sha,
        "attested_tag_object": attested_tag_sha1,
        "cargo_version": cargo_version,
        "assets": verified_assets,
        "attested_asset_digests": attestation_digests,
        "checksums": {
            "filename": manifest_name,
            "sha256": verified_assets[manifest_name]["sha256"],
            "row_count": len(rows),
            "selected_rows": selected,
        },
        "verifications": {
            "release_verify": {"tool": "gh release verify", "exit_code": 0},
            "cosign": cosign_evidence,
            "asset_verify": asset_verifications,
            "attestations": attestation_verifications,
        },
        "verified_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "host": common.host_fingerprint(),
        "commands": recorder.commands,
    }
    common.ensure_no_placeholders(evidence, "verification evidence")
    return evidence


def main(argv: list[str] | None = None) -> int:
    """CLI entry point: verify a release and write evidence JSON."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True, help="release tag, e.g. v0.6.3")
    parser.add_argument("--downloads", type=Path, required=True, help="asset directory")
    parser.add_argument("--evidence", type=Path, required=True, help="evidence output path")
    parser.add_argument("--matrix", type=Path, default=Path("matrix.jsonc"), help="matrix path")
    parser.add_argument(
        "--no-download", action="store_true", help="require assets to be present already"
    )
    args = parser.parse_args(argv)
    recorder = Recorder(SubprocessRunner())
    try:
        targets = common.load_npm_targets(args.matrix)
        evidence = verify_all(
            GhApiClient(recorder),
            recorder,
            args.tag,
            targets,
            args.downloads,
            download=not args.no_download,
        )
    except (NpmPackagingError, OSError, subprocess.SubprocessError) as error:
        print(f"verify_release: {common.redact(str(error))}", file=sys.stderr)
        return 1
    common.write_json(args.evidence, evidence)
    print(
        f"Verified {evidence['tag']} (release {evidence['release']['id']}, "
        f"commit {evidence['commit'][:12]}) -> {args.evidence}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
