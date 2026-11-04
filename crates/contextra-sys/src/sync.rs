//! Modul `sync`: Synchronisation auf physische Datenträger (SPEC 3.1, 8.3).
pub use crate::durability::{flush_dir, flush_dir_sync, flush_file, flush_file_sync, FlushKind};
