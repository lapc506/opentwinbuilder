//! A2A adapter (infrastructure layer) — binds concrete capabilities to the
//! application-layer orchestrator.
//!
//! Each adapter implements `ports::capability::Capability` for an internal tool.
//! Thingport is exposed as `thingport.search` / `thingport.fetch` /
//! `thingport.import` (mirroring the inbound MCP tool names).

use std::sync::Arc;

use serde_json::{json, Value};

use crate::application::a2a::A2AOrchestrator;
use crate::infrastructure::thingport::ThingportClient;
use crate::ports::capability::{Capability, CapabilityResult};
use crate::ports::model_source::ModelSource;

/// Register the Thingport capability with the orchestrator.
pub fn register_thingport(orchestrator: &mut A2AOrchestrator, client: ThingportClient) {
    orchestrator.register(Arc::new(ThingportCapability::new(client)));
}

/// Exposes the Thingport model library as callable A2A capabilities.
pub struct ThingportCapability {
    client: ThingportClient,
}

impl ThingportCapability {
    pub fn new(client: ThingportClient) -> Self {
        Self { client }
    }
}

impl Capability for ThingportCapability {
    fn names(&self) -> Vec<&'static str> {
        vec!["thingport.search", "thingport.fetch", "thingport.import"]
    }

    fn invoke(&self, name: &str, args: Value) -> CapabilityResult {
        match self.dispatch(name, args) {
            Ok(output) => CapabilityResult::ok(output),
            Err(message) => CapabilityResult::err(message),
        }
    }
}

impl ThingportCapability {
    fn dispatch(&self, name: &str, args: Value) -> Result<Value, String> {
        // Jidoka: fail loudly rather than silently no-op when unconfigured.
        if !self.client.is_configured() {
            return Err("thingport not configured (set THINGPORT_URL and THINGPORT_TOKEN)".to_string());
        }
        match name {
            "thingport.search" => {
                let query = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "thingport.search: missing string 'query'".to_string())?;
                let items = self.client.search(query)?;
                let models: Vec<Value> = items
                    .iter()
                    .map(|m| json!({"id": m.id, "name": m.name}))
                    .collect();
                Ok(json!({"models": models}))
            }
            "thingport.import" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "thingport.import: missing string 'id'".to_string())?;
                let _model = self.client.fetch(id)?;
                // TP3: the real Bevy viewport import is pending; the fetch step
                // is wired and the capability reports the honest state.
                Ok(json!({
                    "model_id": id,
                    "fetched": true,
                    "imported_to_viewport": false,
                    "note": "bevy viewport import pending (TP3)"
                }))
            }
            "thingport.fetch" => {
                let id = args
                    .get("id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "thingport.fetch: missing string 'id'".to_string())?;
                let model = self.client.fetch(id)?;
                Ok(json!({"model": model, "imported_to_viewport": false}))
            }
            _ => Err(format!("unhandled capability name: {name}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    /// A throwaway HTTP stub answering Thingport-shaped responses. It serves a
    /// handful of requests (one per `invoke` in a test) on a thread.
    fn spawn_thingport_stub() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind stub");
        let addr = listener.local_addr().expect("local addr");
        let url = format!("http://{addr}");
        thread::spawn(move || {
            for _ in 0..4 {
                let Ok((mut stream, _)) = listener.accept() else { break };
                let mut buf = [0u8; 4096];
                let Ok(n) = stream.read(&mut buf) else { break };
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let body = if req.contains("/api/models?search=") {
                    r#"[{"id":"m1","name":"gear"}]"#
                } else {
                    r#"{"id":"m1","name":"gear"}"#
                };
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(resp.as_bytes());
            }
        });
        url
    }

    fn orchestrator_with_thingport(url: &str) -> A2AOrchestrator {
        let mut orch = A2AOrchestrator::new();
        register_thingport(&mut orch, ThingportClient::new(url.to_string(), "test-token"));
        orch
    }

    #[test]
    fn registers_thingport_names() {
        let orch = orchestrator_with_thingport("http://localhost:80");

        assert!(orch.has("thingport.search"));
        assert!(orch.has("thingport.fetch"));
        assert!(orch.has("thingport.import"));
        assert_eq!(
            orch.list(),
            vec!["thingport.fetch", "thingport.import", "thingport.search"]
        );
    }

    #[test]
    fn thingport_search_dispatches_by_name() {
        let base = spawn_thingport_stub();
        let orch = orchestrator_with_thingport(&base);

        // When the orchestrator dispatches `thingport.search` by name…
        let res = orch.invoke("thingport.search", json!({"query": "gear"}));

        // …the registered Thingport capability runs and returns the stub models.
        assert!(res.ok, "expected ok, got {:?}", res.error);
        assert_eq!(res.output["models"][0]["id"], "m1");
        assert_eq!(res.output["models"][0]["name"], "gear");
    }

    #[test]
    fn thingport_fetch_dispatches_by_name() {
        let base = spawn_thingport_stub();
        let orch = orchestrator_with_thingport(&base);

        let res = orch.invoke("thingport.fetch", json!({"id": "m1"}));

        assert!(res.ok, "expected ok, got {:?}", res.error);
        assert_eq!(res.output["model"]["id"], "m1");
    }

    #[test]
    fn thingport_import_dispatches_by_name() {
        let base = spawn_thingport_stub();
        let orch = orchestrator_with_thingport(&base);

        let res = orch.invoke("thingport.import", json!({"id": "m1"}));

        assert!(res.ok, "expected ok, got {:?}", res.error);
        assert_eq!(res.output["model_id"], "m1");
        assert_eq!(res.output["imported_to_viewport"], false);
    }

    #[test]
    fn unknown_capability_returns_error() {
        let orch = orchestrator_with_thingport("http://localhost:80");

        let res = orch.invoke("no.such.capability", json!({}));

        assert!(!res.ok);
        assert!(res.error.unwrap().contains("unknown capability"));
    }

    #[test]
    fn unconfigured_thingport_rejects() {
        // Jidoka: a not-configured Thingport must refuse, not silently no-op.
        let mut orch = A2AOrchestrator::new();
        register_thingport(&mut orch, ThingportClient::new("", ""));

        let res = orch.invoke("thingport.search", json!({"query": "x"}));

        assert!(!res.ok);
        assert!(res.error.unwrap().contains("not configured"));
    }
}
