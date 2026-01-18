<div align="center">
  <img src="./.assets/poof_logo_bg_tx_384.png" alt="poof logo" width="384"/>

  [![GitHub Release](https://img.shields.io/github/v/release/pirafrank/poof)](https://github.com/pirafrank/poof/releases/latest)
  [![Crates.io](https://img.shields.io/crates/v/poof)](https://crates.io/crates/poof)
  [![Crates.io MSRV](https://img.shields.io/crates/msrv/poof)](https://github.com/pirafrank/poof/blob/main/Cargo.toml)

  [![CI](https://github.com/pirafrank/poof/actions/workflows/ci.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/ci.yml)
  [![Security audit](https://github.com/pirafrank/poof/actions/workflows/security.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/security.yml)
  [![codecov](https://codecov.io/gh/pirafrank/poof/graph/badge.svg?token=UR4XUNOCMO)](https://codecov.io/gh/pirafrank/poof)
  [![dependency status](https://deps.rs/repo/github/pirafrank/poof/status.svg)](https://deps.rs/repo/github/pirafrank/poof)

  [![Licenses](https://github.com/pirafrank/poof/actions/workflows/licenses.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/licenses.yml)
  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
</div>

# poof 🪄 - magic manager of pre-built software

Easy-to-use all-in-one binary with zero-config, zero-install, and zero-dependencies.

You just run `poof install someuser/somerepo` and... *poof!* it is installed and available in your shell.

> *"poof-poof"*
>
> What poof says when it makes awesome pre-built software available for you!
>
> *"I am poof"*
>
> What poof thinks of itself

For more information read below or check the documentation in the wiki. Got an idea? Let's talk in Discussions!

[![GitHub Wiki](https://img.shields.io/badge/GitHub-Wiki-181717?style=flat-square&logo=github&logoColor=white&color=blue)](https://github.com/pirafrank/poof/wiki)
[![GitHub Discussions](https://img.shields.io/badge/GitHub-Discussions-181717?style=flat-square&logo=github&logoColor=white&color=blue)](https://github.com/pirafrank/poof/discussions)

**Note: this project is actively being developed.** I'm making ongoing improvements to the code while trying to maintain stability and up-to-date documentation. However, things may break. If you encounter some issues during this development phase, please [report them](https://github.com/pirafrank/poof/issues). Thank you!

## Features

- **🛠️ Zero-config**: Use it straight away, no yaml, no TOML or other boring configuration
- **📦 Zero-install**: one self-contained binary you just put in `PATH` and `rm` to uninstall
- **🔗 Zero-dependencies**: It runs standalone, no additional software needed
- **👤 User-space**: Designed to work in user-space and be portable. No root access needed to manage your tools
- **🌍 Cross-platform**: Works on Linux and macOS and runs on many popular architectures (FreeBSD support is planned)
- **🚀 Easy to use**: Sensible commands that are easy to remember and to type. Just run `poof help` to know more
- **🧠 Smart**: Automatically detects your setup and downloads the right binaries for it

What's more?

- **⚙️ Written in Rust**: Safe and fast binaries built on reliable dependencies, with linting and formatting applied at commit time
- **0️⃣ Zero-versioned**: Because major versions are [a thing of the past](https://0ver.org/) (and *poof*, albeit magic, is baby).

## Why

More and more often modern tools are built with languages like C/C++, Rust or Go, and offer pre-built binaries. But they aren't always available in standard package managers. Here's where `poof` helps:

- Download and put in `$PATH` binaries from GitHub with a single command
- Install tools discovered on sites like [Terminal Trove](http://terminaltrove.com) instantly
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

1. Get `poof` using one of the methods below:
    - **Pre-built binary**: Download the binary from [latest release](https://github.com/pirafrank/poof/releases), and move it to some directory in your `$PATH`.<br/>You may use the one-liner below:

    ```sh
    curl -fsSL https://raw.githubusercontent.com/pirafrank/poof/main/install.sh | sh
    ```

    - **binstall**: If you have [binstall](https://github.com/cargo-bins/cargo-binstall), you can get the binary using `cargo` and skip compilation:

    ```sh
    cargo binstall poof
    ```

    - **cargo**: Build and install latest release on crates.io using `cargo`:

    ```sh
    cargo install --locked poof
    ```

2. Add poof's `bin` directory to `$PATH` by running:

    ```sh
    poof enable
    ```

    Then reload you shell.

3. **Done!** Now try to install something, for example:

    ```sh
    poof install pirafrank/vault-conductor
    ```

Additional information about [installation](https://github.com/pirafrank/poof/wiki/How-to-install) and [supported platforms](https://github.com/pirafrank/poof/wiki/Supported-platforms) is available in the Wiki.

## Usage

```txt

poof - magic manager of pre-built software

Usage: poof [OPTIONS] <COMMAND>

Commands:
  download  Only download binary for the platform in current directory. No install
  install   Download binary for the platform and install it
  list      List installed binaries and their versions
  use       Make an installed version the one to be used by default
  update    Update installed binaries to their latest versions
  enable    Persistently add poof's bin directory to your shell PATH
  check     Check if poof's bin directory is in the PATH
  clean     Empty cache directory
  info      Show install and environment information
  version   Show version information
  help      Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Increase logging verbosity
  -q, --quiet...    Decrease logging verbosity
  -h, --help        Print help
  -V, --version     Print version

For more information, visit: https://github.com/pirafrank/poof

If you encounter any issues, please report them at:
https://github.com/pirafrank/poof/issues

```

## About poof's `bin` directory

`poof` installs and symlinks binaries in its own data directory.

Having a dedicated directory for `poof` binaries is a good practice, as it allows you to:

- keep them separate from other software installed on your system,
- keep them separate from paths you may manually edit (like `~/.local/bin`),
- easily temporarily disable `poof` by removing the directory from your `$PATH` (read below).

## Disable

poof's `bin` directory by default is added at the beginning of `$PATH` so that it takes precedence over any other version of same-named binary you may have installed other ways.

If you want to halt this behavior, you can [disable it](https://github.com/pirafrank/poof/wiki/Disable) it temporarily or permanently.

## Documentation

Updated documentation for the latest release is available in the [Wiki](https://github.com/pirafrank/poof/wiki).

## Project goals

- Fetch and put in `$PATH` pre-built binaries available on Internet
- Work without requiring buckets, repositories, or registries
- Work out-of-the-box with no setup or configuration needed
- Be designed to in user-space
- Be as cross-platform as possible
- Be easy to use, with sensible and ergonomic commands and options
- Have no external dependencies

## Non-goals

- Build software from source code
- Manage software that doesn't provide pre-built binaries
- Act as a general package manager
- Manage software installed by other tools or package managers
  - Replace or modify binaries installed by other package managers
  - Manage dependencies required by the software
  - Handle language-specific package managers (pip, npm, cargo, etc.)
  - Interface with system package managers (apt, yum, brew, etc.)

## Roadmap

A list of features implemented/to implement is available [in the Wiki](https://github.com/pirafrank/poof/wiki/Features). The list is not final and may change over time.

## Contributing

Contributions are welcome! Please read the [Development](https://github.com/pirafrank/poof/wiki/Development) page for information about how to build, and the [CONTRIBUTING](CONTRIBUTING.md) file for details on how to contribute to this project.
Please make sure to follow the [code of conduct](CODE_OF_CONDUCT.md) when contributing. Thank you!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.

## Acknowledgements

*poof* software is born out of a necessity of mine, yet its name is a tribute to the much more famous [poof](https://fairlyoddparents.fandom.com/wiki/Poof).
