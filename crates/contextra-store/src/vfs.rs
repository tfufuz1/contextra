//! VFS Abstraction Layer for `contextra-store` (SPEC §5.8, D-04, T-LINT-2).
//!
//! Provides filesystem access exclusively through `contextra_ports::vfs` abstractions.
//! Direct usage of `std::fs` or `tokio::fs` is forbidden in production code.

use contextra_ports::vfs::{Vfs, VfsFile, VfsOpenOptions};
use contextra_types::Result;
use std::path::{Path, PathBuf};

/// Default filesystem wrapper implementing POSIX durable operations through ports.
pub struct StorageVfs;

impl StorageVfs {
    #[inline]
    pub fn read_dir(vfs: &dyn Vfs, path: &Path) -> Result<Vec<PathBuf>> {
        vfs.read_dir(path)
    }

    #[inline]
    pub fn read(vfs: &dyn Vfs, path: &Path) -> Result<Vec<u8>> {
        vfs.read(path)
    }

    #[inline]
    pub fn write_atomic(vfs: &dyn Vfs, path: &Path, data: &[u8]) -> Result<()> {
        vfs.write_atomic(path, data)
    }

    #[inline]
    pub fn remove_file(vfs: &dyn Vfs, path: &Path) -> Result<()> {
        vfs.remove_file(path)
    }

    #[inline]
    pub fn rename(vfs: &dyn Vfs, from: &Path, to: &Path) -> Result<()> {
        vfs.rename(from, to)
    }

    #[inline]
    pub fn create_dir_all(vfs: &dyn Vfs, path: &Path) -> Result<()> {
        vfs.create_dir_all(path)
    }

    #[inline]
    pub fn exists(vfs: &dyn Vfs, path: &Path) -> Result<bool> {
        vfs.exists(path)
    }

    #[inline]
    pub fn len(vfs: &dyn Vfs, path: &Path) -> Result<u64> {
        vfs.len(path)
    }

    #[inline]
    pub fn sync_dir(vfs: &dyn Vfs, path: &Path) -> Result<()> {
        vfs.sync_dir(path)
    }

    #[inline]
    pub fn open(vfs: &dyn Vfs, path: &Path, options: &VfsOpenOptions) -> Result<Box<dyn VfsFile>> {
        vfs.open(path, options)
    }
}
