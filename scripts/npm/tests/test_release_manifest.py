"""Tests for the release-to-npm manifest: invariants, enrichment, determinism."""

from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path

from _support import REPO_ROOT, common, synthetic_evidence, synthetic_staging
from npm import release_manifest as rm
from npm.common import NpmPackagingError

GENERATED_AT = "2026-11-09T00:00:00+00:00"


def build(version="0.6.3"):
    """Build a valid pre-publish manifest from synthetic inputs."""
    return rm.build_manifest(
        synthetic_evidence(version),
        synthetic_staging(version),
        evidence_sha256=common.sha256_bytes(b"evidence"),
        staging_sha256=common.sha256_bytes(b"staging"),
        generated_at=GENERATED_AT,
    )


def registry_records(manifest):
    """Registry observations whose bytes match the staged tarballs exactly."""
    records = {}
    for package in manifest["packages"]:
        records[package["name"]] = {
            "version": package["version"],
            "integrity": package["tarball"]["integrity"],
            "tarball": f"{common.NPM_REGISTRY}/{package['name']}/-/pkg-{package['version']}.tgz",
            "shasum": "a" * 40,
            "dist_tags": {"latest": package["version"]},
            "provenance": {"url": f"{common.NPM_REGISTRY}/-/npm/v1/attestations/x"},
            "observed_at": GENERATED_AT,
        }
    return records


def walk_strings(value):
    """Yield every string in a nested JSON structure."""
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for item in value.values():
            yield from walk_strings(item)
    elif isinstance(value, list):
        for item in value:
            yield from walk_strings(item)


def linux_package(manifest):
    """Return the first Linux platform package entry."""
    return next(p for p in manifest["packages"] if p.get("os") == "linux")


class BuildManifestTests(unittest.TestCase):
    """The pre-publish manifest joins evidence and staging without host paths."""

    def setUp(self):
        self.manifest = build()

    def test_describes_exactly_five_packages(self):
        self.assertEqual(len(self.manifest["packages"]), 5)
        kinds = [p["kind"] for p in self.manifest["packages"]]
        self.assertEqual(kinds.count("platform"), 4)
        self.assertEqual(kinds.count("launcher"), 1)

    def test_publish_order_puts_platforms_before_launcher(self):
        order = self.manifest["publish"]["order"]
        platform_names = {
            p["name"] for p in self.manifest["packages"] if p["kind"] == "platform"
        }
        self.assertEqual(len(order), 5)
        self.assertEqual(order[-1], common.BASE_PACKAGE)
        self.assertEqual(set(order[:4]), platform_names)

    def test_all_versions_match_the_release(self):
        for package in self.manifest["packages"]:
            self.assertEqual(package["version"], "0.6.3")
        self.assertEqual(self.manifest["release"]["cargo_version"], "0.6.3")
        self.assertEqual(self.manifest["tag"], "v0.6.3")

    def test_launcher_pins_exact_optional_versions(self):
        launcher = self.manifest["packages"][-1]
        deps = launcher["optional_dependencies"]
        self.assertEqual(len(deps), 4)
        for name, pinned in deps.items():
            self.assertEqual(pinned, "0.6.3")
            self.assertNotIn(pinned[0], "^~><=*")

    def test_status_is_pending_before_publication(self):
        self.assertEqual(self.manifest["publish"]["status"], "pending")
        self.assertEqual(self.manifest["publish"]["method"], "oidc-trusted-publishing")
        self.assertEqual(self.manifest["publish"]["registry"], common.NPM_REGISTRY)
        for package in self.manifest["packages"]:
            self.assertIsNone(package["registry"])

    def test_stable_release_uses_latest(self):
        self.assertEqual(self.manifest["dist_tag"], "latest")
        self.assertEqual(self.manifest["publish"]["dist_tag"], "latest")

    def test_prerelease_uses_next(self):
        manifest = build("0.6.4-rc.1")
        self.assertEqual(manifest["dist_tag"], "next")
        self.assertEqual(manifest["publish"]["dist_tag"], "next")

    def test_contains_no_host_absolute_paths(self):
        for text in walk_strings(self.manifest):
            self.assertFalse(text.startswith("/"), f"absolute path leaked: {text}")
        self.assertNotIn("/tmp/staged", json.dumps(self.manifest))

    def test_contains_no_placeholders(self):
        self.assertNotIn("replace-me", json.dumps(self.manifest))

    def test_references_existing_supply_chain_records_by_digest(self):
        evidence = synthetic_evidence()
        archives = self.manifest["native_artifacts"]["archives"]
        self.assertEqual(len(archives), 4)
        for archive in archives:
            records = archive["referenced_records"]
            for key in ("spdx", "build_environment"):
                name = records[key]["name"]
                self.assertEqual(
                    records[key]["attested_sha256"],
                    evidence["attested_asset_digests"][name],
                )

    def test_is_honest_about_spdx_coverage(self):
        note = self.manifest["supply_chain"]["spdx_provenance_note"]
        self.assertEqual(note, rm.SPDX_PROVENANCE_NOTE)
        for fragment in ("Cargo.toml", "Cargo.lock", "_sbom/", "not a proven SBOM"):
            self.assertIn(fragment, note)
        self.assertIs(self.manifest["supply_chain"]["regenerated_by_npm_packaging"], False)

    def test_records_input_digests(self):
        self.assertEqual(
            self.manifest["inputs"]["evidence_sha256"], common.sha256_bytes(b"evidence")
        )
        self.assertEqual(
            self.manifest["inputs"]["staging_report_sha256"], common.sha256_bytes(b"staging")
        )

    def test_manifest_filename_is_a_workflow_artifact_name(self):
        self.assertEqual(rm.MANIFEST_FILENAME, "release-to-npm-manifest.json")

    def test_rejects_version_disagreement(self):
        with self.assertRaisesRegex(NpmPackagingError, "disagree on version"):
            rm.build_manifest(synthetic_evidence("0.6.3"), synthetic_staging("0.6.4"))

    def test_rejects_commit_disagreement(self):
        with self.assertRaisesRegex(NpmPackagingError, "disagree on commit"):
            rm.build_manifest(synthetic_evidence(), synthetic_staging(commit="8" * 40))

    def test_rejects_release_id_disagreement(self):
        staging = synthetic_staging()
        staging["release_id"] = 1
        with self.assertRaisesRegex(NpmPackagingError, "disagree on release id"):
            rm.build_manifest(synthetic_evidence(), staging)

    def test_rejects_tag_version_disagreement(self):
        evidence = synthetic_evidence()
        evidence["tag"] = "v9.9.9"
        with self.assertRaisesRegex(NpmPackagingError, "tag does not match its version"):
            rm.build_manifest(evidence, synthetic_staging())


class ValidateManifestTests(unittest.TestCase):
    """Every structural and security invariant is enforced, not advisory."""

    def setUp(self):
        self.manifest = build()

    def test_accepts_the_valid_manifest(self):
        rm.validate_manifest(self.manifest)

    def test_rejects_invalid_manifests(self):
        cases = [
            ("missing key", lambda m: m.pop("packages"), "missing required keys"),
            ("schema", lambda m: m.update(schema_version=2), "Unsupported manifest schema_version"),
            ("repository", lambda m: m.update(repository="other/repo"), "Unexpected repository"),
            ("tag", lambda m: m.update(tag="v9.9.9"), "tag does not match its version"),
            ("dist_tag", lambda m: m.update(dist_tag="next"), "disagrees with SemVer"),
            ("mutable", lambda m: m["release"].update(immutable=False), "immutable, published"),
            ("draft", lambda m: m["release"].update(draft=True), "immutable, published"),
            ("commit", lambda m: m["release"].update(commit="abc"), "40-character SHA"),
            ("cargo", lambda m: m["release"].update(cargo_version="0.0.1"), "Cargo version"),
            ("count", lambda m: m["packages"].pop(), "exactly five packages"),
            ("dup names", lambda m: m["packages"][1].update(name=m["packages"][0]["name"]),
             "not unique"),
            ("kinds", lambda m: m["packages"][0].update(kind="launcher"),
             "four platform packages and one launcher"),
            ("launcher name", lambda m: m["packages"][-1].update(name="@pirafrank/other"),
             "Launcher package must be"),
            ("coverage", lambda m: m["packages"][1].update(os="linux", cpu="x64"),
             "platform coverage is incomplete"),
            ("pkg version", lambda m: m["packages"][0].update(version="0.0.1"),
             "version does not match the release"),
            ("tarball sha", lambda m: m["packages"][0]["tarball"].update(sha256="zz"),
             "tarball sha256 is invalid"),
            ("integrity", lambda m: m["packages"][0]["tarball"].update(integrity="sha1-abc"),
             "tarball integrity is invalid"),
            ("triple", lambda m: m["packages"][0].update(rust_target="wrong-triple"),
             "rust_target disagrees with os/cpu"),
            ("archive sha", lambda m: m["packages"][0].update(source_archive_sha256="zz"),
             "source archive sha256 is invalid"),
            ("mode", lambda m: m["packages"][0]["binary"].update(mode="644"),
             "packaged binary mode must be 755"),
            ("static", lambda m: linux_package(m)["binary"].update(static=False),
             "Linux binary must be static musl"),
            ("deps coverage",
             lambda m: m["packages"][-1]["optional_dependencies"].popitem(),
             "must cover all platforms"),
            ("range pin",
             lambda m: m["packages"][-1]["optional_dependencies"].update(
                 {"@pirafrank/poof-linux-x64": "^0.6.3"}),
             "must pin"),
            ("registry", lambda m: m["publish"].update(registry="https://example.invalid"),
             "Manifest registry must be"),
            ("order", lambda m: m["publish"].update(order=list(reversed(m["publish"]["order"]))),
             "platform packages before the launcher"),
            ("publish tag", lambda m: m["publish"].update(dist_tag="next"),
             "Publish dist_tag disagrees"),
            ("archives", lambda m: m["native_artifacts"]["archives"].pop(),
             "four native archives"),
            ("archive digest",
             lambda m: m["native_artifacts"]["archives"][0].update(sha256="zz"),
             "archive sha256 is invalid"),
            ("not an object", lambda m: m.clear(), "missing required keys"),
        ]
        for label, mutate, pattern in cases:
            with self.subTest(case=label):
                candidate = copy.deepcopy(self.manifest)
                mutate(candidate)
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    rm.validate_manifest(candidate)

    def test_rejects_non_object(self):
        with self.assertRaisesRegex(NpmPackagingError, "Manifest must be an object"):
            rm.validate_manifest([])

    def test_rejects_placeholders(self):
        candidate = copy.deepcopy(self.manifest)
        candidate["packages"][0]["tarball"]["filename"] = "0.0.0-replace-me.tgz"
        with self.assertRaises(NpmPackagingError):
            rm.validate_manifest(candidate)


class EnrichWithRegistryTests(unittest.TestCase):
    """Post-publish enrichment must never accept unverified bytes."""

    def setUp(self):
        self.manifest = build()
        self.records = registry_records(self.manifest)

    def test_attaches_registry_facts_and_marks_published(self):
        enriched = rm.enrich_with_registry(self.manifest, self.records)
        self.assertEqual(enriched["publish"]["status"], "published")
        for package in enriched["packages"]:
            registry = package["registry"]
            self.assertEqual(registry["integrity"], package["tarball"]["integrity"])
            self.assertEqual(registry["observed_at"], GENERATED_AT)
            self.assertIsNotNone(registry["tarball"])

    def test_does_not_mutate_the_input(self):
        rm.enrich_with_registry(self.manifest, self.records)
        self.assertEqual(self.manifest["publish"]["status"], "pending")
        for package in self.manifest["packages"]:
            self.assertIsNone(package["registry"])

    def test_rejects_missing_observation(self):
        del self.records[common.BASE_PACKAGE]
        with self.assertRaisesRegex(NpmPackagingError, "No registry observation"):
            rm.enrich_with_registry(self.manifest, self.records)

    def test_rejects_version_mismatch(self):
        self.records[common.BASE_PACKAGE]["version"] = "0.6.4"
        with self.assertRaisesRegex(NpmPackagingError, "registry version != staged version"):
            rm.enrich_with_registry(self.manifest, self.records)

    def test_rejects_integrity_mismatch_as_hard_stop(self):
        name = self.manifest["packages"][0]["name"]
        self.records[name]["integrity"] = "sha512-" + "A" * 88
        with self.assertRaisesRegex(NpmPackagingError, "published bytes were not verified"):
            rm.enrich_with_registry(self.manifest, self.records)

    def test_rejects_unknown_packages(self):
        self.records["@pirafrank/not-a-package"] = {"version": "0.6.3"}
        with self.assertRaisesRegex(NpmPackagingError, "unknown packages"):
            rm.enrich_with_registry(self.manifest, self.records)


class CoreSha256Tests(unittest.TestCase):
    """The deterministic core ignores timestamps and publication state."""

    def setUp(self):
        self.manifest = build()

    def test_stable_across_generated_at(self):
        other = copy.deepcopy(self.manifest)
        other["generated_at"] = "2030-01-01T00:00:00+00:00"
        self.assertEqual(rm.core_sha256(self.manifest), rm.core_sha256(other))

    def test_stable_across_publish_status(self):
        other = copy.deepcopy(self.manifest)
        other["publish"]["status"] = "published"
        self.assertEqual(rm.core_sha256(self.manifest), rm.core_sha256(other))

    def test_stable_across_registry_enrichment(self):
        enriched = rm.enrich_with_registry(self.manifest, registry_records(self.manifest))
        self.assertEqual(rm.core_sha256(self.manifest), rm.core_sha256(enriched))

    def test_changes_when_tarball_bytes_change(self):
        other = copy.deepcopy(self.manifest)
        other["packages"][0]["tarball"]["sha256"] = common.sha256_bytes(b"different")
        self.assertNotEqual(rm.core_sha256(self.manifest), rm.core_sha256(other))

    def test_is_a_sha256_hex_digest(self):
        self.assertRegex(rm.core_sha256(self.manifest), r"^[0-9a-f]{64}$")


class CliTests(unittest.TestCase):
    """The CLI writes the manifest and fails closed on bad inputs."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.dir = Path(self._tmp.name)
        self.evidence = self.dir / "evidence.json"
        self.staging = self.dir / "staging.json"
        self.out = self.dir / rm.MANIFEST_FILENAME
        common.write_json(self.evidence, synthetic_evidence())
        common.write_json(self.staging, synthetic_staging())

    def argv(self, *extra):
        return [
            "--evidence", str(self.evidence),
            "--staging", str(self.staging),
            "--out", str(self.out),
            "--generated-at", GENERATED_AT,
            *extra,
        ]

    def test_writes_manifest(self):
        self.assertEqual(rm.main(self.argv()), 0)
        manifest = common.read_json(self.out)
        self.assertEqual(manifest["publish"]["status"], "pending")
        self.assertEqual(manifest["generated_at"], GENERATED_AT)

    def test_enriches_from_registry_json(self):
        records = self.dir / "registry.json"
        common.write_json(records, registry_records(build()))
        self.assertEqual(rm.main(self.argv("--registry-json", str(records))), 0)
        manifest = common.read_json(self.out)
        self.assertEqual(manifest["publish"]["status"], "published")

    def test_fails_on_missing_input(self):
        self.evidence.unlink()
        self.assertEqual(rm.main(self.argv()), 1)
        self.assertFalse(self.out.exists())

    def test_fails_on_disagreeing_inputs(self):
        common.write_json(self.staging, synthetic_staging("0.6.4"))
        self.assertEqual(rm.main(self.argv()), 1)
        self.assertFalse(self.out.exists())

    def test_output_is_deterministic_json(self):
        self.assertEqual(rm.main(self.argv()), 0)
        text = self.out.read_text(encoding="utf-8")
        self.assertTrue(text.endswith("\n"))
        self.assertEqual(text, common.dump_json(common.read_json(self.out)))


if __name__ == "__main__":
    unittest.main()
