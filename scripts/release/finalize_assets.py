#!/usr/bin/env python3
"""Assemble a complete release without silently overwriting duplicate filenames."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil


def sha256(path):
    """Return the SHA-256 digest of a file."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def finalize(source, destination, version, matrix):
    """Assemble release assets and write their checksums."""
    destination.mkdir(parents=True, exist_ok=False)
    expected = {"poof.1"}
    for entry in matrix["include"]:
        base = f"poof-{version}-{entry['target']}"
        expected.update(
            base + suffix for suffix in (".tar.gz", ".spdx.json", ".build-environment.json")
        )
    files = list(source.rglob("*"))
    files = [path for path in files if path.is_file()]
    names = [path.name for path in files]
    if len(names) != len(set(names)) or set(names) != expected:
        raise ValueError(
            f"Release assets differ: missing={expected - set(names)}, "
            f"unexpected={set(names) - expected}; filenames must be unique"
        )
    for path in files:
        shutil.copyfile(path, destination / path.name)
    for record_path in destination.glob("*.build-environment.json"):
        record = json.loads(record_path.read_text())
        archive = destination / record["artifact"]["name"]
        if (
            record["artifact"]["sha256"] != sha256(archive)
            or record["artifact"]["size_bytes"] != archive.stat().st_size
        ):
            raise ValueError(f"Build record does not match archive: {record_path}")
    manifest = destination / f"poof-{version}-checksums.txt"
    manifest.write_text(
        "".join(f"{sha256(destination / name)}  {name}\n" for name in sorted(expected))
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--matrix", required=True)
    args = parser.parse_args()
    finalize(args.source, args.destination, args.version, json.loads(args.matrix))
