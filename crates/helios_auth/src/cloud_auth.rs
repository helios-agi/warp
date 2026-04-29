//! helios-agi cloud OAuth (stub).
//!
//! Will implement OAuth2 PKCE flow to helios-agi.dev when cloud features
//! are ready. For now, this module always reports "not authenticated" so
//! the terminal can gracefully degrade to offline-only mode.

/// Cloud authentication state for helios-agi team/sync features.
pub struct HeliosCloudAuth {
    // TODO: Store OAuth2 tokens, refresh logic, and session state.
    //
    // Planned implementation:
    //   1. OAuth2 PKCE flow against https://auth.helios-agi.dev
    //   2. Token storage in OS keychain (via keyring crate)
    //   3. Automatic refresh with exponential backoff
    //   4. Graceful degradation — all core features work without cloud auth
}

impl HeliosCloudAuth {
    /// Create a new (unauthenticated) cloud auth handle.
    pub fn new() -> Self {
        Self {}
    }

    /// Returns `true` if the user has an active helios-agi cloud session.
    ///
    /// Currently always returns `false` — cloud auth is not yet implemented.
    pub fn is_authenticated(&self) -> bool {
        false
    }

    // TODO: pub async fn start_login_flow(&mut self) -> Result<()>
    // TODO: pub async fn refresh_token(&mut self) -> Result<()>
    // TODO: pub fn logout(&mut self)
}

impl Default for HeliosCloudAuth {
    fn default() -> Self {
        Self::new()
    }
}
