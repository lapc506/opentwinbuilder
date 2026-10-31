//! MCP port — the Studio's inbound tool surface.
//!
//! The inbound MCP surface is implemented by `infrastructure::mcp_server::StudioMcpServer`
//! over the official `rmcp` (Rust SDK). This module documents the boundary: the
//! application/domain define *what* capabilities exist (versioning, feature factory,
//! BOP, Thingport); the infrastructure adapter exposes them as `rmcp` `#[tool]`s.
//!
//! The A2A orchestrator (outbound dispatch to external agents) is a SEPARATE layer
//! and does not live here.

/// The tool surface the Studio exposes inbound (for documentation / capability report).
pub const INBOUND_TOOLS: &[&str] = &[
    "capability_report",
    "model.commit",
    "model.branch",
    "model.merge",
    "model.history",
    "workspace.protect",
    "feature.generate",
    "feature.test",
    "feature.register",
    "bop.build",
    "bop.link",
    "thingport.search",
    "thingport.fetch",
    "thingport.import",
];
