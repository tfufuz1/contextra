//! Scope-Modul (SPEC §11.1).
//!
//! Scope-Auflösung und -Persistenz für Löschvorgänge.

pub mod codec;
pub mod set;

pub use codec::{decode_scope, encode_scope};
pub use set::{LayerScopeSet, ResolvedLayerScope};
