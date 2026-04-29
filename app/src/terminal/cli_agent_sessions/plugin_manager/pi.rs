//! Plugin manager for Pi (helios-agi) — the default agent for Helios-Terminal.
//!
//! Unlike other CLI agents, Pi is the primary agent in Helios-Terminal.
//! This plugin manager handles detection and setup via npm.

use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;

use async_trait::async_trait;

use super::{CliAgentPluginManager, PluginInstructionStep, PluginInstructions};
use crate::terminal::model::session::LocalCommandExecutor;
use crate::terminal::shell::ShellType;

const MINIMUM_PI_VERSION: &str = "1.0.0";

pub(super) struct PiPluginManager {
    _executor: LocalCommandExecutor,
}

impl PiPluginManager {
    pub(super) fn new(
        shell_path: Option<PathBuf>,
        shell_type: Option<ShellType>,
        _path_env_var: Option<String>,
    ) -> Self {
        let shell_type = shell_type.unwrap_or(ShellType::Bash);
        Self {
            _executor: LocalCommandExecutor::new(shell_path, shell_type),
        }
    }
}

#[async_trait]
impl CliAgentPluginManager for PiPluginManager {
    fn minimum_plugin_version(&self) -> &'static str {
        MINIMUM_PI_VERSION
    }

    fn can_auto_install(&self) -> bool {
        false
    }

    fn is_installed(&self) -> bool {
        // Check if `pi` is on PATH
        Command::new("which")
            .arg("pi")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn install_instructions(&self) -> &'static PluginInstructions {
        &INSTALL_INSTRUCTIONS
    }

    fn update_instructions(&self) -> &'static PluginInstructions {
        &UPDATE_INSTRUCTIONS
    }

    fn supports_update(&self) -> bool {
        true
    }

    fn needs_update(&self) -> bool {
        // For now, we don't check version. User can manually update via npm.
        false
    }
}

static INSTALL_INSTRUCTIONS: LazyLock<PluginInstructions> = LazyLock::new(|| {
    PluginInstructions {
        title: "Set up Pi for Helios-Terminal",
        subtitle: "Pi is the AI agent runtime for Helios-Terminal.",
        steps: &[
            PluginInstructionStep {
                description: "Install Pi using npm:",
                command: "npm install -g @anthropic-ai/pi",
                executable: true,
                link: Some("https://github.com/helios-agi"),
            },
        ],
        post_install_notes: &[
            "Pi will automatically connect to Helios-Terminal via the Warp CLI agent protocol.",
        ],
    }
});

static UPDATE_INSTRUCTIONS: LazyLock<PluginInstructions> = LazyLock::new(|| {
    PluginInstructions {
        title: "Update Pi for Helios-Terminal",
        subtitle: "Update Pi to the latest version.",
        steps: &[
            PluginInstructionStep {
                description: "Update Pi using npm:",
                command: "npm update -g @anthropic-ai/pi",
                executable: true,
                link: Some("https://github.com/helios-agi"),
            },
        ],
        post_install_notes: &[
            "Restart your terminal session to use the updated version.",
        ],
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pi_minimum_version_is_set() {
        let manager = PiPluginManager::new(None, None, None);
        assert_eq!(manager.minimum_plugin_version(), "1.0.0");
    }

    #[test]
    fn pi_cannot_auto_install() {
        let manager = PiPluginManager::new(None, None, None);
        assert!(!manager.can_auto_install());
    }

    #[test]
    fn pi_supports_update() {
        let manager = PiPluginManager::new(None, None, None);
        assert!(manager.supports_update());
    }
}
