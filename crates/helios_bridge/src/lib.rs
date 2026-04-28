//! Helios Bridge — thin integration between Helios-Terminal and Pi runtime
//!
//! Warp = the terminal shell. Pi = the brain. This crate is the plumbing.

pub mod config;
pub mod detector;
pub mod pi_session;
pub mod types;

pub use config::HeliosConfig;
pub use detector::PiDetector;
pub use pi_session::PiSession;
