//! Studio application service — the use-cases of the desktop app.
//!
//! Orchestrates domain rules and port interfaces. It holds no framework types.

use crate::domain::twin::TwinEntity;
use crate::ports::viewport::{SelectionHandle, Viewport};

/// The Studio application service.
pub struct Studio {
    /// Shared selection state (bridges viewport ↔ UI).
    pub selection: SelectionHandle,
}

impl Studio {
    pub fn new(selection: SelectionHandle) -> Self {
        Self { selection }
    }

    /// The currently selected twin (read via port, never via Bevy directly).
    pub fn selected_twin(&self) -> Option<TwinEntity> {
        self.selection
            .selection()
            .map(|id| TwinEntity::new(id.0, "selected-twin"))
    }
}
