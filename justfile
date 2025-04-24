# Default recipe (shows help)
# This must be the first recipe in the file

# Set default shell based on OS
set windows-powershell

# List available recipes
default:
  just --list

# Install git hooks
install-hooks:
  git config core.hooksPath hooks

# Run all tests
test:
  cargo test -- --nocapture

# Run the formatter
fmt:
  cargo fmt

# Run the formatter checks
fmt-check:
  cargo fmt -- --check

# Run the linter
lint:
  cargo clippy -- -D warnings

# Run the formatter and linter
better: fmt lint

# Run pre-commit checks
pre-commit: fmt-check lint

# Run pre-push checks
pre-push: test build

# Generate changelog (git-cliff required)
changelog:
  git cliff -o CHANGELOG.md
  glow CHANGELOG.md

# Build the project
build:
  cargo build

# Build for release
release:
  cargo build --release

# Run cargo artifacts
clean:
  cargo clean

# Clean all downloaded artifacts
clean-dl:
  rm -f *.tar.gz
  rm -f *.tar.bz2
  rm -f *.tar.xz
  rm -f *.zip
  rm -f *.tar
  rm -f *.tgz
  rm -f *.tbz2
  rm -f *.tbz

# Clean all artifacts
clean-all: clean clean-dl

# Generate documentation
docs:
  cargo doc --no-deps --open

# Run benchmarks
bench:
  cargo bench

# Show dependency tree
deps:
  cargo tree

# Update dependencies
update-deps:
  cargo update

# Check for outdated dependencies
outdated-deps:
  cargo outdated --root-deps-only

# Check for security vulnerabilities
audit:
  cargo audit

# Check for license issues
licenses:
  cargo deny check licenses

# Run CI checks
ci: clean fmt-check lint
# note: just test  # do not run tests on CI pipeline via just due freebsd and windows quirks

# Run compliance checks
# Run compliance checks (audit and license validation)
compliance: audit licenses
