"""Tests for version parsing, matrix mapping, and shared helpers."""

from __future__ import annotations

import base64
import copy
import json
import re
import unittest

from _support import REPO_ROOT, common
from npm.common import NpmPackagingError

MATRIX_PATH = REPO_ROOT / "matrix.jsonc"
WORKFLOW_PATH = REPO_ROOT / ".github" / "workflows" / "npm.yml"


def mutated_matrix(change):
    """Load the real matrix, apply a mutation, and return the result."""
    matrix = copy.deepcopy(common.load_matrix(MATRIX_PATH))
    change(matrix)
    return matrix


def entry_for(matrix, triple):
    """Return the matrix entry for a Rust triple."""
    return next(entry for entry in matrix["include"] if entry["target"] == triple)


class VersionTests(unittest.TestCase):
    """Strict SemVer parsing and the tag/Cargo/npm version invariant."""

    def test_accepts_stable_and_prerelease_tags(self):
        for tag, text, prerelease in (
            ("v0.6.3", "0.6.3", None),
            ("v1.0.0", "1.0.0", None),
            ("v0.6.4-rc.1", "0.6.4-rc.1", "rc.1"),
            ("v2.0.0-alpha.1", "2.0.0-alpha.1", "alpha.1"),
            ("v1.2.3-0.3.7", "1.2.3-0.3.7", "0.3.7"),
            ("v1.2.3-x.7.z.92", "1.2.3-x.7.z.92", "x.7.z.92"),
        ):
            with self.subTest(tag=tag):
                version = common.parse_tag(tag)
                self.assertEqual(version.text, text)
                self.assertEqual(version.prerelease, prerelease)
                self.assertEqual(version.tag, tag)

    def test_dist_tag_mapping(self):
        self.assertEqual(common.parse_tag("v0.6.3").dist_tag, "latest")
        self.assertEqual(common.parse_tag("v0.6.4-rc.1").dist_tag, "next")
        self.assertFalse(common.parse_tag("v0.6.3").is_prerelease)
        self.assertTrue(common.parse_tag("v0.6.4-rc.1").is_prerelease)

    def test_rejects_invalid_versions(self):
        for bad in (
            "0.6.3",  # npm version must not carry the leading v
            "v0.6",
            "v0.6.3+build",
            "v0.0.0",
            "v0.0.0-bootstrap.0",
            "v1.2.3-REPLACE-ME",
            "v01.2.3",
            "v1.02.3",
            "v1.2.03",
            "main",
            "",
            "v1.2.3-",
            "v1.2.3-01",
            "v1.2.3+meta",
            "v1.2.3-rc.1+meta",
        ):
            with self.subTest(bad=bad):
                with self.assertRaises(NpmPackagingError):
                    common.parse_tag(bad)

    def test_rejects_non_string_and_empty(self):
        for bad in (None, 123, "", "v"):
            with self.subTest(bad=bad):
                with self.assertRaises(NpmPackagingError):
                    common.parse_tag(bad)

    def test_seed_prerelease_is_valid_but_zero_zero_zero_is_not(self):
        self.assertEqual(common.parse_npm_version("0.0.1-bootstrap.0").text, "0.0.1-bootstrap.0")
        with self.assertRaises(NpmPackagingError):
            common.parse_npm_version("0.0.0-bootstrap.0")

    def test_semver_precedence(self):
        ordered = [
            "1.0.0-alpha",
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0-beta",
            "1.0.0-beta.2",
            "1.0.0-beta.11",
            "1.0.0-rc.1",
            "1.0.0",
            "1.0.1",
            "1.1.0",
            "2.0.0",
        ]
        for lower, higher in zip(ordered, ordered[1:]):
            with self.subTest(lower=lower, higher=higher):
                self.assertEqual(common.compare_versions(lower, higher), -1)
                self.assertEqual(common.compare_versions(higher, lower), 1)
        self.assertEqual(common.compare_versions("1.0.0", "1.0.0"), 0)
        self.assertEqual(common.compare_versions("0.6.10", "0.6.9"), 1)
        self.assertEqual(common.compare_versions("0.6.3-rc.2", "0.6.3-rc.10"), -1)

    def test_cargo_version_from_text(self):
        self.assertEqual(
            common.cargo_version_from_text('[package]\nname = "poof"\nversion = "0.6.3"\n'),
            "0.6.3",
        )

    def test_cargo_version_rejects_missing_and_invalid(self):
        for text in ('[package]\nname = "poof"\n', "[package\nversion=", '[package]\nversion = 3\n'):
            with self.subTest(text=text):
                with self.assertRaises(NpmPackagingError):
                    common.cargo_version_from_text(text)

    def test_cargo_version_rejects_workspace_placeholder(self):
        with self.assertRaises(NpmPackagingError):
            common.cargo_version_from_text('[package]\nversion = "0.0.0"\n')


class MatrixTests(unittest.TestCase):
    """The Rust triple is the single source of npm target truth."""

    def test_real_matrix_yields_four_canonical_targets(self):
        targets = common.load_npm_targets(MATRIX_PATH)
        self.assertEqual(len(targets), 4)
        self.assertEqual(
            [target.platform.rust_target for target in targets],
            list(common.CANONICAL_TRIPLE_ORDER),
        )
        self.assertEqual(
            [target.platform.package_name for target in targets],
            [
                "@pirafrank/poof-linux-x64",
                "@pirafrank/poof-linux-arm64",
                "@pirafrank/poof-darwin-x64",
                "@pirafrank/poof-darwin-arm64",
            ],
        )
        for target in targets:
            self.assertEqual(target.builder, "cargo")
            self.assertTrue(target.runner)

    def test_native_readers_still_parse_the_matrix(self):
        # release.yml uses `grep -v '//' | jq`; build_environment.py drops lines
        # whose stripped form starts with '//'. Both must agree with load_matrix.
        text = MATRIX_PATH.read_text(encoding="utf-8")
        grep_style = json.loads("\n".join(line for line in text.splitlines() if "//" not in line))
        self.assertEqual(grep_style, common.load_matrix(MATRIX_PATH))
        self.assertEqual(len(grep_style["include"]), 16)

    def test_workflow_publish_matrix_matches_the_derived_targets(self):
        # npm.yml hand-copies the four platform entries because a workflow
        # matrix cannot be computed from matrix.jsonc at job-expansion time.
        found = re.findall(
            r"- os: (\S+)\s+target: (\S+)\s+builder: (\S+)\s+"
            r"npm: \{ name: '([^']+)', os: (\w+), cpu: (\w+) \}",
            WORKFLOW_PATH.read_text(encoding="utf-8"),
        )
        self.assertEqual(
            found,
            [
                (
                    target.runner,
                    target.platform.rust_target,
                    target.builder,
                    target.platform.package_name,
                    target.platform.npm_os,
                    target.platform.npm_cpu,
                )
                for target in common.load_npm_targets(MATRIX_PATH)
            ],
        )

    def test_publish_workflow_uses_oidc_without_npm_tokens(self):
        workflow = WORKFLOW_PATH.read_text(encoding="utf-8")
        self.assertNotIn("NODE_AUTH_TOKEN", workflow)
        self.assertNotIn("NPM_TOKEN", workflow)
        self.assertNotIn("registry-url", workflow)
        self.assertEqual(workflow.count("id-token: write"), 2)
        self.assertEqual(workflow.count("environment: publish-npm"), 2)
        self.assertEqual(workflow.count("npm publish \"$TARBALL\" --provenance"), 2)

    def test_matrix_carries_no_npm_specific_entries(self):
        matrix = common.load_matrix(MATRIX_PATH)
        self.assertEqual([entry for entry in matrix["include"] if "npm" in entry], [])

    def test_targets_are_derived_from_the_rust_triple(self):
        for target in common.load_npm_targets(MATRIX_PATH):
            platform = target.platform
            with self.subTest(triple=platform.rust_target):
                self.assertEqual(
                    (platform.npm_os, platform.npm_cpu),
                    common.SUPPORTED_TRIPLES[platform.rust_target],
                )
                self.assertEqual(
                    platform.package_name,
                    f"@pirafrank/poof-{platform.npm_os}-{platform.npm_cpu}",
                )

    def test_derived_asset_names_use_bare_version(self):
        target = common.load_npm_targets(MATRIX_PATH)[0]
        platform = target.platform
        self.assertEqual(
            platform.archive_name("0.6.3"), "poof-0.6.3-x86_64-unknown-linux-musl.tar.gz"
        )
        self.assertEqual(
            platform.spdx_name("0.6.3"), "poof-0.6.3-x86_64-unknown-linux-musl.spdx.json"
        )
        self.assertEqual(
            platform.build_environment_name("0.6.3"),
            "poof-0.6.3-x86_64-unknown-linux-musl.build-environment.json",
        )
        self.assertEqual(platform.package_dirname, "poof-linux-x64")

    def test_manifest_and_bundle_names(self):
        self.assertEqual(common.checksums_manifest_name("0.6.3"), "poof-0.6.3-checksums.txt")
        self.assertEqual(
            common.sigstore_bundle_name("0.6.3"), "poof-0.6.3-checksums.txt.sigstore.json"
        )
        self.assertEqual(
            common.signer_identity("v0.6.3"),
            "https://github.com/pirafrank/poof/.github/workflows/release.yml@refs/tags/v0.6.3",
        )

    def test_missing_supported_triple_is_rejected(self):
        def change(matrix):
            matrix["include"] = [
                entry
                for entry in matrix["include"]
                if entry["target"] != "x86_64-unknown-linux-musl"
            ]

        with self.assertRaisesRegex(NpmPackagingError, "Missing npm target entries"):
            common.npm_targets(mutated_matrix(change))

    def test_npm_key_in_the_matrix_is_rejected(self):
        def change(matrix):
            entry_for(matrix, "x86_64-unknown-linux-musl")["npm"] = {
                "name": "@pirafrank/poof-linux-x64",
                "os": "linux",
                "cpu": "x64",
            }

        with self.assertRaisesRegex(NpmPackagingError, "must not be configured in matrix.jsonc"):
            common.npm_targets(mutated_matrix(change))

    def test_npm_key_on_an_unsupported_triple_is_also_rejected(self):
        def change(matrix):
            entry_for(matrix, "i686-unknown-linux-gnu")["npm"] = {"os": "linux", "cpu": "ia32"}

        with self.assertRaisesRegex(NpmPackagingError, "must not be configured in matrix.jsonc"):
            common.npm_targets(mutated_matrix(change))

    def test_duplicate_target_entry_is_rejected(self):
        def change(matrix):
            entry = entry_for(matrix, "x86_64-unknown-linux-musl")
            matrix["include"].append(copy.deepcopy(entry))

        with self.assertRaisesRegex(NpmPackagingError, "Duplicate matrix entry"):
            common.npm_targets(mutated_matrix(change))

    def test_invalid_builder_and_missing_runner_are_rejected(self):
        def bad_builder(matrix):
            entry_for(matrix, "x86_64-unknown-linux-musl")["builder"] = "make"

        def missing_runner(matrix):
            del entry_for(matrix, "x86_64-unknown-linux-musl")["os"]

        for change, pattern in ((bad_builder, "Invalid builder"), (missing_runner, "Invalid runner")):
            with self.subTest(pattern=pattern):
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    common.npm_targets(mutated_matrix(change))

    def test_inline_comment_is_rejected(self):
        with self.assertRaisesRegex(NpmPackagingError, "inline '//' comment"):
            common.strip_jsonc_comments('{\n  "url": "https://example.com" // note\n}\n')

    def test_whole_line_comments_are_stripped(self):
        text = '// header\n{\n  // inner\n  "include": []\n}\n'
        self.assertEqual(json.loads(common.strip_jsonc_comments(text)), {"include": []})

    def test_load_matrix_rejects_bad_shapes(self):
        import tempfile
        from pathlib import Path

        with tempfile.TemporaryDirectory() as tmp:
            for content in ('{"include": []}', '{"include": {}}', "{}", "[1,2]", "not json"):
                path = Path(tmp) / "matrix.jsonc"
                path.write_text(content, encoding="utf-8")
                with self.subTest(content=content):
                    with self.assertRaises(NpmPackagingError):
                        common.load_matrix(path)


class HelperTests(unittest.TestCase):
    """Digest, base64, redaction, and placeholder helpers."""

    def test_digests_match_known_answer_vectors(self):
        # Published SHA-256 test vector for b"abc" (FIPS 180-4 example).
        self.assertEqual(
            common.sha256_bytes(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        )
        integrity = common.sha512_integrity_bytes(b"abc")
        self.assertTrue(integrity.startswith("sha512-"))
        self.assertEqual(len(base64.b64decode(integrity[len("sha512-") :])), 64)

    def test_file_and_bytes_digests_agree(self):
        import tempfile
        from pathlib import Path

        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "blob"
            path.write_bytes(b"payload" * 1000)
            self.assertEqual(common.sha256_file(path), common.sha256_bytes(path.read_bytes()))
            self.assertEqual(
                common.sha512_integrity_file(path),
                common.sha512_integrity_bytes(path.read_bytes()),
            )

    def test_decode_base64_handles_github_line_wrapping(self):
        raw = b'[package]\nversion = "0.6.3"\n' * 20
        encoded = base64.b64encode(raw).decode("ascii")
        wrapped = "\n".join(encoded[i : i + 60] for i in range(0, len(encoded), 60))
        self.assertIn("\n", wrapped)
        self.assertEqual(common.decode_base64(wrapped, "Cargo.toml"), raw)

    def test_decode_base64_rejects_invalid_input(self):
        for bad in ("!!!!", "abc", None, 12):
            with self.subTest(bad=bad):
                with self.assertRaises(NpmPackagingError):
                    common.decode_base64(bad, "payload")

    def test_redact_removes_credentials(self):
        text = (
            "token=ghp_" + "a" * 30 + " npm_" + "b" * 30 + " Bearer " + "c" * 30
        )
        redacted = common.redact(text)
        self.assertNotIn("ghp_", redacted)
        self.assertNotIn("npm_", redacted)
        self.assertIn("<redacted>", redacted)

    def test_redact_removes_private_keys(self):
        text = "-----BEGIN RSA PRIVATE KEY-----\nMIIE\n-----END RSA PRIVATE KEY-----"
        self.assertEqual(common.redact(text), "<redacted-private-key>")

    def test_ensure_no_placeholders(self):
        common.ensure_no_placeholders({"a": ["real value", 1, None]}, "test")
        for bad in ({"a": "REPLACE-ME"}, {"a": ["TODO"]}, {"a": "placeholder"}):
            with self.subTest(bad=bad):
                with self.assertRaises(NpmPackagingError):
                    common.ensure_no_placeholders(bad, "test")

    def test_ensure_no_placeholders_rejects_credentials(self):
        with self.assertRaisesRegex(NpmPackagingError, "Credential-shaped"):
            common.ensure_no_placeholders({"token": "npm_" + "z" * 30}, "test")

    def test_json_roundtrip_is_deterministic(self):
        import tempfile
        from pathlib import Path

        value = {"b": 1, "a": {"z": 2, "y": 3}}
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "nested" / "out.json"
            common.write_json(path, value)
            text = path.read_text(encoding="utf-8")
            self.assertTrue(text.endswith("\n"))
            self.assertEqual(text, common.dump_json(value))
            self.assertEqual(common.read_json(path), value)
            self.assertLess(text.index('"a"'), text.index('"b"'))

    def test_host_fingerprint_shape(self):
        fingerprint = common.host_fingerprint()
        self.assertEqual(set(fingerprint), {"system", "machine", "python_version"})


if __name__ == "__main__":
    unittest.main()
