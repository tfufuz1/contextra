//! Deterministisches, nicht sparsches Dateisystem mit getrenntem durablem Abbild.

use contextra_storage_ports::traits::{File, FileRole, Fs, FsCaps, FsLock, IoCtx};
use contextra_storage_ports::Error;
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone)]
pub struct MemFs {
    state: Arc<Mutex<FsState>>,
}

#[derive(Default)]
struct FsState {
    files: BTreeMap<PathBuf, Arc<Mutex<MemFileState>>>,
    durable_files: BTreeMap<PathBuf, Arc<Mutex<MemFileState>>>,
    free_bytes: u64,
    locks: BTreeMap<PathBuf, Arc<MemLockCell>>,
}

type MemLockCell = (Mutex<bool>, Condvar);

struct MemFsLock {
    cell: Arc<MemLockCell>,
}

impl FsLock for MemFsLock {}

impl Drop for MemFsLock {
    fn drop(&mut self) {
        let (held, released) = &*self.cell;
        match held.lock() {
            Ok(mut held) => *held = false,
            Err(poisoned) => *poisoned.into_inner() = false,
        }
        released.notify_one();
    }
}

#[derive(Default)]
struct MemFileState {
    data: Vec<u8>,
    durable_data: Vec<u8>,
    allocated: u64,
    logical_len: usize,
    durable_len: usize,
}

impl MemFs {
    pub fn new() -> Self {
        Self { state: Arc::new(Mutex::new(FsState { free_bytes: u64::MAX, ..FsState::default() })) }
    }

    /// Erstellt ein neues Abbild wie nach einem Absturz: nicht synchronisierte
    /// Bytes und nicht durch `sync_dir` bestätigte Namensänderungen entfallen.
    pub fn crash_image(&self) -> Result<Self, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        let mut files = BTreeMap::new();
        for (path, file) in &state.durable_files {
            let file_state = file.lock().map_err(|_| Error::Poisoned { cause: "MemFs file lock" })?;
            let data = file_state.durable_data.clone();
            files.insert(path.clone(), Arc::new(Mutex::new(MemFileState {
                allocated: file_state.allocated,
                data: data.clone(),
                durable_data: data,
                logical_len: file_state.durable_len,
                durable_len: file_state.durable_len,
            })));
        }
        Ok(Self {
            state: Arc::new(Mutex::new(FsState {
                durable_files: files.clone(),
                files,
                free_bytes: state.free_bytes,
                locks: BTreeMap::new(),
            })),
        })
    }

    pub fn set_free_space(&self, bytes: u64) {
        if let Ok(mut state) = self.state.lock() {
            state.free_bytes = bytes;
        }
    }
}

impl Default for MemFs {
    fn default() -> Self { Self::new() }
}

struct MemFile {
    state: Arc<Mutex<MemFileState>>,
    owner: Arc<Mutex<FsState>>,
    role: FileRole,
}

impl File for MemFile {
    fn read_at(&self, off: u64, buf: &mut [u8], _ctx: IoCtx) -> Result<usize, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        let start = usize::try_from(off).map_err(|_| Error::Limit { limit: "read offset" })?;
        if start >= state.logical_len {
            return Ok(0);
        }
        let count = buf.len().min(state.logical_len - start);
        let end = start.checked_add(count).ok_or(Error::Limit { limit: "read range" })?;
        let source = state.data.get(start..end).ok_or(Error::Corrupt { place: "MemFs logical length" })?;
        let target = buf.get_mut(..count).ok_or(Error::Corrupt { place: "MemFs read buffer" })?;
        target.copy_from_slice(source);
        Ok(count)
    }

    fn append_prealloc(&self, data: &[u8], _ctx: IoCtx) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        let end = state.logical_len.checked_add(data.len()).ok_or(Error::Limit { limit: "file size" })?;
        if end as u64 > state.allocated {
            return Err(Error::Io { role: role_name(self.role), kind: ErrorKind::InvalidData });
        }
        let start = state.logical_len;
        let target = state.data.get_mut(start..end).ok_or(Error::Corrupt { place: "MemFs allocation length" })?;
        target.copy_from_slice(data);
        state.logical_len = end;
        Ok(())
    }

    fn preallocate(&self, len: u64, _ctx: IoCtx) -> Result<(), Error> {
        let len = usize::try_from(len).map_err(|_| Error::DiskFull)?;
        let mut owner = self.owner.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        let additional = (len as u64).saturating_sub(state.allocated);
        if additional > owner.free_bytes {
            return Err(Error::DiskFull);
        }
        if len > state.data.len() {
            state.data.resize(len, 0);
        }
        if len > state.durable_data.len() { state.durable_data.resize(len, 0); }
        state.allocated = state.allocated.max(len as u64);
        owner.free_bytes -= additional;
        Ok(())
    }

    fn sync_data(&self, _ctx: IoCtx) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        state.durable_data = state.data.clone();
        state.durable_len = state.logical_len;
        Ok(())
    }

    fn sync_all(&self, ctx: IoCtx) -> Result<(), Error> {
        self.sync_data(ctx)
    }

    fn size(&self) -> Result<u64, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        Ok(state.logical_len as u64)
    }
}

impl Fs for MemFs {
    fn open(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        let file = state.files.get(path).cloned().ok_or(Error::Io { role: role_name(ctx.role), kind: ErrorKind::NotFound })?;
        Ok(Box::new(MemFile { state: file, owner: Arc::clone(&self.state), role: ctx.role }))
    }

    fn create_new(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        if state.files.contains_key(path) {
            return Err(Error::Io { role: role_name(ctx.role), kind: ErrorKind::AlreadyExists });
        }
        let file = Arc::new(Mutex::new(MemFileState::default()));
        state.files.insert(path.to_path_buf(), Arc::clone(&file));
        Ok(Box::new(MemFile { state: file, owner: Arc::clone(&self.state), role: ctx.role }))
    }

    fn rename(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        let file = state.files.remove(from).ok_or(Error::Io { role: role_name(ctx.role), kind: ErrorKind::NotFound })?;
        state.files.remove(to);
        state.files.insert(to.to_path_buf(), file);
        Ok(())
    }

    fn unlink(&self, path: &Path, ctx: IoCtx) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        state.files.remove(path).ok_or(Error::Io { role: role_name(ctx.role), kind: ErrorKind::NotFound })?;
        Ok(())
    }

    fn lock(&self, path: &Path, _ctx: IoCtx) -> Result<Box<dyn FsLock>, Error> {
        let cell = {
            let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
            Arc::clone(state.locks.entry(path.to_path_buf()).or_insert_with(|| {
                Arc::new((Mutex::new(false), Condvar::new()))
            }))
        };
        let (held, released) = &*cell;
        let mut is_held = held.lock().map_err(|_| Error::Poisoned { cause: "MemFs instance lock" })?;
        while *is_held {
            is_held = released.wait(is_held).map_err(|_| Error::Poisoned { cause: "MemFs instance lock" })?;
        }
        *is_held = true;
        drop(is_held);
        Ok(Box::new(MemFsLock { cell }))
    }

    fn sync_dir(&self, _dir: &Path, _ctx: IoCtx) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        state.durable_files = state.files.clone();
        Ok(())
    }

    fn list(&self, _dir: &Path) -> Result<Vec<PathBuf>, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        Ok(state.files.keys().cloned().collect())
    }

    fn free_space(&self, _dir: &Path) -> Result<u64, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned { cause: "MemFs lock" })?;
        Ok(state.free_bytes)
    }

    fn caps(&self) -> FsCaps {
        FsCaps { durable_file_sync: true, durable_dir_sync: true, atomic_rename: true,
            preallocation_reserves_space: true, hard_link: false, reflink: false, punch_hole: false }
    }
}

fn role_name(role: FileRole) -> &'static str {
    match role {
        FileRole::Catalog => "catalog",
        FileRole::Redo => "redo",
        FileRole::Run => "run",
        FileRole::Blob => "blob",
        FileRole::Keys => "keys",
        FileRole::Anchor => "anchor",
        FileRole::Ledger => "ledger",
        FileRole::Format => "format",
        FileRole::PurgeScratch => "purge_scratch",
        FileRole::Other => "other",
    }
}
