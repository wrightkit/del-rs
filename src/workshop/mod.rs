//! DEL-owned Workshop lowering boundary.
//!
//! Binding preparation and canonical WIR lowering intentionally live in
//! separate internal modules. The public entry points remain stable here.

mod context;
mod lower;

pub use lower::{lower_project, lower_project_to_wir, lower_to_wir};
