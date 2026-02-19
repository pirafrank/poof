<div align="center">
  <img src="./.assets/poof_logo_bg_tx_384.png" alt="poof logo" width="192"/>

  <h1>poof 🪄 - magic manager of pre-built software</h1>

  [![GitHub Release](https://img.shields.io/github/v/release/pirafrank/poof)](https://github.com/pirafrank/poof/releases/latest)
  [![Crates.io](https://img.shields.io/crates/v/poof)](https://crates.io/crates/poof)
  [![Crates.io MSRV](https://img.shields.io/crates/msrv/poof)](https://github.com/pirafrank/poof/blob/main/Cargo.toml)
  [![MSRV check](https://github.com/pirafrank/poof/actions/workflows/msrv.yml/badge.svg?branch=main)](https://github.com/pirafrank/poof/actions/workflows/msrv.yml)

  [![CI](https://github.com/pirafrank/poof/actions/workflows/ci.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/ci.yml)
  [![Security audit](https://github.com/pirafrank/poof/actions/workflows/security.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/security.yml)
  [![codecov](https://codecov.io/gh/pirafrank/poof/graph/badge.svg?token=UR4XUNOCMO)](https://codecov.io/gh/pirafrank/poof)
  [![dependency status](https://deps.rs/repo/github/pirafrank/poof/status.svg)](https://deps.rs/repo/github/pirafrank/poof)

  [![Licenses](https://github.com/pirafrank/poof/actions/workflows/licenses.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/licenses.yml)
  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
</div>

Easy-to-use package manager in one-binary. No manifests, formulae, or repositories required.

You just run `poof install someuser/somerepo` and... *poof!* it is installed and available in your shell.

> *"poof-poof"*
>
> What poof says when it makes awesome pre-built software available for you!
>
> *"I am poof"*
>
> What poof thinks of itself

For more information read below or check the documentation. Got an idea? Let's talk in Discussions!

[![Poof Documentation website](https://img.shields.io/badge/Poof-Documentation-181717?style=flat-square&logo=github&logoColor=white&color=blue)](https://poof.fpira.com/docs/intro)
[![GitHub Discussions](https://img.shields.io/badge/GitHub-Discussions-181717?style=flat-square&logo=github&logoColor=white&color=blue)](https://github.com/pirafrank/poof/discussions)

## Features

- **🚀 Easy to use**: Sensible commands that are easy to remember and type. Just run `poof help` to know more
- **👤 User-space**: Designed to work in user-space and be portable. No root access needed to manage your tools
- **🧠 Smart asset selection**: Automatically detects your OS, architecture, and libc (glibc vs musl) to download the right binary for your configuration. Supports multi-tool releases, multi-binary assets, mono-repos, and repositories not following Semantic Versioning
- **📦 Archive format support**: Handles 10+ formats including ZIP, TAR, 7z, and all their compressed variants with magic number validation
- **🔄 Version management**: Install multiple versions of the same tool side-by-side and switch between them instantly with `poof use`
- **🧹 Clean management**: XDG-compliant directory structure with separate cache, data, and bin directories
- **🔍 Helpful error handling**: Fuzzy matching for repository names catches typos, conflict detection warns about existing binaries, and error messages always provide context

### Core Philosophy

- **📃 Zero-maintenance**: Maintainers don’t need to explictly support poof, users don't need to wait for maintainers to add their software to poof
- **🛠️ Zero-config**: Use it straight away, no yaml, no TOML or other boring configuration
- **📦 Zero-install**: One self-contained binary you just put in `PATH` and `rm` to uninstall
- **🔗 Zero-dependencies**: It runs standalone, no additional software needed

### Platform Support

- **🌍 Cross-platform**: Works on Linux and macOS (FreeBSD support is planned)
- **🏗️ Wide architecture support**: 8 architectures on Linux, and both Intel and Apple Silicon on macOS
- **🐚 Shell integration**: Native support for 7 shells (bash, zsh, fish, elvish, nushell, powershell, xonsh) with auto-completions and one-command PATH setup

### What's more?

- **⚙️ Written in Rust**: Safe and fast binaries built on reliable dependencies, with linting and formatting applied at commit time
- **0️⃣ Zero-versioned**: Because major versions are [a thing of the past](https://0ver.org/) (and *poof*, albeit magic, is baby).

## Why

More and more often modern tools are built with languages like C/C++, Rust or Go, and offer pre-built binaries. But they aren't always available in standard package managers.

Here's where `poof` helps:

- Get software instantly upon release: if it's on GitHub Releases, poof can install it
- Download and put in `$PATH` CLI and TUI programs with a single command
- Install tools discovered on sites like [Terminal Trove](http://terminaltrove.com) easily
- Don't wait for your next favorite tool to be supported by maintainers or community
- Test newer versions of tools before they reach official repositories without uninstalling your current version
- Easily install multiple versions of the same tool and switch between them
- Keep your system clean from unnecessary packages and dependencies installed via system package managers
- Configure your CI/CD pipelines to use pre-built binaries without messing with additional requirements
- Install software in sandboxed environments without root access

## Requirements

- Linux or macOS released in the last 10 years, running on one of the supported architectures:
  - Linux (`x86_64`, `aarch64`, `armv7l`, `i686`, `ppc64le`, `s390x`, `riscv64gc`, `loongarch64`)
  - macOS (`x86_64`, `aarch64`)

## Quick start

1. Get `poof` latest stable release using this quick one-liner:

    ```sh
    curl -fsSL https://raw.githubusercontent.com/pirafrank/poof/main/install.sh | sh
    ```

    or via one of other [install methods](https://poof.fpira.com/docs/installation).

2. Add poof's `bin` directory to `$PATH`:

    ```sh
    poof enable
    ```

    Then reload you shell.

3. **🎉 Done!** Now try to install something, for example:

    ```sh
    poof install pirafrank/vault-conductor
    ```

## Usage

Either run:

```sh
poof help
```

or read the [Usage](https://poof.fpira.com/docs/usage) page for additional information.

## About poof's `bin` directory

`poof` installs binaries in its own data directory, then symlinks them to its bin directory.

You can run `poof info` at any time to know where it does store data.

Having a dedicated directory for `poof` binaries is a good practice, as it allows to:

- keep them separate from other software installed on your system,
- keep them away from paths the user may manually interact to (like `~/.local/bin`),
- support multiple side-by-side versions of the same software for easy switch,
- easily temporarily disable `poof` by removing its bin directory from your `$PATH` (read below).

## Disable

poof's `bin` directory by default is added at the beginning of `$PATH` so that it takes precedence over any other version of same-named binary you may have installed other ways.

If you want to halt this behavior, you can either:

- [manually configure](https://poof.fpira.com/docs/shell-configuration) your shell setup,
- [disable it](https://poof.fpira.com/docs/disable), temporarily or permanently.

## Documentation

Updated documentation for the latest release is available in the [Docs website](https://poof.fpira.com/docs/intro).

## Project goals and non-goals

Have a look at [our project goals](https://poof.fpira.com/docs/project-goals).

## Roadmap

A list of features implemented and to implement is available [here](https://poof.fpira.com/docs/roadmap). The list is not final and may change over time.

## Feature requests and Bug reporting

Want to suggest a feature? Found a bug? Please [open an issue](https://github.com/pirafrank/poof/issues). Thank you!

## Contributing

Contributions are welcome! Please read the [Development](https://poof.fpira.com/docs/development-guide) page for information about how to build, and the [CONTRIBUTING](CONTRIBUTING.md) file for details on how to contribute to this project.
Please make sure to follow the [code of conduct](CODE_OF_CONDUCT.md) when contributing. Thank you!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.

## Acknowledgements

*poof* software is born out of a necessity of mine, yet its name is a tribute to the much more famous [poof](https://fairlyoddparents.fandom.com/wiki/Poof).
