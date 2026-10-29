//! Ports layer — interfaces the domain/application depend on.
//!
//! These are traits (the Rust analog of ports/hexagonal boundaries). The
//! infrastructure layer implements them; the application layer consumes them.
//! Nothing here knows about Bevy or Iced.

pub mod mcp;
pub mod model_source;
pub mod viewport;

pub use mcp::McpServer;
pub use model_source::ModelSource;
pub use viewport::Viewport;
