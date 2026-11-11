//! Scope-Modul (SPEC §11.1).
//!
//! Scope-Auflösung, -Persistenz und Single-Flight-Koordination (INV-ERA-3) für Löschvorgänge.

pub mod codec;
pub mod file;
pub mod flight;
pub mod set;
pub mod single_flight;

pub use codec::{decode_scope, encode_scope};
pub use file::{load_scope_file, scope_file_path, store_scope_file};
pub use flight::{ActiveScopeFlight, SingleFlightCoordinator};
pub use set::{LayerScopeSet, ResolvedLayerScope};
pub use single_flight::{
    scope_lock_path, ScopeLockError, ScopeLockGuard, ScopeLockManager, ScopeLockRecord,
};
