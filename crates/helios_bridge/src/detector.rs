//! Pi runtime detection.
//!
//! Checks whether the `pi` binary is installed, its version, and whether
//! provider configuration exists — everything needed before spawning a session.

use std::path::PathBuf;
use std::process::Command;

/// Status of the local Pi runtime installation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PiStatus {
    /// Pi is installed, has providers configured, and is ready to use.
    Ready,
    /// The `pi` binary was not found on PATH.
    NotInstalled,
    /// Pi is installed but `~/.pi/providers.json` is missing.
    NoProviders,
    /// Pi is installed but the version doesn't meet the minimum requirement.
    VersionMismatch {
        found: String,
        required: String,
    },
}

/// Detects the state of the local Pi runtime.
pub struct PiDetector {
    pi_home: PathBuf,
}

impl PiDetector {
    /// Create a new detector that checks the given Pi home directory.
    pub fn new(pi_home: PathBuf) -> Self {
        Self { pi_home }
    }

    /// Create a detector using the default Pi home (`~/.pi`).
    pub fn default_home() -> Self {
        let pi_home = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".pi");
        Self::new(pi_home)
    }

    /// Returns `true` if the `pi` binary is found on PATH.
    pub fn is_pi_installed(&self) -> bool {
        which::which("pi").is_ok()
    }

    /// Runs `pi --version` and returns the version string, if available.
    pub fn pi_version(&self) -> Option<String> {
        Command::new("pi")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| {
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .ok()
                        .map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    }

    /// Returns `true` if `~/.pi/providers.json` exists.
    pub fn has_provider_config(&self) -> bool {
        self.pi_home.join("providers.json").exists()
    }

    /// Run all detection checks and return an overall [`PiStatus`].
    pub fn detect(&self) -> PiStatus {
        if !self.is_pi_installed() {
            return PiStatus::NotInstalled;
        }

        if !self.has_provider_config() {
            return PiStatus::NoProviders;
        }

        // TODO: Add version comparison against a minimum required version.
        // For now, any installed version with providers is considered ready.
        PiStatus::Ready
    }
}
