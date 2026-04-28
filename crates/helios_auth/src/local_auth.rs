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
