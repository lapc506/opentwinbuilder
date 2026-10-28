//! Infrastructure layer — framework adapters.
//!
//! Implements the ports with concrete technologies: Bevy (3D viewport), Iced
//! (UI), and a stateless vitemcp MCP server. This is the only layer that knows
//! about Bevy/Iced/transport.

pub mod mcp_server;
pub mod viewport_bevy;
