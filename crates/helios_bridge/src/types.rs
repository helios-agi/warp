//! Compatibility types for agent actions.
//!
//! These types serve as a bridge between Pi's structured output and Warp's
//! existing agent UI. They will replace the corresponding types from the `ai`
//! crate as the migration progresses.

use std::path::PathBuf;

/// The kind of action an agent can perform.
///
/// TODO: These types will be populated by parsing Pi's structured output.
/// For now they mirror the shapes Warp's UI expects so the bridge can
/// translate between the two worlds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentActionType {
    /// A file creation or edit.
    FileEdit,
    /// A shell command execution.
    BashCommand,
    /// A codebase or web search.
    Search,
    /// Any other action not yet categorised.
    Other(String),
}

/// A file edit produced by an agent action.
#[derive(Debug, Clone)]
pub struct FileEdit {
    /// Absolute or workspace-relative path to the target file.
    pub path: PathBuf,
    /// Full file content after the edit.
    pub content: String,
    /// Optional unified diff showing what changed.
    pub diff: Option<String>,
}
