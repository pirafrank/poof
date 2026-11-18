"""Tests for read-only registry reconciliation and idempotent publish decisions."""

from __future__ import annotations

import copy
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from _support import FakeRunner, common, completed, synthetic_staging
from npm import verify_registry as vr
from npm.common import NpmPackagingError
from npm.verify_release import Recorder

STAGING = synthetic_staging()
VERSION = STAGING["version"]
BASE = common.BASE_PACKAGE
# Captured before any test patches vr.RegistryClient, so the fake factory below
# builds a real client instead of recursing into its own patch.
REGISTRY_CLIENT = vr.RegistryClient
ABSENT_STDERR = "npm error code E404\nnpm error 404 Not Found - GET https://registry.npmjs.org/x"


def observed_for(entry, kind):
    """Build a registry version manifest matching a staged package exactly."""
    payload = {
        "name": entry["package_name"],
        "version": entry["version"],
        "dist": {
            "integrity": entry["tarball"]["sha512_integrity"],
            "shasum": "b" * 40,
            "tarball": f"{common.NPM_REGISTRY}/{entry['package_name']}/-/pkg.tgz",
        },
    }
    if kind == "platform":
        payload["os"] = [entry["os"]]
        payload["cpu"] = [entry["cpu"]]
    else:
        payload["bin"] = {"poof": "bin/poof.js"}
        payload["optionalDependencies"] = dict(entry["optional_dependencies"])
        payload["engines"] = {"node": common.NODE_ENGINE}
    return payload


def all_observed(staging=None):
    """Registry manifests for all five packages, keyed by name@version."""
    staging = staging or STAGING
    payloads = {}
    for entry in staging["platforms"]:
        payloads[entry["package_name"]] = observed_for(entry, "platform")
    payloads[BASE] = observed_for(staging["base"], "launcher")
    return {f"{name}@{VERSION}": payload for name, payload in payloads.items()}


class FakeRegistry:
    """Offline ``npm view`` double that records every invocation."""

    def __init__(self, versions=None, tags=None, failures=None):
        self.versions = versions if versions is not None else {}
        self.tags = tags if tags is not None else {}
        self.failures = failures or {}
        self.runner = FakeRunner(handler=self._handle)
        self.sleeps = []

    def client(self):
        """Return a RegistryClient bound to this fake and a recording sleep."""
        return REGISTRY_CLIENT(Recorder(self.runner), sleep=self.sleeps.append)

    def _handle(self, argv, cwd):
        if argv[:2] != ["npm", "view"]:
            raise AssertionError(f"unexpected command: {argv}")
        spec = argv[2]
        field = argv[3] if len(argv) > 3 and not argv[3].startswith("--") else None
        queue = self.failures.get(spec)
        if queue:
            code, stderr = queue.pop(0)
            return completed(argv, code, "", stderr)
        payload = self.tags.get(spec) if field == "dist-tags" else self.versions.get(spec)
        if payload is None:
            return completed(argv, 1, "", ABSENT_STDERR)
        return completed(argv, 0, json.dumps(payload), "")


class ClassifyFailureTests(unittest.TestCase):
    """A 404 is a legitimate absent state; only transport errors are retryable."""

    def test_absent_markers(self):
        for text in (
            "npm error code E404",
            "404 Not Found",
            "is not in this registry",
            "NPM ERROR CODE E404",
        ):
            with self.subTest(text=text):
                self.assertEqual(vr._classify_failure(text), "absent")

    def test_transient_markers(self):
        for text in (
            "npm error code ETIMEDOUT",
            "ECONNRESET",
            "ECONNREFUSED",
            "EAI_AGAIN",
            "ENOTFOUND",
            "EPIPE",
            "socket hang up",
            "network error",
            "502 Bad Gateway",
            "503 Service Unavailable",
            "504 Gateway Timeout",
            "429 Too Many Requests",
            "500 Internal Server Error",
        ):
            with self.subTest(text=text):
                self.assertEqual(vr._classify_failure(text), "transient")

    def test_fatal_markers(self):
        for text in ("npm error code E403", "ENEEDAUTH", "EAUTHUNKNOWN", "forbidden"):
            with self.subTest(text=text):
                self.assertEqual(vr._classify_failure(text), "fatal")

    def test_absent_wins_over_transient(self):
        self.assertEqual(vr._classify_failure("E404 after 503"), "absent")


class ViewTests(unittest.TestCase):
    """The npm view wrapper is read-only, registry-pinned, and bounded."""

    def test_returns_parsed_json(self):
        fake = FakeRegistry(versions={f"{BASE}@{VERSION}": {"version": VERSION}})
        self.assertEqual(fake.client()._view(f"{BASE}@{VERSION}"), {"version": VERSION})

    def test_absent_returns_none_without_retry(self):
        fake = FakeRegistry()
        self.assertIsNone(fake.client()._view(f"{BASE}@{VERSION}"))
        self.assertEqual(len(fake.runner.calls), 1)
        self.assertEqual(fake.sleeps, [])

    def test_empty_stdout_returns_none(self):
        fake = FakeRegistry()
        fake.runner.handler = lambda argv, cwd: completed(argv, 0, "   ", "")
        self.assertIsNone(fake.client()._view(BASE))

    def test_retries_transient_then_succeeds(self):
        fake = FakeRegistry(
            versions={f"{BASE}@{VERSION}": {"version": VERSION}},
            failures={f"{BASE}@{VERSION}": [(1, "npm error code ETIMEDOUT")]},
        )
        self.assertEqual(fake.client()._view(f"{BASE}@{VERSION}"), {"version": VERSION})
        self.assertEqual(len(fake.runner.calls), 2)
        self.assertEqual(fake.sleeps, [2])

    def test_exhausts_transient_attempts_then_fails(self):
        fake = FakeRegistry(
            failures={f"{BASE}@{VERSION}": [(1, "503 Service Unavailable")] * 4}
        )
        with self.assertRaisesRegex(NpmPackagingError, "npm view .* failed"):
            fake.client()._view(f"{BASE}@{VERSION}")
        self.assertEqual(len(fake.runner.calls), vr.TRANSIENT_ATTEMPTS)
        self.assertEqual(fake.sleeps, list(vr.TRANSIENT_BACKOFF_SECONDS))

    def test_fatal_fails_immediately_without_retry(self):
        fake = FakeRegistry(failures={f"{BASE}@{VERSION}": [(1, "npm error code E403")]})
        with self.assertRaisesRegex(NpmPackagingError, "npm view .* failed"):
            fake.client()._view(f"{BASE}@{VERSION}")
        self.assertEqual(len(fake.runner.calls), 1)
        self.assertEqual(fake.sleeps, [])

    def test_invalid_json_is_fatal(self):
        fake = FakeRegistry()
        fake.runner.handler = lambda argv, cwd: completed(argv, 0, "{not json", "")
        with self.assertRaisesRegex(NpmPackagingError, "invalid JSON"):
            fake.client()._view(BASE)

    def test_every_call_is_read_only_and_registry_pinned(self):
        fake = FakeRegistry(versions=all_observed(), tags={BASE: {"latest": VERSION}})
        vr.reconcile(fake.client(), STAGING)
        self.assertGreater(len(fake.runner.calls), 0)
        for call in fake.runner.calls:
            argv = call["argv"]
            self.assertEqual(argv[:2], ["npm", "view"])
            self.assertIn("--registry", argv)
            self.assertEqual(argv[argv.index("--registry") + 1], common.NPM_REGISTRY)
            self.assertIn("--json", argv)
            for forbidden in ("publish", "dist-tag", "adduser", "login", "unpublish"):
                self.assertNotIn(forbidden, argv)


class VersionManifestTests(unittest.TestCase):
    """Version lookups accept npm's shapes and reject ambiguity."""

    def test_dict_passthrough(self):
        fake = FakeRegistry(versions={f"{BASE}@{VERSION}": {"version": VERSION}})
        self.assertEqual(fake.client().version_manifest(BASE, VERSION), {"version": VERSION})

    def test_list_with_single_match(self):
        fake = FakeRegistry(
            versions={f"{BASE}@{VERSION}": [{"version": VERSION}, {"version": "0.0.1"}]}
        )
        self.assertEqual(fake.client().version_manifest(BASE, VERSION), {"version": VERSION})

    def test_ambiguous_list_is_rejected(self):
        fake = FakeRegistry(
            versions={f"{BASE}@{VERSION}": [{"version": VERSION}, {"version": VERSION}]}
        )
        with self.assertRaisesRegex(NpmPackagingError, "ambiguous results"):
            fake.client().version_manifest(BASE, VERSION)

    def test_list_without_match_is_rejected(self):
        fake = FakeRegistry(versions={f"{BASE}@{VERSION}": [{"version": "0.0.1"}]})
        with self.assertRaisesRegex(NpmPackagingError, "ambiguous results"):
            fake.client().version_manifest(BASE, VERSION)

    def test_scalar_shape_is_rejected(self):
        fake = FakeRegistry(versions={f"{BASE}@{VERSION}": "0.6.3"})
        with self.assertRaisesRegex(NpmPackagingError, "unexpected shape"):
            fake.client().version_manifest(BASE, VERSION)

    def test_absent_returns_none(self):
        self.assertIsNone(FakeRegistry().client().version_manifest(BASE, VERSION))


class DistTagsTests(unittest.TestCase):
    """Dist-tag reads are observations only; this tool never writes tags."""

    def test_absent_returns_empty_mapping(self):
        self.assertEqual(FakeRegistry().client().dist_tags(BASE), {})

    def test_returns_mapping(self):
        fake = FakeRegistry(tags={BASE: {"latest": VERSION, "next": "0.7.0-rc.1"}})
        self.assertEqual(fake.client().dist_tags(BASE)["latest"], VERSION)

    def test_non_mapping_is_rejected(self):
        fake = FakeRegistry(tags={BASE: ["latest"]})
        with self.assertRaisesRegex(NpmPackagingError, "unexpected shape"):
            fake.client().dist_tags(BASE)


class DistTagTransitionTests(unittest.TestCase):
    """Publication must never regress latest or point it at a prerelease."""

    def test_stable_with_no_latest_sets_latest(self):
        result = vr.check_dist_tag_transition(None, common.parse_npm_version("0.6.3"))
        self.assertEqual(result["action"], "set-latest")
        self.assertEqual(result["new_latest"], "0.6.3")
        self.assertEqual(result["dist_tag"], "latest")

    def test_stable_newer_moves_latest_forward(self):
        result = vr.check_dist_tag_transition("0.6.2", common.parse_npm_version("0.6.3"))
        self.assertEqual(result["action"], "set-latest")
        self.assertEqual(result["new_latest"], "0.6.3")

    def test_stable_equal_keeps_latest(self):
        result = vr.check_dist_tag_transition("0.6.3", common.parse_npm_version("0.6.3"))
        self.assertEqual(result["action"], "keep-latest")
        self.assertEqual(result["new_latest"], "0.6.3")

    def test_stable_older_is_refused(self):
        with self.assertRaisesRegex(NpmPackagingError, "would regress 'latest'"):
            vr.check_dist_tag_transition("0.7.0", common.parse_npm_version("0.6.3"))

    def test_prerelease_publishes_to_next_and_keeps_latest(self):
        result = vr.check_dist_tag_transition("0.6.3", common.parse_npm_version("0.6.4-rc.1"))
        self.assertEqual(result["action"], "publish-to-next")
        self.assertEqual(result["dist_tag"], "next")
        self.assertEqual(result["new_latest"], "0.6.3")
        self.assertEqual(result["current_latest"], "0.6.3")

    def test_prerelease_with_no_latest_keeps_none(self):
        result = vr.check_dist_tag_transition(None, common.parse_npm_version("0.6.4-rc.1"))
        self.assertEqual(result["action"], "publish-to-next")
        self.assertIsNone(result["new_latest"])

    def test_prerelease_preserves_bootstrap_latest(self):
        bootstrap = common.BOOTSTRAP_NPM_VERSION
        result = vr.check_dist_tag_transition(bootstrap, common.parse_npm_version("0.6.4-rc.2"))
        self.assertEqual(result["action"], "publish-to-next")
        self.assertEqual(result["current_latest"], bootstrap)
        self.assertEqual(result["new_latest"], bootstrap)

    def test_prerelease_latest_requires_human_repair(self):
        with self.assertRaisesRegex(NpmPackagingError, "already points at prerelease"):
            vr.check_dist_tag_transition("0.6.4-rc.0", common.parse_npm_version("0.6.4-rc.1"))


class CompareToStagedTests(unittest.TestCase):
    """Published bytes and metadata must match what was verified and staged."""

    def setUp(self):
        self.platform = STAGING["platforms"][0]
        self.base = STAGING["base"]
        self.expected_platform = {
            "name": self.platform["package_name"],
            "version": VERSION,
            "kind": "platform",
            "os": self.platform["os"],
            "cpu": self.platform["cpu"],
            "tarball": self.platform["tarball"],
        }
        self.expected_base = {
            "name": BASE,
            "version": VERSION,
            "kind": "launcher",
            "tarball": self.base["tarball"],
            "optional_dependencies": self.base["optional_dependencies"],
        }

    def test_matching_platform_has_no_problems(self):
        observed = observed_for(self.platform, "platform")
        self.assertEqual(
            vr.compare_to_staged(self.platform["package_name"], observed, self.expected_platform),
            [],
        )

    def test_matching_launcher_has_no_problems(self):
        observed = observed_for(self.base, "launcher")
        self.assertEqual(vr.compare_to_staged(BASE, observed, self.expected_base), [])

    def test_detects_integrity_mismatch(self):
        observed = observed_for(self.platform, "platform")
        observed["dist"]["integrity"] = "sha512-" + "A" * 88
        problems = vr.compare_to_staged("x", observed, self.expected_platform)
        self.assertIn("dist.integrity != staged tarball integrity", problems)

    def test_detects_version_mismatch(self):
        observed = observed_for(self.platform, "platform")
        observed["version"] = "0.6.4"
        self.assertTrue(vr.compare_to_staged("x", observed, self.expected_platform))

    def test_detects_os_and_cpu_mismatch(self):
        observed = observed_for(self.platform, "platform")
        observed["os"] = ["darwin"]
        observed["cpu"] = ["arm64"]
        problems = vr.compare_to_staged("x", observed, self.expected_platform)
        self.assertEqual(len(problems), 2)

    def test_platform_must_not_declare_bin(self):
        observed = observed_for(self.platform, "platform")
        observed["bin"] = {"poof": "bin/poof"}
        problems = vr.compare_to_staged("x", observed, self.expected_platform)
        self.assertIn("platform package must not declare a bin entry", problems)

    def test_launcher_bin_must_match(self):
        observed = observed_for(self.base, "launcher")
        observed["bin"] = {"poof": "bin/other.js"}
        self.assertTrue(vr.compare_to_staged(BASE, observed, self.expected_base))

    def test_launcher_pins_must_match_exactly(self):
        observed = observed_for(self.base, "launcher")
        observed["optionalDependencies"][f"{BASE}-linux-x64"] = "^0.6.3"
        problems = vr.compare_to_staged(BASE, observed, self.expected_base)
        self.assertIn("optionalDependencies do not match the exact pinned versions", problems)

    def test_launcher_engine_must_match(self):
        observed = observed_for(self.base, "launcher")
        observed["engines"] = {"node": ">=16"}
        self.assertTrue(vr.compare_to_staged(BASE, observed, self.expected_base))

    def test_lifecycle_scripts_are_rejected(self):
        observed = observed_for(self.base, "launcher")
        observed["scripts"] = {"postinstall": "node install.js"}
        problems = vr.compare_to_staged(BASE, observed, self.expected_base)
        self.assertIn("published package must not declare lifecycle scripts", problems)

    def test_shasum_compared_when_staged_has_one(self):
        observed = observed_for(self.platform, "platform")
        expected = copy.deepcopy(self.expected_platform)
        expected["tarball"] = dict(expected["tarball"], shasum="c" * 40)
        problems = vr.compare_to_staged("x", observed, expected)
        self.assertIn("dist.shasum != staged tarball shasum", problems)


class ReconcileTests(unittest.TestCase):
    """Reconciliation decides publish/skip/stop without mutating the registry."""

    def test_all_absent_plans_full_publish_order(self):
        fake = FakeRegistry()
        result = vr.reconcile(fake.client(), STAGING)
        self.assertFalse(result["all_published"])
        self.assertEqual(len(result["publish_order"]), 5)
        self.assertEqual(result["publish_order"][-1], BASE)
        self.assertEqual(result["observations"], {})
        for decision in result["decisions"].values():
            self.assertEqual(decision["state"], "absent")
            self.assertEqual(decision["action"], "publish")

    def test_all_matching_plans_nothing(self):
        fake = FakeRegistry(versions=all_observed(), tags={BASE: {"latest": VERSION}})
        result = vr.reconcile(fake.client(), STAGING)
        self.assertTrue(result["all_published"])
        self.assertEqual(result["publish_order"], [])
        self.assertEqual(len(result["observations"]), 5)
        for decision in result["decisions"].values():
            self.assertEqual(decision["state"], "match")
            self.assertEqual(decision["action"], "skip")

    def test_partial_publish_plans_only_missing_packages(self):
        versions = all_observed()
        missing = STAGING["platforms"][0]["package_name"]
        del versions[f"{missing}@{VERSION}"]
        result = vr.reconcile(FakeRegistry(versions=versions).client(), STAGING)
        self.assertEqual(result["publish_order"], [missing])
        self.assertFalse(result["all_published"])

    def test_byte_mismatch_is_a_hard_stop(self):
        versions = all_observed()
        name = STAGING["platforms"][0]["package_name"]
        versions[f"{name}@{VERSION}"]["dist"]["integrity"] = "sha512-" + "A" * 88
        with self.assertRaisesRegex(NpmPackagingError, "refusing to continue"):
            vr.reconcile(FakeRegistry(versions=versions).client(), STAGING)

    def test_mismatch_message_names_the_package(self):
        versions = all_observed()
        name = STAGING["platforms"][1]["package_name"]
        versions[f"{name}@{VERSION}"]["dist"]["integrity"] = "sha512-" + "A" * 88
        with self.assertRaisesRegex(NpmPackagingError, name):
            vr.reconcile(FakeRegistry(versions=versions).client(), STAGING)

    def test_require_published_rejects_absent_packages(self):
        with self.assertRaisesRegex(NpmPackagingError, "Expected all five packages published"):
            vr.reconcile(FakeRegistry().client(), STAGING, require_published=True)

    def test_require_published_accepts_matching_registry(self):
        fake = FakeRegistry(versions=all_observed(), tags={BASE: {"latest": VERSION}})
        result = vr.reconcile(fake.client(), STAGING, require_published=True)
        self.assertTrue(result["all_published"])

    def test_prerelease_plans_next_without_touching_latest(self):
        staging = synthetic_staging("0.6.4-rc.1")
        fake = FakeRegistry(tags={BASE: {"latest": "0.6.3"}})
        result = vr.reconcile(fake.client(), staging)
        self.assertEqual(result["dist_tag"], "next")
        self.assertEqual(result["dist_tag_transition"]["action"], "publish-to-next")
        self.assertEqual(result["dist_tag_transition"]["new_latest"], "0.6.3")

    def test_latest_regression_is_refused(self):
        fake = FakeRegistry(tags={BASE: {"latest": "0.7.0"}})
        with self.assertRaisesRegex(NpmPackagingError, "would regress 'latest'"):
            vr.reconcile(fake.client(), STAGING)

    def test_report_shape(self):
        result = vr.reconcile(FakeRegistry().client(), STAGING)
        for key in (
            "schema_version", "registry", "version", "tag", "dist_tag", "observed_at",
            "observations", "decisions", "dist_tags", "dist_tag_transition",
            "publish_order", "all_published",
        ):
            self.assertIn(key, result)
        self.assertEqual(result["registry"], common.NPM_REGISTRY)
        self.assertEqual(result["tag"], "v0.6.3")


class CliTests(unittest.TestCase):
    """The CLI writes a report and fails closed."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self._tmp.cleanup)
        self.dir = Path(self._tmp.name)
        self.staging = self.dir / "staging.json"
        self.out = self.dir / "registry.json"
        common.write_json(self.staging, STAGING)

    def run_main(self, fake, *extra):
        with mock.patch.object(vr, "RegistryClient", lambda *a, **k: fake.client()):
            return vr.main(["--staging", str(self.staging), "--out", str(self.out), *extra])

    def test_writes_report(self):
        self.assertEqual(self.run_main(FakeRegistry()), 0)
        report = common.read_json(self.out)
        self.assertEqual(len(report["publish_order"]), 5)

    def test_require_published_flag_fails_when_absent(self):
        self.assertEqual(self.run_main(FakeRegistry(), "--require-published"), 1)
        self.assertFalse(self.out.exists())

    def test_missing_staging_file_fails(self):
        self.staging.unlink()
        self.assertEqual(self.run_main(FakeRegistry()), 1)

    def test_mismatch_fails_without_writing(self):
        versions = all_observed()
        name = STAGING["platforms"][0]["package_name"]
        versions[f"{name}@{VERSION}"]["dist"]["integrity"] = "sha512-" + "A" * 88
        self.assertEqual(self.run_main(FakeRegistry(versions=versions)), 1)
        self.assertFalse(self.out.exists())


if __name__ == "__main__":
    unittest.main()
