use std::collections::HashMap;

/// Unique identifier for a canvas tab.
pub type CanvasTabId = String;

/// The kind of content a canvas view renders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanvasViewKind {
    /// A WKWebView overlay loading a bundled HTML file.
    WebView { filename: String },
}

/// A single tab in the canvas's internal tab bar.
#[derive(Debug, Clone)]
pub struct CanvasTab {
    pub id: CanvasTabId,
    /// The registry key this tab maps to (e.g., "interview", "helios:brain").
    pub view_key: String,
    /// Display label in the tab bar.
    pub label: String,
    /// Whether this tab is pinned (cannot be closed by bulk close).
    pub pinned: bool,
    /// Optional metadata passed to the view (e.g., interview questions JSON).
    pub metadata: HashMap<String, String>,
    /// Timestamp of last access for LRU eviction.
    pub last_accessed: u64,
}

/// Category of a registered canvas view (for command palette grouping).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasViewCategory {
    Core,
    Intelligence,
    Tools,
    Admin,
}
