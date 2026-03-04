# Changelog

All notable changes to the project will be documented in this file.

## [0.6.1] - 2026-03-04

### 🚀 Features

- Dockerfile for one-off test of `poof`
- Support optional slug in `list` command (#165)
- Show url to release in version info
- Support zstd format (#174)
- Poof as Github action
- Poof as devcontainer feature
- Macports support as installation method (#177)
- Docs improvements (#176)
- Nix support (#179)
- Support distros from the last 10 years in apt and yum/dnf repos (#180)
- Support binaries with arch or OS as extension (#183)

### 🐛 Bug Fixes

- Changed `which` implementation to work after running `unlink` (#172)
- Proper exit codes for warning and error case (#173)
- Improved selection and handling of binaries across platforms (#181)

### 🚜 Refactor

- Rename test files for better readability
- Move tests to own files for clarity

### 📚 Documentation

- Automated devcontainer feature documentation update (#175)

### 🔧 Setup & Quality

- Poof test pipeline
- Change trigger for crates.io publish
- Updated triggers and added Nix checks pipeline
- .deb and .rpm release pipeline
- Make apt/yum packages pipeline idempotent

### ⚙️ Miscellaneous Tasks

- Added tools for AUR asset generation
- New logo
- Bump MSRV to 1.88 (up from 1.85) and add verification pipeline
- Renamed workflows files
- Removed AUR packaging scripts (now handled by AURA)
- Avoid running pre-push checks when not needed
- Ignore github tokens in Cursor IDE
- AUR readme and typo fixes
- Cargo update (#182)
- Added readme to devcontainer test dir
- Update tag matching rule in git-cliff

## [0.6.0] - 2026-02-04

### 🚀 Features

- [**breaking**] `-V` now shows short version. good for scripts
- Added riscv64gc, arm 32 bit, loongarch64
- Improved selector's code to support musl
- New score-based asset selector, rewritten for wider assets support
- Use GITHUB_TOKEN env var (if available) to avoid rate limit
- Musl autodetection, override via `POOF_PREFER_MUSL` env var to prefer musl builds
- Added support for i686, powerpc64le, and s390x
- One-liner installation script (#135)
- Handle multi-asset releases (#141)
- Auto-cleanup cache dir after install (#142)
- New commands `init` and `completions` (#148)
- Generate man pages and publish them with releases (#151)
- New `unlink` and `uninstall` commands (#152)
- New `which` command to know which repository provides a binary (#154)
- New `what` command to know what binaries a slug provides (#155)
- Improved output to sysout to be parsable in scripts (#156)
- [**breaking**] Removed `debug` in favor of `info` command with same output
- [**breaking**] Removed `update --self` option to update poof in favor of other install methods (#157)
- `use` command can run without version to automatically pick and set latest as default (#158)
- Minor UI changes (#160)
- Replace -v flags with RUST_LOG env var (#161)

### 🐛 Bug Fixes

- Handle same-named binaries during install (#143)

### 🚜 Refactor

- Rename update internal method
- Move release pipeline steps into actions for reuse and maintenance
- Improved tests with better environment setup and linting (#153)

### 🧪 Testing

- Improved code coverage (#145)

### 🔧 Setup & Quality

- Group dependabot PRs
- Added rust cache
- Slighty better error messages
- Release workflows for crates, homebrew, and asdf
- AUR workflow

### ⚙️ Miscellaneous Tasks

- Update status badge
- Coderabbit config
- Cargo update (#163)

## [0.5.2] - 2025-12-27

### 🐛 Bug Fixes

- Update self command logic to replace self_update crate with self_replace (#117)

### 🔧 Setup & Quality

- Added git-cliff to cargo plugins in justfile recipe

### ⚙️ Miscellaneous Tasks

- Fix linter issues

## [0.5.1] - 2025-12-08

### 🚀 Features

- *(update)* [**breaking**] Automatically set new version as default after updating an asset (#106)

### 🐛 Bug Fixes

- *(update)* Update args --all and --self cannot be used together
- Better handling of state to avoid panic
- Moving to unit-prefix to address RUSTSEC-2025-0119 #99 (#103)
- Listing installed assets fails or behaves incorrectly with non semver asset versions (#104)

### 🧪 Testing

- Fix tests on macOS (#108)
- Fix deprecation of cargo_bin after dependencies update

### 🔧 Setup & Quality

- Justfile improvements
- *(git)* Enforce conventional commit messages
- *(git)* Avoid running tests for git push -d operations
- Added a bunch of integration tests, unit tests, and code coverage (#101)
- Perform tests on macOS as well and pin runner versions
- Simplify workflow files and improve consistency (#110)
- Justfile improvements
- Just recipe to install cargo plugins

### ◀️ Revert

- Pre-commit recipe as it was to support git partial file commits

### ⚙️ Miscellaneous Tasks

- *(lint)* Fixed linting error for ambiguous cmp reference
- Ignore code coverage reports and allow bzip2-1.0.6 license
- Update cargo.lock
- Added clippy.toml
- Edited coverage recipe
- Cargo update
- Added couple of recipes as utils and split changelog and prepare-release recipes
- Git-cliff config small update
- Cargo update

## [0.5.0] - 2025-06-16

### 🚀 Features

- `poof update` command, introduced anyhow + refactor to return Result (#5)
- Clean command to empty cache dir (#57)
- Support install of unarchived/uncompressed assets (#58)
- Add info about glibc and musl used at build time in 'version' subcommand (#62)
- *(UX)* Clearer info about successfully installed repo and version
- *(commands)* [**breaking**] Change to `use` subcommand args to be more sensible, now `poof use <REPO> <VERSION>` (#63)

### 🐛 Bug Fixes

- Application/octet-stream is not a supported mime type (#55)

### 🚜 Refactor

- Move core functions and files to own modules (#56)
- *(data dirs)* [**breaking**] Prepare data dir structure for a future update. Run migrate_poof_data.sh before updating to 0.5.0

### 🔧 Setup & Quality

- Issue template updates
- Publish sha256 of artifacts in release pipeline (#36)
- Use variables for binary filename in workflows
- Added dependabot configuration
- Gh codespaces env
- Git-cliff config update

### ⚙️ Miscellaneous Tasks

- Added Security Issue template and updated issue links

## [0.4.0] - 2025-05-02

### 🚀 Features

- `info` command to show install and env information
- *(enable)* Add `enable` command with tests (#3)

### 🚜 Refactor

- Data models and version string cleanup
- Move function to github client
- Code cleanup
- Move commands code out of main.rs
- Renamed lib to selector not exposing it as crate lib
- Moved data dir functions to own file
- Moved constants to own file

### 🔧 Setup & Quality

- Organize pipeline checks
- Run CI checks for all branches
- Add pre tag push checks

### ⚙️ Miscellaneous Tasks

- Dependencies update
- Moved roadmap to repository wiki
- Cargo deny settings update

## [0.3.1] - 2025-04-30

### 🐛 Bug Fixes

- 'check' command may not work properly

### 🚜 Refactor

- Args validation regex as static
- Better path traversal in list command, added Asset struct

## [0.3.0] - 2025-04-28

### 🚀 Features

- Better dir structure in data dir
- Minor improvement in messages output to the user
- Better pkg description and repo info
- Added 'check' command to look for poof bin dir in PATH
- Enhanced debug info
- Added 'list' command to show installed binaries and their versions
- Improved installation handling of multiple versions
- Added 'use' command to change symlinked version
- Added validation of passed args

### 🚜 Refactor

- Split up of install function and improvements
- Reorder github interaction code

### 🔧 Setup & Quality

- Schedule compliance workflow

### ⚙️ Miscellaneous Tasks

- Update cargo-deny settings

## [0.2.0] - 2025-04-27

### 🚀 Features

- Support binaries not contained in archives
- Support archive extraction
- Download, extract, and install to bin dir
- Debug command to show sysinfo
- Logging

### 🐛 Bug Fixes

- Do not use release 'name' which may be null
- Check for already installed asset versions
- Use rustls for tls support in reqwest

### 🔧 Setup & Quality

- CI and Release pipelines
- Add script to load PATH
- Compliance pipeline
- Preparing for git hooks
- Format and lint code
- Git cliff tuning
- Enforcing git hooks
- Issue templates
- Ability to manually run CI workflow

### ⚙️ Miscellaneous Tasks

- Justfile adjustments
- License and readme
- Code of conduct and CONTRIBUTING files
- Update dirs dep to 6.0

## [0.1.0-pre] - 2025-04-21

### 🚀 Features

- First working version

### ⚙️ Miscellaneous Tasks

- Vscode config and justfile
- Git-cliff
- Added editorconfig and run fmt

