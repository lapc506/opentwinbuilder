//! Ports layer — interfaces the domain/application depend on.
//!
//! These are traits (the Rust analog of ports/hexagonal boundaries). The
//! infrastructure layer implements them; the application layer consumes them.
//! Nothing here knows about Bevy or Iced.

pub mod capability;
pub mod mcp;
pub mod model_source;
pub mod viewport;

pub use capability::{Capability, CapabilityResult};
pub use model_source::ModelSource;
pub use viewport::Viewport;
pub use viewport::{ImportHandle, ImportRequest};
