use crate::config::Config;
use crate::utils::{Result, TerbulatorError};
use std::fs;
use std::path::PathBuf;

/// Get the default config file path: ~/.config/terbulator/config.yaml
pub fn default_config_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| TerbulatorError::config("HOME environment variable not set"))?;

    let mut path = PathBuf::from(home);
    path.push(".config");
    path.push("terbulator");
    path.push("config.yaml");

    Ok(path)
}

/// Load configuration from file, or return default if file doesn't exist
pub fn load_config(path: Option<PathBuf>) -> Result<Config> {
    let config_path = path.unwrap_or(default_config_path()?);

    if config_path.exists() {
        log::info!("Loading config from: {}", config_path.display());
        let content = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    } else {
        log::info!("Config file not found at {}, using defaults", config_path.display());
        Ok(Config::default())
    }
}

/// Save configuration to file
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = default_config_path()?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let yaml = serde_yaml::to_string(config)?;
    fs::write(&config_path, yaml)?;

    log::info!("Config saved to: {}", config_path.display());
    Ok(())
}

/// Initialize config directory and create default config if it doesn't exist
pub fn init_config(path: Option<PathBuf>) -> Result<Config> {
    let config_path = if let Some(p) = path.clone() {
        p
    } else {
        default_config_path()?
    };

    if !config_path.exists() {
        if path.is_none() {
            // Only auto-create if using default path
            log::info!("Creating default config file at: {}", config_path.display());
            let default_config = Config::default();
            save_config(&default_config)?;
            Ok(default_config)
        } else {
            // If custom path specified but doesn't exist, return error
            Err(TerbulatorError::config(format!(
                "Config file not found: {}",
                config_path.display()
            )))
        }
    } else {
        load_config(path)
    }
}
