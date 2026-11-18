#!/usr/bin/env python3
"""Shared constants, version parsing, and target mapping for poof npm packaging.

Standard library only (Python >= 3.11). ``SUPPORTED_TRIPLES`` below decides
which Rust triples are published to npm, so the repository's existing
``matrix.jsonc`` stays a purely native build matrix with no npm-specific
entries. Package names, os, and cpu are derived from the Rust triple; the
matrix supplies only the builder and runner the native release used.
"""

from __future__ import annotations

import base64
import binascii
import hashlib
import json
import platform as host_platform
import re
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REPOSITORY = "pirafrank/poof"
SCOPE = "@pirafrank"
BASE_PACKAGE = f"{SCOPE}/poof"
BOOTSTRAP_NPM_VERSION = "0.0.1-bootstrap.0"
BINARY_NAME = "poof"
NPM_REGISTRY = "https://registry.npmjs.org"
GITHUB_SERVER_URL = "https://github.com"
SIGNER_WORKFLOW = ".github/workflows/release.yml"
OIDC_ISSUER = "https://token.actions.githubusercontent.com"
SPDX_PREDICATE_TYPE = "https://spdx.dev/Document/v2.3"
BUILD_PROVENANCE_PREDICATE_TYPE = "https://slsa.dev/provenance/v1"
LICENSE = "MIT"
REPOSITORY_URL = f"git+{GITHUB_SERVER_URL}/{REPOSITORY}.git"
HOMEPAGE = f"{GITHUB_SERVER_URL}/{REPOSITORY}"
NODE_ENGINE = ">=18"
DESCRIPTION = (
    "Easy to use zero-config, zero-install, zero-dependencies manager of pre-built "
    "software that works like magic"
)

# Validation oracle only: the four published (npm os, npm cpu) combinations and
# the Rust triple that must produce them. Linux uses static musl; no glibc, no
# Windows, no 32-bit or exotic architectures.
SUPPORTED_TRIPLES = {
    "x86_64-unknown-linux-musl": ("linux", "x64"),
    "aarch64-unknown-linux-musl": ("linux", "arm64"),
    "x86_64-apple-darwin": ("darwin", "x64"),
    "aarch64-apple-darwin": ("darwin", "arm64"),
}
CANONICAL_TRIPLE_ORDER = tuple(SUPPORTED_TRIPLES)
NPM_OS_VALUES = frozenset({"linux", "darwin"})
NPM_CPU_VALUES = frozenset({"x64", "arm64"})

# Observed layout of the native release archives (flat, "./"-prefixed members).
DOC_FILENAMES = frozenset({"LICENSE.md", "README.md", "CHANGELOG.md", "SECURITY.md"})
MAX_ARCHIVE_MEMBERS = 32
MAX_ARCHIVE_UNPACKED_BYTES = 512 * 1024 * 1024
MAX_BINARY_BYTES = 256 * 1024 * 1024
EXECUTABLE_MODE = 0o755

SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
SHA256_PREFIXED_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
COMMIT_SHA_RE = re.compile(r"^[0-9a-f]{40}$")
PACKAGE_NAME_RE = re.compile(r"^@pirafrank/poof-(linux|darwin)-(x64|arm64)$")

# Official SemVer 2.0.0 grammar. Build metadata is captured so it can be
# rejected explicitly: npm versions must not carry a "+build" suffix.
_SEMVER_RE = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)"
    r"(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?"
    r"(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$"
)
_PLACEHOLDER_MARKERS = ("replace-me", "placeholder", "todo", "fixme", "template")
_SECRET_RE = re.compile(
    r"(gh[pousr]_[A-Za-z0-9]{16,}|github_pat_[A-Za-z0-9_]{16,}|npm_[A-Za-z0-9]{16,}"
    r"|Bearer\s+[A-Za-z0-9._\-]{16,})"
)


class NpmPackagingError(Exception):
    """Raised when npm packaging input violates a safety or consistency rule."""


@dataclass(frozen=True)
class Platform:
    """One published npm platform package and its native source archive."""

    rust_target: str
    npm_os: str
    npm_cpu: str

    @property
    def package_name(self) -> str:
        """Scoped npm package name for this platform."""
        return f"{SCOPE}/poof-{self.npm_os}-{self.npm_cpu}"

    @property
    def package_dirname(self) -> str:
        """Filesystem-safe staging directory name (no scope separator)."""
        return f"poof-{self.npm_os}-{self.npm_cpu}"

    def archive_name(self, version: str) -> str:
        """Native release archive filename for this target."""
        return f"{BINARY_NAME}-{version}-{self.rust_target}.tar.gz"

    def spdx_name(self, version: str) -> str:
        """Per-target SPDX asset filename as published by the native workflow."""
        return f"{BINARY_NAME}-{version}-{self.rust_target}.spdx.json"

    def build_environment_name(self, version: str) -> str:
        """Per-target build-environment record filename (observed naming)."""
        return f"{BINARY_NAME}-{version}-{self.rust_target}.build-environment.json"


@dataclass(frozen=True)
class NpmTarget:
    """A published npm platform package, with its native builder and runner."""

    platform: Platform
    builder: str
    runner: str


@dataclass(frozen=True)
class Version:
    """A validated product version shared by the tag, Cargo, and all packages."""

    text: str
    prerelease: str | None

    @property
    def tag(self) -> str:
        """Git tag form, always prefixed with ``v``."""
        return f"v{self.text}"

    @property
    def is_prerelease(self) -> bool:
        """True when the version carries a SemVer prerelease segment."""
        return self.prerelease is not None

    @property
    def dist_tag(self) -> str:
        """npm dist-tag: ``next`` for prereleases, ``latest`` for stable only."""
        return "next" if self.is_prerelease else "latest"


def parse_npm_version(text: str) -> Version:
    """Parse a strict npm version (no leading ``v``, no build metadata)."""
    if not isinstance(text, str) or not text:
        raise NpmPackagingError("npm version must be a non-empty string")
    match = _SEMVER_RE.match(text)
    if match is None:
        raise NpmPackagingError(f"Invalid npm version (expected SemVer without 'v'): {text!r}")
    major, minor, patch, prerelease, build = match.groups()
    if build is not None:
        raise NpmPackagingError(f"npm versions must not carry build metadata: {text!r}")
    if (major, minor, patch) == ("0", "0", "0"):
        raise NpmPackagingError(f"Placeholder version is not publishable: {text!r}")
    lowered = text.lower()
    if any(marker in lowered for marker in _PLACEHOLDER_MARKERS):
        raise NpmPackagingError(f"Placeholder version is not publishable: {text!r}")
    return Version(text=text, prerelease=prerelease)


def parse_tag(tag: str) -> Version:
    """Parse a ``v<semver>`` release tag into the shared npm version."""
    if not isinstance(tag, str) or not tag.startswith("v"):
        raise NpmPackagingError(f"Release tag must start with 'v': {tag!r}")
    return parse_npm_version(tag[1:])


def version_key(version: str) -> tuple:
    """Return a sort key implementing SemVer 2.0.0 precedence rules."""
    parsed = parse_npm_version(version)
    major, minor, patch = (int(part) for part in parsed.text.split("-", 1)[0].split("."))
    if parsed.prerelease is None:
        # A release version always outranks any of its prereleases.
        return (major, minor, patch, 1, ())
    identifiers: list[tuple[int, Any]] = []
    for identifier in parsed.prerelease.split("."):
        if identifier.isdigit():
            identifiers.append((0, int(identifier)))
        else:
            identifiers.append((1, identifier))
    return (major, minor, patch, 0, tuple(identifiers))


def compare_versions(left: str, right: str) -> int:
    """Compare two npm versions: -1, 0, or 1 by SemVer precedence."""
    left_key, right_key = version_key(left), version_key(right)
    if left_key < right_key:
        return -1
    if left_key > right_key:
        return 1
    return 0


def cargo_version_from_text(text: str) -> str:
    """Read ``[package].version`` from Cargo.toml content (tomllib, no regex)."""
    try:
        manifest = tomllib.loads(text)
    except tomllib.TOMLDecodeError as error:
        raise NpmPackagingError(f"Cargo.toml is not valid TOML: {error}") from error
    version = manifest.get("package", {}).get("version")
    if not isinstance(version, str):
        raise NpmPackagingError("Cargo.toml is missing [package].version")
    parse_npm_version(version)
    return version


def cargo_version_from_file(path: Path) -> str:
    """Read the product version from a Cargo.toml checked out at a commit."""
    return cargo_version_from_text(Path(path).read_text(encoding="utf-8"))


def strip_jsonc_comments(text: str) -> str:
    """Drop whole-line ``//`` comments, matching the native matrix readers.

    ``release.yml`` pipes the file through ``grep -v '//'`` and
    ``build_environment.py`` drops lines whose stripped form starts with ``//``.
    An inline ``//`` (for example inside a URL) would be silently deleted by the
    former, so it is rejected here to keep both native readers working.
    """
    kept = []
    for number, line in enumerate(text.splitlines(), start=1):
        if "//" in line:
            if not line.lstrip().startswith("//"):
                raise NpmPackagingError(
                    f"matrix.jsonc line {number} has an inline '//' comment; the native "
                    "release workflow strips whole lines only"
                )
            continue
        kept.append(line)
    return "\n".join(kept)


def load_matrix(path: Path) -> dict:
    """Load and structurally validate the repository release matrix."""
    text = Path(path).read_text(encoding="utf-8")
    try:
        matrix = json.loads(strip_jsonc_comments(text))
    except json.JSONDecodeError as error:
        raise NpmPackagingError(f"matrix.jsonc is not valid JSON after comment removal: {error}")
    if not isinstance(matrix, dict) or not isinstance(matrix.get("include"), list):
        raise NpmPackagingError("matrix.jsonc must contain an 'include' array")
    if not matrix["include"]:
        raise NpmPackagingError("matrix.jsonc 'include' array is empty")
    for entry in matrix["include"]:
        if not isinstance(entry, dict) or not isinstance(entry.get("target"), str):
            raise NpmPackagingError(f"Invalid matrix entry: {entry!r}")
    return matrix


def npm_targets(matrix: dict) -> list[NpmTarget]:
    """Return exactly the four npm-publishable matrix targets, canonically ordered.

    Which triples are published is decided by ``SUPPORTED_TRIPLES``, never by
    the matrix, so an ``npm`` key in ``matrix.jsonc`` is rejected rather than
    honoured. Every supported triple must appear exactly once and carry a valid
    builder and runner; all other native targets are left untouched.
    """
    entries = matrix.get("include")
    if not isinstance(entries, list):
        raise NpmPackagingError("matrix must contain an 'include' array")
    found: dict[str, NpmTarget] = {}
    for entry in entries:
        target = entry.get("target")
        if "npm" in entry:
            raise NpmPackagingError(
                f"Matrix entry {target!r} carries an 'npm' key; npm platforms are derived "
                "from the Rust triple and must not be configured in matrix.jsonc"
            )
        if target not in SUPPORTED_TRIPLES:
            continue
        if target in found:
            raise NpmPackagingError(f"Duplicate matrix entry for npm target {target!r}")
        npm_os, npm_cpu = SUPPORTED_TRIPLES[target]
        platform = Platform(rust_target=target, npm_os=npm_os, npm_cpu=npm_cpu)
        # Guards SUPPORTED_TRIPLES itself: a bad os/cpu pair would otherwise
        # derive a package name that was never reserved on the registry.
        if npm_os not in NPM_OS_VALUES or npm_cpu not in NPM_CPU_VALUES:
            raise NpmPackagingError(f"Unsupported npm platform {npm_os}/{npm_cpu} for {target!r}")
        if not PACKAGE_NAME_RE.match(platform.package_name):
            raise NpmPackagingError(f"Invalid npm package name: {platform.package_name!r}")
        builder, runner = entry.get("builder"), entry.get("os")
        if builder not in ("cargo", "cross"):
            raise NpmPackagingError(f"Invalid builder {builder!r} for {target!r}")
        if not isinstance(runner, str) or not runner:
            raise NpmPackagingError(f"Invalid runner label for {target!r}")
        found[target] = NpmTarget(platform=platform, builder=builder, runner=runner)
    missing = [triple for triple in CANONICAL_TRIPLE_ORDER if triple not in found]
    if missing:
        raise NpmPackagingError(f"Missing npm target entries in matrix.jsonc: {missing}")
    return [found[triple] for triple in CANONICAL_TRIPLE_ORDER]


def load_npm_targets(matrix_path: Path) -> list[NpmTarget]:
    """Convenience loader: read the matrix file and return its npm targets."""
    return npm_targets(load_matrix(matrix_path))


def checksums_manifest_name(version: str) -> str:
    """Aggregate signed checksum manifest filename for a release."""
    return f"{BINARY_NAME}-{version}-checksums.txt"


def sigstore_bundle_name(version: str) -> str:
    """Sigstore bundle filename accompanying the checksum manifest."""
    return f"{checksums_manifest_name(version)}.sigstore.json"


def signer_identity(tag: str) -> str:
    """Expected cosign certificate identity for the native release workflow."""
    return f"{GITHUB_SERVER_URL}/{REPOSITORY}/{SIGNER_WORKFLOW}@refs/tags/{tag}"


def sha256_bytes(data: bytes) -> str:
    """Return the hex SHA-256 digest of bytes."""
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    """Return the hex SHA-256 digest of a file, read in bounded chunks."""
    digest = hashlib.sha256()
    with Path(path).open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def sha512_integrity_bytes(data: bytes) -> str:
    """Return the npm Subresource Integrity string for bytes."""
    return "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode("ascii")


def sha512_integrity_file(path: Path) -> str:
    """Return the npm Subresource Integrity string for a tarball on disk."""
    digest = hashlib.sha512()
    with Path(path).open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return "sha512-" + base64.b64encode(digest.digest()).decode("ascii")


def dump_json(value: Any) -> str:
    """Serialize deterministically: sorted keys, two-space indent, newline."""
    return json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n"


def write_json(path: Path, value: Any) -> None:
    """Write deterministic JSON, creating parent directories as needed."""
    destination = Path(path)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(dump_json(value), encoding="utf-8")


def read_json(path: Path) -> Any:
    """Read a JSON document from disk."""
    return json.loads(Path(path).read_text(encoding="utf-8"))


def decode_base64(content: str, label: str) -> bytes:
    """Decode base64 that may be line-wrapped, rejecting any other invalid input.

    The GitHub contents API wraps base64 at 60 characters, so whitespace is
    removed before strict validation; non-alphabet characters still fail.
    """
    if not isinstance(content, str):
        raise NpmPackagingError(f"{label} is not a base64 string")
    try:
        return base64.b64decode("".join(content.split()), validate=True)
    except (binascii.Error, ValueError) as error:
        raise NpmPackagingError(f"{label} is not valid base64: {error}") from error


def redact(text: str) -> str:
    """Remove credential-shaped substrings from evidence text."""
    redacted = _SECRET_RE.sub("<redacted>", text)
    return re.sub(
        r"-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
        "<redacted-private-key>",
        redacted,
        flags=re.DOTALL,
    )


def ensure_no_placeholders(value: Any, where: str = "manifest") -> None:
    """Reject placeholder markers and credential-shaped strings in output."""
    if isinstance(value, dict):
        for key, item in value.items():
            ensure_no_placeholders(item, f"{where}.{key}")
    elif isinstance(value, list):
        for index, item in enumerate(value):
            ensure_no_placeholders(item, f"{where}[{index}]")
    elif isinstance(value, str):
        lowered = value.lower()
        for marker in _PLACEHOLDER_MARKERS:
            if marker in lowered:
                raise NpmPackagingError(f"Placeholder marker {marker!r} in {where}: {value!r}")
        if _SECRET_RE.search(value):
            raise NpmPackagingError(f"Credential-shaped value in {where}")


def host_fingerprint() -> dict:
    """Describe the host performing checks, for honest provenance records."""
    return {
        "system": host_platform.system(),
        "machine": host_platform.machine(),
        "python_version": host_platform.python_version(),
    }
