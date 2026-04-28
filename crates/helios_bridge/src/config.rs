//! Configuration for the Pi runtime bridge.
//!
//! Loads settings from `~/.helios-terminal/config.toml` with sane defaults.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Bridge configuration connecting Helios-Terminal to the Pi runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeliosConfig {
    /// Path to the `pi` binary. Defaults to auto-detection from PATH.
    pub pi_binary: PathBuf,

    /// Default agent identity to use when spawning Pi (e.g. "helios").
    pub default_agent: String,

    /// Pi home directory containing providers, skills, and extensions.
    pub pi_home: PathBuf,

    /// Whether to enable helios-agi cloud auth for team/sync features.
    pub cloud_auth: bool,
}

impl Default for HeliosConfig {
    fn default() -> Self {
        let pi_binary = which::which("pi")
            .unwrap_or_else(|_| PathBuf::from("pi"));

        let pi_home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pi");

        Self {
            pi_binary,
            default_agent: "helios".to_string(),
            pi_home,
            cloud_auth: false,
        }
    }
}

impl HeliosConfig {
    /// Load configuration from `~/.helios-terminal/config.toml`, falling back
    /// to [`Default`] values for any missing fields.
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_file_path();

        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let config: Self = toml::from_str(&contents)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    /// Returns the path to the configuration file:
    /// `~/.helios-terminal/config.toml`
    pub fn config_file_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".helios-terminal")
            .join("config.toml")
    }
}
