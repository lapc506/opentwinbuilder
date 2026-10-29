//! Stateless vitemcp MCP server — infrastructure adapter of the `McpServer` port.
//!
//! Targets protocol revision 2024-11-05 with NO `Mcp-Session-Id` (stateless).
//! This adapter knows the wire protocol; it does not hold domain logic.

use serde_json::{json, Value};

use crate::infrastructure::thingport::ThingportClient;
use crate::ports::mcp::McpServer;
use crate::ports::model_source::ModelSource;

/// Protocol revision we speak — stateless, no session.
pub const PROTOCOL_VERSION: &str = "2024-11-05";

/// A stateless vitemcp server implementation.
#[derive(Debug, Clone)]
pub struct ViteMcpServer {
    pub name: &'static str,
    pub version: &'static str,
    /// Thingport model library source (on-demand assets).
    pub thingport: ThingportClient,
}

impl Default for ViteMcpServer {
    fn default() -> Self {
        Self {
            name: "opentwinbuilder",
            version: env!("CARGO_PKG_VERSION"),
            thingport: ThingportClient::new("", ""),
        }
    }
}

impl ViteMcpServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the Thingport source (URL + API token).
    pub fn with_thingport(mut self, thingport: ThingportClient) -> Self {
        self.thingport = thingport;
        self
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
                },
                {
                    "name": "thingport.search",
                    "description": "Search the Thingport model library (no download)",
                    "inputSchema": { "type": "object", "properties": {
                        "query": { "type": "string" }
                    } },
                },
                {
                    "name": "thingport.fetch",
                    "description": "Fetch one model by id from Thingport (on-demand download)",
                    "inputSchema": { "type": "object", "properties": {
                        "id": { "type": "string" }
                    } },
                },
                {
                    "name": "thingport.import",
                    "description": "Fetch a model and signal it for import into the viewport",
                    "inputSchema": { "type": "object", "properties": {
                        "id": { "type": "string" }
                    } },
                },
            ]
        })
    }

    fn call_tool(&self, name: &str, args: Value) -> Value {
        let ask = |key: &str| -> Result<String, String> {
            args.get(key)
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .ok_or_else(|| format!("missing arg '{key}'"))
        };
        let not_configured = || err("thingport not configured (set THINGPORT_URL and THINGPORT_TOKEN)".into());
        match name {
            "capability_report" => json!({
                "protocol": PROTOCOL_VERSION,
                "stateless": true,
                "session_id": Value::Null,
                "capabilities": ["3d-viewport", "entity-picking", "rv-odt", "opensusd", "thingport"],
            }),
            "thingport.search" => {
                let query = match ask("query") {
                    Ok(q) => q,
                    Err(e) => return err(e),
                };
                if !self.thingport.is_configured() {
                    return not_configured();
                }
                match self.thingport.search(&query) {
                    Ok(items) => json!({
                        "models": items.iter().map(|m| json!({"id": m.id, "name": m.name})).collect::<Vec<_>>()
                    }),
                    Err(e) => err(e),
                }
            }
            "thingport.fetch" | "thingport.import" => {
                let id = match ask("id") {
                    Ok(i) => i,
                    Err(e) => return err(e),
                };
                if !self.thingport.is_configured() {
                    return not_configured();
                }
                match self.thingport.fetch(&id) {
                    Ok(model) => json!({
                        "model": model,
                        "imported_to_viewport": name == "thingport.import",
                    }),
                    Err(e) => err(e),
                }
            }
            _ => err(format!("unknown tool: {name}")),
        }
    }
}

fn err(msg: String) -> Value {
    json!({
        "isError": true,
        "content": [ { "type": "text", "text": msg } ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stateless_protocol_no_session() {
        let s = ViteMcpServer::new();
        assert_eq!(s.protocol_version(), PROTOCOL_VERSION);
        assert_eq!(s.list_tools()["tools"].as_array().unwrap().len(), 4);
        let res = s.call_tool("capability_report", json!({}));
        assert!(res["stateless"].as_bool().unwrap());
        assert!(res["session_id"].is_null());
    }

    #[test]
    fn thingport_unconfigured_reports_error() {
        let s = ViteMcpServer::new();
        let res = s.call_tool("thingport.search", json!({"query": "simready"}));
        assert_eq!(res["isError"].as_bool(), Some(true));
        let txt = res["content"][0]["text"].as_str().unwrap_or("");
        assert!(txt.contains("not configured"), "got: {txt}");
    }
}
