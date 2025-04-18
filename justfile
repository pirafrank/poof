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

# Clean build artifacts
clean:
  cargo clean

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

# Default recipe (shows help)
default:
  @just --list
