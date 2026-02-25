# Nix assets

This directory holds Nix-related assets and supporting files for the
[`flake.nix`](../flake.nix) at the repository root.

## Using the flake

### Install into your Nix profile

```sh
nix profile install github:pirafrank/poof
```

### Run without installing

```sh
nix run github:pirafrank/poof
```

### Build locally

From inside the repository:

```sh
nix build .#default
# binary available at ./result/bin/poof
```

### Development shell

The flake does not ship a dedicated dev shell; use the standard Cargo
toolchain (see [`rust-toolchain.toml`](../rust-toolchain.toml)) for day-to-day
development.  `cargo build` and `cargo test` work unchanged — the env-var
overrides used by the Nix build are only injected when building through Nix.

## How the build works

`flake.nix` uses `pkgs.rustPlatform.buildRustPackage` with a `cargoLock`
pointing to `Cargo.lock`, so the build is fully reproducible with no network
access at build time.

`build.rs` checks for two environment variables before falling back to its
normal behavior:

| Variable | Nix value | Fallback (non-Nix) |
|---|---|---|
| `GIT_COMMIT_HASH` | `self.rev` (flake git revision, or `"unknown"` for dirty trees) | `git rev-parse HEAD` |
| `BUILD_DATE` | Derived from `self.lastModifiedDate` in `YYYY-MM-DD HH:MM:SS UTC` format | `chrono::Utc::now()` |

This keeps `cargo build` and CI fully functional outside Nix without any
changes.

## Adding Nix assets

Place any additional Nix files needed by the flake (overlays, derivation
helpers, etc.) in this directory and `import` / `callPackage` them from
`flake.nix`.
