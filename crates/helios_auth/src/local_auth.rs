//! Pi local authentication via `~/.pi/providers.json`.
//!
//! This layer works entirely offline — it reads the provider configuration
//! that the Pi runtime already manages and exposes it to Helios-Terminal
//! for status display and provider selection.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A single AI provider entry from `~/.pi/providers.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name (e.g. "anthropic", "openai", "google").
    pub name: String,
    /// Whether this provider has a valid API key configured.
    #[serde(default)]
    pub has_key: bool,
}

/// Reads and validates Pi's local provider configuration.
pub struct PiLocalAuth {
    pi_home: PathBuf,
}

impl PiLocalAuth {
    /// Create a new local auth reader for the given Pi home directory.
    pub fn new(pi_home: PathBuf) -> Self {
        Self { pi_home }
    }

    /// Create a local auth reader using the default Pi home (`~/.pi`).
    pub fn default_home() -> Self {
        let pi_home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pi");
        Self::new(pi_home)
    }

    /// Returns `true` if `~/.pi/providers.json` exists.
    pub fn has_providers(&self) -> bool {
        self.providers_path().exists()
    }

    /// Read and parse provider configurations from `providers.json`.
    pub fn get_providers(&self) -> anyhow::Result<Vec<ProviderConfig>> {
        let path = self.providers_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let contents = std::fs::read_to_string(&path)?;
        let providers: Vec<ProviderConfig> = serde_json::from_str(&contents)?;
        Ok(providers)
    }

    fn providers_path(&self) -> PathBuf {
        self.pi_home.join("providers.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_providers_json_round_trip() {
        let json = r#"[{"name": "anthropic", "has_key": true}, {"name": "openai", "has_key": false}]"#;
        let providers: Vec<ProviderConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(providers.len(), 2);
        assert_eq!(providers[0].name, "anthropic");
        assert!(providers[0].has_key);
        assert_eq!(providers[1].name, "openai");
        assert!(!providers[1].has_key);
    }

    #[test]
    fn parse_providers_with_missing_fields_uses_defaults() {
        // Test that #[serde(default)] works
        let json = r#"[{"name": "google"}]"#;
        let providers: Vec<ProviderConfig> = serde_json::from_str(json).unwrap();
        assert_eq!(providers[0].name, "google");
        assert!(!providers[0].has_key); // default is false
    }

    #[test]
    fn default_home_returns_pi_directory() {
        let auth = PiLocalAuth::default_home();
        assert!(auth.pi_home.ends_with(".pi"));
    }
}
