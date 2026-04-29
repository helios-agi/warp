//! Pi runtime detection.
//!
//! Checks whether the `pi` binary is installed, its version, and whether
//! provider configuration exists — everything needed before spawning a session.

#![allow(clippy::disallowed_types)]

use std::path::PathBuf;
use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

/// Create a Command that doesn't flash a console window on Windows.
fn pi_command(program: &str) -> Command {
    #[cfg(windows)]
    {
        let mut cmd = Command::new(program);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        cmd
    }
    #[cfg(not(windows))]
    {
        Command::new(program)
    }
}

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
        which::which("helios").is_ok() || which::which("pi").is_ok()
    }

    /// Runs `pi --version` and returns the version string, if available.
    pub fn pi_version(&self) -> Option<String> {
        self.pi_version_with_binary("helios")
            .or_else(|| self.pi_version_with_binary("pi"))
    }

    /// Runs `<binary> --version` and returns the version string, if available.
    /// Exposed for testing.
    fn pi_version_with_binary(&self, binary: &str) -> Option<String> {
        pi_command(binary)
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
        let status = self.detect_with_binary("helios");
        if status != PiStatus::NotInstalled {
            status
        } else {
            self.detect_with_binary("pi")
        }
    }

    /// Run detection with a custom binary name (for testing).
    pub fn detect_with_binary(&self, binary: &str) -> PiStatus {
        if which::which(binary).is_err() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_returns_not_installed_when_pi_missing() {
        let detector = PiDetector::default_home();
        let status = detector.detect_with_binary("nonexistent-binary-xyz-12345");
        assert!(matches!(status, PiStatus::NotInstalled));
    }

    #[test]
    fn pi_status_variants() {
        // Ensure all variants are constructible
        let _ready = PiStatus::Ready;
        let _not_installed = PiStatus::NotInstalled;
        let _no_providers = PiStatus::NoProviders;
        let _version_mismatch = PiStatus::VersionMismatch {
            found: "0.1.0".to_string(),
            required: "0.2.0".to_string(),
        };
    }
}
