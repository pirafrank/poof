<div align="center">
  <img src="./.assets/poof_logo_bg_tx_384.png" alt="poof logo" width="384"/>
</div>

# poof 🪄 - magic manager of pre-built software

[![Release](https://github.com/pirafrank/poof/actions/workflows/release.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/release.yml)
[![Security audit](https://github.com/pirafrank/poof/actions/workflows/compliance.yml/badge.svg)](https://github.com/pirafrank/poof/actions/workflows/compliance.yml)

Easy to use zero-config, zero-install, zero-dependencies binary manager in user-space that works like magic.

You just run `poof install someuser/somerepo` and... *poof!* it is installed and available in your shell.

> *"poof-poof"*
>
> What poof says when it makes awesome pre-built software available for you!
>
> *"I am poof"*
>
> What poof thinks of itself

**Note: this project is actively being developed.** I'm making ongoing improvements to the code while trying to maintain stability and up-to-date documentation. However, things may break. If you encounter some issues during this development phase, please [report them](https://github.com/pirafrank/poof/issues). Thank you!

## Features

- **🛠️ Zero-config**: Works immediately without any setup required for first run
- **📦 Zero-install**: Simply download the binary for your platform and use it right away
- **🔗 Zero-dependencies**: It runs standalone without requiring any additional software
- **👤 User-space**: Designed to work in user-space, no root access needed
- **🌍 Cross-platform**: Works on Linux and macOS (FreeBSD and Windows support is planned)
- **🚀 Easy to use**: Just run `poof` and it will do the rest

Bonus:

- **⚙️ Written in Rust**: with linting and formatting applied at commit time
- **0️⃣ Zero-versioned**: because major versions are [a thing of the past](https://0ver.org/) (and *poof*, albeit magic, is baby).

## Why

More and more often modern tools are built with languages like Rust and Go, and offer pre-built binaries. But they aren't always available in standard package managers. Here's where `poof` helps:

- Download and put in `$PATH` binaries from GitHub with a single command
- Install tools discovered on sites like [Terminal Trove](http://terminaltrove.com) instantly
- Test newer versions of tools before they reach official repositories without uninstalling your current version
- Fast jump on interesting utilities you read about without hassle
- Prefer use of pre-built, portable, self-contained binaries without involving system package managers

## Quick start

1. Get `poof` using one of the methods below:
    - **Pre-built binary**: Download the binary from [latest release](https://github.com/pirafrank/poof/releases), and move it to `$PATH`.
    - **binstall**: If you have [binstall](https://github.com/cargo-bins/cargo-binstall), you can get the binary using `cargo` and skip compilation:

    ```sh
    cargo binstall exif_renamer
    ```

    - **cargo**: Build and install latest release on crates.io using `cargo`:

    ```sh
    cargo install --locked poof
    ```

    - **From source**: Build and install from source code on GitHub:

    ```sh
    cargo install --locked --git https://github.com/pirafrank/poof --tag VERSION
    ```

    Note: Replace `VERSION` with the desired version to install. Not specifying a tag will install from `main` branch. `main` branch should be stable, but it's unreleased software and may contain bugs or breaking changes. It should considered beta quality software.

2. Add poof's `bin` directory to `$PATH`. Paste the following to your `~/.bashrc` or `~/.zshrc`:

    On Linux:

    ```txt
    export PATH="${HOME}/.local/share/poof/bin:${PATH}"
    ```

    On macOS:

    ```txt
    export PATH="${HOME}/Library/Application Support/poof/bin:${PATH}"
    ```

3. Done! Now try to install something, for example:

    ```txt
    poof install pirafrank/rust_exif_renamer
    ```

## About poof's `bin` directory

After installing `poof`, you need to add its `bin` directory to your `$PATH`. Be sure it is at the beginning of your `$PATH` so that it takes precedence over any other version of the same binary you may have installed other ways.

Having a dedicated directory for `poof` binaries is a good practice, as it allows you to:

- keep them separate from other software installed on your system,
- keep them separate from paths you may manually edit (like `~/.local/bin`),
- easily temporarily disable `poof` by removing the directory from your `$PATH` (see below).

## Usage

```txt

poof - magic manager of pre-built software

Usage: poof [OPTIONS] <COMMAND>

Commands:
  download  Only download binary for the platform in current directory. No install
  install   Download binary for the platform and install it
  use       Make an installed version the one to be used by default
  list      List installed binaries and their versions
  check     Check if poof's bin directory is in the PATH
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

## Disable

If you want to disable `poof`, you can do so by removing its `bin` directory from your `$PATH`.

You can do this by commenting out the line you added to your `~/.bashrc` or `~/.zshrc` file, or by removing the directory from `$PATH` variable in your shell session.

## Uninstall

To uninstall `poof`, just delete the binary from your `$PATH`.

If you have installed it using `cargo`, you can uninstall it with:

```sh
cargo uninstall poof
```

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

A list of features implemented/to implement is available in the [ROADMAP](ROADMAP.md) file. The list is not final and may change over time.

## Contributing

Contributions are welcome! Please read the [CONTRIBUTING](CONTRIBUTING.md) file for details on how to contribute to this project.
Please make sure to follow the [code of conduct](CODE_OF_CONDUCT.md) when contributing.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE.md) file for details.

## Acknowledgements

*poof* software is born out of a necessity of mine, yet its name is a tribute to the much more famous [poof](https://fairlyoddparents.fandom.com/wiki/Poof).
