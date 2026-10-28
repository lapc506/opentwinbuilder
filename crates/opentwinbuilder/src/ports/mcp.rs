//! MCP port — the Studio's tool surface as an interface.
//!
//! The application layer defines *what* tools exist; the infrastructure layer
//! provides the concrete stateless vitemcp server that speaks the protocol.

use serde_json::Value;

/// A single MCP tool = one operation the Studio exposes.
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
}

/// Stateless MCP server contract. No session state anywhere.
pub trait McpServer {
    /// Protocol revision we speak (stateless).
    fn protocol_version(&self) -> &'static str;
    /// `tools/list`.
    fn list_tools(&self) -> Value;
    /// `tools/call`.
    fn call_tool(&self, name: &str, args: Value) -> Value;
}
