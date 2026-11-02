#!/usr/bin/env python3
"""Collect allowlisted release build information; never serialize the environment."""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess
import tomllib


def command(*args, required=False):
    """Return stdout, or null for an unavailable optional probe."""
    try:
        result = subprocess.run(args, capture_output=True, text=True, check=True)
        return result.stdout.strip()
    except (OSError, subprocess.CalledProcessError):
        if required:
            raise
        return None


def digest(path):
    """Return a file's SHA-256 digest, or None when it does not exist."""
    path = Path(path)
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.is_file() else None


def hashed_text(value):
    """Return the SHA-256 digest of a string."""
    return hashlib.sha256(value.encode()).hexdigest()


def write(path, data):
    """Write JSON data as UTF-8, creating parent directories when needed."""
    Path(path).parent.mkdir(parents=True, exist_ok=True)
    Path(path).write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def native_tools(prefix=()):
    """Collect version strings for available native compiler and linker tools."""
    probes = {
        "cc": ["cc", "--version"],
        "gcc": ["gcc", "--version"],
        "clang": ["clang", "--version"],
        "ld": ["ld", "--version"],
        "ar": ["ar", "--version"],
        "musl_gcc": ["musl-gcc", "--version"],
    }
    return {key: command(*prefix, *value) for key, value in probes.items()}


def prepare_cross(target):
    """Pin the Cross image to its resolved digest and record container details."""
    config = tomllib.loads(Path("Cross.toml").read_text(encoding="utf-8"))
    image = (
        config.get("target", {}).get(target, {}).get("image", f"ghcr.io/cross-rs/{target}:0.2.5")
    )
    if not isinstance(image, str):
        raise ValueError("Cross image must be an explicit image reference")
    command("docker", "pull", image, required=True)
    info = json.loads(command("docker", "image", "inspect", image, required=True))[0]
    resolved = info["RepoDigests"][0]
    # Force the subsequent build to use exactly the image inspected here.
    key = "CROSS_TARGET_" + target.upper().replace("-", "_") + "_IMAGE"
    with open(os.environ["GITHUB_ENV"], "a", encoding="utf-8") as output:
        output.write(f"{key}={resolved}\n")
    prefix = ("docker", "run", "--rm", "--entrypoint", "/bin/sh", resolved, "-c")
    linker_key = "CARGO_TARGET_" + target.upper().replace("-", "_") + "_LINKER"
    container_env = dict(item.split("=", 1) for item in info["Config"]["Env"])
    linker = container_env.get(linker_key)
    tools = command(
        *prefix,
        "for tool in ${CROSS_TOOLCHAIN_PREFIX}gcc "
        "${CROSS_TOOLCHAIN_PREFIX}ld ${CROSS_TOOLCHAIN_PREFIX}ar; do "
        'command -v "$tool" && "$tool" --version; done',
    )
    write(
        "_temp/cross-environment.json",
        {
            "image": image,
            "image_digest": resolved,
            "container_architecture": info["Architecture"],
            "runtime": "docker",
            "runtime_versions": json.loads(
                command("docker", "version", "--format", "{{json .}}", required=True)
            ),
            "native_toolchain": native_tools(("docker", "run", "--rm", resolved)),
            "target_toolchain": tools,
            "linker": linker,
            "linker_version": command("docker", "run", "--rm", resolved, linker, "--version")
            if linker
            else None,
            "version": command("cross", "--version", required=True),
            "installation": {"source": "crates.io", "version": "0.2.5", "locked": True},
            "config_sha256": digest("Cross.toml"),
            "custom_dockerfile": None,
            "base_image": None,
        },
    )


def collect(args):
    """Collect allowlisted source, runner, toolchain, and dependency metadata."""
    archive = Path(args.archive)
    metadata_text = command(
        "cargo",
        "metadata",
        "--locked",
        "--format-version",
        "1",
        "--filter-platform",
        args.target,
        required=True,
    )
    metadata = json.loads(metadata_text)
    tree = command("cargo", "tree", "--locked", "--target", args.target, required=True)
    manifests = {
        str(Path(package["manifest_path"]).relative_to(Path.cwd())): digest(
            package["manifest_path"]
        )
        for package in metadata["packages"]
        if package["id"] in metadata["workspace_members"]
    }
    github_keys = (
        "REPOSITORY REPOSITORY_ID WORKFLOW WORKFLOW_REF WORKFLOW_SHA RUN_ID "
        "RUN_ATTEMPT RUN_NUMBER EVENT_NAME REF REF_NAME SHA ACTOR SERVER_URL"
    ).split()
    config = tomllib.loads(Path(".cargo/config.toml").read_text(encoding="utf-8"))
    target_config = config.get("target", {}).get(args.target, {})
    linker_key = "CARGO_TARGET_" + args.target.upper().replace("-", "_") + "_LINKER"
    linker = os.environ.get(linker_key, target_config.get("linker"))
    # Flags can contain arbitrary literals, including secrets. Record presence only.
    flag_keys = (
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CFLAGS",
        "CXXFLAGS",
        "LDFLAGS",
        "CARGO_TARGET_" + args.target.upper().replace("-", "_") + "_RUSTFLAGS",
        "CFLAGS_" + args.target.replace("-", "_"),
        "CXXFLAGS_" + args.target.replace("-", "_"),
    )
    flags = {
        key: {"value": "[REDACTED]", "sha256": hashed_text(os.environ[key])}
        if os.environ.get(key)
        else None
        for key in flag_keys
    }
    source_dirty = bool(
        command("git", "status", "--porcelain", "--untracked-files=no", required=True)
    )
    record = {
        "schema_version": 1,
        "artifact": {
            "name": archive.name,
            "target": args.target,
            "sha256": digest(archive),
            "size_bytes": archive.stat().st_size,
        },
        "source": {
            "repository": os.environ.get("GITHUB_REPOSITORY"),
            "commit": command("git", "rev-parse", "HEAD", required=True),
            "tag": os.environ.get("GITHUB_REF_NAME")
            if os.environ.get("GITHUB_REF_TYPE") == "tag"
            else None,
            "worktree_dirty": source_dirty,
            "submodules": (command("git", "submodule", "status", "--recursive") or "").splitlines(),
            "lfs_objects": command("git", "lfs", "ls-files", "--long"),
        },
        "github_actions": {key.lower(): os.environ.get("GITHUB_" + key) for key in github_keys},
        "runner": {
            "runs_on": args.runner,
            "os": os.environ.get("RUNNER_OS"),
            "architecture": os.environ.get("RUNNER_ARCH"),
            "environment": os.environ.get("RUNNER_ENVIRONMENT"),
            "kernel": command("uname", "-a"),
            "cpu_architecture": command("uname", "-m"),
            "os_release": Path("/etc/os-release").read_text(encoding="utf-8")
            if Path("/etc/os-release").exists()
            else command("sw_vers"),
            "image_version": os.environ.get("ImageVersion"),
        },
        "toolchain": {
            "rustc_version_verbose": command("rustc", "-vV", required=True),
            "cargo_version_verbose": command("cargo", "--version", "--verbose", required=True),
            "active_toolchain": command("rustup", "show", "active-toolchain", required=True),
            "toolchains": command("rustup", "toolchain", "list", required=True).splitlines(),
            "installed_targets": command(
                "rustup", "target", "list", "--installed", required=True
            ).splitlines(),
            "toolchain_files": {p: digest(p) for p in ("rust-toolchain.toml", "rust-toolchain")},
        },
        "build": {
            "builder": args.builder,
            "target": args.target,
            "target_architecture": args.target.split("-")[0],
            "workspace_packages": [
                {"name": package["name"], "version": package["version"]}
                for package in metadata["packages"]
                if package["id"] in metadata["workspace_members"]
            ],
            "command": [args.builder, "build", "--target", args.target, "--release", "--locked"],
            "profile": "release",
            "default_features": True,
            "additional_features": [],
            "resolved_features": {
                node["id"]: node["features"] for node in metadata["resolve"]["nodes"]
            },
            "flags": flags,
            "cargo_home": os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")),
            "cargo_target_dir": os.environ.get("CARGO_TARGET_DIR"),
            "matrix_entry": json.loads(args.matrix),
        },
        "native_toolchain": {
            "scope": "host",
            "versions": native_tools(),
            "linker": linker,
            "linker_version": command(*shlex.split(linker), "--version") if linker else None,
            "macos_sdk_version": command("xcrun", "--sdk", "macosx", "--show-sdk-version"),
            "xcode_version": command("xcodebuild", "-version"),
            "active_developer_directory": command("xcode-select", "-p"),
            "deployment_target": os.environ.get("MACOSX_DEPLOYMENT_TARGET"),
        },
        "cross": json.loads(Path("_temp/cross-environment.json").read_text(encoding="utf-8"))
        if args.builder == "cross"
        else None,
        "inputs": {
            "cargo_lock_sha256": digest("Cargo.lock"),
            "manifests": manifests,
            "cargo_config": {p: digest(p) for p in (".cargo/config.toml", ".cargo/config")},
            "cargo_home_config": {
                name: digest(Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))) / name)
                for name in ("config", "config.toml")
            },
            "dependency_metadata_sha256": hashed_text(metadata_text),
            "dependency_tree_sha256": hashed_text(tree),
            "matrix_sha256": digest("matrix.jsonc"),
            "build_script_sha256": digest("build.rs"),
            "collector_sha256": digest(__file__),
            "workflow_sha256": digest(".github/workflows/release.yml"),
        },
        "captured_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    }
    write(args.output, record)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("prepare-cross", "collect"))
    parser.add_argument("--target", required=True)
    parser.add_argument("--archive")
    parser.add_argument("--output")
    parser.add_argument("--runner")
    parser.add_argument("--builder", choices=("cargo", "cross"))
    parser.add_argument("--matrix")
    arguments = parser.parse_args()
    if arguments.mode == "prepare-cross":
        prepare_cross(arguments.target)
    else:
        collect(arguments)
