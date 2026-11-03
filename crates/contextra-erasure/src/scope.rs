//! Scope-Modul (SPEC §11.1).
//!
//! Scope-Auflösung und -Persistenz für Löschvorgänge.
pub mod set;
pub mod codec;

pub use set::{LayerScopeSet, ResolvedLayerScope};
pub use codec::{decode_scope, encode_scope};
