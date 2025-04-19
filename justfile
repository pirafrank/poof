# Default recipe (shows help)
# This must be the first recipe in the file

# Set default shell based on OS
set windows-powershell

default:
  just --list

# Run all tests
test:
  cargo test -- --nocapture

# Run the formatter
fmt:
  cargo fmt

# Run the linter
lint:
  cargo clippy -- -D warnings

# Run the formatter and linter
check: fmt lint

# Build the project
build:
  cargo build

# Build for release
release:
  cargo build --release

# Run cargo clean
clean:
  cargo clean

# Clean all downloaded artifacts
clean_dl:
  rm -f *.tar.gz
  rm -f *.tar.bz2
  rm -f *.tar.xz
  rm -f *.zip
  rm -f *.tar

# Clean build artifacts
clean_all:
  just clean
  just clean_dl

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
