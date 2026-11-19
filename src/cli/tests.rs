use super::*;
use clap::CommandFactory;

/// Every subcommand must document the release that introduced it, so users and
/// agents can check compatibility against the short version from `poof -V`.
#[test]
fn all_subcommands_have_valid_since_marker() {
    for sub in Cli::command().get_subcommands() {
        let name = sub.get_name();
        // clap synthesizes a `help` subcommand; it is not a poof command.
        if name == "help" {
            continue;
        }
        let long_about = sub
            .get_long_about()
            .unwrap_or_else(|| panic!("`{name}` is missing its `Since:` documentation"));
        let text = long_about.to_string();
        let since = text
            .lines()
            .find_map(|line| line.strip_prefix("Since: "))
            .unwrap_or_else(|| panic!("`{name}` has no `Since: <version>` line in its help"));
        semver::Version::parse(since.trim())
            .unwrap_or_else(|e| panic!("`{name}` has invalid Since version `{since}`: {e}"));
    }
}
