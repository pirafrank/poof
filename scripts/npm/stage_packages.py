#!/usr/bin/env python3
"""Stage and pack the five poof npm packages from verified native archives.

Consumes the verification evidence produced by ``verify_release.py`` and the
native release archives it attested. Never rebuilds Rust, never downloads from
the launcher path, and never publishes: it only extracts the root ``poof``
executable, validates it, writes manifests, and runs ``npm pack``.

Safety rules enforced here (fail closed):
* only flat archive layouts with exactly one regular root ``poof`` member;
* no traversal, absolute paths, symlinks/hardlinks, devices, or extra executables;
* ELF/Mach-O magic plus CPU architecture must match the npm target;
* Linux binaries must be static musl (no ELF interpreter, no DT_NEEDED, no GLIBC);
* the packaged executable is the only file given mode 0755;
* tarball contents are asserted against an explicit allowlist.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import tarfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from npm import common
from npm.common import NpmPackagingError, NpmTarget, Platform

ELF_MAGIC = b"\x7fELF"
ELFCLASS64 = 2
ELFDATA2LSB = 1
ET_EXEC = 2
ET_DYN = 3
EM_X86_64 = 62
EM_AARCH64 = 183
PT_INTERP = 3
PT_DYNAMIC = 2
SHT_DYNAMIC = 6
DT_NULL = 0
DT_NEEDED = 1
ELF_MACHINE_BY_CPU = {"x64": EM_X86_64, "arm64": EM_AARCH64}

MH_MAGIC_64 = 0xFEEDFACF
MH_CIGAM_64 = 0xCFFAEDFE
MH_MAGIC_32 = 0xFEEDFACE
MH_CIGAM_32 = 0xCEFAEDFE
FAT_MAGIC = 0xCAFEBABE
FAT_CIGAM = 0xBEBAFECA
MH_EXECUTE = 2
CPU_TYPE_X86_64 = 0x01000007
CPU_TYPE_ARM64 = 0x0100000C
MACHO_CPU_BY_CPU = {"x64": CPU_TYPE_X86_64, "arm64": CPU_TYPE_ARM64}

# Magic prefixes that must never appear in a documentation member.
FORBIDDEN_DOC_MAGIC = (
    ELF_MAGIC,
    b"\xcf\xfa\xed\xfe",
    b"\xce\xfa\xed\xfe",
    b"\xfe\xed\xfa\xce",
    b"\xfe\xed\xfa\xcf",
    b"\xca\xfe\xba\xbe",
    b"PK\x03\x04",
    b"7z\xbc\xaf\x27\x1c",
    b"\x1f\x8b",
)
MAX_DOC_BYTES = 8 * 1024 * 1024
WINDOWS_DRIVE_RE = re.compile(r"^[A-Za-z]:")
NPM_TARBALL_PREFIX = "package/"


class CommandRunner(Protocol):
    """Abstraction over subprocess execution so tests can inject fakes."""

    def run(
        self,
        argv: list[str],
        cwd: Path | None = None,
        env: dict[str, str] | None = None,
        timeout: int | None = None,
    ) -> subprocess.CompletedProcess:
        """Run a command and return its completed process."""
        ...


class SubprocessRunner:
    """Real subprocess runner with captured output and bounded runtime."""

    def run(
        self,
        argv: list[str],
        cwd: Path | None = None,
        env: dict[str, str] | None = None,
        timeout: int | None = None,
    ) -> subprocess.CompletedProcess:
        """Run argv without a shell, capturing stdout/stderr as text."""
        merged_env = dict(os.environ)
        if env:
            merged_env.update(env)
        return subprocess.run(
            argv,
            cwd=str(cwd) if cwd else None,
            env=merged_env,
            capture_output=True,
            text=True,
            timeout=timeout,
            check=False,
        )


def normalize_member_name(raw: str) -> str:
    """Normalize an archive member name to a safe flat relative path.

    Rejects NUL bytes, absolute paths, Windows drive prefixes, backslashes,
    parent/current directory traversal, and nested paths. A single leading
    ``./`` (the layout produced by the native release workflow) is accepted.
    """
    if not isinstance(raw, str) or not raw:
        raise NpmPackagingError("Archive member name must be a non-empty string")
    if "\x00" in raw:
        raise NpmPackagingError(f"Archive member name contains NUL: {raw!r}")
    if "\\" in raw:
        raise NpmPackagingError(f"Archive member name contains a backslash: {raw!r}")
    name = raw
    if name.startswith("./"):
        name = name[2:]
    if name.startswith("./") or name.startswith("/"):
        raise NpmPackagingError(f"Unsafe archive member path: {raw!r}")
    if WINDOWS_DRIVE_RE.match(name):
        raise NpmPackagingError(f"Windows-style archive member path: {raw!r}")
    if not name:
        raise NpmPackagingError(f"Archive member name resolves to empty: {raw!r}")
    parts = name.split("/")
    for part in parts:
        if part in ("", ".", ".."):
            raise NpmPackagingError(f"Unsafe archive member path component in {raw!r}")
    if len(parts) != 1:
        raise NpmPackagingError(f"Nested archive member is not allowed: {raw!r}")
    if name.startswith("."):
        raise NpmPackagingError(f"Hidden archive member is not allowed: {raw!r}")
    return name


def _is_root_dir_entry(member: tarfile.TarInfo) -> bool:
    """True for the archive's single ``.``/``./`` directory entry."""
    return member.isdir() and member.name in (".", "./")


def validate_members(members: list[tarfile.TarInfo], archive_label: str) -> tarfile.TarInfo:
    """Validate the full member list and return the single ``poof`` member.

    Enforces the flat layout, member/type restrictions, size bounds, duplicate
    detection, and the rule that no member other than ``poof`` is executable.
    """
    if len(members) > common.MAX_ARCHIVE_MEMBERS:
        raise NpmPackagingError(
            f"{archive_label}: too many archive members ({len(members)} > "
            f"{common.MAX_ARCHIVE_MEMBERS})"
        )
    total = 0
    root_dirs = 0
    seen: dict[str, tarfile.TarInfo] = {}
    binary_member: tarfile.TarInfo | None = None
    for member in members:
        if _is_root_dir_entry(member):
            root_dirs += 1
            if root_dirs > 1:
                raise NpmPackagingError(f"{archive_label}: duplicate root directory entry")
            continue
        name = normalize_member_name(member.name)
        if name in seen:
            raise NpmPackagingError(f"{archive_label}: duplicate archive member {name!r}")
        seen[name] = member
        if member.isdir():
            raise NpmPackagingError(
                f"{archive_label}: nested directory {member.name!r} not allowed"
            )
        if not member.isfile():
            kind = "symlink" if member.issym() else "hardlink" if member.islnk() else "special file"
            raise NpmPackagingError(
                f"{archive_label}: {kind} member {member.name!r} is not allowed"
            )
        if member.size < 0 or member.size > common.MAX_ARCHIVE_UNPACKED_BYTES:
            raise NpmPackagingError(f"{archive_label}: implausible size for {name!r}")
        total += member.size
        if total > common.MAX_ARCHIVE_UNPACKED_BYTES:
            raise NpmPackagingError(f"{archive_label}: unpacked size exceeds safety bound")
        if name == common.BINARY_NAME:
            if member.size == 0 or member.size > common.MAX_BINARY_BYTES:
                raise NpmPackagingError(f"{archive_label}: implausible executable size")
            if binary_member is not None:
                raise NpmPackagingError(f"{archive_label}: duplicate 'poof' executable")
            binary_member = member
            continue
        if member.mode & 0o111:
            raise NpmPackagingError(
                f"{archive_label}: unexpected executable member {name!r} (mode {member.mode:o})"
            )
    if binary_member is None:
        raise NpmPackagingError(f"{archive_label}: no root 'poof' executable found")
    unexpected = set(seen) - {common.BINARY_NAME} - common.DOC_FILENAMES
    if unexpected:
        raise NpmPackagingError(f"{archive_label}: unexpected archive members {sorted(unexpected)}")
    return binary_member


def _assert_docs_are_not_binaries(archive: tarfile.TarFile, members: list[tarfile.TarInfo]) -> None:
    """Read a bounded prefix of each doc member and reject executable magic."""
    for member in members:
        if _is_root_dir_entry(member):
            continue
        name = normalize_member_name(member.name)
        if name == common.BINARY_NAME:
            continue
        if member.size > MAX_DOC_BYTES:
            raise NpmPackagingError(f"Documentation member {name!r} is implausibly large")
        stream = archive.extractfile(member)
        if stream is None:
            raise NpmPackagingError(f"Cannot read documentation member {name!r}")
        head = stream.read(8)
        stream.close()
        if any(head.startswith(magic) for magic in FORBIDDEN_DOC_MAGIC):
            raise NpmPackagingError(f"Documentation member {name!r} has executable/archive magic")


def extract_binary(archive_path: Path, expected_archive_sha256: str) -> tuple[bytes, dict]:
    """Verify the archive digest, validate members, and read only root ``poof``.

    Returns the executable bytes plus extraction evidence. The source archive
    and its permissions are never modified; only the packaged copy gets 0755.
    """
    archive_path = Path(archive_path)
    actual = common.sha256_file(archive_path)
    if actual != expected_archive_sha256:
        raise NpmPackagingError(
            f"{archive_path.name}: SHA-256 mismatch (expected {expected_archive_sha256}, "
            f"got {actual})"
        )
    with tarfile.open(archive_path, "r:gz") as archive:
        members = archive.getmembers()
        binary_member = validate_members(members, archive_path.name)
        _assert_docs_are_not_binaries(archive, members)
        stream = archive.extractfile(binary_member)
        if stream is None:
            raise NpmPackagingError(f"{archive_path.name}: cannot read 'poof' member")
        data = stream.read(common.MAX_BINARY_BYTES + 1)
        stream.close()
    if len(data) > common.MAX_BINARY_BYTES:
        raise NpmPackagingError(f"{archive_path.name}: executable exceeds size bound")
    if not data:
        raise NpmPackagingError(f"{archive_path.name}: executable is empty")
    evidence = {
        "archive_sha256": actual,
        "member_name": binary_member.name,
        "member_mode": f"{binary_member.mode & 0o7777:o}",
        "member_count": len(members),
        "doc_members": sorted(
            normalize_member_name(m.name)
            for m in members
            if not _is_root_dir_entry(m) and normalize_member_name(m.name) != common.BINARY_NAME
        ),
    }
    return data, evidence


def _inspect_elf(data: bytes, expected_machine: int, label: str) -> dict:
    """Validate an ELF64 little-endian executable and prove static musl linking."""
    if len(data) < 64:
        raise NpmPackagingError(f"{label}: truncated ELF header")
    if data[4] != ELFCLASS64:
        raise NpmPackagingError(f"{label}: only ELF64 executables are supported")
    if data[5] != ELFDATA2LSB:
        raise NpmPackagingError(f"{label}: only little-endian ELF is supported")
    try:
        e_type, e_machine = struct.unpack_from("<HH", data, 16)
        e_phoff = struct.unpack_from("<Q", data, 32)[0]
        e_shoff = struct.unpack_from("<Q", data, 40)[0]
        e_phentsize, e_phnum = struct.unpack_from("<HH", data, 54)
        e_shentsize, e_shnum = struct.unpack_from("<HH", data, 58)
    except struct.error as error:
        raise NpmPackagingError(f"{label}: malformed ELF header: {error}") from error
    if e_machine != expected_machine:
        raise NpmPackagingError(
            f"{label}: ELF machine {e_machine} does not match expected {expected_machine}"
        )
    if e_type not in (ET_EXEC, ET_DYN):
        raise NpmPackagingError(f"{label}: ELF type {e_type} is not an executable")

    interpreter = None
    has_pt_dynamic = False
    for index in range(e_phnum):
        offset = e_phoff + index * e_phentsize
        if offset + 56 > len(data):
            raise NpmPackagingError(f"{label}: truncated ELF program header")
        p_type = struct.unpack_from("<I", data, offset)[0]
        if p_type == PT_DYNAMIC:
            has_pt_dynamic = True
        if p_type == PT_INTERP:
            p_offset = struct.unpack_from("<Q", data, offset + 8)[0]
            p_filesz = struct.unpack_from("<Q", data, offset + 32)[0]
            if p_offset + p_filesz > len(data) or p_filesz > 4096:
                raise NpmPackagingError(f"{label}: implausible ELF interpreter segment")
            interpreter = data[p_offset : p_offset + p_filesz].split(b"\x00")[0].decode(
                "utf-8", "replace"
            )
    if interpreter is not None:
        raise NpmPackagingError(
            f"{label}: dynamically linked ELF (interpreter {interpreter!r}); "
            "only static musl binaries may be published"
        )

    needed: list[int] = []
    if e_shoff:
        for index in range(e_shnum):
            offset = e_shoff + index * e_shentsize
            if offset + 64 > len(data):
                raise NpmPackagingError(f"{label}: truncated ELF section header")
            sh_type = struct.unpack_from("<I", data, offset + 4)[0]
            if sh_type != SHT_DYNAMIC:
                continue
            sh_offset = struct.unpack_from("<Q", data, offset + 24)[0]
            sh_size = struct.unpack_from("<Q", data, offset + 32)[0]
            sh_entsize = struct.unpack_from("<Q", data, offset + 56)[0] or 16
            if sh_offset + sh_size > len(data):
                raise NpmPackagingError(f"{label}: truncated ELF dynamic section")
            for entry in range(sh_size // sh_entsize):
                position = sh_offset + entry * sh_entsize
                d_tag = struct.unpack_from("<q", data, position)[0]
                if d_tag == DT_NULL:
                    break
                if d_tag == DT_NEEDED:
                    needed.append(struct.unpack_from("<Q", data, position + 8)[0])
    elif has_pt_dynamic:
        # No section headers to inspect: cannot prove the absence of DT_NEEDED.
        raise NpmPackagingError(
            f"{label}: PT_DYNAMIC present but section headers are stripped; "
            "static linking cannot be proven"
        )
    if needed:
        raise NpmPackagingError(
            f"{label}: ELF declares {len(needed)} DT_NEEDED entries; "
            "only static musl binaries may be published"
        )
    if b"GLIBC_" in data:
        raise NpmPackagingError(f"{label}: GLIBC symbol versions found; glibc build rejected")
    return {
        "format": "elf64",
        "endianness": "little",
        "elf_type": "ET_EXEC" if e_type == ET_EXEC else "ET_DYN",
        "machine": e_machine,
        "has_interpreter": False,
        "has_pt_dynamic": has_pt_dynamic,
        "dt_needed_count": 0,
        "glibc_symbols": False,
        "static": True,
        "libc": "musl (static)",
    }


def _inspect_macho(data: bytes, expected_cpu: int, label: str) -> dict:
    """Validate a thin 64-bit Mach-O executable and its CPU type."""
    if len(data) < 32:
        raise NpmPackagingError(f"{label}: truncated Mach-O header")
    magic = struct.unpack_from("<I", data, 0)[0]
    if magic in (FAT_MAGIC, FAT_CIGAM):
        raise NpmPackagingError(f"{label}: universal (fat) Mach-O is not supported")
    if magic in (MH_MAGIC_32, MH_CIGAM_32):
        raise NpmPackagingError(f"{label}: only 64-bit Mach-O executables are supported")
    if magic == MH_MAGIC_64:
        endian = "<"
    elif magic == MH_CIGAM_64:
        endian = ">"
    else:
        raise NpmPackagingError(f"{label}: unrecognized Mach-O magic {magic:#x}")
    try:
        cputype, _cpusubtype, filetype = struct.unpack_from(endian + "iiI", data, 4)
    except struct.error as error:
        raise NpmPackagingError(f"{label}: malformed Mach-O header: {error}") from error
    if cputype != expected_cpu:
        raise NpmPackagingError(
            f"{label}: Mach-O cputype {cputype:#x} does not match expected {expected_cpu:#x}"
        )
    if filetype != MH_EXECUTE:
        raise NpmPackagingError(f"{label}: Mach-O filetype {filetype} is not MH_EXECUTE")
    return {
        "format": "mach-o64",
        "endianness": "little" if endian == "<" else "big",
        "cputype": f"{cputype:#x}",
        "filetype": "MH_EXECUTE",
        "universal": False,
        "static": None,
    }


def inspect_binary(data: bytes, platform: Platform) -> dict:
    """Detect the executable format and verify it matches the npm target."""
    label = platform.package_name
    if data[:4] == ELF_MAGIC:
        if platform.npm_os != "linux":
            raise NpmPackagingError(f"{label}: ELF binary packaged for a non-Linux target")
        evidence = _inspect_elf(data, ELF_MACHINE_BY_CPU[platform.npm_cpu], label)
    elif len(data) >= 4 and struct.unpack_from("<I", data, 0)[0] in (
        MH_MAGIC_64,
        MH_CIGAM_64,
        MH_MAGIC_32,
        MH_CIGAM_32,
        FAT_MAGIC,
        FAT_CIGAM,
    ):
        if platform.npm_os != "darwin":
            raise NpmPackagingError(f"{label}: Mach-O binary packaged for a non-macOS target")
        evidence = _inspect_macho(data, MACHO_CPU_BY_CPU[platform.npm_cpu], label)
    else:
        raise NpmPackagingError(f"{label}: unrecognized executable format (expected ELF or Mach-O)")
    evidence["sha256"] = common.sha256_bytes(data)
    evidence["size_bytes"] = len(data)
    evidence["mode"] = f"{common.EXECUTABLE_MODE:o}"
    return evidence


def platform_package_json(target: NpmTarget, version: str) -> dict:
    """Build the platform package manifest: no bin field, exact version, public."""
    platform = target.platform
    return {
        "name": platform.package_name,
        "version": version,
        "description": f"Native poof binary for {platform.npm_os}/{platform.npm_cpu} "
        f"({platform.rust_target})",
        "license": common.LICENSE,
        "os": [platform.npm_os],
        "cpu": [platform.npm_cpu],
        "files": ["bin/poof", "LICENSE.md"],
        "publishConfig": {"access": "public"},
        "repository": {"type": "git", "url": common.REPOSITORY_URL},
        "homepage": common.HOMEPAGE,
        "bugs": {"url": f"{common.HOMEPAGE}/issues"},
    }


def base_package_json(template: dict, version: str, targets: list[NpmTarget]) -> dict:
    """Render the launcher manifest from the template with exact pinned versions."""
    manifest = json.loads(json.dumps(template))
    if manifest.get("name") != common.BASE_PACKAGE:
        raise NpmPackagingError(f"Template name must be {common.BASE_PACKAGE!r}")
    manifest["version"] = version
    manifest["private"] = False
    expected_deps = {target.platform.package_name: version for target in targets}
    manifest["optionalDependencies"] = expected_deps
    if manifest.get("bin") != {"poof": "bin/poof.js"}:
        raise NpmPackagingError("Template must map bin.poof to bin/poof.js")
    if manifest.get("engines", {}).get("node") != common.NODE_ENGINE:
        raise NpmPackagingError(f"Template engines.node must be {common.NODE_ENGINE!r}")
    if manifest.get("publishConfig", {}).get("access") != "public":
        raise NpmPackagingError("Template publishConfig.access must be 'public'")
    if "scripts" in manifest:
        raise NpmPackagingError("Launcher package must not declare lifecycle scripts")
    if "dependencies" in manifest:
        raise NpmPackagingError("Launcher package must not declare runtime dependencies")
    if "os" in manifest or "cpu" in manifest:
        raise NpmPackagingError(
            "Launcher package must not restrict os/cpu: it has to emit actionable errors"
        )
    if manifest.get("repository", {}).get("url") != common.REPOSITORY_URL:
        raise NpmPackagingError(
            "repository.url must exactly match the GitHub repository for trusted publishing"
        )
    common.ensure_no_placeholders(manifest, "base package.json")
    for name, pinned in manifest["optionalDependencies"].items():
        if pinned != version:
            raise NpmPackagingError(f"optionalDependency {name} must pin exact version {version}")
    return manifest


def _write_executable(path: Path, data: bytes) -> None:
    """Write bytes and set mode 0755 on the packaged copy only."""
    path.write_bytes(data)
    path.chmod(common.EXECUTABLE_MODE)


def stage_platform_package(
    out_dir: Path,
    target: NpmTarget,
    version: str,
    binary_data: bytes,
    license_text: str,
) -> Path:
    """Create a platform package directory containing only the allowed files."""
    package_dir = Path(out_dir) / target.platform.package_dirname
    if package_dir.exists():
        raise NpmPackagingError(f"Staging directory already exists: {package_dir}")
    (package_dir / "bin").mkdir(parents=True)
    _write_executable(package_dir / "bin" / common.BINARY_NAME, binary_data)
    (package_dir / "LICENSE.md").write_text(license_text, encoding="utf-8")
    common.write_json(package_dir / "package.json", platform_package_json(target, version))
    return package_dir


def stage_base_package(
    out_dir: Path,
    version: str,
    targets: list[NpmTarget],
    npm_dir: Path,
    license_text: str,
) -> Path:
    """Create the launcher package directory from the template and launcher source."""
    npm_dir = Path(npm_dir)
    template_path = npm_dir / "package.json.template"
    launcher_path = npm_dir / "bin" / "poof.js"
    readme_path = npm_dir / "README.md"
    for required in (template_path, launcher_path, readme_path):
        if not required.is_file():
            raise NpmPackagingError(f"Missing launcher source file: {required}")
    template = common.read_json(template_path)
    manifest = base_package_json(template, version, targets)
    package_dir = Path(out_dir) / "poof"
    if package_dir.exists():
        raise NpmPackagingError(f"Staging directory already exists: {package_dir}")
    (package_dir / "bin").mkdir(parents=True)
    _write_executable(package_dir / "bin" / "poof.js", launcher_path.read_bytes())
    shutil.copyfile(readme_path, package_dir / "README.md")
    (package_dir / "LICENSE.md").write_text(license_text, encoding="utf-8")
    common.write_json(package_dir / "package.json", manifest)
    return package_dir


def parse_pack_output(stdout: str, package_name: str) -> dict:
    """Extract one package's pack record from npm's JSON output.

    npm <= 10 emits an array of records; npm 12 emits an object keyed by package
    name. Both shapes are accepted, and anything ambiguous is rejected.
    """
    text = stdout.strip()
    if not text:
        raise NpmPackagingError("npm pack produced no output")
    start_candidates = [index for index in (text.find("{"), text.find("[")) if index >= 0]
    if not start_candidates:
        raise NpmPackagingError(f"npm pack output is not JSON: {text[:200]!r}")
    try:
        payload = json.loads(text[min(start_candidates) :])
    except json.JSONDecodeError as error:
        raise NpmPackagingError(f"npm pack output is not valid JSON: {error}") from error
    if isinstance(payload, dict):
        if package_name in payload:
            record = payload[package_name]
        elif len(payload) == 1:
            record = next(iter(payload.values()))
        else:
            raise NpmPackagingError(f"npm pack output has no entry for {package_name}")
    elif isinstance(payload, list):
        matches = [
            item
            for item in payload
            if isinstance(item, dict) and item.get("name") == package_name
        ]
        if len(matches) != 1:
            raise NpmPackagingError(
                f"npm pack output must contain exactly one {package_name} entry"
            )
        record = matches[0]
    else:
        raise NpmPackagingError("Unexpected npm pack JSON shape")
    if not isinstance(record, dict):
        raise NpmPackagingError("npm pack record must be an object")
    if record.get("name") != package_name:
        raise NpmPackagingError(
            f"npm pack name mismatch: {record.get('name')!r} != {package_name!r}"
        )
    return record


def assert_pack_record(
    record: dict,
    package_name: str,
    version: str,
    expected_files: dict[str, int],
) -> None:
    """Assert the pack record's identity, file allowlist, modes, and integrity."""
    if record.get("version") != version:
        raise NpmPackagingError(
            f"{package_name}: pack version {record.get('version')!r} != {version!r}"
        )
    integrity = record.get("integrity", "")
    if not isinstance(integrity, str) or not integrity.startswith("sha512-"):
        raise NpmPackagingError(f"{package_name}: pack record lacks sha512 integrity")
    files = record.get("files")
    if not isinstance(files, list):
        raise NpmPackagingError(f"{package_name}: pack record has no file list")
    actual = {}
    for item in files:
        path = item.get("path")
        if not isinstance(path, str):
            raise NpmPackagingError(f"{package_name}: pack file entry without a path")
        if path in actual:
            raise NpmPackagingError(f"{package_name}: duplicate pack file {path!r}")
        actual[path] = int(item.get("mode", 0)) & 0o777
    if set(actual) != set(expected_files):
        raise NpmPackagingError(
            f"{package_name}: pack file allowlist mismatch: "
            f"unexpected={sorted(set(actual) - set(expected_files))} "
            f"missing={sorted(set(expected_files) - set(actual))}"
        )
    for path, mode in expected_files.items():
        if actual[path] != mode:
            raise NpmPackagingError(
                f"{package_name}: {path} mode {actual[path]:o} != expected {mode:o}"
            )
    if record.get("entryCount") != len(expected_files):
        raise NpmPackagingError(f"{package_name}: unexpected entryCount in pack record")


def npm_pack(
    runner: CommandRunner,
    package_dir: Path,
    package_name: str,
    version: str,
    expected_files: dict[str, int],
    dry_run: bool,
) -> dict:
    """Run ``npm pack`` in a package directory and validate its JSON record."""
    argv = ["npm", "pack", "--json"]
    if dry_run:
        argv.append("--dry-run")
    env = {
        "npm_config_update_notifier": "false",
        "npm_config_fund": "false",
        "npm_config_audit": "false",
    }
    result = runner.run(argv, cwd=Path(package_dir), env=env, timeout=600)
    if result.returncode != 0:
        raise NpmPackagingError(
            f"{package_name}: npm pack failed ({result.returncode}): "
            f"{common.redact(result.stderr or result.stdout)}"
        )
    record = parse_pack_output(result.stdout, package_name)
    assert_pack_record(record, package_name, version, expected_files)
    return record


def inspect_npm_tarball(
    tarball_path: Path,
    package_name: str,
    expected_files: dict[str, int],
) -> dict:
    """Independently inspect the produced tarball's entries, types, and modes."""
    tarball_path = Path(tarball_path)
    with tarfile.open(tarball_path, "r:gz") as archive:
        members = archive.getmembers()
    actual: dict[str, int] = {}
    for member in members:
        if not member.name.startswith(NPM_TARBALL_PREFIX):
            raise NpmPackagingError(
                f"{package_name}: tarball entry outside package/: {member.name}"
            )
        relative = member.name[len(NPM_TARBALL_PREFIX) :]
        if not relative or relative.startswith("/") or ".." in relative.split("/"):
            raise NpmPackagingError(f"{package_name}: unsafe tarball entry {member.name!r}")
        if not member.isfile():
            raise NpmPackagingError(f"{package_name}: non-regular tarball entry {member.name!r}")
        if relative in actual:
            raise NpmPackagingError(f"{package_name}: duplicate tarball entry {relative!r}")
        actual[relative] = member.mode & 0o777
    if set(actual) != set(expected_files):
        raise NpmPackagingError(
            f"{package_name}: tarball allowlist mismatch: "
            f"unexpected={sorted(set(actual) - set(expected_files))} "
            f"missing={sorted(set(expected_files) - set(actual))}"
        )
    for path, mode in expected_files.items():
        if actual[path] != mode:
            raise NpmPackagingError(
                f"{package_name}: tarball {path} mode {actual[path]:o} != expected {mode:o}"
            )
    return {
        "filename": tarball_path.name,
        "sha256": common.sha256_file(tarball_path),
        "sha512_integrity": common.sha512_integrity_file(tarball_path),
        "size_bytes": tarball_path.stat().st_size,
        "entries": [{"path": path, "mode": f"{mode:o}"} for path, mode in sorted(actual.items())],
    }


PLATFORM_FILES = {"package.json": 0o644, "bin/poof": 0o755, "LICENSE.md": 0o644}
BASE_FILES = {
    "package.json": 0o644,
    "bin/poof.js": 0o755,
    "README.md": 0o644,
    "LICENSE.md": 0o644,
}


def _move_tarball(package_dir: Path, record: dict, tarball_dir: Path) -> Path:
    """Move the packed tarball out of the package directory into the output dir."""
    filename = record.get("filename")
    if not isinstance(filename, str) or not filename.endswith(".tgz"):
        raise NpmPackagingError("npm pack record has no tarball filename")
    if "/" in filename or "\\" in filename or filename.startswith("."):
        raise NpmPackagingError(f"Unsafe tarball filename from npm pack: {filename!r}")
    source = Path(package_dir) / filename
    if not source.is_file():
        raise NpmPackagingError(f"npm pack did not produce {source}")
    tarball_dir = Path(tarball_dir)
    tarball_dir.mkdir(parents=True, exist_ok=True)
    destination = tarball_dir / filename
    if destination.exists():
        raise NpmPackagingError(f"Tarball already exists: {destination}")
    shutil.move(str(source), str(destination))
    return destination


def load_evidence(path: Path) -> dict:
    """Load and sanity-check verification evidence from verify_release.py."""
    evidence = common.read_json(path)
    if evidence.get("schema_version") != 1:
        raise NpmPackagingError("Unsupported verification evidence schema_version")
    version = evidence.get("version")
    common.parse_npm_version(version)
    if evidence.get("tag") != f"v{version}":
        raise NpmPackagingError("Evidence tag does not match its version")
    release = evidence.get("release", {})
    if release.get("immutable") is not True:
        raise NpmPackagingError("Evidence does not attest an immutable release")
    if release.get("draft") is not False:
        raise NpmPackagingError("Evidence references a draft release")
    commit = evidence.get("commit")
    if not isinstance(commit, str) or not common.COMMIT_SHA_RE.match(commit):
        raise NpmPackagingError("Evidence has no resolved 40-character commit SHA")
    if evidence.get("cargo_version") != version:
        raise NpmPackagingError("Evidence Cargo version does not match the release version")
    return evidence


def stage_all(
    evidence: dict,
    downloads_dir: Path,
    out_dir: Path,
    npm_dir: Path,
    license_path: Path,
    runner: CommandRunner,
) -> dict:
    """Stage and pack all five packages, returning a deterministic staging report."""
    version = evidence["version"]
    parsed = common.parse_npm_version(version)
    targets = common.load_npm_targets(Path(npm_dir).parent / "matrix.jsonc")
    downloads_dir, out_dir = Path(downloads_dir), Path(out_dir)
    license_text = Path(license_path).read_text(encoding="utf-8")
    tarball_dir = out_dir / "tarballs"
    packages_dir = out_dir / "packages"
    packages_dir.mkdir(parents=True, exist_ok=True)

    platform_reports = []
    for target in targets:
        platform = target.platform
        archive_name = platform.archive_name(version)
        archive_path = downloads_dir / archive_name
        expected_sha = evidence["assets"][archive_name]["sha256"]
        binary_data, extraction = extract_binary(archive_path, expected_sha)
        inspection = inspect_binary(binary_data, platform)
        package_dir = stage_platform_package(
            packages_dir, target, version, binary_data, license_text
        )
        dry_record = npm_pack(
            runner, package_dir, platform.package_name, version, PLATFORM_FILES, dry_run=True
        )
        record = npm_pack(
            runner, package_dir, platform.package_name, version, PLATFORM_FILES, dry_run=False
        )
        tarball_path = _move_tarball(package_dir, record, tarball_dir)
        tarball = inspect_npm_tarball(tarball_path, platform.package_name, PLATFORM_FILES)
        if tarball["sha512_integrity"] != record["integrity"]:
            raise NpmPackagingError(
                f"{platform.package_name}: tarball integrity differs from npm pack record"
            )
        platform_reports.append(
            {
                "package_name": platform.package_name,
                "version": version,
                "os": platform.npm_os,
                "cpu": platform.npm_cpu,
                "rust_target": platform.rust_target,
                "builder": target.builder,
                "runner": target.runner,
                "archive": {"name": archive_name, "sha256": expected_sha},
                "extraction": extraction,
                "binary": inspection,
                "tarball": {**tarball, "path": str(tarball_path)},
                "pack_dry_run": {
                    "entryCount": dry_record.get("entryCount"),
                    "unpackedSize": dry_record.get("unpackedSize"),
                },
            }
        )

    base_dir = stage_base_package(packages_dir, version, targets, Path(npm_dir), license_text)
    base_name = common.BASE_PACKAGE
    npm_pack(runner, base_dir, base_name, version, BASE_FILES, dry_run=True)
    base_record = npm_pack(runner, base_dir, base_name, version, BASE_FILES, dry_run=False)
    base_tarball_path = _move_tarball(base_dir, base_record, tarball_dir)
    base_tarball = inspect_npm_tarball(base_tarball_path, base_name, BASE_FILES)
    if base_tarball["sha512_integrity"] != base_record["integrity"]:
        raise NpmPackagingError(f"{base_name}: tarball integrity differs from npm pack record")
    launcher_path = base_dir / "bin" / "poof.js"
    report = {
        "schema_version": 1,
        "version": version,
        "tag": parsed.tag,
        "dist_tag": parsed.dist_tag,
        "commit": evidence["commit"],
        "release_id": evidence["release"]["id"],
        "host": common.host_fingerprint(),
        "platforms": platform_reports,
        "base": {
            "package_name": base_name,
            "version": version,
            "optional_dependencies": {
                target.platform.package_name: version for target in targets
            },
            "launcher": {
                "path": "bin/poof.js",
                "sha256": common.sha256_file(launcher_path),
                "mode": f"{launcher_path.stat().st_mode & 0o777:o}",
            },
            "tarball": {**base_tarball, "path": str(base_tarball_path)},
        },
    }
    common.ensure_no_placeholders(report, "staging report")
    return report


def main(argv: list[str] | None = None) -> int:
    """CLI entry point for staging npm packages from verified release assets."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True, help="verify_release.py output")
    parser.add_argument("--downloads", type=Path, required=True, help="verified asset directory")
    parser.add_argument("--out", type=Path, required=True, help="staging output directory")
    parser.add_argument("--npm-dir", type=Path, default=Path("npm"), help="launcher source dir")
    parser.add_argument("--license", type=Path, default=Path("LICENSE.md"), help="LICENSE.md path")
    parser.add_argument("--report", type=Path, help="write the staging report here")
    args = parser.parse_args(argv)
    try:
        evidence = load_evidence(args.evidence)
        report = stage_all(
            evidence,
            args.downloads,
            args.out,
            args.npm_dir,
            args.license,
            SubprocessRunner(),
        )
    except (NpmPackagingError, OSError, subprocess.SubprocessError) as error:
        print(f"stage_packages: {common.redact(str(error))}", file=sys.stderr)
        return 1
    destination = args.report or (args.out / "staging-report.json")
    common.write_json(destination, report)
    print(f"Staged 5 packages for {report['tag']} -> {destination}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
