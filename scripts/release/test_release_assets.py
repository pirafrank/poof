"""Exercise release assembly against missing, duplicate, and modified inputs."""

import json
import os
import shutil
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from build_environment import collect, prepare_cross
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
            (root / "Cross.toml").write_text('[target.example-target]\nimage="image:tag"')
            resolved = "image@sha256:" + "a" * 64

            def probe(*args, required=False):
                if args[:3] == ("docker", "image", "inspect"):
                    return json.dumps(
                        [
                            {
                                "RepoDigests": [resolved],
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
                patch("build_environment.command", side_effect=probe),
            ):
                prepare_cross("example-target")
            self.assertEqual(
                (root / "job-env").read_text(), f"CROSS_TARGET_EXAMPLE_TARGET_IMAGE={resolved}\n"
            )
            self.assertEqual(
                json.loads((root / "_temp/cross-environment.json").read_text())["image_digest"],
                resolved,
            )


if __name__ == "__main__":
    unittest.main()
