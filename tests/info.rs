// Test for the 'info' subcommand
#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*; // Add methods on commands
    use predicates::prelude::*; // Used for writing assertions
    use std::process::Command; // Run programs

    #[test]
    fn test_show_info_simulated() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("poof")?;
        cmd.arg("info")
            .assert()
            .success()
            .stdout(predicate::str::contains("poof"))
            .stdout(predicate::str::contains("Platform Information:"))
            .stdout(predicate::str::contains("OS family :"))
            .stdout(predicate::str::contains("OS type   :"))
            .stdout(predicate::str::contains("OS version:"))
            .stdout(predicate::str::contains("Arch      :"))
            .stdout(predicate::str::contains("Endianness:"))
            .stderr(predicate::str::is_empty());

        Ok(())
    }
}
