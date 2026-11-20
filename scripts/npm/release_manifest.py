#!/usr/bin/env python3
"""Build the release-to-npm manifest: one auditable record per publication.

The manifest joins the read-only verification evidence with the staging report
so a reviewer can trace every published npm tarball back to an immutable GitHub
release asset, its signed checksums, and its attestations.

It is written as a **workflow artifact**, never as a release asset: the native
release's asset set is immutable and must not be modified by npm packaging.

Existing native supply-chain records (``*.spdx.json``,
``*.build-environment.json``) are referenced by their attested digests and are
never regenerated here. See ``SPDX_PROVENANCE_NOTE`` for what those records
actually cover.
"""

from __future__ import annotations

import argparse
import copy
import json
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from npm import common
from npm.common import NpmPackagingError

SCHEMA_VERSION = 1
MANIFEST_FILENAME = "release-to-npm-manifest.json"

# Honesty about the existing native SBOM: anchore/sbom-action scans a copy of
# Cargo.toml/Cargo.lock placed under _sbom/, so the per-target SPDX document is a
# Rust dependency inventory, not a proven inventory of the shipped archive bytes.
SPDX_PROVENANCE_NOTE = (
    "The native release's per-target *.spdx.json documents are produced by "
    "anchore/sbom-action scanning copies of Cargo.toml and Cargo.lock under _sbom/. "
    "They are a Rust dependency inventory, not a proven SBOM of the shipped archive "
    "contents. npm packaging references these existing records as-is and never "
    "regenerates them. A separate archive-content SBOM audit is out of scope."
)

REQUIRED_TOP_LEVEL = (
    "schema_version",
    "repository",
    "tag",
    "version",
    "dist_tag",
    "release",
    "native_artifacts",
    "supply_chain",
    "packages",
    "publish",
    "inputs",
)


def _strip_environment_paths(report: dict) -> dict:
    """Copy a staging report without host-specific absolute paths."""
    cleaned = copy.deepcopy(report)
    for entry in cleaned.get("platforms", []):
        entry.get("tarball", {}).pop("path", None)
    cleaned.get("base", {}).get("tarball", {}).pop("path", None)
    return cleaned


def _native_artifacts(evidence: dict, staging: dict) -> dict:
    """Describe the native inputs, including referenced SPDX/build-env records."""
    version = evidence["version"]
    attested = evidence.get("attested_asset_digests", {})
    archives = []
    for platform_report in staging["platforms"]:
        target = platform_report["rust_target"]
        spdx_name = f"{common.BINARY_NAME}-{version}-{target}.spdx.json"
        build_env_name = f"{common.BINARY_NAME}-{version}-{target}.build-environment.json"
        archives.append(
            {
                "name": platform_report["archive"]["name"],
                "sha256": platform_report["archive"]["sha256"],
                "rust_target": target,
                "builder": platform_report["builder"],
                "runner": platform_report["runner"],
                "referenced_records": {
                    "spdx": {"name": spdx_name, "attested_sha256": attested.get(spdx_name)},
                    "build_environment": {
                        "name": build_env_name,
                        "attested_sha256": attested.get(build_env_name),
                    },
                },
            }
        )
    checksums = evidence["checksums"]
    bundle_name = common.sigstore_bundle_name(version)
    return {
        "checksums": {"name": checksums["filename"], "sha256": checksums["sha256"]},
        "sigstore_bundle": {
            "name": bundle_name,
            "sha256": evidence["assets"][bundle_name]["sha256"],
        },
        "archives": archives,
    }


def _packages(evidence: dict, staging: dict) -> list[dict]:
    """Describe all five packages, with registry fields left null pre-publish."""
    packages = []
    for platform_report in staging["platforms"]:
        packages.append(
            {
                "name": platform_report["package_name"],
                "version": platform_report["version"],
                "kind": "platform",
                "os": platform_report["os"],
                "cpu": platform_report["cpu"],
                "rust_target": platform_report["rust_target"],
                "tarball": {
                    "filename": platform_report["tarball"]["filename"],
                    "sha256": platform_report["tarball"]["sha256"],
                    "integrity": platform_report["tarball"]["sha512_integrity"],
                    "size_bytes": platform_report["tarball"]["size_bytes"],
                },
                "binary": {
                    "sha256": platform_report["binary"]["sha256"],
                    "size_bytes": platform_report["binary"]["size_bytes"],
                    "mode": platform_report["binary"]["mode"],
                    "format": platform_report["binary"]["format"],
                    "static": platform_report["binary"].get("static"),
                },
                "source_archive_sha256": platform_report["archive"]["sha256"],
                "registry": None,
            }
        )
    base = staging["base"]
    packages.append(
        {
            "name": base["package_name"],
            "version": base["version"],
            "kind": "launcher",
            "os": None,
            "cpu": None,
            "rust_target": None,
            "tarball": {
                "filename": base["tarball"]["filename"],
                "sha256": base["tarball"]["sha256"],
                "integrity": base["tarball"]["sha512_integrity"],
                "size_bytes": base["tarball"]["size_bytes"],
            },
            "launcher": base["launcher"],
            "optional_dependencies": base["optional_dependencies"],
            "registry": None,
        }
    )
    return packages


def build_manifest(
    evidence: dict,
    staging: dict,
    evidence_sha256: str | None = None,
    staging_sha256: str | None = None,
    generated_at: str | None = None,
) -> dict:
    """Join verification evidence and staging output into the pre-publish manifest."""
    if evidence.get("version") != staging.get("version"):
        raise NpmPackagingError("Evidence and staging report disagree on version")
    if evidence.get("commit") != staging.get("commit"):
        raise NpmPackagingError("Evidence and staging report disagree on commit")
    if evidence.get("release", {}).get("id") != staging.get("release_id"):
        raise NpmPackagingError("Evidence and staging report disagree on release id")
    version = common.parse_npm_version(evidence["version"])
    if evidence["tag"] != version.tag:
        raise NpmPackagingError("Evidence tag does not match its version")
    cleaned = _strip_environment_paths(staging)
    packages = _packages(evidence, cleaned)
    platform_names = [package["name"] for package in packages if package["kind"] == "platform"]
    manifest = {
        "schema_version": SCHEMA_VERSION,
        "repository": common.REPOSITORY,
        "tag": evidence["tag"],
        "version": version.text,
        "dist_tag": version.dist_tag,
        "generated_at": generated_at
        or datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "release": {
            "id": evidence["release"]["id"],
            "immutable": evidence["release"]["immutable"],
            "draft": evidence["release"]["draft"],
            "prerelease": evidence["release"]["prerelease"],
            "published_at": evidence["release"]["published_at"],
            "html_url": evidence["release"]["html_url"],
            "commit": evidence["commit"],
            "tag_object": evidence.get("tag_object"),
            "attested_tag_object": evidence.get("attested_tag_object"),
            "cargo_version": evidence["cargo_version"],
        },
        "native_artifacts": _native_artifacts(evidence, cleaned),
        "supply_chain": {
            "cosign": evidence["verifications"]["cosign"],
            "release_attestation": evidence["verifications"]["release_verify"],
            "asset_attestations": evidence["verifications"]["attestations"],
            "spdx_provenance_note": SPDX_PROVENANCE_NOTE,
            "regenerated_by_npm_packaging": False,
        },
        "packages": packages,
        "publish": {
            "registry": common.NPM_REGISTRY,
            "order": platform_names + [common.BASE_PACKAGE],
            "dist_tag": version.dist_tag,
            "status": "pending",
            "method": "oidc-trusted-publishing",
        },
        "inputs": {
            "evidence_sha256": evidence_sha256,
            "staging_report_sha256": staging_sha256,
            "verification_host": evidence.get("host"),
            "staging_host": staging.get("host"),
        },
    }
    validate_manifest(manifest)
    return manifest


def validate_manifest(manifest: Any) -> None:
    """Validate the manifest's structure, invariants, and cross-references."""
    if not isinstance(manifest, dict):
        raise NpmPackagingError("Manifest must be an object")
    missing = [key for key in REQUIRED_TOP_LEVEL if key not in manifest]
    if missing:
        raise NpmPackagingError(f"Manifest is missing required keys: {missing}")
    if manifest["schema_version"] != SCHEMA_VERSION:
        raise NpmPackagingError(
            f"Unsupported manifest schema_version {manifest['schema_version']!r}"
        )
    if manifest["repository"] != common.REPOSITORY:
        raise NpmPackagingError(f"Unexpected repository {manifest['repository']!r}")
    version = common.parse_npm_version(manifest["version"])
    if manifest["tag"] != version.tag:
        raise NpmPackagingError("Manifest tag does not match its version")
    if manifest["dist_tag"] != version.dist_tag:
        raise NpmPackagingError("Manifest dist_tag disagrees with SemVer prerelease status")
    release = manifest["release"]
    if release.get("immutable") is not True or release.get("draft") is not False:
        raise NpmPackagingError("Manifest must reference an immutable, published release")
    if not common.COMMIT_SHA_RE.match(str(release.get("commit"))):
        raise NpmPackagingError("Manifest release.commit is not a 40-character SHA")
    if release.get("cargo_version") != version.text:
        raise NpmPackagingError("Manifest Cargo version does not match the release version")
    packages = manifest["packages"]
    if not isinstance(packages, list) or len(packages) != 5:
        raise NpmPackagingError("Manifest must describe exactly five packages")
    names = [package["name"] for package in packages]
    if len(set(names)) != 5:
        raise NpmPackagingError("Manifest package names are not unique")
    platforms = [package for package in packages if package["kind"] == "platform"]
    launchers = [package for package in packages if package["kind"] == "launcher"]
    if len(platforms) != 4 or len(launchers) != 1:
        raise NpmPackagingError("Manifest must have four platform packages and one launcher")
    if launchers[0]["name"] != common.BASE_PACKAGE:
        raise NpmPackagingError(f"Launcher package must be {common.BASE_PACKAGE!r}")
    combos = {(package["os"], package["cpu"]) for package in platforms}
    if combos != set(common.SUPPORTED_TRIPLES.values()):
        raise NpmPackagingError(f"Manifest platform coverage is incomplete: {sorted(combos)}")
    for package in packages:
        if package["version"] != version.text:
            raise NpmPackagingError(f"{package['name']}: version does not match the release")
        tarball = package["tarball"]
        if not common.SHA256_RE.match(str(tarball.get("sha256"))):
            raise NpmPackagingError(f"{package['name']}: tarball sha256 is invalid")
        integrity = tarball.get("integrity")
        if not isinstance(integrity, str) or not integrity.startswith("sha512-"):
            raise NpmPackagingError(f"{package['name']}: tarball integrity is invalid")
        if package["kind"] == "platform":
            expected_triple = dict(
                (value, key) for key, value in common.SUPPORTED_TRIPLES.items()
            )[(package["os"], package["cpu"])]
            if package["rust_target"] != expected_triple:
                raise NpmPackagingError(
                    f"{package['name']}: rust_target disagrees with os/cpu"
                )
            if not common.SHA256_RE.match(str(package.get("source_archive_sha256"))):
                raise NpmPackagingError(f"{package['name']}: source archive sha256 is invalid")
            if package["binary"].get("mode") != "755":
                raise NpmPackagingError(f"{package['name']}: packaged binary mode must be 755")
            if package["os"] == "linux" and package["binary"].get("static") is not True:
                raise NpmPackagingError(f"{package['name']}: Linux binary must be static musl")
        else:
            deps = package.get("optional_dependencies") or {}
            if set(deps) != {platform["name"] for platform in platforms}:
                raise NpmPackagingError("Launcher optionalDependencies must cover all platforms")
            for name, pinned in deps.items():
                if pinned != version.text:
                    raise NpmPackagingError(f"Launcher must pin {name} to exact {version.text}")
    publish = manifest["publish"]
    if publish.get("registry") != common.NPM_REGISTRY:
        raise NpmPackagingError(f"Manifest registry must be {common.NPM_REGISTRY!r}")
    if publish.get("order") != [package["name"] for package in platforms] + [common.BASE_PACKAGE]:
        raise NpmPackagingError("Publish order must list platform packages before the launcher")
    if publish.get("dist_tag") != version.dist_tag:
        raise NpmPackagingError("Publish dist_tag disagrees with the version")
    archives = manifest["native_artifacts"]["archives"]
    if len(archives) != 4:
        raise NpmPackagingError("Manifest must reference four native archives")
    for archive in archives:
        if not common.SHA256_RE.match(str(archive.get("sha256"))):
            raise NpmPackagingError(f"{archive.get('name')}: archive sha256 is invalid")
    common.ensure_no_placeholders(manifest, "manifest")


def enrich_with_registry(manifest: dict, records: dict[str, dict]) -> dict:
    """Attach post-publish registry facts, asserting they match the staged bytes.

    ``records`` maps package name to the registry observation produced by
    ``observations`` field in the ``verify_registry.py`` report. A mismatch
    between the published integrity and the staged tarball integrity is a hard
    failure: it would mean the registry holds bytes that were never verified.
    """
    enriched = copy.deepcopy(manifest)
    seen = set()
    for package in enriched["packages"]:
        name = package["name"]
        record = records.get(name)
        if record is None:
            raise NpmPackagingError(f"No registry observation for {name}")
        seen.add(name)
        if record.get("version") != package["version"]:
            raise NpmPackagingError(f"{name}: registry version != staged version")
        integrity = record.get("integrity")
        if integrity != package["tarball"]["integrity"]:
            raise NpmPackagingError(
                f"{name}: registry integrity {integrity!r} != staged tarball integrity "
                f"{package['tarball']['integrity']!r}; published bytes were not verified"
            )
        package["registry"] = {
            "integrity": integrity,
            "tarball": record.get("tarball"),
            "shasum": record.get("shasum"),
            "dist_tags": record.get("dist_tags"),
            "provenance": record.get("provenance"),
            "observed_at": record.get("observed_at"),
        }
    unexpected = set(records) - seen
    if unexpected:
        raise NpmPackagingError(f"Registry observations for unknown packages: {sorted(unexpected)}")
    enriched["publish"]["status"] = "published"
    validate_manifest(enriched)
    return enriched


def core_sha256(manifest: dict) -> str:
    """Hash the deterministic core, excluding timestamps and registry enrichment."""
    core = copy.deepcopy(manifest)
    core.pop("generated_at", None)
    core.get("publish", {}).pop("status", None)
    for package in core.get("packages", []):
        package["registry"] = None
    return common.sha256_bytes(common.dump_json(core).encode("utf-8"))


def main(argv: list[str] | None = None) -> int:
    """CLI entry point: build (and optionally enrich) the release-to-npm manifest."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True, help="verify_release.py output")
    parser.add_argument("--staging", type=Path, required=True, help="stage_packages.py report")
    parser.add_argument("--out", type=Path, required=True, help="manifest output path")
    parser.add_argument(
        "--registry-json", type=Path, help="verify_registry.py reconciliation report to attach"
    )
    parser.add_argument("--generated-at", help="override timestamp (reproducibility/testing)")
    args = parser.parse_args(argv)
    try:
        evidence = common.read_json(args.evidence)
        staging = common.read_json(args.staging)
        manifest = build_manifest(
            evidence,
            staging,
            evidence_sha256=common.sha256_file(args.evidence),
            staging_sha256=common.sha256_file(args.staging),
            generated_at=args.generated_at,
        )
        if args.registry_json:
            report = common.read_json(args.registry_json)
            manifest = enrich_with_registry(manifest, report["observations"])
    except (NpmPackagingError, OSError, KeyError, json.JSONDecodeError) as error:
        print(f"release_manifest: {common.redact(str(error))}", file=sys.stderr)
        return 1
    common.write_json(args.out, manifest)
    print(
        f"Wrote {args.out} for {manifest['tag']} "
        f"(status={manifest['publish']['status']}, core={core_sha256(manifest)[:12]})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
