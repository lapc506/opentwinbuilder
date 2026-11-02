//! Application layer — use cases / orchestration.
//!
//! Depends only on domain + ports. No Bevy, no Iced, no MCP transport details.

pub mod a2a;
pub mod studio;

pub use a2a::A2AOrchestrator;
pub use studio::Studio;
