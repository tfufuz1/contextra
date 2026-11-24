//! Deterministische Fehlerhülle für I/O-Operationen (SP-ENV, WP-03.2).

use contextra_storage_ports::traits::{File, FileRole, Fs, FsCaps, FsLock, IoCtx};
use contextra_storage_ports::Error;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    Eio,
    Enospc,
    TornWrite { keep: usize },
    LostFsync,
    RenameReorder,
    ShortRead,
    BitFlip { offset: u64 },
}

#[derive(Debug, Clone, Copy)]
pub struct FaultPlan {
    /// Nullbasierter Index über alle Datei- und Dateisystemoperationen.
    pub at_op: u64,
    pub fault: Fault,
}

struct Runtime {
    next_op: AtomicU64,
    plan: Mutex<Option<FaultPlan>>,
    bytes_read: [AtomicU64; 10],
    #[cfg(feature = "testing")]
    ghost: Mutex<Vec<Vec<u8>>>,
}

pub struct FaultFs<F: Fs> {
    inner: F,
    runtime: Arc<Runtime>,
}

impl<F: Fs> FaultFs<F> {
    pub fn new(inner: F, plan: FaultPlan) -> Self {
        Self {
            inner,
            runtime: Arc::new(Runtime {
                next_op: AtomicU64::new(0),
                plan: Mutex::new(Some(plan)),
                bytes_read: [const { AtomicU64::new(0) }; 10],
                #[cfg(feature = "testing")]
                ghost: Mutex::new(Vec::new()),
            }),
        }
    }

    pub fn op_count(&self) -> u64 {
        self.runtime.next_op.load(Ordering::Acquire)
    }

    pub fn bytes_read(&self, role: FileRole) -> u64 {
        self.runtime.bytes_read.get(role_index(role))
            .map(|counter| counter.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub fn inner(&self) -> &F { &self.inner }

    #[cfg(feature = "testing")]
    pub fn ghost_journal(&self) -> Result<Vec<Vec<u8>>, Error> {
        self.runtime.ghost.lock()
            .map(|journal| journal.clone())
            .map_err(|_| Error::Poisoned { cause: "FaultFs journal lock" })
    }
}

impl FaultFs<super::memfs::MemFs> {
    pub fn crash_image(&self) -> Result<super::memfs::MemFs, Error> {
        self.inner.crash_image()
    }
}

struct FaultFile {
    inner: Box<dyn File>,
    runtime: Arc<Runtime>,
    role: FileRole,
}

impl File for FaultFile {
    fn read_at(&self, off: u64, buf: &mut [u8], _ctx: IoCtx) -> Result<usize, Error> {
        let fault = take_fault(&self.runtime)?;
        if matches!(fault, Some(Fault::Eio)) { return Err(io_error(self.role)); }
        let limit = if matches!(fault, Some(Fault::ShortRead)) && !buf.is_empty() { buf.len() - 1 } else { buf.len() };
        let read_buffer = buf.get_mut(..limit).ok_or(Error::Corrupt { place: "FaultFs read range" })?;
        let read = self.inner.read_at(off, read_buffer, _ctx)?;
        if let Some(Fault::BitFlip { offset }) = fault {
            if offset >= off && offset - off < read as u64 {
                let index = (offset - off) as usize;
                if let Some(byte) = buf.get_mut(index) { *byte ^= 1; }
            }
        }
        if let Some(counter) = self.runtime.bytes_read.get(role_index(self.role)) {
            counter.fetch_add(read as u64, Ordering::Relaxed);
        }
        Ok(read)
    }

    fn append_prealloc(&self, data: &[u8], ctx: IoCtx) -> Result<(), Error> {
        match take_fault(&self.runtime)? {
            Some(Fault::Eio | Fault::RenameReorder | Fault::BitFlip { .. } | Fault::ShortRead | Fault::LostFsync | Fault::Enospc) => Err(io_error(self.role)),
            Some(Fault::TornWrite { keep }) => {
                let partial = data.get(..keep.min(data.len())).ok_or(Error::Corrupt { place: "FaultFs torn write range" })?;
                self.inner.append_prealloc(partial, ctx)?;
                #[cfg(feature = "testing")]
                if let Ok(mut ghost) = self.runtime.ghost.lock() { ghost.push(data.to_vec()); }
                Err(io_error(self.role))
            }
            None => {
                self.inner.append_prealloc(data, ctx)?;
                #[cfg(feature = "testing")]
                if let Ok(mut ghost) = self.runtime.ghost.lock() { ghost.push(data.to_vec()); }
                Ok(())
            }
        }
    }

    fn preallocate(&self, len: u64, ctx: IoCtx) -> Result<(), Error> {
        match take_fault(&self.runtime)? {
            Some(Fault::Enospc) => Err(Error::DiskFull),
            Some(_) => Err(io_error(self.role)),
            None => self.inner.preallocate(len, ctx),
        }
    }

    fn sync_data(&self, ctx: IoCtx) -> Result<(), Error> {
        match take_fault(&self.runtime)? {
            Some(Fault::LostFsync) => Ok(()),
            Some(_) => Err(io_error(self.role)),
            None => self.inner.sync_data(ctx),
        }
    }

    fn sync_all(&self, ctx: IoCtx) -> Result<(), Error> {
        match take_fault(&self.runtime)? {
            Some(Fault::LostFsync) => Ok(()),
            Some(_) => Err(io_error(self.role)),
            None => self.inner.sync_all(ctx),
        }
    }

    fn size(&self) -> Result<u64, Error> { self.inner.size() }
}

impl<F: Fs> Fs for FaultFs<F> {
    fn open(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error> {
        match take_fault(&self.runtime)? { Some(_) => Err(io_error(ctx.role)), None => {
            let file = self.inner.open(path, ctx)?;
            Ok(Box::new(FaultFile { inner: file, runtime: Arc::clone(&self.runtime), role: ctx.role }))
        }}
    }

    fn create_new(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(ctx.role)); }
        let file = self.inner.create_new(path, ctx)?;
        Ok(Box::new(FaultFile { inner: file, runtime: Arc::clone(&self.runtime), role: ctx.role }))
    }

    fn rename(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error> {
        match take_fault(&self.runtime)? {
            Some(Fault::RenameReorder) => Err(io_error(ctx.role)),
            Some(_) => Err(io_error(ctx.role)),
            None => self.inner.rename(from, to, ctx),
        }
    }

    fn unlink(&self, path: &Path, ctx: IoCtx) -> Result<(), Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(ctx.role)); }
        self.inner.unlink(path, ctx)
    }

    fn lock(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn FsLock>, Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(ctx.role)); }
        self.inner.lock(path, ctx)
    }

    fn link(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(ctx.role)); }
        self.inner.link(from, to, ctx)
    }

    fn reflink(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(ctx.role)); }
        self.inner.reflink(from, to, ctx)
    }

    fn punch_hole(&self, path: &Path, offset: u64, length: u64, ctx: IoCtx) -> Result<(), Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(ctx.role)); }
        self.inner.punch_hole(path, offset, length, ctx)
    }

    fn sync_dir(&self, dir: &Path, ctx: IoCtx) -> Result<(), Error> {
        if matches!(take_fault(&self.runtime)?, Some(Fault::LostFsync)) { return Ok(()); }
        self.inner.sync_dir(dir, ctx)
    }

    fn list(&self, dir: &Path) -> Result<Vec<PathBuf>, Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(FileRole::Other)); }
        self.inner.list(dir)
    }

    fn free_space(&self, dir: &Path) -> Result<u64, Error> {
        if take_fault(&self.runtime)?.is_some() { return Err(io_error(FileRole::Other)); }
        self.inner.free_space(dir)
    }

    fn caps(&self) -> FsCaps { self.inner.caps() }
}

fn take_fault(runtime: &Runtime) -> Result<Option<Fault>, Error> {
    let index = runtime.next_op.fetch_add(1, Ordering::AcqRel);
    let mut plan = runtime.plan.lock().map_err(|_| Error::Poisoned { cause: "FaultFs plan lock" })?;
    Ok(if plan.is_some_and(|selected| selected.at_op == index) { plan.take().map(|p| p.fault) } else { None })
}

fn io_error(role: FileRole) -> Error {
    let role = match role {
        FileRole::Catalog => "catalog", FileRole::Redo => "redo", FileRole::Run => "run",
        FileRole::Blob => "blob", FileRole::Keys => "keys", FileRole::Anchor => "anchor",
        FileRole::Ledger => "ledger", FileRole::Format => "format", FileRole::PurgeScratch => "purge_scratch",
        FileRole::Other => "other",
    };
    Error::Io { role, kind: ErrorKind::Other }
}

fn role_index(role: FileRole) -> usize {
    match role {
        FileRole::Catalog => 0, FileRole::Redo => 1, FileRole::Run => 2,
        FileRole::Blob => 3, FileRole::Keys => 4, FileRole::Anchor => 5,
        FileRole::Ledger => 6, FileRole::Format => 7, FileRole::PurgeScratch => 8, FileRole::Other => 9,
    }
}
