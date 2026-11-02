//! Capability port — the smallest unit the A2A orchestrator dispatches to.
//!
//! The Studio's *inbound* surface is the MCP server (`infrastructure::mcp_server`).
//! This port models the *outbound* side: a named, invokeable agent/tool (e.g.
//! `thingport.search`). The application-layer orchestrator holds only this
//! interface; concrete implementations (Thingport, CAD, …) live in infrastructure.

use serde_json::Value;

/// Outcome of invoking a capability.
#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityResult {
    /// Whether invocation succeeded.
    pub ok: bool,
    /// Structured output when `ok` is true, `Value::Null` otherwise.
    pub output: Value,
    /// Error message when `ok` is false.
    pub error: Option<String>,
}

impl CapabilityResult {
    /// Build a successful result.
    pub fn ok(output: Value) -> Self {
        Self {
            ok: true,
            output,
            error: None,
        }
    }

    /// Build a failed result (Jidoka: suppressed/blank results are not success).
    pub fn err(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            output: Value::Null,
            error: Some(message.into()),
        }
    }
}

/// A callable capability in the A2A orchestrator.
///
/// An implementation answers to one or more names. The name the caller actually
/// invoked is passed to [`Capability::invoke`] so a multi-name capability can
/// route internally (Thingport answers to `thingport.search` / `thingport.fetch`
/// / `thingport.import`).
pub trait Capability {
    /// The name(s) this capability is registered under and answers to.
    fn names(&self) -> Vec<&'static str>;

    /// Invoke the capability by name with JSON arguments.
    fn invoke(&self, name: &str, args: Value) -> CapabilityResult;
}
