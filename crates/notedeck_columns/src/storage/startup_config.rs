use notedeck::{storage, DataPath, DataPathType, Directory};
use tracing::{debug, error, info};

use crate::startup_config::StartupConfig;

pub static STARTUP_CONFIG_FILE: &str = "startup_config.json";

/// Load startup configuration from disk
/// Returns None if file doesn't exist or can't be parsed
pub fn load_startup_config(path: &DataPath) -> Option<StartupConfig> {
    let data_path = path.path(DataPathType::Setting);

    let config_str = match Directory::new(data_path).get_file(STARTUP_CONFIG_FILE.to_owned()) {
        Ok(s) => s,
        Err(e) => {
            debug!(
                "No startup config file found at {}: {}. This is normal for first-time setup.",
                STARTUP_CONFIG_FILE, e
            );
            return None;
        }
    };

    match serde_json::from_str::<StartupConfig>(&config_str) {
        Ok(config) => {
            info!("Loaded startup configuration from {}", STARTUP_CONFIG_FILE);
            Some(config)
        }
        Err(e) => {
            error!("Could not parse startup config: {}", e);
            None
        }
    }
}

/// Save startup configuration to disk (optional - mainly for reference)
pub fn save_startup_config(path: &DataPath, config: &StartupConfig) {
    let serialized_config = match serde_json::to_string_pretty(config) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not serialize startup config: {}", e);
            return;
        }
    };

    let data_path = path.path(DataPathType::Setting);

    if let Err(e) = storage::write_file(
        &data_path,
        STARTUP_CONFIG_FILE.to_string(),
        &serialized_config,
    ) {
        error!(
            "Could not write startup config to file {}: {}",
            STARTUP_CONFIG_FILE, e
        );
    } else {
        debug!("Successfully wrote startup config to {}", STARTUP_CONFIG_FILE);
    }
}
