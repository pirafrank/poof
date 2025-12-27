# Test Fixtures

This directory contains test fixtures and utilities for integration testing.

Available fixtures follows.

## `test_env.rs` - Test Environment

Provides `TestFixture` for setting up isolated test environments with temporary directories.

**Usage:**
```rust
let fixture = TestFixture::new()?;
fixture.create_fake_installation("user/repo", "1.0.0")?;
```

## `mock_github.rs` - Mock GitHub API Server

Provides `MockGitHub` for mocking GitHub API responses without making real network calls.

**Usage:**

```rust
use super::common::fixtures::mock_github::MockGitHub;

// Create a mock server
let mut mock_github = MockGitHub::new();

// Mock up-to-date response
let _m = mock_github.mock_poof_update_get_version("v1.0.0");

// Run command with mock API URL
let output = cmd
    .arg("update")
    .arg("--self")
    .env("POOF_GITHUB_API_URL", mock_github.base_url())
    .output()?;
```

**Available Methods:**

- `mock_latest_release(repo, tag, assets)` - Mock the latest release endpoint
- `mock_release_by_tag(repo, tag, assets)` - Mock a specific release by tag
- `mock_not_found(repo)` - Mock a 404 response
- `mock_network_error(repo)` - Mock a 500 error
- `mock_poof_update_get_version(version)` - Mock poof self-update check returning the given version

### `POOF_GITHUB_API_URL` env var

Override the GitHub API base URL. Used by tests to point to the mock server instead of the real GitHub API.

**Default:** `https://api.github.com/repos`

**Test Usage:**
```rust
.env("POOF_GITHUB_API_URL", mock_github.base_url())
```

### For Any GitHub API Tests

The same pattern works for testing `install`, `update <repo>`, etc.:

```rust
let mut mock_github = MockGitHub::new();
let _m = mock_github.mock_latest_release(
    "testuser/testrepo",
    "v2.0.0",
    vec![/* assets */]
);

let output = Command::new(cargo::cargo_bin!("poof"))
    .arg("install")
    .arg("testuser/testrepo")
    .env("POOF_GITHUB_API_URL", mock_github.base_url())
    .output()?;
```

### Important Notes

1. **Keep mock in scope:** The `_m` variable must stay in scope for the mock to work
2. **Use env on Command:** Set `POOF_GITHUB_API_URL` on the Command, not globally
3. **Serial tests:** Use `#[serial]` if tests modify shared state
4. **Check stderr:** Log messages go to stderr, not stdout

### Implementation Details

The mock server uses the `mockito` crate to create HTTP mock servers.
