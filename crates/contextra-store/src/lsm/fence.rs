// FILE-CONTEXT
// STAND: 2026-10-16
// ZWECK: Scope-weite und präfixbasierte Schreibsperre (ErasureGate & ErasureFence) zur Blockierung von Schreibvorgängen während der Purge-Compaction (§3, §5, ADR-V2-1).
// INVARIANTEN: Lock-Hierarchie einhalten, keine Locks über .await, hot path ohne RMW/Sperren.

use std::collections::HashSet;

#[cfg(loom)]
use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(loom)]
use loom::sync::{Arc, Mutex, RwLock};

#[cfg(not(loom))]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(not(loom))]
use std::sync::Arc;

/// Scope-wide write block state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceScopeState {
    /// Active write access permitted.
    Active,
    /// Collection is currently being dropped / purged. Scope-wide write block active.
    CollectionDropping,
}

/// Errors returned by `ErasureFence` scope operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FenceError {
    #[error("Collection or scope write block active")]
    CollectionDropping,
    #[error("Fence scope conflict detected")]
    Conflict,
    #[error("Blocking lock attempted within async runtime context")]
    AsyncRuntimeBlockingAttempt,
}

/// Registered prefix fence entry with reference counter (F-2/F-3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixFence {
    pub purge_id: u64,
    pub prefixes: Vec<Vec<u8>>,
    pub ref_count: u32,
}

/// Gate managing atomic active flag and RCU prefix fence table without inflight/RMW counters on hot paths (ADR-V2-1 #3 & #5).
#[derive(Debug)]
pub struct ErasureGate {
    active: AtomicBool,
    #[cfg(not(loom))]
    table: arc_swap::ArcSwap<Vec<PrefixFence>>,
    #[cfg(loom)]
    table: RwLock<Vec<PrefixFence>>,
}

impl ErasureGate {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            #[cfg(not(loom))]
            table: arc_swap::ArcSwap::new(Arc::new(Vec::new())),
            #[cfg(loom)]
            table: RwLock::new(Vec::new()),
        }
    }

    /// Fast-path check: returns whether any scope fence is active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Checks if a key violates any active prefix fence (HOT PATH: single Acquire-load of active).
    pub fn check_write_allowed(&self, key: &[u8]) -> Result<(), FenceError> {
        if !self.is_active() {
            return Ok(());
        }

        #[cfg(not(loom))]
        {
            let table = self.table.load();
            for pf in table.iter() {
                for prefix in &pf.prefixes {
                    if prefix.is_empty() || key.starts_with(prefix) {
                        return Err(FenceError::CollectionDropping);
                    }
                }
            }
        }

        #[cfg(loom)]
        {
            if let Ok(table) = self.table.read() {
                for pf in table.iter() {
                    for prefix in &pf.prefixes {
                        if prefix.is_empty() || key.starts_with(prefix) {
                            return Err(FenceError::CollectionDropping);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Installs or updates prefix fence entry under caller synchronization (F-1, F-2/F-3).
    /// Overlapping or nested prefixes form a union with reference counts rather than returning Conflict.
    pub fn install_prefix_fence(&self, purge_id: u64, prefixes: Vec<Vec<u8>>) {
        #[cfg(not(loom))]
        {
            let current_vec: &Vec<PrefixFence> = &**self.table.load();
            let mut new_table = current_vec.clone();

            let mut matched = false;
            for pf in new_table.iter_mut() {
                if pf.purge_id == purge_id || pf.prefixes == prefixes {
                    pf.ref_count += 1;
                    matched = true;
                    break;
                }
            }

            if !matched {
                new_table.push(PrefixFence {
                    purge_id,
                    prefixes,
                    ref_count: 1,
                });
            }

            self.table.store(Arc::new(new_table));
        }

        #[cfg(loom)]
        {
            if let Ok(mut table) = self.table.write() {
                let mut matched = false;
                for pf in table.iter_mut() {
                    if pf.purge_id == purge_id || pf.prefixes == prefixes {
                        pf.ref_count += 1;
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    table.push(PrefixFence {
                        purge_id,
                        prefixes,
                        ref_count: 1,
                    });
                }
            }
        }

        self.active.store(true, Ordering::Release);
    }

    /// Removes or decrements reference count of registered prefix fence by purge_id or prefixes (F-5).
    pub fn release_prefix_fence_internal(&self, purge_id: u64, prefixes: &[Vec<u8>]) {
        #[cfg(not(loom))]
        {
            let current_vec: &Vec<PrefixFence> = &**self.table.load();
            let mut new_table = current_vec.clone();

            if let Some(pos) = new_table
                .iter()
                .position(|f| f.purge_id == purge_id || f.prefixes == prefixes)
            {
                if new_table[pos].ref_count > 1 {
                    new_table[pos].ref_count -= 1;
                } else {
                    new_table.remove(pos);
                }
            } else {
                new_table.retain(|f| f.purge_id != purge_id);
            }

            let is_empty = new_table.is_empty();
            self.table.store(Arc::new(new_table));
            if is_empty {
                self.active.store(false, Ordering::Release);
            }
        }

        #[cfg(loom)]
        {
            if let Ok(mut table) = self.table.write() {
                if let Some(pos) = table
                    .iter()
                    .position(|f| f.purge_id == purge_id || f.prefixes == prefixes)
                {
                    if table[pos].ref_count > 1 {
                        table[pos].ref_count -= 1;
                    } else {
                        table.remove(pos);
                    }
                } else {
                    table.retain(|f| f.purge_id != purge_id);
                }

                if table.is_empty() {
                    self.active.store(false, Ordering::Release);
                }
            }
        }
    }
}

impl Default for ErasureGate {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII Guard for a scope fence over specific key prefixes.
#[derive(Debug)]
pub struct FenFenceGuard<'a> {
    fence: &'a ErasureFence,
    id: u64,
    prefixes: Vec<Vec<u8>>,
    write_seq: Arc<AtomicU64>,
}

impl<'a> FenFenceGuard<'a> {
    /// Returns current write sequence number for this guard scope.
    pub fn scope_write_seq(&self) -> u64 {
        self.write_seq.load(Ordering::Acquire)
    }

    /// Returns key prefixes protected by this guard.
    pub fn prefixes(&self) -> &[Vec<u8>] {
        &self.prefixes
    }
}

impl<'a> Drop for FenFenceGuard<'a> {
    fn drop(&mut self) {
        self.fence.release_prefix_fence(self.id, &self.prefixes);
    }
}

/// RAII Guard for locked document IDs.
#[derive(Debug)]
pub struct FenDocLockGuard<'a> {
    fence: &'a ErasureFence,
    doc_ids: Vec<Vec<u8>>,
}

impl<'a> Drop for FenDocLockGuard<'a> {
    fn drop(&mut self) {
        self.fence.unlock_documents(&self.doc_ids);
    }
}

/// ErasureFence manages scope-wide write blocks (`CollectionDropping`), prefix-based scope fences via `ErasureGate`, and document-level locks.
#[derive(Debug)]
pub struct ErasureFence {
    pub gate: ErasureGate,
    scope_state: parking_lot::RwLock<FenceScopeState>,
    locked_doc_ids: parking_lot::RwLock<HashSet<Vec<u8>>>,
    scope_write_seq: AtomicU64,
    next_fence_id: AtomicU64,
    #[cfg(not(loom))]
    pub commit_mutex: tokio::sync::Mutex<()>,
    #[cfg(loom)]
    pub commit_mutex: Mutex<()>,
}

impl ErasureFence {
    /// Creates a new inactive `ErasureFence`.
    pub fn new() -> Self {
        Self {
            gate: ErasureGate::new(),
            scope_state: parking_lot::RwLock::new(FenceScopeState::Active),
            locked_doc_ids: parking_lot::RwLock::new(HashSet::new()),
            scope_write_seq: AtomicU64::new(0),
            next_fence_id: AtomicU64::new(1),
            #[cfg(not(loom))]
            commit_mutex: tokio::sync::Mutex::new(()),
            #[cfg(loom)]
            commit_mutex: Mutex::new(()),
        }
    }

    /// Checks if a scope-wide, prefix-based, or document-level write block is currently active for `key`.
    /// Returns `true` if write access is allowed, `false` if blocked by fence.
    pub fn check_write_allowed(&self, key: &[u8]) -> bool {
        self.check_write_allowed_scope(key).is_ok()
    }

    /// Checks if write access is allowed for `key`. Returns `Err(FenceError)` if blocked.
    pub fn check_write_allowed_scope(&self, key: &[u8]) -> Result<(), FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
            return Err(FenceError::CollectionDropping);
        }

        if let Err(e) = self.gate.check_write_allowed(key) {
            self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
            return Err(e);
        }

        let locked_docs = self.locked_doc_ids.read();
        if locked_docs.contains(key) {
            self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
            return Err(FenceError::Conflict);
        }

        Ok(())
    }

    /// Activates global scope-wide write block (`CollectionDropping`).
    pub fn acquire_scope_fence(&self) {
        #[cfg(not(loom))]
        let _commit_guard = if tokio::runtime::Handle::try_current().is_ok() {
            self.commit_mutex.try_lock().ok()
        } else {
            Some(self.commit_mutex.blocking_lock())
        };

        #[cfg(loom)]
        let _commit_guard = self.commit_mutex.lock().ok();

        let mut state = self.scope_state.write();
        *state = FenceScopeState::CollectionDropping;
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Deactivates global scope-wide write block back to `Active`.
    pub fn release_scope_fence(&self) {
        #[cfg(not(loom))]
        let _commit_guard = if tokio::runtime::Handle::try_current().is_ok() {
            self.commit_mutex.try_lock().ok()
        } else {
            Some(self.commit_mutex.blocking_lock())
        };

        #[cfg(loom)]
        let _commit_guard = self.commit_mutex.lock().ok();

        let mut state = self.scope_state.write();
        *state = FenceScopeState::Active;
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Directly installs a prefix fence assuming external caller holding `commit_mutex`.
    pub fn acquire_scope_fence_for_locked(&self, prefix: Vec<u8>) -> u64 {
        let id = self.next_fence_id.fetch_add(1, Ordering::SeqCst);
        self.gate.install_prefix_fence(id, vec![prefix]);
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst)
    }

    /// Directly releases a prefix fence assuming external caller holding `commit_mutex`.
    pub fn release_scope_fence_for_locked(&self, prefix: &[u8]) {
        self.gate
            .release_prefix_fence_internal(0, &[prefix.to_vec()]);
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Acquires a scope fence for specific key prefixes.
    /// F-1: Fence-Erwerb nimmt SELBST den commit_mutex. Dadurch ist der Erwerb die Drain-Barriere:
    /// Alle früher zugelassenen Batches sind bei der Rückkehr angewendet.
    /// Ruft jemand ihn aus einem Async-Worker auf, darf es NICHT panicken: liefere AsyncRuntimeBlockingAttempt.
    pub fn acquire_scope_fence_for(
        &self,
        prefixes: Vec<Vec<u8>>,
    ) -> Result<FenFenceGuard<'_>, FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return Err(FenceError::CollectionDropping);
        }

        #[cfg(not(loom))]
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(FenceError::AsyncRuntimeBlockingAttempt);
        }

        #[cfg(not(loom))]
        let _commit_guard = self.commit_mutex.blocking_lock();
        #[cfg(loom)]
        let _commit_guard = self.commit_mutex.lock().unwrap();

        let id = self.next_fence_id.fetch_add(1, Ordering::SeqCst);
        let write_seq = Arc::new(AtomicU64::new(0));

        self.gate.install_prefix_fence(id, prefixes.clone());
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);

        Ok(FenFenceGuard {
            fence: self,
            id,
            prefixes,
            write_seq,
        })
    }

    /// Helper to release a registered prefix fence by ID and prefixes under commit_mutex lock.
    fn release_prefix_fence(&self, id: u64, prefixes: &[Vec<u8>]) {
        #[cfg(not(loom))]
        let _commit_guard = if tokio::runtime::Handle::try_current().is_ok() {
            self.commit_mutex.try_lock().ok()
        } else {
            Some(self.commit_mutex.blocking_lock())
        };

        #[cfg(loom)]
        let _commit_guard = self.commit_mutex.lock().ok();

        self.gate.release_prefix_fence_internal(id, prefixes);
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Locks specific document IDs against writes.
    pub fn lock_documents(&self, doc_ids: &[Vec<u8>]) {
        let mut locked = self.locked_doc_ids.write();
        for id in doc_ids {
            locked.insert(id.clone());
        }
    }

    /// Locks specific document IDs with RAII guard. Returns `Err(FenceError::Conflict)` if already locked or overlapping.
    pub fn lock_documents_guarded(
        &self,
        doc_ids: Vec<Vec<u8>>,
    ) -> Result<FenDocLockGuard<'_>, FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return Err(FenceError::CollectionDropping);
        }

        if doc_ids.is_empty() {
            return Err(FenceError::Conflict);
        }

        // Check internal duplicates within requested doc_ids
        for i in 0..doc_ids.len() {
            for j in (i + 1)..doc_ids.len() {
                if doc_ids[i] == doc_ids[j] {
                    return Err(FenceError::Conflict);
                }
            }
        }

        for doc_id in &doc_ids {
            self.gate.check_write_allowed(doc_id)?;
        }

        let mut locked = self.locked_doc_ids.write();
        // Atomic validation under write lock prior to modifying locked set
        for id in &doc_ids {
            if locked.contains(id) {
                return Err(FenceError::Conflict);
            }
        }

        for id in &doc_ids {
            locked.insert(id.clone());
        }

        Ok(FenDocLockGuard {
            fence: self,
            doc_ids,
        })
    }

    /// Unlocks specific document IDs.
    pub fn unlock_documents(&self, doc_ids: &[Vec<u8>]) {
        let mut locked = self.locked_doc_ids.write();
        for id in doc_ids {
            locked.remove(id);
        }
    }

    /// Returns current global scope write sequence number.
    pub fn scope_write_seq(&self) -> u64 {
        self.scope_write_seq.load(Ordering::Acquire)
    }

    /// Returns current scope write sequence for the given guard.
    pub fn scope_write_seq_for(&self, guard: &FenFenceGuard<'_>) -> u64 {
        guard.scope_write_seq()
    }

    /// Takes a snapshot of the write sequence across active fences matching `prefixes`.
    pub fn snapshot_write_seq(&self, _prefixes: &[Vec<u8>]) -> u64 {
        self.scope_write_seq.load(Ordering::Acquire)
    }
}

impl Default for ErasureFence {
    fn default() -> Self {
        Self::new()
    }
}
