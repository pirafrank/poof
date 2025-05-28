// Test for the 'version' subcommand
#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*; // Add methods on commands
    use predicates::prelude::*; // Used for writing assertions
    use std::process::Command; // Run programs

    #[test]
    fn test_command_exists() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("poof")?;
        cmd.arg("version")
            .assert()
            .success()
            .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")))
            .stdout(predicate::str::contains("Commit"))
            .stdout(predicate::str::contains("Build Date"))
            .stderr(predicate::str::is_empty());
        Ok(())
    }
}
