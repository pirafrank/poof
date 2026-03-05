# Alpine APK packages

This directory contains the build tooling for creating Alpine Linux (APK)
packages for poof.

## Files

- `Dockerfile`: Alpine-based image used to build `.apk` packages.
  It accepts `ALPINE_VERSION` as a build arg (default: `3.21`).
- `entrypoint.sh`: Container entrypoint. Sets up the abuild signing key,
  fills in the `APKBUILD.template`, runs `abuild -F package`, and copies
  the output `.apk` to `/output/`.
- `APKBUILD.template`: Alpine package build script template. Placeholders
  (`@@PKGVER@@`, `@@PKGREL@@`, `@@ARCH@@`) are substituted at build time by
  `entrypoint.sh`.

## Variables

In [apk_matrix.jsonc](./apk_matrix.jsonc) matrix file and in [apk.yml](../../.github/workflows/apk.yml)
workflow file:

- `alpine_version`s  should match with `ALPINE_VERSIONS`
- `alpine_arch`s should match with `APK_ARCHS`

## Build workflow

Packages are built and published by `.github/workflows/apk.yml`.

```txt
release / workflow_dispatch
        │
        ▼
    checks          ← resolves app version and release tag
        │
        ▼
download_and_package  ← matrix: 6 arches × 2 Alpine versions (v3.20, v3.21)
  • downloads pre-built binary from GitHub release
  • builds Docker image from this Dockerfile
  • runs container to produce a signed .apk
        │
        ▼
    publish_apk     ← assembles repo, creates APKINDEX, syncs to Cloudflare R2
```

## Storage layout

```txt
apk/
  v3.20/
    x86_64/   aarch64/   armv7/   ppc64le/   riscv64/   s390x/
      *.apk   APKINDEX.tar.gz
  v3.21/  (same)
  <pubkeyname>.rsa.pub
```
