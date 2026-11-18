#!/usr/bin/env python3
"""Read-only npm registry reconciliation for idempotent, safe publication.

Answers three questions without mutating anything:

* which of the five packages are absent, already published with *matching*
  bytes, or published with *mismatching* bytes (a hard stop, never overwritten);
* whether publishing this version would regress the ``latest`` dist-tag or
  point ``latest`` at a prerelease;
* what the registry actually holds after publication, for the manifest.

A 404 is a legitimate "absent" state, not an error. Only transport-shaped
failures are retried, and only a bounded number of times.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from npm import common
from npm.common import NpmPackagingError
from npm.verify_release import CommandRunner, Recorder, SubprocessRunner

TRANSIENT_MARKERS = (
    "etimedout",
    "econnreset",
    "econnrefused",
    "eai_again",
    "enotfound",
    "epipe",
    "socket hang up",
    "network",
    "500",
    "502",
    "503",
    "504",
    "429",
)
ABSENT_MARKERS = ("e404", "404 not found", "is not in this registry")
TRANSIENT_ATTEMPTS = 4
TRANSIENT_BACKOFF_SECONDS = (2, 5, 10)


def _classify_failure(text: str) -> str:
    """Classify npm stderr as 'absent', 'transient', or 'fatal'."""
    lowered = text.lower()
    if any(marker in lowered for marker in ABSENT_MARKERS):
        return "absent"
    if any(marker in lowered for marker in TRANSIENT_MARKERS):
        return "transient"
    return "fatal"


class RegistryClient:
    """Read-only npm registry client pinned to the public npm registry."""

    def __init__(
        self,
        recorder: Recorder,
        registry: str = common.NPM_REGISTRY,
        sleep: Callable[[float], None] = time.sleep,
    ) -> None:
        """Bind the client to a recorder, registry URL, and sleep function."""
        self._recorder = recorder
        self._registry = registry
        self._sleep = sleep

    def _view(self, spec: str, field: str | None = None) -> Any | None:
        """Run ``npm view`` returning parsed JSON, or None when absent."""
        argv = ["npm", "view", spec, "--json", "--registry", self._registry]
        if field:
            argv.insert(3, field)
        last_text = ""
        for attempt in range(TRANSIENT_ATTEMPTS):
            result = self._recorder.run(argv, timeout=300)
            if result.returncode == 0:
                text = (result.stdout or "").strip()
                if not text:
                    return None
                try:
                    return json.loads(text)
                except json.JSONDecodeError as error:
                    raise NpmPackagingError(
                        f"npm view {spec} returned invalid JSON: {error}"
                    ) from error
            last_text = f"{result.stderr or ''}{result.stdout or ''}"
            kind = _classify_failure(last_text)
            if kind == "absent":
                return None
            if kind == "transient" and attempt + 1 < TRANSIENT_ATTEMPTS:
                self._sleep(
                    TRANSIENT_BACKOFF_SECONDS[min(attempt, len(TRANSIENT_BACKOFF_SECONDS) - 1)]
                )
                continue
            break
        raise NpmPackagingError(
            f"npm view {spec} failed: {common.redact(last_text)}"
        )

    def version_manifest(self, name: str, version: str) -> dict | None:
        """Return the published version manifest, or None when absent."""
        payload = self._view(f"{name}@{version}")
        if payload is None:
            return None
        if isinstance(payload, list):
            matches = [item for item in payload if item.get("version") == version]
            if len(matches) != 1:
                raise NpmPackagingError(f"npm view {name}@{version} returned ambiguous results")
            payload = matches[0]
        if not isinstance(payload, dict):
            raise NpmPackagingError(f"npm view {name}@{version} returned an unexpected shape")
        return payload

    def dist_tags(self, name: str) -> dict:
        """Return the package's dist-tags, or an empty mapping when absent."""
        payload = self._view(name, "dist-tags")
        if payload is None:
            return {}
        if not isinstance(payload, dict):
            raise NpmPackagingError(f"npm view {name} dist-tags returned an unexpected shape")
        return payload


def check_dist_tag_transition(current_latest: str | None, version: common.Version) -> dict:
    """Assert publishing this version cannot regress or poison ``latest``.

    Prereleases publish to ``next`` and must leave ``latest`` untouched. Stable
    releases may only move ``latest`` forward (or keep it equal).
    """
    if version.is_prerelease:
        if current_latest is not None and current_latest != common.BOOTSTRAP_NPM_VERSION:
            existing = common.parse_npm_version(current_latest)
            if existing.is_prerelease:
                raise NpmPackagingError(
                    f"Registry 'latest' already points at prerelease {current_latest!r}; "
                    "refusing to continue until a human repairs the dist-tags"
                )
        return {
            "action": "publish-to-next",
            "current_latest": current_latest,
            "new_latest": current_latest,
            "dist_tag": "next",
        }
    if current_latest is None:
        return {
            "action": "set-latest",
            "current_latest": None,
            "new_latest": version.text,
            "dist_tag": "latest",
        }
    comparison = common.compare_versions(version.text, current_latest)
    if comparison < 0:
        raise NpmPackagingError(
            f"Publishing {version.text} would regress 'latest' from {current_latest!r}; "
            "refusing to publish"
        )
    return {
        "action": "set-latest" if comparison > 0 else "keep-latest",
        "current_latest": current_latest,
        "new_latest": version.text if comparison > 0 else current_latest,
        "dist_tag": "latest",
    }


def compare_to_staged(name: str, observed: dict, expected: dict) -> list[str]:
    """Return the list of mismatches between registry bytes and staged bytes."""
    problems: list[str] = []
    dist = observed.get("dist") or {}
    if observed.get("version") != expected["version"]:
        problems.append(f"version {observed.get('version')!r} != {expected['version']!r}")
    if dist.get("integrity") != expected["tarball"]["sha512_integrity"]:
        problems.append("dist.integrity != staged tarball integrity")
    if dist.get("shasum") and expected["tarball"].get("shasum"):
        if dist["shasum"] != expected["tarball"]["shasum"]:
            problems.append("dist.shasum != staged tarball shasum")
    if expected["kind"] == "platform":
        if observed.get("os") != [expected["os"]]:
            problems.append(f"os {observed.get('os')!r} != [{expected['os']!r}]")
        if observed.get("cpu") != [expected["cpu"]]:
            problems.append(f"cpu {observed.get('cpu')!r} != [{expected['cpu']!r}]")
        if observed.get("bin"):
            problems.append("platform package must not declare a bin entry")
    else:
        if observed.get("bin") != {"poof": "bin/poof.js"}:
            problems.append(f"bin {observed.get('bin')!r} != {{'poof': 'bin/poof.js'}}")
        deps = observed.get("optionalDependencies") or {}
        if deps != expected["optional_dependencies"]:
            problems.append("optionalDependencies do not match the exact pinned versions")
        if (observed.get("engines") or {}).get("node") != common.NODE_ENGINE:
            problems.append(f"engines.node != {common.NODE_ENGINE!r}")
    if observed.get("scripts"):
        problems.append("published package must not declare lifecycle scripts")
    return problems


def _expected_packages(staging: dict) -> list[dict]:
    """Derive the expected registry shape for all five packages from staging."""
    expected = []
    for platform_report in staging["platforms"]:
        expected.append(
            {
                "name": platform_report["package_name"],
                "version": platform_report["version"],
                "kind": "platform",
                "os": platform_report["os"],
                "cpu": platform_report["cpu"],
                "tarball": platform_report["tarball"],
            }
        )
    base = staging["base"]
    expected.append(
        {
            "name": base["package_name"],
            "version": base["version"],
            "kind": "launcher",
            "tarball": base["tarball"],
            "optional_dependencies": base["optional_dependencies"],
        }
    )
    return expected


def reconcile(
    client: RegistryClient,
    staging: dict,
    require_published: bool = False,
) -> dict:
    """Observe the registry and decide, per package, what publication may do."""
    version = common.parse_npm_version(staging["version"])
    expected_packages = _expected_packages(staging)
    observations: dict[str, dict] = {}
    decisions: dict[str, dict] = {}
    now = datetime.now(timezone.utc).isoformat(timespec="seconds")

    for expected in expected_packages:
        name = expected["name"]
        observed = client.version_manifest(name, version.text)
        if observed is None:
            decisions[name] = {"state": "absent", "action": "publish", "problems": []}
            continue
        problems = compare_to_staged(name, observed, expected)
        dist = observed.get("dist") or {}
        tags = client.dist_tags(name)
        observations[name] = {
            "version": observed.get("version"),
            "integrity": dist.get("integrity"),
            "tarball": dist.get("tarball"),
            "shasum": dist.get("shasum"),
            "dist_tags": tags,
            "provenance": dist.get("attestations"),
            "observed_at": now,
        }
        if problems:
            decisions[name] = {"state": "mismatch", "action": "stop", "problems": problems}
        else:
            decisions[name] = {"state": "match", "action": "skip", "problems": []}

    base_tags = client.dist_tags(common.BASE_PACKAGE)
    transition = check_dist_tag_transition(base_tags.get("latest"), version)

    mismatches = {name: d for name, d in decisions.items() if d["state"] == "mismatch"}
    absent = [name for name, d in decisions.items() if d["state"] == "absent"]
    if require_published and absent:
        raise NpmPackagingError(
            f"Expected all five packages published, but these are absent: {sorted(absent)}"
        )
    if mismatches:
        detail = "; ".join(
            f"{name}: {', '.join(info['problems'])}" for name, info in sorted(mismatches.items())
        )
        raise NpmPackagingError(
            "Registry holds different bytes for an already-published version; "
            f"refusing to continue (immutable versions must never be overwritten). {detail}"
        )
    return {
        "schema_version": 1,
        "registry": common.NPM_REGISTRY,
        "version": version.text,
        "tag": version.tag,
        "dist_tag": version.dist_tag,
        "observed_at": now,
        "observations": observations,
        "decisions": decisions,
        "dist_tags": {common.BASE_PACKAGE: base_tags},
        "dist_tag_transition": transition,
        "publish_order": [
            expected["name"]
            for expected in expected_packages
            if decisions[expected["name"]]["action"] == "publish"
        ],
        "all_published": not absent,
    }


def main(argv: list[str] | None = None) -> int:
    """CLI entry point: reconcile the registry against a staging report."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--staging", type=Path, required=True, help="stage_packages.py report")
    parser.add_argument("--out", type=Path, required=True, help="reconciliation output path")
    parser.add_argument(
        "--require-published",
        action="store_true",
        help="fail unless all five packages are already published and matching",
    )
    args = parser.parse_args(argv)
    recorder = Recorder(SubprocessRunner())
    try:
        staging = common.read_json(args.staging)
        result = reconcile(RegistryClient(recorder), staging, args.require_published)
    except (NpmPackagingError, OSError, subprocess.SubprocessError, KeyError) as error:
        print(f"verify_registry: {common.redact(str(error))}", file=sys.stderr)
        return 1
    common.write_json(args.out, result)
    states = {name: info["state"] for name, info in sorted(result["decisions"].items())}
    print(f"Registry reconciliation for {result['tag']}: {states}")
    print(f"Publish order: {result['publish_order'] or '(none — already published)'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
