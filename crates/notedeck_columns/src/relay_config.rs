use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tracing::info;

/// Global relay configuration (not tied to user accounts)
/// This determines which relays the app connects to for fetching events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayConfig {
    /// List of relay URLs to monitor
    pub relays: BTreeSet<String>,
}

impl RelayConfig {
    pub fn new() -> Self {
        Self {
            relays: BTreeSet::new(),
        }
    }

    /// Create a default configuration with some popular relays
    pub fn default_relays() -> Self {
        let mut relays = BTreeSet::new();

        // Add some default public relays
        relays.insert("wss://relay.damus.io".to_string());
        relays.insert("wss://relay.nostr.band".to_string());
        relays.insert("wss://nos.lol".to_string());

        Self { relays }
    }

    pub fn add_relay(&mut self, url: String) -> bool {
        let inserted = self.relays.insert(url.clone());
        if inserted {
            info!("Added relay: {}", url);
        }
        inserted
    }

    pub fn remove_relay(&mut self, url: &str) -> bool {
        let removed = self.relays.remove(url);
        if removed {
            info!("Removed relay: {}", url);
        }
        removed
    }

    pub fn has_relay(&self, url: &str) -> bool {
        self.relays.contains(url)
    }

    pub fn get_relays(&self) -> &BTreeSet<String> {
        &self.relays
    }

    pub fn is_empty(&self) -> bool {
        self.relays.is_empty()
    }

    pub fn len(&self) -> usize {
        self.relays.len()
    }
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self::default_relays()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_config() {
        let mut config = RelayConfig::new();
        assert!(config.is_empty());

        config.add_relay("wss://relay.example.com".to_string());
        assert_eq!(config.len(), 1);
        assert!(config.has_relay("wss://relay.example.com"));

        config.remove_relay("wss://relay.example.com");
        assert!(config.is_empty());
    }

    #[test]
    fn test_default_relays() {
        let config = RelayConfig::default_relays();
        assert!(!config.is_empty());
        assert!(config.has_relay("wss://relay.damus.io"));
    }
}
