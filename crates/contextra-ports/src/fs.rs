//! Filesystem port abstractions (VFS).
//!
//! Provides synchronous, crash-consistent virtual file system contracts
//! per SPEC §8.5 and Contract M4.

pub use crate::vfs::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions};

/// Prelude for filesystem ports.
pub mod prelude {
    pub use super::{VerifiedVia, Vfs, VfsFile, VfsOpenOptions};
}
