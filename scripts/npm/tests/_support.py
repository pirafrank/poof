"""Shared test helpers: path bootstrap, fake runners/APIs, binary and tar builders.

Everything here is synthetic and offline. Real captured GitHub API responses
live in ``scripts/npm/fixtures`` and are used by the verification tests.
"""

from __future__ import annotations

import io
import struct
import subprocess
import sys
import tarfile
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parents[2]
if str(SCRIPTS_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPTS_DIR))

REPO_ROOT = SCRIPTS_DIR.parent
FIXTURES = SCRIPTS_DIR / "npm" / "fixtures"

from npm import common  # noqa: E402
from npm.stage_packages import (  # noqa: E402
    CPU_TYPE_X86_64,
    DT_NEEDED,
    DT_NULL,
    EM_X86_64,
    ET_EXEC,
    FAT_MAGIC,
    MH_EXECUTE,
    MH_MAGIC_64,
    PT_DYNAMIC,
    PT_INTERP,
    SHT_DYNAMIC,
)


def completed(argv, code=0, stdout="", stderr=""):
    """Build a CompletedProcess stand-in for fake runners."""
    return subprocess.CompletedProcess(argv, code, stdout, stderr)


class FakeRunner:
    """Scriptable command runner recording every invocation."""

    def __init__(self, handler=None, default=None):
        self.calls = []
        self.handler = handler
        self.default = default if default is not None else completed([], 0, "", "")

    def run(self, argv, cwd=None, env=None, timeout=None):
        self.calls.append({"argv": list(argv), "cwd": cwd, "env": env, "timeout": timeout})
        if self.handler is not None:
            result = self.handler(list(argv), cwd)
            if result is not None:
                return result
        return self.default

    def argv_list(self):
        return [call["argv"] for call in self.calls]


class FakeApi:
    """Fixture-backed GitHub API client that rejects unexpected paths."""

    def __init__(self, routes):
        self.routes = dict(routes)
        self.calls = []

    def get_json(self, path):
        self.calls.append(path)
        if path not in self.routes:
            raise AssertionError(f"unexpected API path: {path}")
        return self.routes[path]


def build_elf(
    machine=EM_X86_64,
    elf_type=ET_EXEC,
    interpreter=None,
    needed=(),
    pt_dynamic=False,
    glibc=False,
    omit_sections=False,
    elf_class=2,
    endian_flag=1,
):
    """Construct a minimal but structurally valid ELF64 image for gate tests."""
    payload = b""
    interp_offset = None
    if interpreter is not None:
        interp_offset = len(payload)
        encoded = interpreter.encode() + b"\x00"
        payload += encoded
    entries = [struct.pack("<qQ", DT_NEEDED, value) for value in needed]
    entries.append(struct.pack("<qQ", DT_NULL, 0))
    dynamic_offset = None
    has_dynamic = bool(needed) or pt_dynamic
    if has_dynamic:
        dynamic_offset = len(payload)
        payload += b"".join(entries)
    if glibc:
        payload += b"GLIBC_2.17\x00"

    program_headers = []
    if interpreter is not None:
        program_headers.append((PT_INTERP, interp_offset, len(interpreter.encode()) + 1))
    if has_dynamic:
        program_headers.append((PT_DYNAMIC, dynamic_offset, len(entries) * 16))
    section_headers = []
    if has_dynamic and not omit_sections:
        section_headers.append((SHT_DYNAMIC, dynamic_offset, len(entries) * 16, 16))

    phoff = 64
    shoff = phoff + len(program_headers) * 56
    payload_offset = shoff + len(section_headers) * 64

    header = struct.pack(
        "<4sBBBBB7xHHIQQQIHHHHHH",
        b"\x7fELF",
        elf_class,
        endian_flag,
        1,
        0,
        0,
        elf_type,
        machine,
        1,
        0,
        phoff,
        0 if omit_sections else shoff,
        0,
        64,
        56,
        len(program_headers),
        64,
        len(section_headers),
        0,
    )
    phdr_bytes = b""
    for p_type, offset, filesz in program_headers:
        phdr_bytes += struct.pack(
            "<IIQQQQQQ", p_type, 0, payload_offset + offset, 0, 0, filesz, filesz, 8
        )
    shdr_bytes = b""
    for sh_type, offset, size, entsize in section_headers:
        shdr_bytes += struct.pack(
            "<IIQQQQIIQQ", 0, sh_type, 0, 0, payload_offset + offset, size, 0, 0, 8, entsize
        )
    return header + phdr_bytes + shdr_bytes + payload


def build_macho(cputype=CPU_TYPE_X86_64, filetype=MH_EXECUTE, magic=MH_MAGIC_64, endian="<"):
    """Construct a minimal 64-bit Mach-O header for gate tests."""
    return struct.pack(endian + "IiiIIIII", magic, cputype, 0, filetype, 0, 0, 0, 0) + b"\x00" * 64


def build_fat_macho():
    """Construct a universal (fat) Mach-O header, which must be rejected."""
    return struct.pack(">II", FAT_MAGIC, 2) + b"\x00" * 64


def make_tar_gz(path, members):
    """Write a gzipped tar with explicit member types, modes, and payloads.

    ``members`` is a list of dicts with keys: name, data (bytes), mode, and
    optional type (tarfile type constant), linkname, size override.
    """
    with tarfile.open(path, "w:gz") as archive:
        for spec in members:
            info = tarfile.TarInfo(spec["name"])
            info.mode = spec.get("mode", 0o644)
            info.type = spec.get("type", tarfile.REGTYPE)
            data = spec.get("data", b"")
            if info.type == tarfile.REGTYPE:
                info.size = spec.get("size", len(data))
            else:
                info.size = 0
            info.linkname = spec.get("linkname", "")
            archive.addfile(info, io.BytesIO(data) if info.type == tarfile.REGTYPE else None)
    return Path(path)


def doc_members(prefix="./"):
    """The four documentation members present in real poof release archives."""
    return [
        {"name": f"{prefix}{name}", "data": f"{name} text\n".encode(), "mode": 0o644}
        for name in sorted(common.DOC_FILENAMES)
    ]


def native_archive_members(binary_data, prefix="./", binary_mode=0o755):
    """A realistic flat release archive layout: root dir, poof, and four docs."""
    return (
        [{"name": prefix, "type": tarfile.DIRTYPE, "mode": 0o755}]
        + [{"name": f"{prefix}poof", "data": binary_data, "mode": binary_mode}]
        + doc_members(prefix)
    )


def npm_tarball_members(files, prefix="package/"):
    """Build tarball members in npm's ``package/`` layout from a path->bytes map."""
    return [
        {"name": f"{prefix}{name}", "data": data, "mode": mode}
        for name, (data, mode) in files.items()
    ]


def tar_info(name, mode=0o644, kind=tarfile.REGTYPE, size=16, linkname=""):
    """Build a TarInfo for member-validation tests without writing an archive."""
    info = tarfile.TarInfo(name)
    info.mode = mode
    info.type = kind
    info.size = size if kind == tarfile.REGTYPE else 0
    info.linkname = linkname
    return info


class FakePackRunner:
    """Offline ``npm pack`` double that really produces a tarball on disk.

    Mirrors the two npm JSON shapes and the dry-run/real distinction so
    ``stage_all`` can be exercised end to end without invoking npm.
    """

    def __init__(self, files_by_name, shape="object"):
        self.files_by_name = files_by_name
        self.shape = shape
        self.calls = []

    def run(self, argv, cwd=None, env=None, timeout=None):
        import base64
        import hashlib
        import json

        self.calls.append({"argv": list(argv), "cwd": str(cwd), "env": env})
        if argv[:2] != ["npm", "pack"]:
            raise AssertionError(f"unexpected command: {argv}")
        cwd = Path(cwd)
        manifest = json.loads((cwd / "package.json").read_text(encoding="utf-8"))
        name, version = manifest["name"], manifest["version"]
        expected = self.files_by_name[name]
        payload = {rel: (cwd / rel).read_bytes() for rel in sorted(expected)}
        modes = {rel: (cwd / rel).stat().st_mode & 0o777 for rel in payload}
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w:gz") as archive:
            for rel, data in payload.items():
                info = tarfile.TarInfo(f"package/{rel}")
                info.size = len(data)
                info.mode = modes[rel]
                archive.addfile(info, io.BytesIO(data))
        blob = buffer.getvalue()
        filename = f"{name.replace('@', '').replace('/', '-')}-{version}.tgz"
        dry_run = "--dry-run" in argv
        if not dry_run:
            (cwd / filename).write_bytes(blob)
        record = {
            "name": name,
            "version": version,
            "filename": filename,
            "integrity": "sha512-" + base64.b64encode(hashlib.sha512(blob).digest()).decode(),
            "shasum": hashlib.sha1(blob).hexdigest(),
            "files": [
                {"path": rel, "size": len(payload[rel]), "mode": modes[rel]}
                for rel in payload
            ],
            "entryCount": len(payload),
            "unpackedSize": sum(len(data) for data in payload.values()),
        }
        text = (
            json.dumps({name: record})
            if self.shape == "object"
            else json.dumps([record])
        )
        return completed(argv, 0, text, "")


def synthetic_evidence(version="0.6.3", commit="9" * 40):
    """Build verification evidence with the shape verify_release.py emits."""
    from npm import common as c

    tag = f"v{version}"
    assets = {}
    attested = {}
    for triple in c.CANONICAL_TRIPLE_ORDER:
        platform = c.Platform(rust_target=triple, npm_os=c.SUPPORTED_TRIPLES[triple][0],
                              npm_cpu=c.SUPPORTED_TRIPLES[triple][1])
        for name in (
            platform.archive_name(version),
            platform.spdx_name(version),
            platform.build_environment_name(version),
        ):
            digest = c.sha256_bytes(name.encode())
            assets[name] = {"sha256": digest, "size": 1024}
            attested[name] = digest
    for name in (c.checksums_manifest_name(version), c.sigstore_bundle_name(version)):
        digest = c.sha256_bytes(name.encode())
        assets[name] = {"sha256": digest, "size": 512}
        attested[name] = digest
    return {
        "schema_version": 1,
        "version": version,
        "tag": tag,
        "commit": commit,
        "tag_object": "a" * 40,
        "attested_tag_object": "a" * 40,
        "cargo_version": version,
        "release": {
            "id": 438203540,
            "immutable": True,
            "draft": False,
            "prerelease": c.parse_npm_version(version).is_prerelease,
            "published_at": "2026-11-08T11:20:00Z",
            "html_url": f"{c.GITHUB_SERVER_URL}/{c.REPOSITORY}/releases/tag/{tag}",
        },
        "assets": assets,
        "attested_asset_digests": attested,
        "checksums": {
            "filename": c.checksums_manifest_name(version),
            "sha256": assets[c.checksums_manifest_name(version)]["sha256"],
        },
        "verifications": {
            "cosign": {"status": "ok", "identity": c.signer_identity(tag)},
            "release_verify": {"status": "ok", "subjects": 52},
            "attestations": {"status": "ok", "verified": 8},
        },
        "host": {"system": "Linux", "machine": "x86_64", "python_version": "3.13.2"},
    }


def synthetic_staging(version="0.6.3", commit="9" * 40, targets=None):
    """Build a staging report with the shape stage_packages.py emits."""
    from npm import common as c

    targets = targets or c.load_npm_targets(REPO_ROOT / "matrix.jsonc")
    platforms = []
    for target in targets:
        platform = target.platform
        archive_name = platform.archive_name(version)
        binary_blob = f"binary-for-{platform.rust_target}".encode()
        tarball_blob = f"tarball-for-{platform.package_name}".encode()
        platforms.append(
            {
                "package_name": platform.package_name,
                "version": version,
                "os": platform.npm_os,
                "cpu": platform.npm_cpu,
                "rust_target": platform.rust_target,
                "builder": target.builder,
                "runner": target.runner,
                "archive": {"name": archive_name, "sha256": c.sha256_bytes(archive_name.encode())},
                "binary": {
                    "sha256": c.sha256_bytes(binary_blob),
                    "size_bytes": len(binary_blob),
                    "mode": "755",
                    "format": "elf64" if platform.npm_os == "linux" else "mach-o64",
                    "static": True if platform.npm_os == "linux" else None,
                },
                "tarball": {
                    "filename": f"{platform.package_dirname}-{version}.tgz",
                    "sha256": c.sha256_bytes(tarball_blob),
                    "sha512_integrity": c.sha512_integrity_bytes(tarball_blob),
                    "size_bytes": len(tarball_blob),
                    "path": f"/tmp/staged/tarballs/{platform.package_dirname}-{version}.tgz",
                },
            }
        )
    base_blob = b"tarball-for-base"
    return {
        "schema_version": 1,
        "version": version,
        "tag": f"v{version}",
        "dist_tag": c.parse_npm_version(version).dist_tag,
        "commit": commit,
        "release_id": 438203540,
        "host": {"system": "Linux", "machine": "x86_64", "python_version": "3.13.2"},
        "platforms": platforms,
        "base": {
            "package_name": c.BASE_PACKAGE,
            "version": version,
            "optional_dependencies": {t.platform.package_name: version for t in targets},
            "launcher": {"path": "bin/poof.js", "sha256": c.sha256_bytes(b"launcher"), "mode": "755"},
            "tarball": {
                "filename": f"pirafrank-poof-{version}.tgz",
                "sha256": c.sha256_bytes(base_blob),
                "sha512_integrity": c.sha512_integrity_bytes(base_blob),
                "size_bytes": len(base_blob),
                "path": f"/tmp/staged/tarballs/pirafrank-poof-{version}.tgz",
            },
        },
    }
