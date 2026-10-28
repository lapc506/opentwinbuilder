//! Digital-twin domain model (rv-ODT flavor).
//!
//! Pure types only — no Bevy, no Iced, no MCP, no IO. This is the core the
//! application layer orchestrates and the infrastructure adapts.

use serde::{Deserialize, Serialize};

/// Strong id for a twin object. Newtype keeps ids distinct from raw strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TwinId(pub String);

/// A "digital twin" entity in the viewport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwinEntity {
    pub id: TwinId,
    pub name: String,
}

/// The selection state maintained by the viewport and mirrored to the UI.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub picked: Option<TwinId>,
}

impl TwinEntity {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: TwinId(id.into()),
            name: name.into(),
        }
    }
}
