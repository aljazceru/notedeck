use serde::{Deserialize, Serialize};

/// Startup configuration for initial relay and account setup
/// This file should be manually created by the user in ~/.local/share/notedeck/settings/startup_config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupConfig {
    /// Single relay URL to connect to (e.g., "wss://relay.damus.io")
    pub relay: Option<String>,

    /// Private key in nsec format (e.g., "nsec1...")
    /// This will be used to create the user's account
    pub nsec: Option<String>,
}

impl StartupConfig {
    pub fn new() -> Self {
        Self {
            relay: None,
            nsec: None,
        }
    }

    pub fn with_relay(mut self, relay: String) -> Self {
        self.relay = Some(relay);
        self
    }

    pub fn with_nsec(mut self, nsec: String) -> Self {
        self.nsec = Some(nsec);
        self
    }

    pub fn is_configured(&self) -> bool {
        self.relay.is_some() || self.nsec.is_some()
    }
}

impl Default for StartupConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_config() {
        let config = StartupConfig::new()
            .with_relay("wss://relay.damus.io".to_string())
            .with_nsec("nsec1test".to_string());

        assert!(config.is_configured());
        assert_eq!(config.relay, Some("wss://relay.damus.io".to_string()));
        assert_eq!(config.nsec, Some("nsec1test".to_string()));
    }
}
