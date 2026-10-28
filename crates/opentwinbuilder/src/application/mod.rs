//! Application layer — use cases / orchestration.
//!
//! Depends only on domain + ports. No Bevy, no Iced, no MCP transport details.

pub mod studio;

pub use studio::Studio;
