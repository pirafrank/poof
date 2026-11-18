# poof

Easy to use zero-config, zero-install, zero-dependencies manager of pre-built software that
works like magic.

`poof` downloads pre-built binaries from GitHub Releases and manages them entirely in user
space: no root, no system package manager, no daemon, no configuration files.

This npm package is a **thin launcher**. It installs the native `poof` executable for your
platform through an optional platform package and forwards every argument to it unchanged.

## Requirements

- Node.js **>= 18**
- One of the supported platforms below

| Operating system | Architecture | npm package                        |
| ---------------- | ------------ | ---------------------------------- |
| Linux (static musl) | x64       | `@pirafrank/poof-linux-x64`        |
| Linux (static musl) | arm64     | `@pirafrank/poof-linux-arm64`      |
| macOS            | x64 (Intel)  | `@pirafrank/poof-darwin-x64`       |
| macOS            | arm64 (Apple silicon) | `@pirafrank/poof-darwin-arm64` |

Windows is **not** supported. Linux binaries are statically linked against musl, so they run
on glibc and musl distributions alike, including Alpine and scratch-based containers.

## Install

```sh
# Global install: puts `poof` on your PATH
npm install --global @pirafrank/poof

# One-off use without installing
npx --yes @pirafrank/poof --help

# Project-local install
npm install --save-dev @pirafrank/poof
```

Then use it like the native CLI:

```sh
poof install pirafrank/poof
poof list
poof which rg
```

## How it works

The launcher resolves the native executable with Node module resolution and spawns it with
`stdio: 'inherit'` and `shell: false`. Arguments, exit codes, and signals are passed through
unchanged, so `poof` behaves exactly like the native binary.

The launcher performs **no network access at runtime**. It never downloads, extracts, or
updates anything: the executable is already installed by npm. All download and update
behavior belongs to `poof` itself and is unchanged by this packaging.

## Troubleshooting

### `poof: native package @pirafrank/poof-<os>-<cpu> is not installed`

The optional platform package was skipped or removed. Common causes:

- **Optional dependencies were disabled.** `npm install --omit=optional` (or `--no-optional`,
  or `optional=false` in `.npmrc`) skips the native package. Reinstall with them enabled:

  ```sh
  npm install --global --include=optional @pirafrank/poof
  ```

- **An incomplete or pruned install.** A interrupted install, a `node_modules` cleanup, or a
  lockfile that predates the platform packages can leave the tree incomplete. Remove and
  reinstall:

  ```sh
  npm uninstall --global @pirafrank/poof
  npm install --global @pirafrank/poof
  ```

- **A package-manager layout that hides sibling packages.** Some pnpm and Yarn PnP
  configurations do not place scoped sibling packages where Node can resolve them. Use npm,
  or configure your package manager to hoist `@pirafrank/*` (for pnpm,
  `public-hoist-pattern[]=@pirafrank/*`), or install `poof` from a native release archive
  instead.

### `poof: unsupported platform`

Your operating system or CPU is not one of the four supported targets. The launcher prints
the detected platform and the full supported list. Build from source or download a native
release archive from <https://github.com/pirafrank/poof/releases>.

### `poof: failed to execute ... (EACCES)` or `(ENOENT)`

The native package is installed but its executable could not be started. Check that the file
exists and is executable (mode `0755`), and that the filesystem is not mounted `noexec` and
no security software is blocking it. Reinstalling usually repairs a damaged installation.

## Verifying what you installed

Every release is built by GitHub Actions from a tagged commit, signed with Sigstore, and
published with npm provenance. You can inspect the provenance of an installed version with:

```sh
npm audit signatures
```

Native release archives, checksums, SPDX inventories, and attestations are published on the
[GitHub Releases page](https://github.com/pirafrank/poof/releases).

## Documentation

Full CLI documentation, shell integration, and the man page live in the repository:
<https://github.com/pirafrank/poof>

## License

MIT — see [LICENSE.md](./LICENSE.md).
