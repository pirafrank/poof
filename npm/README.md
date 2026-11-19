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

## Verifying what you installed

Every release is built by GitHub Actions from a tagged commit, signed with Sigstore, and
published with npm provenance. You can inspect the provenance of an installed version with:

```sh
npm audit signatures
```

Native release archives, checksums, SPDX inventories, and attestations are published on the
[GitHub Releases page](https://github.com/pirafrank/poof/releases).

## How it works

The launcher resolves the native executable with Node module resolution and spawns it with
`stdio: 'inherit'` and `shell: false`. Arguments, exit codes, and signals are passed through
unchanged, so `poof` behaves exactly like the native binary.

The launcher performs **no network access at runtime**. It never downloads, extracts, or
updates anything: the executable is already installed by npm. All download and update
behavior belongs to `poof` itself and is unchanged by this packaging.

## Documentation

Full CLI documentation, shell integration, and more is available at [poof.fpira.com](https://poof.fpira.com).

## Troubleshooting

See the [npm troubleshooting guide](https://poof.fpira.com/docs/npm-troubleshooting)
for help with missing native packages, unsupported platforms, and executable launch errors.

## Repository

The source code is available on GitHub: <https://github.com/pirafrank/poof>.

## License

MIT

See [LICENSE.md](https://github.com/pirafrank/poof/blob/main/LICENSE.md).
