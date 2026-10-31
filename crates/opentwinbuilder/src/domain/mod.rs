//! Domain layer — pure rules, no framework/IO.
//!
//! Per Explicit Architecture (Herberto Graca): the domain sits in the centre with
//! zero knowledge of Bevy, Iced, MCP, or any adapter. Newtypes and the twin model
//! live here.

pub mod twin;
pub mod versioning;
pub mod featurescript;
pub mod bop;

pub use twin::*;
pub use versioning::*;
pub use featurescript::*;
pub use bop::*;
