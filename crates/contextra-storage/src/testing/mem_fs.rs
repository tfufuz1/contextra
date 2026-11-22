//! # In-Memory Dateisystem (MemFs)
//!
//! Deterministisches Dateisystem für Tests mit Trennung von Page Cache und dauerhaftem Zustand
//! nach SPEC §5.1, §19.2 und IMPL §6 WP-1.3.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_storage_ports::{DirEntry, File, Fs, FsCaps, LockGuard, LockMode, OpenMode, Perm, ReadReq};
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
struct FileContent {
    data: Vec<u8>,
    durable: Vec<u8>,
    perm: Perm,
}

struct MemFileHandle {
    content: Arc<RwLock<FileContent>>,
}

impl File for MemFileHandle {
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize> {
        let inner = self.content.read().unwrap();
        let off = usize::try_from(off).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        if off >= inner.data.len() {
            return Ok(0);
        }
        let available = &inner.data[off..];
        let n = std::cmp::min(buf.len(), available.len());
        buf[..n].copy_from_slice(&available[..n]);
        Ok(n)
    }

    fn read_batch(&self, reqs: &mut [ReadReq]) -> io::Result<()> {
        for req in reqs.iter_mut() {
            let n = self.read_at(req.buf, req.offset)?;
            if n < req.buf.len() { req.buf[n..].fill(0); }
        }
        Ok(())
    }

    fn write_all_at(&self, buf: &[u8], off: u64) -> io::Result<()> {
        let mut inner = self.content.write().unwrap();
        let off = usize::try_from(off).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let end = off.checked_add(buf.len()).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Overflow"))?;
        if inner.data.len() < end { inner.data.resize(end, 0); }
        inner.data[off..end].copy_from_slice(buf);
        Ok(())
    }

    fn len(&self) -> io::Result<u64> {
        Ok(self.content.read().unwrap().data.len() as u64)
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        let mut inner = self.content.write().unwrap();
        let len = usize::try_from(len).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        inner.data.resize(len, 0);
        Ok(())
    }

    fn sync_data(&self) -> io::Result<()> {
        let mut inner = self.content.write().unwrap();
        inner.durable = inner.data.clone();
        Ok(())
    }

    fn sync_all(&self) -> io::Result<()> { self.sync_data() }

    fn preallocate(&self, len: u64) -> io::Result<()> {
        let mut inner = self.content.write().unwrap();
        let len = usize::try_from(len).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        if inner.data.len() < len { inner.data.resize(len, 0); }
        Ok(())
    }
}

fn is_direct_child(child: &Path, parent: &Path) -> bool {
    let cp = match child.parent() { Some(p) => p, None => return false };
    if cp == parent { return true; }
    (parent == Path::new(".") || parent == Path::new("")) && (cp == Path::new(".") || cp == Path::new(""))
}

#[derive(Clone)]
struct Snapshot {
    files: BTreeMap<PathBuf, (Vec<u8>, Perm)>,
    dirs: BTreeSet<PathBuf>,
}

struct Inner {
    files: BTreeMap<PathBuf, Arc<RwLock<FileContent>>>,
    durable_files: BTreeMap<PathBuf, Arc<RwLock<FileContent>>>,
    dirs: BTreeSet<PathBuf>,
    durable_dirs: BTreeSet<PathBuf>,
    locks: BTreeSet<PathBuf>,
    snapshots: BTreeMap<u64, Snapshot>,
    next_snap_id: u64,
}

/// In-Memory Dateisystem für Tests nach SPEC §19.2.
#[derive(Clone)]
pub struct MemFs {
    inner: Arc<RwLock<Inner>>,
}

impl Default for MemFs {
    fn default() -> Self { Self::new() }
}

impl MemFs {
    /// Erstellt ein neues, leeres `MemFs`.
    pub fn new() -> Self {
        let mut dirs = BTreeSet::new();
        dirs.insert(PathBuf::from("/"));
        dirs.insert(PathBuf::from("."));
        Self {
            inner: Arc::new(RwLock::new(Inner {
                files: BTreeMap::new(),
                durable_files: BTreeMap::new(),
                dirs: dirs.clone(),
                durable_dirs: dirs,
                locks: BTreeSet::new(),
                snapshots: BTreeMap::new(),
                next_snap_id: 1,
            })),
        }
    }

    /// Simuliert einen Absturz: Verwirft alle nicht synchronisierten Datei- und Verzeichnisänderungen.
    pub fn crash(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.locks.clear();
        for file in inner.durable_files.values() {
            let mut f = file.write().unwrap();
            f.data = f.durable.clone();
        }
        inner.files = inner.durable_files.clone();
        inner.dirs = inner.durable_dirs.clone();
    }

    /// Simuliert einen Absturz mit zerrissenen Schreibvorgängen (Sektorgröße 512 B oder 4 KiB).
    pub fn crash_torn(&self, sector_size: usize) {
        let mut inner = self.inner.write().unwrap();
        inner.locks.clear();
        let sec = if sector_size == 0 { 512 } else { sector_size };
        for file in inner.durable_files.values() {
            let mut f = file.write().unwrap();
            if f.data.len() > f.durable.len() {
                let uncommitted = f.data.len() - f.durable.len();
                let full = uncommitted / sec;
                if full > 0 {
                    f.durable = f.data[..f.durable.len() + (full * sec)].to_vec();
                }
            }
            f.data = f.durable.clone();
        }
        inner.files = inner.durable_files.clone();
        inner.dirs = inner.durable_dirs.clone();
    }

    /// Simuliert einen Absturz mit umgeordneten unvollständigen Schreibvorgängen.
    pub fn crash_reordered(&self, seed: u64) {
        let mut inner = self.inner.write().unwrap();
        inner.locks.clear();
        let mut prng = if seed == 0 { 0x12345678_deadbeef } else { seed };
        for file in inner.durable_files.values() {
            let mut f = file.write().unwrap();
            if f.data.len() > f.durable.len() {
                prng ^= prng << 13; prng ^= prng >> 7; prng ^= prng << 17;
                let keep = (prng as usize) % (f.data.len() - f.durable.len() + 1);
                f.durable = f.data[..f.durable.len() + keep].to_vec();
            }
            f.data = f.durable.clone();
        }
        inner.files = inner.durable_files.clone();
        inner.dirs = inner.durable_dirs.clone();
    }

    /// Erstellt einen Schnappschuss des gesamten dauerhaften Zustands (F-10).
    pub fn take_snapshot(&self) -> u64 {
        let mut inner = self.inner.write().unwrap();
        let id = inner.next_snap_id;
        inner.next_snap_id = inner.next_snap_id.saturating_add(1);
        let mut files = BTreeMap::new();
        for (p, h) in &inner.durable_files {
            let f = h.read().unwrap();
            files.insert(p.clone(), (f.durable.clone(), f.perm));
        }
        let dirs = inner.durable_dirs.clone();
        inner.snapshots.insert(id, Snapshot { files, dirs });
        id
    }

    /// Setzt den gesamten Zustand auf einen früheren Schnappschuss zurück (F-10).
    pub fn restore_snapshot(&self, snap_id: u64) -> io::Result<()> {
        let mut inner = self.inner.write().unwrap();
        let snap = inner.snapshots.get(&snap_id).cloned().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Schnappschuss nicht gefunden")
        })?;
        inner.locks.clear();
        inner.durable_dirs = snap.dirs.clone();
        inner.dirs = snap.dirs;
        let mut new_durable = BTreeMap::new();
        for (path, (durable_data, perm)) in snap.files {
            let h = Arc::new(RwLock::new(FileContent {
                data: durable_data.clone(), durable: durable_data, perm,
            }));
            new_durable.insert(path, h);
        }
        inner.durable_files = new_durable.clone();
        inner.files = new_durable;
        Ok(())
    }
}

struct MemLockGuard { path: PathBuf, inner: Arc<RwLock<Inner>> }
impl LockGuard for MemLockGuard {}
impl Drop for MemLockGuard {
    fn drop(&mut self) { self.inner.write().unwrap().locks.remove(&self.path); }
}

impl Fs for MemFs {
    fn create_dir_all(&self, p: &Path, _perm: Perm) -> io::Result<()> {
        let mut inner = self.inner.write().unwrap();
        let mut curr = PathBuf::new();
        for comp in p.components() {
            curr.push(comp);
            inner.dirs.insert(curr.clone());
        }
        Ok(())
    }

    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>> {
        let mut inner = self.inner.write().unwrap();
        match mode {
            OpenMode::Read | OpenMode::ReadWrite => {
                let file = inner.files.get(p).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "Datei nicht gefunden")
                })?;
                Ok(Box::new(MemFileHandle { content: Arc::clone(file) }))
            }
            OpenMode::CreateNew(perm) => {
                if inner.files.contains_key(p) {
                    return Err(io::Error::new(io::ErrorKind::AlreadyExists, "Datei existiert bereits"));
                }
                let handle = Arc::new(RwLock::new(FileContent {
                    data: Vec::new(), durable: Vec::new(), perm,
                }));
                inner.files.insert(p.to_path_buf(), Arc::clone(&handle));
                Ok(Box::new(MemFileHandle { content: handle }))
            }
        }
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut inner = self.inner.write().unwrap();
        let handle = inner.files.remove(from).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Quelldatei nicht gefunden")
        })?;
        inner.files.insert(to.to_path_buf(), handle);
        Ok(())
    }

    fn remove_file(&self, p: &Path) -> io::Result<()> {
        let mut inner = self.inner.write().unwrap();
        inner.files.remove(p).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Datei nicht gefunden"))?;
        Ok(())
    }

    fn remove_dir(&self, p: &Path) -> io::Result<()> {
        let mut inner = self.inner.write().unwrap();
        if !inner.dirs.remove(p) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Verzeichnis nicht gefunden"));
        }
        Ok(())
    }

    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>> {
        let inner = self.inner.read().unwrap();
        let mut entries = Vec::new();
        for (f_path, handle) in &inner.files {
            if is_direct_child(f_path, p) {
                if let Some(name) = f_path.file_name().and_then(|n| n.to_str()) {
                    let len = handle.read().unwrap().data.len() as u64;
                    entries.push(DirEntry { name: name.to_string(), len });
                }
            }
        }
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(entries)
    }

    fn sync_dir(&self, p: &Path) -> io::Result<()> {
        let mut inner = self.inner.write().unwrap();
        if inner.dirs.contains(p) { inner.durable_dirs.insert(p.to_path_buf()); }

        let working_matching: Vec<(PathBuf, Arc<RwLock<FileContent>>)> = inner
            .files.iter().filter(|(path, _)| is_direct_child(path, p))
            .map(|(path, h)| (path.clone(), Arc::clone(h))).collect();
        for (path, h) in working_matching {
            inner.durable_files.insert(path, h);
        }

        let current_keys: BTreeSet<PathBuf> = inner.files.keys().cloned().collect();
        inner.durable_files.retain(|path, _| {
            if is_direct_child(path, p) { current_keys.contains(path) } else { true }
        });

        let working_subdirs: Vec<PathBuf> = inner.dirs.iter()
            .filter(|d| is_direct_child(d, p)).cloned().collect();
        for d in working_subdirs { inner.durable_dirs.insert(d); }

        let current_dirs = inner.dirs.clone();
        inner.durable_dirs.retain(|d| {
            if is_direct_child(d, p) { current_dirs.contains(d) } else { true }
        });
        Ok(())
    }

    fn lock(&self, p: &Path, _mode: LockMode) -> io::Result<Box<dyn LockGuard>> {
        let mut inner = self.inner.write().unwrap();
        if inner.locks.contains(p) {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "Dateisperre bereits gehalten"));
        }
        inner.locks.insert(p.to_path_buf());
        Ok(Box::new(MemLockGuard { path: p.to_path_buf(), inner: Arc::clone(&self.inner) }))
    }

    fn caps(&self) -> FsCaps {
        FsCaps {
            durable_file_sync: true, durable_dir_sync: true, atomic_rename: true,
            private_perm: true, no_follow_root: true,
        }
    }

    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()> {
        let mut inner = self.inner.write().unwrap();
        let handle = inner.files.get(from).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Quelldatei nicht gefunden")
        })?;
        let clone_handle = Arc::clone(handle);
        inner.files.insert(to.to_path_buf(), clone_handle);
        Ok(())
    }
}
