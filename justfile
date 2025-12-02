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

# Build the project
build:
  cargo build

# Run all tests
test:
  cargo test -- --nocapture

# Run only integration tests
test-integration:
  cargo test --test integration -- --nocapture

# Run the formatter
fmt:
  cargo fmt

# Run the formatter (checks only)
fmt-check:
  cargo fmt -- --check

# Run the linter on all files (exclude dependencies)
lint:
  cargo clippy -- --no-deps -D warnings

# Auto-fix clippy warnings
fix:
  cargo clippy -- --no-deps -D warnings --fix --allow-dirty

# Run the formatter and linter on all source files
better: fmt lint

# Run pre-commit checks on staged files
pre-commit: fmt-check lint

# Run pre-push checks
pre-push: build test

# Run pre-push checks with tags
pre-push-tag:
  hooks_scripts/pre-push-tag.sh

# Generate changelog (git-cliff required)
changelog version:
  git cliff --tag {{version}} -o CHANGELOG.md
  glow CHANGELOG.md | less

# Prepare release
prepare-release version:
  cargo set-version {{version}}
  git cliff --tag {{version}} -o CHANGELOG.md

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
ci: clean fmt-check lint build test

# Run compliance checks
# Run compliance checks (audit and license validation)
compliance: audit licenses

# Run code coverage
coverage:
  cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

