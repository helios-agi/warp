//! Helios Auth — hybrid authentication for Helios-Terminal
//!
//! Layer 1: Pi local auth (~/.pi/providers.json) — always works, offline capable
//! Layer 2: helios-agi cloud OAuth — optional, unlocks team/sync features

pub mod cloud_auth;
pub mod local_auth;

pub use cloud_auth::HeliosCloudAuth;
pub use local_auth::PiLocalAuth;
