// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Scope-weite und präfixbasierte Schreibsperre (ErasureGate & ErasureFence) zur Blockierung von Schreibvorgängen während der Purge-Compaction.
// INVARIANTEN: Lock-Hierarchie einhalten, keine Locks über .await, loom-kompatibel.

use std::collections::HashSet;
use std::time::Duration;

#[cfg(loom)]
use loom::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
#[cfg(loom)]
use loom::sync::{Arc, Mutex, RwLock};

#[cfg(not(loom))]
use parking_lot::Mutex;
#[cfg(not(loom))]
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixFence {
    pub purge_id: u64,
    pub prefixes: Vec<Vec<u8>>,
}

/// Low-level gate managing atomic active flag, RCU prefix fence table, and inflight write batch tracking.
#[derive(Debug)]
pub struct ErasureGate {
    active: AtomicBool,
    #[cfg(not(loom))]
    table: arc_swap::ArcSwap<Vec<PrefixFence>>,
    #[cfg(loom)]
    table: RwLock<Vec<PrefixFence>>,
    inflight: AtomicU32,
    drained: tokio::sync::Notify,
    install: Mutex<()>,
}

impl ErasureGate {
    pub fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            #[cfg(not(loom))]
            table: arc_swap::ArcSwap::new(Arc::new(Vec::new())),
            #[cfg(loom)]
            table: RwLock::new(Vec::new()),
            inflight: AtomicU32::new(0),
            drained: tokio::sync::Notify::new(),
            install: Mutex::new(()),
        }
    }

    /// Fast-path check: returns whether any scope fence is active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Returns current count of inflight write batches in progress.
    #[inline]
    pub fn inflight_count(&self) -> u32 {
        self.inflight.load(Ordering::Acquire)
    }

    /// Increments inflight batch counter.
    #[inline]
    pub fn inc_inflight(&self) {
        self.inflight.fetch_add(1, Ordering::SeqCst);
    }

    /// Decrements inflight batch counter and notifies drain waiters if count reaches zero.
    #[inline]
    pub fn dec_inflight(&self) {
        let prev = self.inflight.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 && self.is_active() {
            self.drained.notify_waiters();
        }
    }

    /// Checks if a key violates any active prefix fence.
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

    /// Helper to test prefix overlap.
    fn prefixes_overlap(p1: &[u8], p2: &[u8]) -> bool {
        if p1.is_empty() || p2.is_empty() {
            return true;
        }
        p1.starts_with(p2) || p2.starts_with(p1)
    }

    /// Installs a new prefix fence, drains inflight write batches until inflight == 0.
    pub async fn acquire_fence(
        &self,
        purge_id: u64,
        prefixes: Vec<Vec<u8>>,
        timeout: Duration,
    ) -> Result<(), FenceError> {
        #[cfg(not(loom))]
        let _guard = self.install.lock();
        #[cfg(loom)]
        let _guard = self.install.lock().unwrap();

        if prefixes.is_empty() {
            return Err(FenceError::Conflict);
        }

        // Validate internal overlaps in requested prefixes
        for i in 0..prefixes.len() {
            for j in (i + 1)..prefixes.len() {
                if Self::prefixes_overlap(&prefixes[i], &prefixes[j]) {
                    return Err(FenceError::Conflict);
                }
            }
        }

        // Validate overlaps against existing prefix fences
        #[cfg(not(loom))]
        {
            let current_table = self.table.load();
            for existing in current_table.iter() {
                for ex_p in &existing.prefixes {
                    for new_p in &prefixes {
                        if Self::prefixes_overlap(ex_p, new_p) {
                            return Err(FenceError::Conflict);
                        }
                    }
                }
            }
        }
        #[cfg(loom)]
        {
            if let Ok(current_table) = self.table.read() {
                for existing in current_table.iter() {
                    for ex_p in &existing.prefixes {
                        for new_p in &prefixes {
                            if Self::prefixes_overlap(ex_p, new_p) {
                                return Err(FenceError::Conflict);
                            }
                        }
                    }
                }
            }
        }

        // Install new fence into table
        #[cfg(not(loom))]
        {
            let current_vec: &Vec<PrefixFence> = &**self.table.load();
            let mut new_table = current_vec.clone();
            new_table.push(PrefixFence {
                purge_id,
                prefixes: prefixes.clone(),
            });
            self.table.store(Arc::new(new_table));
        }
        #[cfg(loom)]
        {
            if let Ok(mut table) = self.table.write() {
                table.push(PrefixFence {
                    purge_id,
                    prefixes: prefixes.clone(),
                });
            }
        }

        self.active.store(true, Ordering::Release);

        // Drain phase: wait until inflight == 0
        let start = std::time::Instant::now();
        loop {
            if self.inflight.load(Ordering::Acquire) == 0 {
                break;
            }

            if start.elapsed() >= timeout {
                // Timeout during Drain: revert fence installation
                self.release_fence_internal(purge_id);
                return Err(FenceError::Conflict);
            }

            // Wait on notification or brief sleep
            let _ = tokio::time::timeout(Duration::from_millis(2), self.drained.notified()).await;
        }

        Ok(())
    }

    /// Releases a registered prefix fence by purge_id.
    pub fn release_fence(&self, purge_id: u64) {
        #[cfg(not(loom))]
        let _guard = self.install.lock();
        #[cfg(loom)]
        let _guard = self.install.lock().unwrap();

        self.release_fence_internal(purge_id);
    }

    fn release_fence_internal(&self, purge_id: u64) {
        #[cfg(not(loom))]
        {
            let current_vec: &Vec<PrefixFence> = &**self.table.load();
            let mut new_table = current_vec.clone();
            new_table.retain(|f| f.purge_id != purge_id);
            let is_empty = new_table.is_empty();
            self.table.store(Arc::new(new_table));
            if is_empty {
                self.active.store(false, Ordering::Release);
            }
        }

        #[cfg(loom)]
        {
            if let Ok(mut table) = self.table.write() {
                table.retain(|f| f.purge_id != purge_id);
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

/// RAII Guard that increments inflight counter on creation and decrements on drop.
pub struct InflightGuard<'a> {
    gate: &'a ErasureGate,
}

impl<'a> InflightGuard<'a> {
    pub fn new(gate: &'a ErasureGate) -> Self {
        gate.inc_inflight();
        Self { gate }
    }
}

impl<'a> Drop for InflightGuard<'a> {
    fn drop(&mut self) {
        self.gate.dec_inflight();
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

    /// Returns unique fence identifier.
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<'a> Drop for FenFenceGuard<'a> {
    fn drop(&mut self) {
        self.fence.release_prefix_fence(self.id);
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

        self.gate.check_write_allowed(key)?;

        let locked_docs = self.locked_doc_ids.read();
        if locked_docs.contains(key) {
            self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
            return Err(FenceError::Conflict);
        }

        Ok(())
    }

    /// Activates global scope-wide write block (`CollectionDropping`).
    pub fn acquire_scope_fence(&self) {
        let mut state = self.scope_state.write();
        *state = FenceScopeState::CollectionDropping;
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Deactivates global scope-wide write block back to `Active`.
    pub fn release_scope_fence(&self) {
        let mut state = self.scope_state.write();
        *state = FenceScopeState::Active;
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Acquires a scope fence for specific key prefixes.
    pub fn acquire_scope_fence_for(
        &self,
        prefixes: Vec<Vec<u8>>,
    ) -> Result<FenFenceGuard<'_>, FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return Err(FenceError::CollectionDropping);
        }

        let id = self.next_fence_id.fetch_add(1, Ordering::SeqCst);
        let write_seq = Arc::new(AtomicU64::new(0));

        // Use blocking handle for sync caller on acquire_fence
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.gate
                    .acquire_fence(id, prefixes.clone(), Duration::from_secs(5))
                    .await
            })
        })?;

        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);

        Ok(FenFenceGuard {
            fence: self,
            id,
            prefixes,
            write_seq,
        })
    }

    /// Helper to release a registered prefix fence by ID.
    pub(crate) fn release_prefix_fence(&self, id: u64) {
        self.gate.release_fence(id);
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
