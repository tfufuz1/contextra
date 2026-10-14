// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Deterministic simulation filesystem (SimFs) with real-world fault injection.
// INVARIANTEN: Zero panic, fully deterministic, seed-driven crash fidelity.

use crate::dst::rng::DstRng;
use crate::sim_fs::vfs_mirror::{SimOpenOptions, SimVfs, SimVfsFile};
use parking_lot::RwLock;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, ErrorKind};
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Normalizes a path by removing cur-dirs and parent-dir steps safely.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(c) => {
                components.push(c);
            }
            Component::RootDir | Component::Prefix(_) => {}
        }
    }
    let mut buf = PathBuf::new();
    for c in components {
        buf.push(c);
    }
    buf
}

/// Helper to get normalized parent path or "." if root.
fn parent_path(path: &Path) -> PathBuf {
    let norm = normalize_path(path);
    match norm.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// Configuration options for `SimFs` fault injection.
#[derive(Debug, Clone)]
pub struct SimFsConfig {
    /// Sector size in bytes for torn writes (typically 512 or 4096).
    pub sector_size: usize,
    /// Fail writes after N successful write calls.
    pub fail_writes_after: Option<usize>,
    /// Fail syncs after N successful sync calls.
    pub fail_syncs_after: Option<usize>,
    /// Fail reads after N successful read calls.
    pub fail_reads_after: Option<usize>,
    /// Trigger ENOSPC after N write calls.
    pub enospc_after: Option<usize>,
    /// Probability of bit flip on read (0.0 to 1.0).
    pub bit_flip_prob: f64,
}

impl Default for SimFsConfig {
    fn default() -> Self {
        Self {
            sector_size: 4096,
            fail_writes_after: None,
            fail_syncs_after: None,
            fail_reads_after: None,
            enospc_after: None,
            bit_flip_prob: 0.0,
        }
    }
}

/// Internal record of a file's state in `SimFs`.
#[derive(Debug, Clone, Default)]
struct FileRecord {
    durable_data: Vec<u8>,
    cache_data: Vec<u8>,
    dirty: bool,
}

/// Internal state of `SimFs`.
#[derive(Debug, Default)]
struct SimFsState {
    durable_files: BTreeMap<PathBuf, FileRecord>,
    cache_files: BTreeMap<PathBuf, FileRecord>,
    durable_dirs: BTreeSet<PathBuf>,
    cache_dirs: BTreeSet<PathBuf>,
    pending_renames: Vec<(PathBuf, PathBuf)>,
}

/// Deterministic Simulation Filesystem.
#[derive(Debug, Clone)]
pub struct SimFs {
    state: Arc<RwLock<SimFsState>>,
    config: Arc<RwLock<SimFsConfig>>,
    read_count: Arc<AtomicUsize>,
    write_count: Arc<AtomicUsize>,
    sync_count: Arc<AtomicUsize>,
    crashed: Arc<AtomicBool>,
}

impl Default for SimFs {
    fn default() -> Self {
        Self::new()
    }
}

impl SimFs {
    /// Creates a new `SimFs`.
    pub fn new() -> Self {
        let mut dirs = BTreeSet::new();
        dirs.insert(PathBuf::from("."));
        Self {
            state: Arc::new(RwLock::new(SimFsState {
                durable_dirs: dirs.clone(),
                cache_dirs: dirs,
                ..Default::default()
            })),
            config: Arc::new(RwLock::new(SimFsConfig::default())),
            read_count: Arc::new(AtomicUsize::new(0)),
            write_count: Arc::new(AtomicUsize::new(0)),
            sync_count: Arc::new(AtomicUsize::new(0)),
            crashed: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Sets the fault configuration.
    pub fn set_config(&self, config: SimFsConfig) {
        *self.config.write() = config;
    }

    /// Resets crash state and counters.
    pub fn reset_counters(&self) {
        self.read_count.store(0, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.sync_count.store(0, Ordering::SeqCst);
        self.crashed.store(false, Ordering::SeqCst);
    }

    /// Simulates a crash using a deterministic seed.
    ///
    /// Applies deterministic rules (unpersisted write loss, torn writes, reordering,
    /// missing directory sync reversion) to resolve un-synced changes into durable state.
    pub fn crash(&self, seed: u64) {
        self.crashed.store(true, Ordering::SeqCst);
        let mut rng = DstRng::new(seed);
        let mut state = self.state.write();
        let cfg = self.config.read();
        let sector_size = if cfg.sector_size == 0 {
            4096
        } else {
            cfg.sector_size
        };

        let mut durable_updates = Vec::new();

        // 1. Resolve file payload unpersisted writes
        for (path, rec) in state.cache_files.iter_mut() {
            if rec.dirty && rec.cache_data != rec.durable_data {
                let choice = rng.next_range(1, 100);
                if choice <= 40 {
                    // Loss: unpersisted write disappears
                    rec.cache_data = rec.durable_data.clone();
                    rec.dirty = false;
                } else if choice <= 80 {
                    // Torn write: payload survives up to sector boundary
                    let common_len = rec.durable_data.len().min(rec.cache_data.len());
                    let mut diff_idx = common_len;
                    for i in 0..common_len {
                        if rec.durable_data.get(i) != rec.cache_data.get(i) {
                            diff_idx = i;
                            break;
                        }
                    }
                    let truncated_len = ((diff_idx / sector_size) + 1) * sector_size;
                    let cap_len = truncated_len.min(rec.cache_data.len());

                    let mut torn_buf = rec.durable_data.clone();
                    if torn_buf.len() < cap_len {
                        torn_buf.resize(cap_len, 0);
                    }
                    if let Some(src_slice) = rec.cache_data.get(..cap_len) {
                        if let Some(dst_slice) = torn_buf.get_mut(..cap_len) {
                            dst_slice.copy_from_slice(src_slice);
                        }
                    }
                    rec.durable_data = torn_buf.clone();
                    rec.cache_data = torn_buf;
                    rec.dirty = false;
                } else {
                    // Reordered survival: write survives
                    rec.durable_data = rec.cache_data.clone();
                    rec.dirty = false;
                }
            } else {
                rec.cache_data = rec.durable_data.clone();
                rec.dirty = false;
            }

            durable_updates.push((path.clone(), rec.clone()));
        }

        for (path, rec) in durable_updates {
            state.durable_files.insert(path, rec);
        }

        // 2. Resolve directory unpersisted mutations
        let renames = std::mem::take(&mut state.pending_renames);
        for (from, to) in renames {
            let choice = rng.next_range(1, 100);
            if choice <= 33 {
                // Revert rename: restore 'from', remove 'to'
                if let Some(rec) = state.durable_files.get(&from).cloned() {
                    state.cache_files.insert(from.clone(), rec);
                }
                state.cache_files.remove(&to);
            } else if choice <= 66 {
                // Both exist (duplicate entry)
                if let Some(rec) = state.durable_files.get(&from).cloned() {
                    state.cache_files.insert(from, rec.clone());
                    state.cache_files.insert(to, rec);
                }
            } else {
                // Rename succeeded
                state.durable_files.remove(&from);
                if let Some(rec) = state.cache_files.get(&to).cloned() {
                    state.durable_files.insert(to, rec);
                }
            }
        }

        // 3. Align durable and cache maps post-crash
        state.cache_files = state.durable_files.clone();
        state.cache_dirs = state.durable_dirs.clone();
    }

    /// Computes a deterministic hash over all durable and cache file contents for determinism checks.
    pub fn content_hash(&self) -> u64 {
        let state = self.state.read();
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for (path, rec) in &state.cache_files {
            for &b in path.as_os_str().as_encoded_bytes() {
                hash ^= u64::from(b);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            for &b in &rec.cache_data {
                hash ^= u64::from(b);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        hash
    }

    fn check_write_fault(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                ErrorKind::BrokenPipe,
                "SimFs simulated crash",
            ));
        }
        let cfg = self.config.read();
        let count = self.write_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_writes_after {
            if count >= limit {
                return Err(io::Error::new(
                    ErrorKind::WriteZero,
                    "Simulated write failure",
                ));
            }
        }
        if let Some(limit) = cfg.enospc_after {
            if count >= limit {
                return Err(io::Error::from_raw_os_error(28)); // ENOSPC
            }
        }
        Ok(())
    }

    fn check_read_fault(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                ErrorKind::BrokenPipe,
                "SimFs simulated crash",
            ));
        }
        let cfg = self.config.read();
        let count = self.read_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_reads_after {
            if count >= limit {
                return Err(io::Error::other("Simulated read I/O error"));
            }
        }
        Ok(())
    }

    fn check_sync_fault(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(
                ErrorKind::BrokenPipe,
                "SimFs simulated crash",
            ));
        }
        let cfg = self.config.read();
        let count = self.sync_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_syncs_after {
            if count >= limit {
                return Err(io::Error::other("Simulated fsync I/O failure"));
            }
        }
        Ok(())
    }
}

impl SimVfs for SimFs {
    fn open(&self, path: &Path, opts: &SimOpenOptions) -> io::Result<Box<dyn SimVfsFile>> {
        let norm = normalize_path(path);
        let mut state = self.state.write();

        let exists = state.cache_files.contains_key(&norm);

        if opts.create_new && exists {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                "File already exists",
            ));
        }

        if !opts.create && !opts.create_new && !exists {
            return Err(io::Error::new(ErrorKind::NotFound, "File not found"));
        }

        if !exists && (opts.create || opts.create_new) {
            state.cache_files.insert(
                norm.clone(),
                FileRecord {
                    durable_data: Vec::new(),
                    cache_data: Vec::new(),
                    dirty: true,
                },
            );
        }

        if opts.truncate {
            if let Some(rec) = state.cache_files.get_mut(&norm) {
                rec.cache_data.clear();
                rec.dirty = true;
            }
        }

        Ok(Box::new(SimFsFileImpl {
            sim_fs: self.clone(),
            path: norm,
        }))
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        let norm = normalize_path(path);
        let mut state = self.state.write();
        state.cache_dirs.insert(norm);
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let norm = normalize_path(path);
        let mut state = self.state.write();
        if state.cache_files.remove(&norm).is_none() {
            return Err(io::Error::new(ErrorKind::NotFound, "File not found"));
        }
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let norm_from = normalize_path(from);
        let norm_to = normalize_path(to);

        let mut state = self.state.write();
        let rec = state
            .cache_files
            .remove(&norm_from)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Source file not found"))?;

        state.cache_files.insert(norm_to.clone(), rec);
        state.pending_renames.push((norm_from, norm_to));
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> io::Result<Vec<PathBuf>> {
        let norm = normalize_path(path);
        let state = self.state.read();

        let mut results = BTreeSet::new();
        for file_path in state.cache_files.keys() {
            if parent_path(file_path) == norm {
                results.insert(file_path.clone());
            }
        }
        for dir_path in &state.cache_dirs {
            if parent_path(dir_path) == norm && dir_path != &norm {
                results.insert(dir_path.clone());
            }
        }

        Ok(results.into_iter().collect())
    }

    fn exists(&self, path: &Path) -> io::Result<bool> {
        let norm = normalize_path(path);
        let state = self.state.read();
        Ok(state.cache_files.contains_key(&norm) || state.cache_dirs.contains(&norm))
    }

    fn sync_dir(&self, path: &Path) -> io::Result<()> {
        self.check_sync_fault()?;
        let norm = normalize_path(path);
        let mut state = self.state.write();

        // Commit pending renames affecting this directory
        let renames = std::mem::take(&mut state.pending_renames);
        let mut uncommitted = Vec::new();
        for (from, to) in renames {
            if parent_path(&from) == norm || parent_path(&to) == norm {
                state.durable_files.remove(&from);
                if let Some(rec) = state.cache_files.get(&to).cloned() {
                    state.durable_files.insert(to, rec);
                }
            } else {
                uncommitted.push((from, to));
            }
        }
        state.pending_renames = uncommitted;

        // Commit directory existence
        if state.cache_dirs.contains(&norm) {
            state.durable_dirs.insert(norm.clone());
        }

        // Commit children file entries existence
        let mut updates = Vec::new();
        for (fpath, rec) in &state.cache_files {
            if parent_path(fpath) == norm {
                updates.push((fpath.clone(), rec.clone()));
            }
        }

        for (fpath, rec) in updates {
            state.durable_files.insert(fpath, rec);
        }

        Ok(())
    }
}

/// Implementation of `SimVfsFile` for `SimFs`.
struct SimFsFileImpl {
    sim_fs: SimFs,
    path: PathBuf,
}

impl SimVfsFile for SimFsFileImpl {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        self.sim_fs.check_read_fault()?;
        let state = self.sim_fs.state.read();
        let rec = state
            .cache_files
            .get(&self.path)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found"))?;

        let start = match usize::try_from(offset) {
            Ok(s) => s,
            Err(_) => return Ok(0),
        };

        if start >= rec.cache_data.len() {
            return Ok(0);
        }

        let slice = &rec.cache_data[start..];
        let to_copy = buf.len().min(slice.len());
        if let Some(dst) = buf.get_mut(..to_copy) {
            if let Some(src) = slice.get(..to_copy) {
                dst.copy_from_slice(src);
            }
        }

        Ok(to_copy)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        self.sim_fs.check_write_fault()?;
        let mut state = self.sim_fs.state.write();
        let rec = state
            .cache_files
            .get_mut(&self.path)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found"))?;

        let start = usize::try_from(offset).unwrap_or(usize::MAX);
        let end = start.saturating_add(buf.len());

        if rec.cache_data.len() < end {
            rec.cache_data.resize(end, 0);
        }

        if let Some(dst) = rec.cache_data.get_mut(start..end) {
            dst.copy_from_slice(buf);
        }
        rec.dirty = true;

        Ok(buf.len())
    }

    fn set_len(&self, len: u64) -> io::Result<()> {
        self.sim_fs.check_write_fault()?;
        let mut state = self.sim_fs.state.write();
        let rec = state
            .cache_files
            .get_mut(&self.path)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found"))?;

        let new_len = usize::try_from(len).unwrap_or(usize::MAX);
        rec.cache_data.resize(new_len, 0);
        if rec.durable_data.len() > new_len {
            rec.durable_data.resize(new_len, 0);
        }
        rec.dirty = true;
        Ok(())
    }

    fn len(&self) -> io::Result<u64> {
        let state = self.sim_fs.state.read();
        let rec = state
            .cache_files
            .get(&self.path)
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found"))?;
        Ok(rec.cache_data.len() as u64)
    }

    fn is_empty(&self) -> io::Result<bool> {
        Ok(self.len()? == 0)
    }

    fn sync_data(&self) -> io::Result<()> {
        if let Err(e) = self.sim_fs.check_sync_fault() {
            // Kernel page cache clearing on fsync error: un-synced dirty changes discarded
            let mut state = self.sim_fs.state.write();
            if let Some(rec) = state.cache_files.get_mut(&self.path) {
                rec.cache_data = rec.durable_data.clone();
                rec.dirty = false;
            }
            return Err(e);
        }

        let mut state = self.sim_fs.state.write();
        if let Some(rec) = state.cache_files.get_mut(&self.path) {
            rec.durable_data = rec.cache_data.clone();
            rec.dirty = false;
            let cloned_rec = rec.clone();
            state.durable_files.insert(self.path.clone(), cloned_rec);
        }
        Ok(())
    }

    fn sync_all(&self) -> io::Result<()> {
        self.sync_data()?;
        let parent = parent_path(&self.path);
        self.sim_fs.sync_dir(&parent)
    }
}
