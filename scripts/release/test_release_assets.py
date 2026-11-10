"""Exercise release assembly against missing, duplicate, and modified inputs."""

import json
import os
import shutil
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from jsonschema import Draft202012Validator

from build_environment import CROSS_VERSION, collect, prepare_cross
from finalize_assets import finalize, sha256


class ReleaseAssetsTests(unittest.TestCase):
    """Check the finalizer's manifest and input validation."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.root)
        self.source = self.root / "incoming"
        self.source.mkdir()
        self.output = self.root / "artifacts"
        self.matrix = {"include": [{"target": "example-target"}]}
        self.base = "poof-0.6.3-example-target"
        archive = self.source / (self.base + ".tar.gz")
        archive.write_bytes(b"archive fixture")
        (self.source / (self.base + ".build-environment.json")).write_text(
            json.dumps(
                {
                    "artifact": {
                        "name": archive.name,
                        "sha256": sha256(archive),
                        "size_bytes": archive.stat().st_size,
                    }
                }
            )
        )
        (self.source / (self.base + ".spdx.json")).write_text("{}")
        (self.source / "poof.1").write_text("man page")

    def assemble(self):
        """Run finalization with the standard fixture inputs."""
        finalize(self.source, self.output, "0.6.3", self.matrix)

    def test_manifest_is_sorted_and_covers_all_inputs(self):
        """The manifest is sorted and contains the digest of every expected asset."""
        self.assemble()
        lines = (self.output / "poof-0.6.3-checksums.txt").read_text().splitlines()
        names = [line.split("  ")[1] for line in lines]
        self.assertEqual(names, sorted(path.name for path in self.source.iterdir()))
        for line in lines:
            checksum, name = line.split("  ")
            self.assertEqual(checksum, sha256(self.output / name))

    def test_modified_archive_is_rejected(self):
        """Reject a build record whose archive digest or size does not match."""
        (self.source / (self.base + ".tar.gz")).write_bytes(b"substitution")
        with self.assertRaisesRegex(ValueError, "does not match"):
            self.assemble()

    def test_duplicate_basename_is_rejected(self):
        """Reject different source paths with the same release filename."""
        duplicate = self.source / "duplicate"
        duplicate.mkdir()
        (duplicate / "poof.1").write_text("replacement")
        with self.assertRaisesRegex(ValueError, "unique"):
            self.assemble()

    def test_missing_sbom_is_rejected(self):
        """Reject an incomplete release asset set."""
        (self.source / (self.base + ".spdx.json")).unlink()
        with self.assertRaisesRegex(ValueError, "missing"):
            self.assemble()


class EnvironmentTests(unittest.TestCase):
    """Check metadata collection and Cross image pinning."""

    def test_collected_cargo_and_cross_records_match_schema(self):
        """Validate collector output for native and Cross builds against the schema.
        Catch drifts between the collector and schema in CI"""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old_cwd = Path.cwd()
            self.addCleanup(os.chdir, old_cwd)
            os.chdir(root)
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text("")
            (root / ".github/workflows").mkdir(parents=True)
            (root / ".github/workflows/release.yml").write_text("workflow fixture")
            (root / "Cargo.toml").write_text('[package]\nname="poof"\nversion="1.0.0"')
            (root / "Cargo.lock").write_text("lock fixture")
            (root / "build.rs").write_text("fn main() {}")
            archive = root / "poof.tar.gz"
            archive.write_bytes(b"archive fixture")

            cargo_target = "x86_64-unknown-linux-musl"
            cross_target = "x86_64-unknown-linux-gnu"
            cargo_entry = {
                "os": "ubuntu-24.04",
                "target": cargo_target,
                "builder": "cargo",
            }
            cross_entry = {
                "os": "ubuntu-24.04",
                "target": cross_target,
                "builder": "cross",
            }
            (root / "matrix.jsonc").write_text(
                json.dumps({"include": [cargo_entry, cross_entry]})
            )
            repository = f"ghcr.io/cross-rs/{cross_target}"
            resolved_digest = repository + "@sha256:" + "a" * 64
            (root / "Cross.toml").write_text(
                f'[target."{cross_target}"]\nimage="{repository}:{CROSS_VERSION}"\n'
            )

            metadata = {
                "packages": [
                    {
                        "id": "poof 1.0.0",
                        "name": "poof",
                        "version": "1.0.0",
                        "manifest_path": str(root / "Cargo.toml"),
                    }
                ],
                "workspace_members": ["poof 1.0.0"],
                "resolve": {"nodes": [{"id": "poof 1.0.0", "features": []}]},
            }

            def probe(*args, required=False):
                if args[:2] == ("cargo", "metadata"):
                    return json.dumps(metadata)
                if args[:3] == ("docker", "image", "inspect"):
                    return json.dumps(
                        [
                            {
                                "RepoDigests": [resolved_digest],
                                "Os": "linux",
                                "Architecture": "amd64",
                                "Config": {"Env": []},
                            }
                        ]
                    )
                if args[:2] == ("docker", "version"):
                    return "{}"
                return "fixture"

            schema_path = Path(__file__).with_name("build-environment.schema.json")
            validator = Draft202012Validator(json.loads(schema_path.read_text()))
            env_path = root / "github-env"
            with (
                patch.dict(
                    os.environ,
                    {
                        "RUNNER_OS": "Linux",
                        "RUNNER_ARCH": "X64",
                        "GITHUB_ENV": str(env_path),
                    },
                ),
                patch("build_environment.command", side_effect=probe),
            ):
                for builder, target, matrix_entry in (
                    ("cargo", cargo_target, cargo_entry),
                    ("cross", cross_target, cross_entry),
                ):
                    if builder == "cross":
                        prepare_cross(target)
                    collect(
                        SimpleNamespace(
                            archive=str(archive),
                            target=target,
                            runner="ubuntu-24.04",
                            builder=builder,
                            matrix=json.dumps(matrix_entry),
                            output="record.json",
                        )
                    )
                    with self.subTest(builder=builder):
                        validator.validate(json.loads((root / "record.json").read_text()))

    def test_collector_allowlists_and_redacts_environment(self):
        """Record build metadata without writing secret environment values."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old_cwd = Path.cwd()
            self.addCleanup(os.chdir, old_cwd)
            os.chdir(root)
            (root / ".cargo").mkdir()
            (root / ".cargo/config.toml").write_text("")
            (root / "Cargo.toml").write_text('[package]\nname="poof"\nversion="0.6.3"')
            (root / "archive.tar.gz").write_bytes(b"archive")
            metadata = {
                "packages": [
                    {
                        "id": "poof",
                        "name": "poof",
                        "version": "0.6.3",
                        "manifest_path": str(root / "Cargo.toml"),
                    }
                ],
                "workspace_members": ["poof"],
                "resolve": {"nodes": []},
            }

            def probe(*args, required=False):
                if args[:2] == ("cargo", "metadata"):
                    return json.dumps(metadata)
                if args[:3] == ("git", "status", "--porcelain"):
                    return ""
                if required:
                    return "fixture"
                return "fixture"

            args = SimpleNamespace(
                archive="archive.tar.gz",
                target="example-target",
                runner="ubuntu-24.04",
                builder="cargo",
                matrix="{}",
                output="record.json",
            )
            with (
                patch.dict(
                    os.environ,
                    {"RUSTFLAGS": "private-flag-value", "REGISTRY_TOKEN": "private-token-value"},
                ),
                patch("build_environment.command", side_effect=probe),
            ):
                collect(args)
            text = (root / "record.json").read_text()
            record = json.loads(text)
            self.assertNotIn("private-flag-value", text)
            self.assertNotIn("private-token-value", text)
            self.assertEqual(record["build"]["flags"]["RUSTFLAGS"]["value"], "[REDACTED]")
            self.assertEqual(record["artifact"]["sha256"], sha256(root / "archive.tar.gz"))

    def test_cross_digest_is_passed_to_actual_build(self):
        """Pass the inspected Cross image digest to the later build step."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old_cwd = Path.cwd()
            self.addCleanup(os.chdir, old_cwd)
            os.chdir(root)
            target = "x86_64-unknown-linux-gnu"
            repository = "ghcr.io/cross-rs/" + target
            (root / "matrix.jsonc").write_text(
                json.dumps({"include": [{"target": target, "builder": "cross"}]})
            )
            (root / "Cross.toml").write_text(
                f'[target."{target}"]\nimage="{repository}:{CROSS_VERSION}"'
            )
            resolved = repository + "@sha256:" + "a" * 64

            def probe(*args, required=False):
                if args[:3] == ("docker", "image", "inspect"):
                    return json.dumps(
                        [
                            {
                                "RepoDigests": [resolved],
                                "Os": "linux",
                                "Architecture": "amd64",
                                "Config": {"Env": []},
                            }
                        ]
                    )
                if args[:2] == ("docker", "version"):
                    return "{}"
                if required:
                    return "version fixture"
                return "version fixture"

            with (
                patch.dict(os.environ, {"GITHUB_ENV": str(root / "job-env")}),
                patch.dict(os.environ, {"RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}),
                patch("build_environment.command", side_effect=probe),
            ):
                prepare_cross(target)
            self.assertEqual(
                (root / "job-env").read_text(), f"CROSS_TARGET_X86_64_UNKNOWN_LINUX_GNU_IMAGE={resolved}\n"
            )
            record = json.loads((root / "_temp/cross-environment.json").read_text())
            self.assertEqual(record["image_digest"], resolved)
            self.assertTrue(record["verification"]["verified"])
            self.assertEqual(
                record["verification"]["expected_platform"],
                {"os": "linux", "architecture": "amd64"},
            )

    def test_cross_rejects_invalid_image_identity_and_target(self):
        """Reject unknown targets and images outside the exact target repository."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old_cwd = Path.cwd()
            self.addCleanup(os.chdir, old_cwd)
            os.chdir(root)
            target = "x86_64-unknown-linux-gnu"
            (root / "matrix.jsonc").write_text(json.dumps({"include": [
                {"target": target, "builder": "cross"}
            ]}))
            (root / "Cross.toml").write_text(
                f'[target."{target}"]\nimage="ghcr.io/other/{target}:tag"'
            )
            with patch.dict(os.environ, {"RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}):
                with self.assertRaisesRegex(ValueError, "repository"):
                    prepare_cross(target)
            with self.assertRaisesRegex(ValueError, "Unsupported Cross target"):
                prepare_cross("../../invalid")

    def test_cross_rejects_missing_matching_digest(self):
        """Require an explicit digest for the exact configured repository."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old_cwd = Path.cwd()
            self.addCleanup(os.chdir, old_cwd)
            os.chdir(root)
            target = "x86_64-unknown-linux-gnu"
            repository = "ghcr.io/cross-rs/" + target
            (root / "matrix.jsonc").write_text(json.dumps({"include": [
                {"target": target, "builder": "cross"}
            ]}))
            (root / "Cross.toml").write_text(
                f'[target."{target}"]\nimage="{repository}:tag"'
            )

            def probe(*args, required=False):
                if args[:3] == ("docker", "image", "inspect"):
                    return json.dumps([{
                        "RepoDigests": ["ghcr.io/cross-rs/unrelated@sha256:" + "a" * 64],
                        "Os": "linux", "Architecture": "amd64", "Config": {"Env": []},
                    }])
                if required:
                    return "{}"
                return "fixture"

            with (
                patch.dict(os.environ, {"RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}),
                patch("build_environment.command", side_effect=probe),
            ):
                with self.assertRaisesRegex(ValueError, "exactly one resolved digest"):
                    prepare_cross(target)
            self.assertFalse((root / "job-env").exists())

    def test_cross_platform_verification_failure_stops_before_override(self):
        """Reject a digest-resolved image whose platform does not match its target."""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            old_cwd = Path.cwd()
            self.addCleanup(os.chdir, old_cwd)
            os.chdir(root)
            target = "x86_64-unknown-linux-gnu"
            repository = "ghcr.io/cross-rs/" + target
            (root / "matrix.jsonc").write_text(json.dumps({"include": [
                {"target": target, "builder": "cross"}
            ]}))
            (root / "Cross.toml").write_text(
                f'[target."{target}"]\nimage="{repository}:tag"'
            )
            resolved = repository + "@sha256:" + "b" * 64

            def probe(*args, required=False):
                if args[:3] == ("docker", "image", "inspect"):
                    return json.dumps([{
                        "RepoDigests": [resolved], "Os": "linux", "Architecture": "arm64",
                        "Config": {"Env": []},
                    }])
                if required:
                    return "{}"
                return "fixture"

            with (
                patch.dict(os.environ, {"RUNNER_OS": "Linux", "RUNNER_ARCH": "X64"}),
                patch("build_environment.command", side_effect=probe),
            ):
                with self.assertRaisesRegex(ValueError, "platform mismatch"):
                    prepare_cross(target)
            self.assertFalse((root / "job-env").exists())


if __name__ == "__main__":
    unittest.main()
