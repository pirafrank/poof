// Test for 'clap' handling of non-existent subcommands
#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*; // Add methods on commands
    use predicates::prelude::*; // Used for writing assertions
    use std::process::Command; // Run programs

    #[test]
    fn test_command_does_not_exist() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("poof")?;
        cmd.arg("versioning")
            .assert()
            .failure()
            .stderr(predicate::str::contains("unrecognized subcommand"));
        Ok(())
    }
}
