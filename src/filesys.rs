
// ~/.config/APPNAME/config.json
pub fn get_config_dir() -> Option<String> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let home_dir = dirs::home_dir()?;
        let config_dir = home_dir.join(".config").join(app_name);
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).ok()?;
        }
        Some(config_dir.display().to_string())
    } else {
        None
    }
}

// ~/.local/share/APPNAME
pub fn get_data_dir() -> Option<String> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let home_dir = dirs::home_dir()?;
        let data_dir = home_dir.join(".local").join("share").join(app_name);
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).ok()?;
        }
        Some(data_dir.display().to_string())
    } else {
        None
    }
}
