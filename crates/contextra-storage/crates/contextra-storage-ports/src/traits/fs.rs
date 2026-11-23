//! Fs/File-Port (SP-ENV). Jede Operation trägt IoCtx für FaultFs-Indizierung.
//! Lock-Operation (flock/OFD) folgt in WP-03.3.
use crate::error::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileRole {
    Catalog,
    Redo,
    Run,
    Blob,
    Keys,
    Anchor,
    Ledger,
    Format,
    PurgeScratch,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct IoCtx {
    pub role: FileRole,
    pub purpose: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FsCaps {
    pub durable_file_sync: bool,
    pub durable_dir_sync: bool,
    pub atomic_rename: bool,
    pub preallocation_reserves_space: bool,
    pub reflink: bool,
    pub punch_hole: bool,
}

pub trait File: Send + Sync {
    fn read_at(&self, off: u64, buf: &mut [u8], ctx: IoCtx) -> Result<usize, Error>;
    fn append_prealloc(&self, data: &[u8], ctx: IoCtx) -> Result<(), Error>;
    /// Schreibt Nullen, nie sparse (AN-10). ENOSPC tritt nur hier auf.
    fn preallocate(&self, len: u64, ctx: IoCtx) -> Result<(), Error>;
    fn sync_data(&self, ctx: IoCtx) -> Result<(), Error>;
    fn sync_all(&self, ctx: IoCtx) -> Result<(), Error>;
    fn size(&self) -> Result<u64, Error>;
}

pub trait Fs: Send + Sync {
    fn open(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error>;
    fn create_new(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error>;
    fn rename(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error>;
    fn unlink(&self, path: &Path, ctx: IoCtx) -> Result<(), Error>;
    fn sync_dir(&self, dir: &Path, ctx: IoCtx) -> Result<(), Error>;
    fn list(&self, dir: &Path) -> Result<Vec<PathBuf>, Error>;
    fn free_space(&self, dir: &Path) -> Result<u64, Error>;
    fn caps(&self) -> FsCaps;
}
