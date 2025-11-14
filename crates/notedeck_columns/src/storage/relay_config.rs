use notedeck::{storage, DataPath, DataPathType, Directory};
use tracing::{debug, error};

use crate::relay_config::RelayConfig;

pub static RELAY_CONFIG_FILE: &str = "relay_config.json";

pub fn load_relay_config(path: &DataPath) -> Option<RelayConfig> {
    let data_path = path.path(DataPathType::Setting);

    let relay_config_str = match Directory::new(data_path).get_file(RELAY_CONFIG_FILE.to_owned()) {
        Ok(s) => s,
        Err(e) => {
            error!(
                "Could not read relay config from file {}: {}",
                RELAY_CONFIG_FILE, e
            );
            return None;
        }
    };

    serde_json::from_str::<RelayConfig>(&relay_config_str).ok()
}

pub fn save_relay_config(path: &DataPath, relay_config: &RelayConfig) {
    let serialized_relay_config = match serde_json::to_string_pretty(relay_config) {
        Ok(s) => s,
        Err(e) => {
            error!("Could not serialize relay config: {}", e);
            return;
        }
    };

    let data_path = path.path(DataPathType::Setting);

    if let Err(e) = storage::write_file(
        &data_path,
        RELAY_CONFIG_FILE.to_string(),
        &serialized_relay_config,
    ) {
        error!(
            "Could not write relay config to file {}: {}",
            RELAY_CONFIG_FILE, e
        );
    } else {
        debug!("Successfully wrote relay config to {}", RELAY_CONFIG_FILE);
    }
}
