//! Fs/File-Port (SPEC §3.2). Jede Operation trägt IoCtx für FaultFs-Indizierung.
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
    pub hard_link: bool,
    pub reflink: bool,
    pub punch_hole: bool,
}

/// One caller-owned output buffer in a batched ordered read request.
pub struct ReadRange<'a> {
    pub offset: u64,
    pub buffer: &'a mut [u8],
    pub bytes_read: usize,
}

pub trait File: Send + Sync {
    fn read_at(&self, off: u64, buf: &mut [u8], ctx: IoCtx) -> Result<usize, Error>;
    /// Reads caller-owned buffers in the supplied order. Short reads are
    /// preserved in each `bytes_read`; implementations must not reorder.
    fn read_ranges(&self, ranges: &mut [ReadRange<'_>], ctx: IoCtx) -> Result<(), Error> {
        for range in ranges {
            let count = self.read_at(range.offset, &mut *range.buffer, ctx)?;
            if count > range.buffer.len() {
                return Err(Error::Corrupt { place: "filesystem read length" });
            }
            range.bytes_read = count;
        }
        Ok(())
    }
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
    /// Acquires a process/host lock for `path`. Implementations without a
    /// locking primitive must fail closed instead of returning a dummy lock.
    fn lock(&self, _path: &Path, _ctx: IoCtx) -> Result<Box<dyn FsLock>, Error> {
        Err(Error::Unsupported { what: "filesystem lock" })
    }
    /// Optional filesystem capabilities. Unsupported implementations return
    /// a typed error and callers must preserve correctness without them.
    fn link(&self, _from: &Path, _to: &Path, _ctx: IoCtx) -> Result<(), Error> {
        Err(Error::Unsupported { what: "hard link" })
    }
    fn reflink(&self, _from: &Path, _to: &Path, _ctx: IoCtx) -> Result<(), Error> {
        Err(Error::Unsupported { what: "reflink" })
    }
    fn punch_hole(&self, _path: &Path, _offset: u64, _length: u64, _ctx: IoCtx) -> Result<(), Error> {
        Err(Error::Unsupported { what: "hole punching" })
    }
    fn sync_dir(&self, dir: &Path, ctx: IoCtx) -> Result<(), Error>;
    fn list(&self, dir: &Path) -> Result<Vec<PathBuf>, Error>;
    fn free_space(&self, dir: &Path) -> Result<u64, Error>;
    fn caps(&self) -> FsCaps;
}

/// RAII handle for a filesystem lock. Dropping releases the lock.
pub trait FsLock: Send + Sync {}
