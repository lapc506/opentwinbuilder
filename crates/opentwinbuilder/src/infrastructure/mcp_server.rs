//! Stateless vitemcp MCP server — infrastructure adapter of the `McpServer` port.
//!
//! Targets protocol revision 2024-11-05 with NO `Mcp-Session-Id` (stateless).
//! This adapter knows the wire protocol; it does not hold domain logic.

use serde_json::{json, Value};

use crate::ports::mcp::McpServer;

/// Protocol revision we speak — stateless, no session.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// A stateless vitemcp server implementation.
#[derive(Debug, Clone)]
pub struct ViteMcpServer {
    pub name: &'static str,
    pub version: &'static str,
}

impl Default for ViteMcpServer {
    fn default() -> Self {
        Self {
            name: "opentwinbuilder",
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

impl ViteMcpServer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl McpServer for ViteMcpServer {
    fn protocol_version(&self) -> &'static str {
        PROTOCOL_VERSION
    }

    fn list_tools(&self) -> Value {
        json!({
            "tools": [
                {
                    "name": "capability_report",
                    "description": "Report the Studio's stateless vitemcp capabilities",
                    "inputSchema": { "type": "object", "properties": {} },
                }
            ]
        })
    }

    fn call_tool(&self, name: &str, _args: Value) -> Value {
        match name {
            "capability_report" => json!({
                "protocol": PROTOCOL_VERSION,
                "stateless": true,
                "session_id": Value::Null,
                "capabilities": ["3d-viewport", "entity-picking", "rv-odt", "opensusd"],
            }),
            _ => json!({
                "isError": true,
                "content": [ { "type": "text", "text": format!("unknown tool: {name}") } ],
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stateless_protocol_no_session() {
        let s = ViteMcpServer::new();
        assert_eq!(s.protocol_version(), PROTOCOL_VERSION);
        assert_eq!(s.list_tools()["tools"].as_array().unwrap().len(), 1);
        let res = s.call_tool("capability_report", json!({}));
        assert!(res["stateless"].as_bool().unwrap());
        assert!(res["session_id"].is_null());
    }
}
