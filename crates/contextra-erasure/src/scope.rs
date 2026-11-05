//! Scope-Modul (SPEC §11.1).
//!
//! Scope-Auflösung und -Persistenz für Löschvorgänge.

pub mod codec;
pub mod file;
pub mod set;

pub use codec::{decode_scope, encode_scope};
pub use file::{load_scope_file, scope_file_path, store_scope_file};
pub use set::{LayerScopeSet, ResolvedLayerScope};
