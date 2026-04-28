//! Pi subprocess management.
//!
//! Wraps a long-running `pi` child process, providing methods to send input,
//! check liveness, and tear down the session.

use std::path::Path;

use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

use crate::config::HeliosConfig;

/// A running Pi session backed by a tokio child process.
pub struct PiSession {
    child: Child,
}

impl PiSession {
    /// Spawn a new Pi session in the given working directory.
    ///
    /// The process runs `pi --agent <agent>` as a subprocess with piped
    /// stdin/stdout/stderr so the terminal can relay I/O.
    ///
    /// **Environment inheritance:** The Pi subprocess inherits ALL parent
    /// environment variables by default. This is intentional — Pi needs
    /// provider API keys (ANTHROPIC_API_KEY, OPENAI_API_KEY, etc.) from
    /// the environment to authenticate with LLM providers.
    pub async fn spawn(config: &HeliosConfig, cwd: &Path) -> anyhow::Result<Self> {
        let child = Command::new(&config.pi_binary)
            .args(["--agent", &config.default_agent])
            .current_dir(cwd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        Ok(Self { child })
    }

    /// Write a line of input to the Pi process's stdin.
    pub async fn send_input(&mut self, input: &str) -> anyhow::Result<()> {
        if let Some(stdin) = self.child.stdin.as_mut() {
            stdin.write_all(input.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
            Ok(())
        } else {
            anyhow::bail!("Pi session stdin is not available")
        }
    }

    /// Check whether the Pi process is still running.
    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Kill the Pi process.
    pub async fn kill(&mut self) -> anyhow::Result<()> {
        self.child.kill().await?;
        Ok(())
    }

    // TODO: Structured output parsing — parse Pi's stdout for tool calls,
    //       diffs, and agent actions to map into Warp's action_result types.

    // TODO: PTY integration — replace raw stdin/stdout pipes with a proper
    //       pseudo-terminal for full interactive support.

    // TODO: Session restore — persist session state so Pi can resume after
    //       terminal restart.
}
