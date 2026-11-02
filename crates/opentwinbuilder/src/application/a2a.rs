//! A2A orchestration (application layer) — the *outbound* capability dispatch.
//!
//! The Studio's inbound surface is the MCP server (`infrastructure::mcp_server`).
//! This module is the opposite side: it lets an orchestrator register callable
//! capabilities (agents/tools) by name and invoke them. It depends only on the
//! `ports::capability` interface — never on concrete infrastructure, Bevy, or
//! MCP transport details.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::ports::capability::{Capability, CapabilityResult};

/// Minimal A2A orchestrator: a name → capability registry with dispatch by name.
#[derive(Default)]
pub struct A2AOrchestrator {
    registry: HashMap<String, Arc<dyn Capability + Send + Sync>>,
}

impl A2AOrchestrator {
    /// Create an empty orchestrator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a capability under every name it answers to.
    pub fn register(&mut self, capability: Arc<dyn Capability + Send + Sync>) {
        for name in capability.names() {
            self.registry.insert(name.to_string(), Arc::clone(&capability));
        }
    }

    /// Whether a capability is registered under `name`.
    pub fn has(&self, name: &str) -> bool {
        self.registry.contains_key(name)
    }

    /// All registered capability names (sorted, for a stable capability report).
    pub fn list(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.registry.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    /// Dispatch a call to a capability by its registered name.
    pub fn invoke(&self, name: &str, args: Value) -> CapabilityResult {
        match self.registry.get(name) {
            Some(capability) => capability.invoke(name, args),
            None => CapabilityResult::err(format!("unknown capability: {name}")),
        }
    }
}
