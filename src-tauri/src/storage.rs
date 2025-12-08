use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;
use crate::security::AuthorizedDevice;

const CONFIG_FILE: &str = "config.json";

fn get_config_path() -> Result<PathBuf, String> {
    let proj_dirs = ProjectDirs::from("com", "scanlink", "ScanLink")
        .ok_or("Failed to get project directories")?;

    let config_dir = proj_dirs.config_dir();
    fs::create_dir_all(config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(config_dir.join(CONFIG_FILE))
}

/// Load config from disk (standalone function)
pub fn load() -> AppConfig {
    match get_config_path() {
        Ok(path) => {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_json::from_str(&content) {
                            Ok(config) => {
                                log::info!("Config loaded from {:?}", path);
                                config
                            }
                            Err(e) => {
                                log::warn!("Failed to parse config: {}. Using default.", e);
                                AppConfig::default()
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read config: {}. Using default.", e);
                        AppConfig::default()
                    }
                }
            } else {
                log::info!("No config file found, using default");
                AppConfig::default()
            }
        }
        Err(e) => {
            log::warn!("Failed to get config path: {}. Using default.", e);
            AppConfig::default()
        }
    }
}

/// Save config to disk (standalone function)
pub fn save(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path()?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    log::info!("Config saved to {:?}", path);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Master token (persistent, only changes on explicit regeneration)
    pub master_token: Option<String>,
    /// Secret key for AES encryption (base64 encoded)
    pub secret_key: Option<String>,
    /// List of authorized devices
    #[serde(default)]
    pub authorized_devices: HashMap<String, AuthorizedDevice>,
    /// Auto-start with Windows
    #[serde(default)]
    pub auto_start: bool,
    /// Minimize to tray on close
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    /// Start minimized (in tray)
    #[serde(default)]
    pub start_minimized: bool,
}

fn default_true() -> bool {
    true
}

impl AppConfig {
    pub fn add_device(&mut self, device: AuthorizedDevice) {
        self.authorized_devices.insert(device.device_id.clone(), device);
    }

    pub fn remove_device(&mut self, device_id: &str) -> bool {
        self.authorized_devices.remove(device_id).is_some()
    }

    pub fn revoke_all_devices(&mut self) {
        self.authorized_devices.clear();
    }

    pub fn is_device_authorized(&self, device_id: &str) -> bool {
        self.authorized_devices.contains_key(device_id)
    }

    #[allow(dead_code)] // Reserved for future device management feature
    pub fn get_device(&self, device_id: &str) -> Option<&AuthorizedDevice> {
        self.authorized_devices.get(device_id)
    }

    #[allow(dead_code)] // Reserved for future device management feature
    pub fn get_device_mut(&mut self, device_id: &str) -> Option<&mut AuthorizedDevice> {
        self.authorized_devices.get_mut(device_id)
    }
}
