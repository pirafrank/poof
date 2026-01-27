//! Integration tests for the 'enable' command

use assert_cmd::cargo;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[serial]
#[test]
fn test_enable_creates_bashrc_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("bash")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that .bashrc was created/modified
    let bashrc_path = temp_home.path().join(".bashrc");
    assert!(
        bashrc_path.exists(),
        ".bashrc file should be created by enable command"
    );

    let contents = fs::read_to_string(&bashrc_path)?;
    assert!(
        contents.contains("eval \"$(poof init --shell bash)\""),
        ".bashrc should contain eval line"
    );
    assert!(
        contents.contains("# added by poof"),
        ".bashrc should contain comment marker"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_creates_zshrc_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("zsh")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/zsh");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that .zshrc was created/modified
    let zshrc_path = temp_home.path().join(".zshrc");
    assert!(
        zshrc_path.exists(),
        ".zshrc file should be created by enable command"
    );

    let contents = fs::read_to_string(&zshrc_path)?;
    assert!(
        contents.contains("eval \"$(poof init --shell zsh)\""),
        ".zshrc should contain eval line"
    );
    assert!(
        contents.contains("# added by poof"),
        ".zshrc should contain comment marker"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_creates_fish_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("fish")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/fish");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that fish config was created/modified
    let fish_config = temp_home
        .path()
        .join(".config")
        .join("fish")
        .join("config.fish");
    assert!(
        fish_config.exists(),
        "fish config file should be created by enable command"
    );

    let contents = fs::read_to_string(&fish_config)?;
    assert!(
        contents.contains("fish_add_path"),
        "fish config should contain fish_add_path"
    );
    assert!(
        contents.contains("# added by poof"),
        "fish config should contain comment marker"
    );
    assert!(
        contents.contains(bin_dir.to_string_lossy().as_ref()),
        "fish config should contain bin directory"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_creates_elvish_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("elvish")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/elvish");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that elvish config was created/modified
    let elvish_config = temp_home
        .path()
        .join(".config")
        .join("elvish")
        .join("rc.elv");
    assert!(
        elvish_config.exists(),
        "elvish config file should be created by enable command"
    );

    let contents = fs::read_to_string(&elvish_config)?;
    assert!(
        contents.contains("eval \"(poof init --shell elvish)\""),
        "elvish config should contain eval line"
    );
    assert!(
        contents.contains("# added by poof"),
        "elvish config should contain comment marker"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_creates_nushell_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("nushell")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/nu");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that nushell config was created/modified
    let nushell_config = temp_home
        .path()
        .join(".config")
        .join("nushell")
        .join("env.nu");
    assert!(
        nushell_config.exists(),
        "nushell config file should be created by enable command"
    );

    let contents = fs::read_to_string(&nushell_config)?;
    assert!(
        contents.contains("$env.PATH"),
        "nushell config should contain $env.PATH"
    );
    assert!(
        contents.contains("# added by poof"),
        "nushell config should contain comment marker"
    );
    assert!(
        contents.contains(bin_dir.to_string_lossy().as_ref()),
        "nushell config should contain bin directory"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_creates_xonsh_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("xonsh")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/xonsh");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that xonsh config was created/modified
    let xonsh_config = temp_home.path().join(".xonshrc");
    assert!(
        xonsh_config.exists(),
        "xonsh config file should be created by enable command"
    );

    let contents = fs::read_to_string(&xonsh_config)?;
    assert!(
        contents.contains("$PATH.insert"),
        "xonsh config should contain $PATH.insert"
    );
    assert!(
        contents.contains("# added by poof"),
        "xonsh config should contain comment marker"
    );
    assert!(
        contents.contains(bin_dir.to_string_lossy().as_ref()),
        "xonsh config should contain bin directory"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_creates_powershell_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("powershell")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/pwsh");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that powershell config was created/modified
    let powershell_config = temp_home
        .path()
        .join(".config")
        .join("powershell")
        .join("Microsoft.PowerShell_profile.ps1");
    assert!(
        powershell_config.exists(),
        "powershell config file should be created by enable command"
    );

    let contents = fs::read_to_string(&powershell_config)?;
    assert!(
        contents.contains("Invoke-Expression"),
        "powershell config should contain Invoke-Expression"
    );
    assert!(
        contents.contains("poof init --shell powershell"),
        "powershell config should contain poof init command"
    );
    assert!(
        contents.contains("# added by poof"),
        "powershell config should contain comment marker"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let _bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let _bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&_bin_dir)?;

    // Run enable twice
    let mut cmd1 = Command::new(cargo::cargo_bin!("poof"));
    cmd1.arg("enable")
        .arg("--shell")
        .arg("bash")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash");
    #[cfg(target_os = "linux")]
    {
        cmd1.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    cmd1.output()?;

    let mut cmd2 = Command::new(cargo::cargo_bin!("poof"));
    cmd2.arg("enable")
        .arg("--shell")
        .arg("bash")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash");
    #[cfg(target_os = "linux")]
    {
        cmd2.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    cmd2.output()?;

    // Check that eval line appears only once
    let bashrc_path = temp_home.path().join(".bashrc");
    assert!(
        bashrc_path.exists(),
        ".bashrc file should be created by enable command"
    );

    let contents = fs::read_to_string(&bashrc_path)?;
    let eval_line = "eval \"$(poof init --shell bash)\"";
    let count = contents.matches(eval_line).count();
    assert_eq!(
        count, 1,
        "Eval line should appear exactly once, found {} times",
        count
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_preserves_existing_content() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let _bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let _bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&_bin_dir)?;

    // Pre-seed .bashrc with existing content
    let bashrc_path = temp_home.path().join(".bashrc");
    fs::write(&bashrc_path, "PRE_EXISTING_LINE\n")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable")
        .arg("--shell")
        .arg("bash")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash");
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Verify existing content is preserved
    let contents = fs::read_to_string(&bashrc_path)?;
    assert!(
        contents.contains("PRE_EXISTING_LINE"),
        "Existing content should be preserved"
    );
    assert!(
        contents.contains("eval \"$(poof init --shell bash)\""),
        "Eval line should be added"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_zsh_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let _bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let _bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&_bin_dir)?;

    // Run enable twice with zsh
    let mut cmd1 = Command::new(cargo::cargo_bin!("poof"));
    cmd1.arg("enable")
        .arg("--shell")
        .arg("zsh")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/zsh");
    #[cfg(target_os = "linux")]
    {
        cmd1.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    cmd1.output()?;

    let mut cmd2 = Command::new(cargo::cargo_bin!("poof"));
    cmd2.arg("enable")
        .arg("--shell")
        .arg("zsh")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/zsh");
    #[cfg(target_os = "linux")]
    {
        cmd2.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    cmd2.output()?;

    // Check that eval line appears only once
    let zshrc_path = temp_home.path().join(".zshrc");
    assert!(
        zshrc_path.exists(),
        ".zshrc file should be created by enable command"
    );

    let contents = fs::read_to_string(&zshrc_path)?;
    let eval_line = "eval \"$(poof init --shell zsh)\"";
    let count = contents.matches(eval_line).count();
    assert_eq!(
        count, 1,
        "Eval line should appear exactly once in zsh, found {} times",
        count
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_requires_shell_argument() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure (platform-specific)
    #[cfg(target_os = "linux")]
    let _bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    #[cfg(target_os = "macos")]
    let _bin_dir = temp_home
        .path()
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&_bin_dir)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("enable").env("HOME", temp_home.path());
    #[cfg(target_os = "linux")]
    {
        cmd.env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        );
    }
    let output = cmd.output()?;

    // Command should fail without --shell argument
    assert!(
        !output.status.success(),
        "Enable command should fail without --shell argument"
    );

    Ok(())
}

#[serial]
#[test]
fn test_enable_all_shells_are_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let shells = vec![
        ("bash", "/bin/bash", ".bashrc"),
        ("zsh", "/usr/bin/zsh", ".zshrc"),
        ("fish", "/usr/bin/fish", ".config/fish/config.fish"),
        ("elvish", "/usr/bin/elvish", ".config/elvish/rc.elv"),
        ("nu", "/usr/bin/nu", ".config/nushell/env.nu"),
        ("xonsh", "/usr/bin/xonsh", ".xonshrc"),
        (
            "pwsh",
            "/usr/bin/pwsh",
            ".config/powershell/Microsoft.PowerShell_profile.ps1",
        ),
    ];

    for (shell_name, shell_path, config_file) in shells {
        let temp_home = TempDir::new()?;

        // Create bin directory structure (platform-specific)
        #[cfg(target_os = "linux")]
        let _bin_dir = temp_home
            .path()
            .join(".local")
            .join("share")
            .join("poof")
            .join("bin");
        #[cfg(target_os = "macos")]
        let _bin_dir = temp_home
            .path()
            .join("Library")
            .join("Application Support")
            .join("poof")
            .join("bin");
        fs::create_dir_all(&_bin_dir)?;

        // Run enable twice
        for _ in 0..2 {
            let mut cmd = Command::new(cargo::cargo_bin!("poof"));
            cmd.arg("enable")
                .arg("--shell")
                .arg(shell_name)
                .env("HOME", temp_home.path())
                .env("SHELL", shell_path);
            #[cfg(target_os = "linux")]
            {
                cmd.env(
                    "XDG_DATA_HOME",
                    temp_home.path().join(".local").join("share"),
                );
            }
            cmd.output()?;
        }

        let config_path = temp_home.path().join(config_file);
        let contents = fs::read_to_string(&config_path)?;

        // Should only have one "# added by poof" marker
        assert_eq!(
            contents.matches("# added by poof").count(),
            1,
            "{}: comment marker should appear exactly once",
            shell_name
        );
    }

    Ok(())
}
