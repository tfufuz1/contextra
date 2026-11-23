//! SP-CATALOG: einzige Wahrheit ueber Dateien, Gruppen-Commit, Reservierungen.
pub mod edit;
pub mod file;
pub mod ops;
pub mod record;
pub mod reserve;
pub mod rollover;
pub mod state;

pub use ops::{CatalogOp, FileKind};
pub use record::{CatalogIntegrity, CatalogRecord};
pub use state::{CatalogState, FileMeta};
