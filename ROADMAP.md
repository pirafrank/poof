# Roadmap

## Overview

This document outlines both implemented and planned features for *poof*. The roadmap is organized by feature categories, commands, supported platforms, and compatible archive types.

Items are listed in no particular order. The list is not final and may change over time.

Got a feature request? Open an [issue](https://github.com/pirafrank/poof/labels/feature%20request).

## Features

- [x] Support for GitHub releases API (version 2022-11-28)
- [x] Automatic understanding of the correct binary for the platform *poof* is running on
- [x] Download and extract it to cache dir, install to data dir, and symlink to bin directory
- [x] Select `latest` version for download if no version is specified
- [ ] Download and install of user-defined version of the binary
- [x] Use tag_name from serde parsed API output as subdir for all installs, including *latest*
- [x] Support archives that contain a directory with the same name as the archive inside
- [ ] Easier install via shell script
- [ ] Install via `cargo install poof`
- [ ] Install via `cargo binstall poof`
- [ ] Add proper logging to sysout
- [x] Add debug info
- [ ] Shell integration
- [ ] Support providers other than github
  - [ ] GitLab.com
  - [ ] Codeberg
- [x] CI pipeline
- [x] Release pipeline

## Commands

- [x] `help`, show help information about poof
- [ ] `enable`, add poof's bin directory to `$PATH` in `~/.bashrc` or `~/.zshrc`
- [ ] `disable`, remove poof's bin directory from `$PATH` in current shell session
- [x] `get`, only download the binary for current platform to current directory
- [x] `install`, download the binary for current platform and install to poof's bin directory
- [x] `list`, list all installed binaries
- [x] `use`, make an installed version the one to be used by default
- [ ] `update`, update an installed binary to the latest version
- [ ] `update --all`, update all installed binaries to their latest versions
- [ ] `update --self`, use poof to update poof itself
- [ ] `remove`, remove symlink to a binary installed in poof
- [ ] `purge`, remove all installed binaries
- [x] `check`, check if poof's bin directory is in the `$PATH`
- [x] `version`, show version information about poof
- [x] `debug`, show debug information for troubleshooting
- [ ] `export`, export installed apps and their version in JSON or YAML format
- [ ] `import`, import installed apps and their version in JSON or YAML format
- [ ] `hold`, disabl updates for an app
- [ ] `unhold`, re-enable updates for an app
- [x] Support for `--help` flag for all commands
- [ ] Support shortened versions of commands (e.g. `poof i` for `poof install`)

## Platform support

- [x] Linux x86_64 support
- [x] Linux aarch64 support
- [x] macOS x86_64 support
- [x] macOS aarch64 support
- [ ] FreeBSD x86_64 support
- [ ] Windows x86_64 support
- [ ] Windows aarch64 support

## Archive types

Archive types of pre-built software to download and install:

- [x] non compressed archives
- [x] support for `application/zip` archives
- [x] support for `application/gzip` archives
- [x] support for `application/x-gtar` archives
- [x] support for `application/x-xz` archives
- [x] support for `application/x-bzip2` archives
- [x] support for `application/x-tar` archives
- [x] support for `application/x-7z-compressed` archives
- [x] support for `application/x-tar` archives (experimental)
- [x] support for `application/x-7z-compressed` archives (experimental)
