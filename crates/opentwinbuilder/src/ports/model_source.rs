//! ModelSource port — fetch 3D models/assets on demand from an external library
//! (e.g. a self-hosted Thingport instance). The application/domain depend on this
//! interface; the infrastructure layer implements it with the concrete HTTP client.

use serde_json::Value;

/// A single model summary returned by a source (no download yet).
#[derive(Debug, Clone)]
pub struct ModelSummary {
    pub id: String,
    pub name: String,
    pub source: &'static str,
}

/// On-demand model/asset source.
pub trait ModelSource {
    /// Search/browse the library without downloading anything.
    fn search(&self, query: &str) -> Result<Vec<ModelSummary>, String>;

    /// Fetch one model by id (the on-demand download).
    fn fetch(&self, id: &str) -> Result<Value, String>;
}
