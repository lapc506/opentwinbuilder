//! Viewport port — the 3D surface as an interface.
//!
//! The application layer reads selection state through this trait without knowing
//! it is Bevy underneath. Infrastructure implements it with the Bevy plugin.

use crate::domain::twin::{Selection, TwinId};

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
