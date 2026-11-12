//! Kapselt Datei- und Verzeichnisoperationen für `contextra-store`.
//!
//! Invarianten:
//! - INV-FSYNC-1: Dauerhafte Persistierung durch fdatasync/fsync vor Bestätigung.
//! - Atomares Schreiben: tmp-Datei -> sync_all -> rename -> parent sync.

pub mod types;
pub mod real_fs;
pub mod store_io;

pub use types::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions};
pub use real_fs::{RealFs, RealFsFile};
pub use store_io::StoreIo;
