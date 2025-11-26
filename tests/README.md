# Test Suite for poof

This directory contains comprehensive tests for the poof CLI application.

## Test Organization

Tests are organized into the following structure:

```
tests/
├── common/           # Shared test utilities and fixtures
│   └── mod.rs        # TestFixture and helper functions
├── unit/             # Unit tests for standalone commands
│   ├── mod.rs
│   ├── version.rs    # Tests for 'version' command
│   ├── info.rs       # Tests for 'info' command
│   ├── check.rs      # Tests for 'check' command
│   ├── clap.rs       # Tests for command-line parsing
│   └── error_handling.rs  # Error handling tests
├── integration/      # Integration tests for stateful commands
│   ├── mod.rs
│   ├── list.rs       # Tests for 'list' command
│   ├── make_default.rs  # Tests for 'use' command (make_default)
│   ├── enable.rs     # Tests for 'enable' command
│   ├── download.rs   # Tests for 'download' command
│   ├── install.rs    # Tests for 'install' command
│   ├── update.rs     # Tests for 'update' command
│   └── error_handling.rs  # Error handling in stateful commands
├── clap.rs           # Legacy test (kept for compatibility)
├── info.rs           # Legacy test (kept for compatibility)
└── version.rs        # Legacy test (kept for compatibility)
```

## Test Categories

### Standalone Commands (Unit Tests)

These commands don't depend on prior state:
- **version**: Shows version information
- **info**: Shows platform and environment information
- **check**: Checks if bin directory is in PATH
- **help**: Command-line help (implicit via clap)

### Stateful Commands (Integration Tests)

These commands depend on prior command execution:
- **download**: Downloads binaries to current directory (requires network for real tests)
- **install**: Downloads and installs binaries (requires network for real tests)
- **list**: Lists installed binaries (requires install)
- **use**: Sets default version (requires install)
- **update**: Updates installed binaries (requires install)
- **enable**: Adds bin directory to PATH

## Test Utilities

### TestFixture

The `TestFixture` struct in `tests/common/mod.rs` provides:
- Temporary directory setup
- Environment variable management
- Fake installation creation
- Automatic cleanup

All tests use temporary file systems and never touch the actual file system.

## Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --test version
cargo test --test info
cargo test --test check

# Run only integration tests
cargo test --test list
cargo test --test make_default
cargo test --test enable
cargo test --test download
cargo test --test install
cargo test --test update

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_version_command_exists
```

## Test Best Practices

1. **No File System Changes**: All tests use `tempfile::TempDir` for temporary directories
2. **Serial Execution**: Stateful tests use `#[serial_test::serial]` to prevent race conditions
3. **Environment Isolation**: Tests set and restore environment variables
4. **Error Handling**: Tests use `Result<(), Box<dyn std::error::Error>>` for proper error propagation
5. **Descriptive Names**: Test names follow `test_<function>_<scenario>_<expected_result>` pattern

## Notes

- Integration tests that require network access (like `install`) may be skipped in CI or use mocks
- The `enable` command tests modify shell RC files in temporary directories only
- All tests are designed to be idempotent and safe to run multiple times
