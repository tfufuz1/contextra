//! Bounded FD LRU. Returned `Arc<File>` values pin handles while readers use them.

use contextra_storage_ports::traits::{File, FileRole, Fs, IoCtx};
use contextra_storage_ports::Error;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};

const IO: IoCtx = IoCtx { role: FileRole::Run, purpose: "run_table_cache_open" };

pub(crate) fn run_io() -> IoCtx { IoCtx { role: FileRole::Run, purpose: "run_table_read" } }

struct Entry { file: Arc<dyn File>, last_used: u64 }
#[derive(Default)]
struct State {
    files: HashMap<PathBuf, Entry>,
    opening: HashSet<PathBuf>,
    tick: u64,
}

/// Cache capacity is a hard bound, including opens in progress. An entry is
/// evictable only when the cache owns its sole strong reference.
pub struct TableCache {
    capacity: usize,
    state: Mutex<State>,
    changed: Condvar,
}

impl TableCache {
    pub fn new(max_open_files: usize) -> Result<Self, Error> {
        if max_open_files == 0 { return Err(Error::Limit { limit: "max_open_files must be nonzero" }); }
        Ok(Self { capacity: max_open_files, state: Mutex::new(State::default()), changed: Condvar::new() })
    }

    pub fn pin(&self, fs: &dyn Fs, path: &Path) -> Result<Arc<dyn File>, Error> {
        let key = path.to_path_buf();
        loop {
            let mut state = self.lock()?;
            if let Some(entry) = state.files.get(&key) {
                let file = Arc::clone(&entry.file);
                state.tick = state.tick.checked_add(1).ok_or(Error::Limit { limit: "table cache LRU clock" })?;
                let tick = state.tick;
                if let Some(entry) = state.files.get_mut(&key) { entry.last_used = tick; }
                return Ok(file);
            }
            if state.opening.contains(&key) {
                state = self.changed.wait(state).map_err(|_| Error::Poisoned { cause: "table cache lock poisoned" })?;
                drop(state);
                continue;
            }
            while state.files.len().checked_add(state.opening.len()).is_none_or(|n| n >= self.capacity) {
                if !evict_one(&mut state) {
                    return Err(Error::Transient { role: "run_table_cache_pinned" });
                }
            }
            state.opening.insert(key.clone());
            drop(state);

            // Opening performs cold I/O; it must not run under the cache mutex.
            let opened = fs.open(&key, IO).map(Arc::<dyn File>::from);
            let mut state = self.lock()?;
            state.opening.remove(&key);
            match opened {
                Ok(file) => {
                    state.tick = match state.tick.checked_add(1) {
                        Some(tick) => tick,
                        None => {
                            self.changed.notify_all();
                            return Err(Error::Limit { limit: "table cache LRU clock" });
                        }
                    };
                    let tick = state.tick;
                    state.files.insert(key.clone(), Entry { file: Arc::clone(&file), last_used: tick });
                    self.changed.notify_all();
                    return Ok(file);
                }
                Err(error) => {
                    self.changed.notify_all();
                    return Err(error);
                }
            }
        }
    }

    pub fn resident(&self) -> Result<usize, Error> { Ok(self.lock()?.files.len()) }
    pub fn capacity(&self) -> usize { self.capacity }

    /// Drops every unpinned handle. Returns the number of handles closed.
    pub fn evict_unpinned(&self) -> Result<usize, Error> {
        let mut state = self.lock()?;
        let before = state.files.len();
        state.files.retain(|_, entry| Arc::strong_count(&entry.file) > 1);
        let removed = before.saturating_sub(state.files.len());
        self.changed.notify_all();
        Ok(removed)
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, State>, Error> {
        self.state.lock().map_err(|_| Error::Poisoned { cause: "table cache lock poisoned" })
    }
}

fn evict_one(state: &mut State) -> bool {
    let candidate = state.files.iter()
        .filter(|(_, entry)| Arc::strong_count(&entry.file) == 1)
        .min_by_key(|(_, entry)| entry.last_used)
        .map(|(path, _)| path.clone());
    candidate.is_some_and(|path| state.files.remove(&path).is_some())
}
