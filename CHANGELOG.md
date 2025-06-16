# Changelog

All notable changes to the project will be documented in this file.

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

