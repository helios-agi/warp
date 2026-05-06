use std::collections::HashMap;
use std::sync::OnceLock;

use super::types::{CanvasViewCategory, CanvasViewKind};

/// A registered canvas view definition.
#[derive(Debug, Clone)]
pub struct CanvasViewRegistration {
    /// Unique key (e.g., "interview", "helios:brain", "design-deck").
    pub key: String,
    /// Human-readable label for tab bar and command palette.
    pub label: String,
    /// Category for command palette grouping.
    pub category: CanvasViewCategory,
    /// What kind of content this view renders.
    pub kind: CanvasViewKind,
    /// Optional icon name.
    pub icon: Option<String>,
}

static REGISTRY: OnceLock<std::sync::RwLock<HashMap<String, CanvasViewRegistration>>> =
    OnceLock::new();

fn registry() -> &'static std::sync::RwLock<HashMap<String, CanvasViewRegistration>> {
    REGISTRY.get_or_init(|| std::sync::RwLock::new(HashMap::new()))
}

/// Register a canvas view. Call during app init.
pub fn register_canvas_view(reg: CanvasViewRegistration) {
    let mut map = registry().write().unwrap();
    if map.contains_key(&reg.key) {
        log::warn!(
            "[canvas-registry] Duplicate registration for key '{}', overwriting.",
            reg.key
        );
    }
    map.insert(reg.key.clone(), reg);
}

/// Resolve a view key to its registration.
pub fn resolve_canvas_view(key: &str) -> Option<CanvasViewRegistration> {
    let map = registry().read().unwrap();
    map.get(key).cloned()
}

/// Get all registered views (for command palette).
pub fn all_canvas_views() -> Vec<CanvasViewRegistration> {
    let map = registry().read().unwrap();
    map.values().cloned().collect()
}

/// Get views filtered by category.
#[allow(dead_code)]
pub fn canvas_views_by_category(category: CanvasViewCategory) -> Vec<CanvasViewRegistration> {
    let map = registry().read().unwrap();
    map.values()
        .filter(|r| r.category == category)
        .cloned()
        .collect()
}
