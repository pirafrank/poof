"""Tests for archive safety, binary gates, manifest rendering, and packing."""

from __future__ import annotations

import copy
import json
import tarfile
import tempfile
import unittest
from pathlib import Path

from _support import (
    REPO_ROOT,
    FakePackRunner,
    build_elf,
    build_fat_macho,
    build_macho,
    common,
    completed,
    make_tar_gz,
    native_archive_members,
    synthetic_evidence,
    tar_info,
)
from npm import stage_packages as sp
from npm.common import NpmPackagingError

NPM_DIR = REPO_ROOT / "npm"
TARGETS = common.load_npm_targets(REPO_ROOT / "matrix.jsonc")
LINUX_X64 = next(t for t in TARGETS if t.platform.package_dirname == "poof-linux-x64")
LINUX_ARM64 = next(t for t in TARGETS if t.platform.package_dirname == "poof-linux-arm64")
DARWIN_X64 = next(t for t in TARGETS if t.platform.package_dirname == "poof-darwin-x64")
DARWIN_ARM64 = next(t for t in TARGETS if t.platform.package_dirname == "poof-darwin-arm64")


def binary_for(target):
    """A structurally valid synthetic executable for an npm target."""
    if target.platform.npm_os == "linux":
        machine = sp.EM_X86_64 if target.platform.npm_cpu == "x64" else sp.EM_AARCH64
        # The real x64 musl build is a static PIE: ET_DYN with PT_DYNAMIC but
        # zero DT_NEEDED. Reproduce that shape so the gate is tested honestly.
        return build_elf(
            machine=machine,
            elf_type=sp.ET_DYN if target.platform.npm_cpu == "x64" else sp.ET_EXEC,
            pt_dynamic=target.platform.npm_cpu == "x64",
        )
    cpu = sp.CPU_TYPE_X86_64 if target.platform.npm_cpu == "x64" else sp.CPU_TYPE_ARM64
    return build_macho(cputype=cpu)


class MemberNameTests(unittest.TestCase):
    """Archive member names must be flat, relative, and traversal-free."""

    def test_accepts_flat_names(self):
        for raw, expected in (("./poof", "poof"), ("poof", "poof"), ("./LICENSE.md", "LICENSE.md")):
            with self.subTest(raw=raw):
                self.assertEqual(sp.normalize_member_name(raw), expected)

    def test_rejects_unsafe_names(self):
        for raw in (
            "",
            None,
            "poof\x00evil",
            "sub\\poof",
            "../poof",
            "/poof",
            "./../poof",
            "././poof",
            "C:poof",
            "c:/poof",
            "sub/poof",
            "./",
            ".",
            "..",
            ".hidden",
            "a//b",
            "poof/.",
        ):
            with self.subTest(raw=raw):
                with self.assertRaises(NpmPackagingError):
                    sp.normalize_member_name(raw)


class ValidateMembersTests(unittest.TestCase):
    """The full member list is validated before anything is extracted."""

    def members(self, extra=(), binary_mode=0o755, binary_size=4096):
        base = [tar_info("./", kind=tarfile.DIRTYPE)]
        base.append(tar_info("./poof", mode=binary_mode, size=binary_size))
        base.extend(tar_info(f"./{name}") for name in sorted(common.DOC_FILENAMES))
        return base + list(extra)

    def test_accepts_real_layout(self):
        member = sp.validate_members(self.members(), "test.tar.gz")
        self.assertEqual(member.name, "./poof")

    def test_rejects_duplicate_binary(self):
        # The duplicate-name guard fires before the binary-specific guard.
        with self.assertRaisesRegex(NpmPackagingError, "duplicate archive member 'poof'"):
            sp.validate_members(self.members([tar_info("./poof", size=10)]), "a")

    def test_rejects_duplicate_root_directory(self):
        with self.assertRaisesRegex(NpmPackagingError, "duplicate root directory"):
            sp.validate_members(self.members([tar_info("./", kind=tarfile.DIRTYPE)]), "a")

    def test_rejects_duplicate_document(self):
        with self.assertRaisesRegex(NpmPackagingError, "duplicate archive member"):
            sp.validate_members(self.members([tar_info("./LICENSE.md")]), "a")

    def test_rejects_links_and_devices(self):
        cases = (
            (tarfile.SYMTYPE, "symlink"),
            (tarfile.LNKTYPE, "hardlink"),
            (tarfile.CHRTYPE, "special file"),
            (tarfile.BLKTYPE, "special file"),
            (tarfile.FIFOTYPE, "special file"),
        )
        for kind, label in cases:
            with self.subTest(kind=kind):
                members = [tar_info("./", kind=tarfile.DIRTYPE)]
                members.append(tar_info("./poof", kind=kind, linkname="/etc/passwd"))
                with self.assertRaisesRegex(NpmPackagingError, label):
                    sp.validate_members(members, "a")

    def test_rejects_nested_directory(self):
        with self.assertRaisesRegex(NpmPackagingError, "nested directory"):
            sp.validate_members(self.members([tar_info("./sub", kind=tarfile.DIRTYPE)]), "a")

    def test_rejects_nested_path_with_trailing_slash(self):
        with self.assertRaisesRegex(NpmPackagingError, "Unsafe archive member path component"):
            sp.validate_members(self.members([tar_info("./sub/", kind=tarfile.DIRTYPE)]), "a")

    def test_rejects_extra_executable(self):
        members = [tar_info("./", kind=tarfile.DIRTYPE), tar_info("./poof", size=4096)]
        members.append(tar_info("./LICENSE.md", mode=0o755))
        with self.assertRaisesRegex(NpmPackagingError, "unexpected executable member"):
            sp.validate_members(members, "a")

    def test_rejects_unexpected_member(self):
        with self.assertRaisesRegex(NpmPackagingError, "unexpected archive members"):
            sp.validate_members(self.members([tar_info("./extra.txt")]), "a")

    def test_rejects_missing_binary(self):
        members = [tar_info("./", kind=tarfile.DIRTYPE)]
        members.extend(tar_info(f"./{name}") for name in sorted(common.DOC_FILENAMES))
        with self.assertRaisesRegex(NpmPackagingError, "no root 'poof'"):
            sp.validate_members(members, "a")

    def test_rejects_too_many_members(self):
        extra = [tar_info(f"./file{index}.txt") for index in range(common.MAX_ARCHIVE_MEMBERS)]
        with self.assertRaisesRegex(NpmPackagingError, "too many archive members"):
            sp.validate_members(self.members(extra), "a")

    def test_rejects_implausible_binary_sizes(self):
        for size in (0, common.MAX_BINARY_BYTES + 1):
            with self.subTest(size=size):
                with self.assertRaisesRegex(NpmPackagingError, "implausible executable size"):
                    sp.validate_members(self.members(binary_size=size), "a")

    def test_rejects_excessive_total_size(self):
        extra = [
            tar_info(f"./{name}", size=common.MAX_ARCHIVE_UNPACKED_BYTES)
            for name in sorted(common.DOC_FILENAMES)
        ]
        members = [tar_info("./", kind=tarfile.DIRTYPE), tar_info("./poof", size=4096)] + extra
        with self.assertRaisesRegex(NpmPackagingError, "unpacked size exceeds"):
            sp.validate_members(members, "a")


class ExtractBinaryTests(unittest.TestCase):
    """Only the root executable is read, and only after digest verification."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.addCleanup(self._tmp.cleanup)

    def archive(self, members, name="poof-0.6.3-x86_64-unknown-linux-musl.tar.gz"):
        path = make_tar_gz(self.tmp / name, members)
        return path, common.sha256_file(path)

    def test_extracts_only_the_binary(self):
        data = build_elf()
        path, digest = self.archive(native_archive_members(data))
        extracted, evidence = sp.extract_binary(path, digest)
        self.assertEqual(extracted, data)
        self.assertEqual(evidence["archive_sha256"], digest)
        self.assertEqual(evidence["member_name"], "./poof")
        self.assertEqual(evidence["member_mode"], "755")
        self.assertEqual(evidence["member_count"], 6)
        self.assertEqual(evidence["doc_members"], sorted(common.DOC_FILENAMES))

    def test_rejects_digest_mismatch(self):
        path, _ = self.archive(native_archive_members(build_elf()))
        with self.assertRaisesRegex(NpmPackagingError, "SHA-256 mismatch"):
            sp.extract_binary(path, "0" * 64)

    def test_source_archive_permissions_are_untouched(self):
        path, digest = self.archive(native_archive_members(build_elf()))
        path.chmod(0o644)
        sp.extract_binary(path, digest)
        self.assertEqual(path.stat().st_mode & 0o777, 0o644)

    def test_rejects_symlink_member(self):
        members = native_archive_members(build_elf())
        members.append(
            {"name": "./evil", "type": tarfile.SYMTYPE, "linkname": "/etc/passwd", "mode": 0o777}
        )
        path, digest = self.archive(members)
        with self.assertRaisesRegex(NpmPackagingError, "symlink"):
            sp.extract_binary(path, digest)

    def test_rejects_traversal_member(self):
        members = native_archive_members(build_elf())
        members.append({"name": "../evil", "data": b"x", "mode": 0o644})
        path, digest = self.archive(members)
        with self.assertRaises(NpmPackagingError):
            sp.extract_binary(path, digest)

    def test_rejects_document_with_executable_magic(self):
        members = native_archive_members(build_elf())
        for member in members:
            if member["name"] == "./README.md":
                member["data"] = build_elf()
        path, digest = self.archive(members)
        with self.assertRaisesRegex(NpmPackagingError, "executable/archive magic"):
            sp.extract_binary(path, digest)

    def test_rejects_oversized_document(self):
        members = native_archive_members(build_elf())
        for member in members:
            if member["name"] == "./README.md":
                member["data"] = b"\x00" * (sp.MAX_DOC_BYTES + 1)
        path, digest = self.archive(members)
        with self.assertRaisesRegex(NpmPackagingError, "implausibly large"):
            sp.extract_binary(path, digest)


class InspectBinaryTests(unittest.TestCase):
    """ELF/Mach-O format, CPU, and static-musl gates."""

    def test_static_elf_exec(self):
        evidence = sp.inspect_binary(build_elf(), LINUX_X64.platform)
        self.assertEqual(evidence["format"], "elf64")
        self.assertEqual(evidence["elf_type"], "ET_EXEC")
        self.assertTrue(evidence["static"])
        self.assertEqual(evidence["libc"], "musl (static)")
        self.assertEqual(evidence["mode"], "755")
        self.assertFalse(evidence["has_interpreter"])
        self.assertEqual(evidence["dt_needed_count"], 0)

    def test_static_pie_elf_is_accepted(self):
        # ET_DYN + PT_DYNAMIC with zero DT_NEEDED is a static PIE, not dynamic.
        evidence = sp.inspect_binary(build_elf(elf_type=sp.ET_DYN, pt_dynamic=True), LINUX_X64.platform)
        self.assertEqual(evidence["elf_type"], "ET_DYN")
        self.assertTrue(evidence["has_pt_dynamic"])
        self.assertTrue(evidence["static"])

    def test_arm64_elf(self):
        evidence = sp.inspect_binary(build_elf(machine=sp.EM_AARCH64), LINUX_ARM64.platform)
        self.assertEqual(evidence["machine"], sp.EM_AARCH64)
        self.assertTrue(evidence["static"])

    def test_rejects_dynamic_linking(self):
        cases = (
            ({"interpreter": "/lib64/ld-linux-x86-64.so.2"}, "dynamically linked"),
            ({"needed": (1, 2)}, "DT_NEEDED"),
            ({"glibc": True}, "GLIBC symbol versions"),
            ({"pt_dynamic": True, "omit_sections": True}, "cannot be proven"),
        )
        for kwargs, pattern in cases:
            with self.subTest(kwargs=kwargs):
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    sp.inspect_binary(build_elf(**kwargs), LINUX_X64.platform)

    def test_rejects_wrong_elf_machine(self):
        with self.assertRaisesRegex(NpmPackagingError, "does not match expected"):
            sp.inspect_binary(build_elf(machine=sp.EM_AARCH64), LINUX_X64.platform)

    def test_rejects_elf_for_darwin_and_macho_for_linux(self):
        with self.assertRaisesRegex(NpmPackagingError, "non-Linux"):
            sp.inspect_binary(build_elf(), DARWIN_X64.platform)
        with self.assertRaisesRegex(NpmPackagingError, "non-macOS"):
            sp.inspect_binary(build_macho(), LINUX_X64.platform)

    def test_rejects_malformed_elf(self):
        cases = (
            (build_elf()[:40], "truncated ELF header"),
            (build_elf(elf_class=1), "only ELF64"),
            (build_elf(endian_flag=2), "little-endian"),
            (build_elf(elf_type=1), "not an executable"),
        )
        for data, pattern in cases:
            with self.subTest(pattern=pattern):
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    sp.inspect_binary(data, LINUX_X64.platform)

    def test_macho_cpu_types(self):
        for target, cpu in ((DARWIN_X64, "0x1000007"), (DARWIN_ARM64, "0x100000c")):
            with self.subTest(target=target.platform.package_name):
                expected = sp.CPU_TYPE_X86_64 if cpu == "0x1000007" else sp.CPU_TYPE_ARM64
                evidence = sp.inspect_binary(build_macho(cputype=expected), target.platform)
                self.assertEqual(evidence["format"], "mach-o64")
                self.assertEqual(evidence["cputype"], cpu)
                self.assertEqual(evidence["filetype"], "MH_EXECUTE")
                self.assertIsNone(evidence["static"])

    def test_rejects_wrong_macho_cpu(self):
        with self.assertRaisesRegex(NpmPackagingError, "does not match expected"):
            sp.inspect_binary(build_macho(cputype=sp.CPU_TYPE_ARM64), DARWIN_X64.platform)

    def test_rejects_unsupported_macho_variants(self):
        cases = (
            (build_fat_macho(), "universal"),
            (build_macho(magic=sp.MH_MAGIC_32), "only 64-bit"),
            (build_macho(filetype=6), "not MH_EXECUTE"),
            (build_macho()[:16], "truncated Mach-O"),
            (b"\x00\x01\x02\x03" + b"\x00" * 64, "unrecognized executable format"),
        )
        for data, pattern in cases:
            with self.subTest(pattern=pattern):
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    sp.inspect_binary(data, DARWIN_X64.platform)

    def test_macho_inspector_rejects_unknown_magic_directly(self):
        with self.assertRaisesRegex(NpmPackagingError, "unrecognized Mach-O magic"):
            sp._inspect_macho(b"\x00\x01\x02\x03" + b"\x00" * 64, sp.CPU_TYPE_X86_64, "label")

    def test_accepts_big_endian_macho_header(self):
        # A big-endian file stores MH_MAGIC_64 byte-swapped, which reads back as
        # MH_CIGAM_64 when interpreted little-endian.
        data = build_macho(magic=sp.MH_MAGIC_64, endian=">")
        evidence = sp.inspect_binary(data, DARWIN_X64.platform)
        self.assertEqual(evidence["endianness"], "big")

    def test_rejects_unknown_format(self):
        for data in (b"", b"MZ" + b"\x00" * 62, b"#!/bin/sh\n"):
            with self.subTest(data=data[:8]):
                with self.assertRaisesRegex(NpmPackagingError, "unrecognized executable format"):
                    sp.inspect_binary(data, LINUX_X64.platform)


class PackageJsonTests(unittest.TestCase):
    """Manifest rendering: no bin on platforms, exact pins on the launcher."""

    def test_platform_manifest(self):
        manifest = sp.platform_package_json(LINUX_X64, "0.6.3")
        self.assertEqual(manifest["name"], "@pirafrank/poof-linux-x64")
        self.assertEqual(manifest["version"], "0.6.3")
        self.assertEqual(manifest["os"], ["linux"])
        self.assertEqual(manifest["cpu"], ["x64"])
        self.assertEqual(manifest["files"], ["bin/poof", "LICENSE.md"])
        self.assertEqual(manifest["publishConfig"], {"access": "public"})
        self.assertEqual(manifest["license"], "MIT")
        self.assertNotIn("bin", manifest)
        self.assertNotIn("scripts", manifest)
        self.assertNotIn("dependencies", manifest)
        common.ensure_no_placeholders(manifest, "platform manifest")

    def test_base_manifest_from_real_template(self):
        template = common.read_json(NPM_DIR / "package.json.template")
        manifest = sp.base_package_json(template, "0.6.3", TARGETS)
        self.assertEqual(manifest["name"], "@pirafrank/poof")
        self.assertEqual(manifest["version"], "0.6.3")
        self.assertIs(manifest["private"], False)
        self.assertEqual(manifest["bin"], {"poof": "bin/poof.js"})
        self.assertEqual(manifest["engines"]["node"], ">=18")
        self.assertEqual(
            manifest["optionalDependencies"],
            {target.platform.package_name: "0.6.3" for target in TARGETS},
        )
        for pinned in manifest["optionalDependencies"].values():
            self.assertEqual(pinned, "0.6.3")
        self.assertNotIn("os", manifest)
        self.assertNotIn("cpu", manifest)
        self.assertNotIn("scripts", manifest)
        self.assertNotIn("dependencies", manifest)

    def test_base_manifest_rejects_bad_templates(self):
        good = common.read_json(NPM_DIR / "package.json.template")
        mutations = (
            lambda m: m.update(name="@pirafrank/other"),
            lambda m: m.update(bin={"poof": "bin/other.js"}),
            lambda m: m["engines"].update(node=">=16"),
            lambda m: m["publishConfig"].update(access="restricted"),
            lambda m: m.update(scripts={"postinstall": "node evil.js"}),
            lambda m: m.update(dependencies={"left-pad": "1.0.0"}),
            lambda m: m.update(os=["linux"]),
            lambda m: m.update(cpu=["x64"]),
            lambda m: m["repository"].update(url="git+https://github.com/evil/poof.git"),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(mutation=index):
                template = copy.deepcopy(good)
                mutate(template)
                with self.assertRaises(NpmPackagingError):
                    sp.base_package_json(template, "0.6.3", TARGETS)


class PackOutputTests(unittest.TestCase):
    """Both npm pack JSON shapes are accepted; ambiguity is rejected."""

    def record(self, name="@pirafrank/poof-linux-x64"):
        return {
            "name": name,
            "version": "0.6.3",
            "filename": "pirafrank-poof-linux-x64-0.6.3.tgz",
            "integrity": "sha512-" + "A" * 88,
            "shasum": "0" * 40,
            "files": [{"path": "bin/poof", "size": 10, "mode": 0o755}],
            "entryCount": 1,
        }

    def test_object_shape_npm12(self):
        record = self.record()
        text = json.dumps({record["name"]: record})
        self.assertEqual(sp.parse_pack_output(text, record["name"]), record)

    def test_array_shape_npm10(self):
        record = self.record()
        self.assertEqual(sp.parse_pack_output(json.dumps([record]), record["name"]), record)

    def test_leading_noise_is_tolerated(self):
        record = self.record()
        text = "npm notice something\n" + json.dumps([record])
        self.assertEqual(sp.parse_pack_output(text, record["name"]), record)

    def test_rejects_bad_output(self):
        record = self.record()
        cases = (
            "",
            "   ",
            "no json here",
            "{not json",
            json.dumps([record, record]),
            json.dumps({"a": record, "b": record}),
            json.dumps([self.record("@pirafrank/other")]),
            json.dumps({"@pirafrank/other": self.record("@pirafrank/other")}),
            json.dumps(["string"]),
        )
        for text in cases:
            with self.subTest(text=text[:40]):
                with self.assertRaises(NpmPackagingError):
                    sp.parse_pack_output(text, record["name"])


class AssertPackRecordTests(unittest.TestCase):
    """Pack records are checked against an explicit allowlist and modes."""

    def setUp(self):
        self.record = {
            "name": "@pirafrank/poof-linux-x64",
            "version": "0.6.3",
            "integrity": "sha512-" + "A" * 88,
            "files": [
                {"path": "package.json", "mode": 0o644},
                {"path": "bin/poof", "mode": 0o755},
                {"path": "LICENSE.md", "mode": 0o644},
            ],
            "entryCount": 3,
        }

    def test_accepts_matching_record(self):
        sp.assert_pack_record(self.record, self.record["name"], "0.6.3", sp.PLATFORM_FILES)

    def test_rejects_mismatches(self):
        def mutate(record, change):
            clone = copy.deepcopy(record)
            change(clone)
            return clone

        cases = (
            (lambda r: r.update(version="0.6.4"), "pack version"),
            (lambda r: r.update(integrity="sha1-abc"), "sha512 integrity"),
            (lambda r: r.pop("integrity"), "sha512 integrity"),
            (lambda r: r.update(files="nope"), "no file list"),
            (lambda r: r["files"].append({"path": "extra", "mode": 0o644}), "allowlist mismatch"),
            (lambda r: r["files"].pop(), "allowlist mismatch"),
            (lambda r: r["files"][1].update(mode=0o644), "mode"),
            (lambda r: r["files"].append(dict(r["files"][0])), "duplicate pack file"),
            (lambda r: r.update(entryCount=99), "entryCount"),
            (lambda r: r["files"].append({"mode": 0o644}), "without a path"),
        )
        for change, pattern in cases:
            with self.subTest(pattern=pattern):
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    sp.assert_pack_record(
                        mutate(self.record, change), self.record["name"], "0.6.3", sp.PLATFORM_FILES
                    )


class NpmTarballTests(unittest.TestCase):
    """The produced tarball is inspected independently of npm's own report."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.addCleanup(self._tmp.cleanup)
        self.files = {
            "package.json": (b"{}", 0o644),
            "bin/poof": (b"binary", 0o755),
            "LICENSE.md": (b"MIT", 0o644),
        }

    def tarball(self, members, name="pkg.tgz"):
        return make_tar_gz(self.tmp / name, members)

    def test_accepts_correct_tarball(self):
        from _support import npm_tarball_members

        path = self.tarball(npm_tarball_members(self.files))
        report = sp.inspect_npm_tarball(path, "@pirafrank/poof-linux-x64", sp.PLATFORM_FILES)
        self.assertEqual(
            report["entries"],
            [
                {"path": "LICENSE.md", "mode": "644"},
                {"path": "bin/poof", "mode": "755"},
                {"path": "package.json", "mode": "644"},
            ],
        )
        self.assertEqual(report["sha256"], common.sha256_file(path))
        self.assertEqual(report["sha512_integrity"], common.sha512_integrity_file(path))

    def test_rejects_unsafe_or_unexpected_entries(self):
        from _support import npm_tarball_members

        base = npm_tarball_members(self.files)
        cases = (
            ([{"name": "evil", "data": b"x", "mode": 0o644}], "outside package/"),
            ([{"name": "package/../evil", "data": b"x", "mode": 0o644}], "unsafe tarball entry"),
            (
                [{"name": "package/evil", "type": tarfile.SYMTYPE, "linkname": "/etc/passwd"}],
                "non-regular tarball entry",
            ),
            ([{"name": "package/bin/poof", "data": b"x", "mode": 0o755}], "duplicate tarball entry"),
            ([{"name": "package/extra.txt", "data": b"x", "mode": 0o644}], "allowlist mismatch"),
        )
        for extra, pattern in cases:
            with self.subTest(pattern=pattern):
                path = self.tarball(base + extra, name=f"{abs(hash(pattern))}.tgz")
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    sp.inspect_npm_tarball(path, "@pirafrank/poof-linux-x64", sp.PLATFORM_FILES)

    def test_rejects_wrong_mode(self):
        from _support import npm_tarball_members

        wrong = dict(self.files)
        wrong["bin/poof"] = (b"binary", 0o644)
        path = self.tarball(npm_tarball_members(wrong))
        with self.assertRaisesRegex(NpmPackagingError, "mode"):
            sp.inspect_npm_tarball(path, "@pirafrank/poof-linux-x64", sp.PLATFORM_FILES)


class StagingTests(unittest.TestCase):
    """Staged directories contain only allowed files with correct modes."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.addCleanup(self._tmp.cleanup)

    def test_platform_staging(self):
        package = sp.stage_platform_package(
            self.tmp, LINUX_X64, "0.6.3", build_elf(), "MIT license text\n"
        )
        self.assertEqual(
            sorted(p.relative_to(package).as_posix() for p in package.rglob("*") if p.is_file()),
            ["LICENSE.md", "bin/poof", "package.json"],
        )
        self.assertEqual((package / "bin" / "poof").stat().st_mode & 0o777, 0o755)
        self.assertEqual((package / "package.json").stat().st_mode & 0o777, 0o644)
        self.assertEqual((package / "LICENSE.md").stat().st_mode & 0o777, 0o644)

    def test_staging_refuses_to_overwrite(self):
        sp.stage_platform_package(self.tmp, LINUX_X64, "0.6.3", build_elf(), "MIT\n")
        with self.assertRaisesRegex(NpmPackagingError, "already exists"):
            sp.stage_platform_package(self.tmp, LINUX_X64, "0.6.3", build_elf(), "MIT\n")

    def test_base_staging_does_not_modify_sources(self):
        launcher = NPM_DIR / "bin" / "poof.js"
        original_mode = launcher.stat().st_mode & 0o777
        original_digest = common.sha256_file(launcher)
        package = sp.stage_base_package(self.tmp, "0.6.3", TARGETS, NPM_DIR, "MIT\n")
        self.assertEqual((package / "bin" / "poof.js").stat().st_mode & 0o777, 0o755)
        self.assertEqual(common.sha256_file(package / "bin" / "poof.js"), original_digest)
        self.assertEqual(launcher.stat().st_mode & 0o777, original_mode)
        self.assertEqual(
            sorted(p.relative_to(package).as_posix() for p in package.rglob("*") if p.is_file()),
            ["LICENSE.md", "README.md", "bin/poof.js", "package.json"],
        )
        self.assertNotIn("bin/poof", [p.as_posix() for p in package.rglob("poof")])

    def test_base_staging_requires_sources(self):
        with self.assertRaisesRegex(NpmPackagingError, "Missing launcher source file"):
            sp.stage_base_package(self.tmp, "0.6.3", TARGETS, self.tmp, "MIT\n")


class LoadEvidenceTests(unittest.TestCase):
    """Staging refuses evidence that does not attest an immutable release."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.addCleanup(self._tmp.cleanup)

    def write(self, evidence):
        path = self.tmp / "evidence.json"
        common.write_json(path, evidence)
        return path

    def test_accepts_valid_evidence(self):
        evidence = synthetic_evidence()
        self.assertEqual(sp.load_evidence(self.write(evidence))["version"], "0.6.3")

    def test_rejects_invalid_evidence(self):
        cases = (
            (lambda e: e.update(schema_version=2), "schema_version"),
            (lambda e: e.update(tag="v0.6.4"), "tag does not match"),
            (lambda e: e["release"].update(immutable=False), "immutable"),
            (lambda e: e["release"].update(draft=True), "draft"),
            (lambda e: e.update(commit="abc"), "40-character"),
            (lambda e: e.update(cargo_version="0.6.4"), "Cargo version"),
            (lambda e: e.update(version="0.0.0"), "Placeholder"),
        )
        for change, pattern in cases:
            with self.subTest(pattern=pattern):
                evidence = synthetic_evidence()
                change(evidence)
                with self.assertRaisesRegex(NpmPackagingError, pattern):
                    sp.load_evidence(self.write(evidence))


class StageAllTests(unittest.TestCase):
    """End-to-end offline staging with a faithful npm pack double."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.tmp = Path(self._tmp.name)
        self.addCleanup(self._tmp.cleanup)
        self.downloads = self.tmp / "downloads"
        self.downloads.mkdir()
        self.out = self.tmp / "staged"
        self.evidence = synthetic_evidence()
        for target in TARGETS:
            name = target.platform.archive_name("0.6.3")
            path = make_tar_gz(self.downloads / name, native_archive_members(binary_for(target)))
            self.evidence["assets"][name]["sha256"] = common.sha256_file(path)
        self.runner = FakePackRunner(
            {
                **{t.platform.package_name: sp.PLATFORM_FILES for t in TARGETS},
                common.BASE_PACKAGE: sp.BASE_FILES,
            }
        )

    def stage(self):
        return sp.stage_all(
            self.evidence,
            self.downloads,
            self.out,
            NPM_DIR,
            REPO_ROOT / "LICENSE.md",
            self.runner,
        )

    def test_stages_five_packages(self):
        report = self.stage()
        self.assertEqual(len(report["platforms"]), 4)
        self.assertEqual(report["base"]["package_name"], "@pirafrank/poof")
        self.assertEqual(report["dist_tag"], "latest")
        tarballs = sorted(p.name for p in (self.out / "tarballs").iterdir())
        self.assertEqual(len(tarballs), 5)
        for platform_report in report["platforms"]:
            self.assertEqual(platform_report["binary"]["mode"], "755")
            if platform_report["os"] == "linux":
                self.assertTrue(platform_report["binary"]["static"])
            self.assertEqual(
                platform_report["tarball"]["sha512_integrity"],
                common.sha512_integrity_file(Path(platform_report["tarball"]["path"])),
            )
        # Tarball entry paths are package-relative; absolute host paths live
        # only in the staging report and are stripped from the manifest.
        for entry in report["base"]["tarball"]["entries"]:
            self.assertFalse(entry["path"].startswith("/"))
            self.assertNotIn("..", entry["path"].split("/"))
        # Every npm invocation ran inside a package directory, never a shell.
        for call in self.runner.calls:
            self.assertEqual(call["argv"][:2], ["npm", "pack"])
            self.assertIn("--json", call["argv"])

    def test_separate_release_tree_supplies_package_contents(self):
        import shutil
        source = self.tmp / "release-source"
        shutil.copytree(NPM_DIR, source / "npm")
        shutil.copyfile(REPO_ROOT / "matrix.jsonc", source / "matrix.jsonc")
        launcher = b"// tagged launcher\n"
        (source / "npm/bin/poof.js").write_bytes(launcher)
        (source / "npm/README.md").write_text("Tagged readme\n")
        template = common.read_json(source / "npm/package.json.template")
        template["description"] = "Tagged description"
        common.write_json(source / "npm/package.json.template", template)
        matrix = common.load_matrix(source / "matrix.jsonc")
        for entry in matrix["include"]:
            entry["os"] = "tagged-runner"
        common.write_json(source / "matrix.jsonc", matrix)
        (source / "LICENSE.md").write_text("Tagged license\n")
        report = sp.stage_all(self.evidence, self.downloads, self.out, source / "npm",
                              source / "LICENSE.md", self.runner)
        base = self.out / "packages/poof"
        self.assertEqual((base / "bin/poof.js").read_bytes(), launcher)
        self.assertEqual((base / "README.md").read_text(), "Tagged readme\n")
        self.assertEqual((base / "LICENSE.md").read_text(), "Tagged license\n")
        self.assertEqual(common.read_json(base / "package.json")["description"], "Tagged description")
        self.assertTrue(all(p["runner"] == "tagged-runner" for p in report["platforms"]))

    def test_both_pack_json_shapes_are_exercised(self):
        self.stage()
        self.runner.shape = "array"
        second = self.tmp / "staged-array"
        sp.stage_all(
            self.evidence, self.downloads, second, NPM_DIR, REPO_ROOT / "LICENSE.md", self.runner
        )
        self.assertEqual(len(list((second / "tarballs").iterdir())), 5)

    def test_rejects_tampered_archive(self):
        name = LINUX_X64.platform.archive_name("0.6.3")
        path = self.downloads / name
        path.write_bytes(path.read_bytes() + b"tampered")
        with self.assertRaisesRegex(NpmPackagingError, "SHA-256 mismatch"):
            self.stage()

    def test_rejects_wrong_architecture_binary(self):
        name = LINUX_X64.platform.archive_name("0.6.3")
        path = make_tar_gz(
            self.downloads / name, native_archive_members(build_elf(machine=sp.EM_AARCH64))
        )
        self.evidence["assets"][name]["sha256"] = common.sha256_file(path)
        with self.assertRaisesRegex(NpmPackagingError, "does not match expected"):
            self.stage()

    def test_rejects_glibc_binary_even_with_valid_digest(self):
        name = LINUX_ARM64.platform.archive_name("0.6.3")
        path = make_tar_gz(
            self.downloads / name,
            native_archive_members(build_elf(machine=sp.EM_AARCH64, glibc=True)),
        )
        self.evidence["assets"][name]["sha256"] = common.sha256_file(path)
        with self.assertRaisesRegex(NpmPackagingError, "GLIBC"):
            self.stage()

    def test_pack_failure_is_reported(self):
        class FailingRunner:
            def run(self, argv, cwd=None, env=None, timeout=None):
                return completed(argv, 1, "", "npm ERR! code E403")

        with self.assertRaisesRegex(NpmPackagingError, "npm pack failed"):
            sp.stage_all(
                self.evidence,
                self.downloads,
                self.tmp / "fail",
                NPM_DIR,
                REPO_ROOT / "LICENSE.md",
                FailingRunner(),
            )


if __name__ == "__main__":
    unittest.main()
