//! Viewport port — the 3D surface as an interface.
//!
//! The application layer reads selection state through this trait without knowing
//! it is Bevy underneath. Infrastructure implements it with the Bevy plugin.

use crate::domain::twin::{Selection, TwinId};

/// A request to bring a fetched model into the 3D scene.
#[derive(Debug, Clone)]
pub struct ImportRequest {
    pub model_id: String,
    pub name: String,
    pub source: &'static str,
    /// Raw fetched model payload from the source (e.g. Thingport JSON).
    pub data: serde_json::Value,
}

/// Shared inbound queue of pending model imports. The application/MCP tool push
/// requests here; the concrete viewport adapter (Bevy) drains them and spawns
/// scene entities. Rust-analog of an event/observer boundary, like `SelectionHandle`.
#[derive(Debug, Default, Clone, bevy::ecs::resource::Resource)]
pub struct ImportHandle(pub std::sync::Arc<std::sync::Mutex<Vec<ImportRequest>>>);

impl ImportHandle {
    /// Queue one import request. Fails if the shared queue is poisoned/locked.
    pub fn push(&self, req: ImportRequest) -> Result<(), String> {
        self.0
            .lock()
            .map(|mut q| q.push(req))
            .map_err(|e| format!("import queue lock poisoned: {e}"))
    }

    /// Drain all pending import requests.
    pub fn take_all(&self) -> Result<Vec<ImportRequest>, String> {
        self.0
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .map_err(|e| format!("import queue lock poisoned: {e}"))
    }
}

/// The interactive 3D viewport. Selection is the key read the UI binds to.
pub trait Viewport {
    /// Current selection (hover/click) as maintained by the viewport.
    fn selection(&self) -> Option<TwinId>;
}

/// A shared handle the viewport writes and the UI reads.
/// Rust-analog of an event/observer boundary: the concrete Bevy adapter mutates
/// this; the Iced side consumes it.
#[derive(Debug, Default, Clone, bevy::ecs::resource::Resource)]
pub struct SelectionHandle(pub std::sync::Arc<std::sync::Mutex<Selection>>);

impl SelectionHandle {
    pub fn set(&self, sel: Selection) {
        if let Ok(mut g) = self.0.lock() {
            *g = sel;
        }
    }
    pub fn get(&self) -> Selection {
        self.0.lock().map(|g| g.clone()).unwrap_or_default()
    }
}

impl Viewport for SelectionHandle {
    fn selection(&self) -> Option<TwinId> {
        self.get().picked
    }
}
