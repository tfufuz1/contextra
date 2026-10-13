// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Scope-weite und präfixbasierte Schreibsperre (ErasureFence) zur Blockierung von Schreibvorgängen während der Purge-Compaction.
// INVARIANTEN: Lock-Hierarchie einhalten, keine Locks über .await, loom-kompatibel.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
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

#[derive(Debug)]
struct RegisteredPrefixFence {
    id: u64,
    prefixes: Vec<Vec<u8>>,
    write_seq: Arc<AtomicU64>,
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

/// ErasureFence manages scope-wide write blocks (`CollectionDropping`), prefix-based scope fences, and document-level locks.
#[derive(Debug)]
pub struct ErasureFence {
    scope_state: parking_lot::RwLock<FenceScopeState>,
    locked_doc_ids: parking_lot::RwLock<HashSet<Vec<u8>>>,
    scope_write_seq: AtomicU64,
    next_fence_id: AtomicU64,
    prefix_fences: parking_lot::RwLock<Vec<RegisteredPrefixFence>>,
}

impl ErasureFence {
    /// Creates a new inactive `ErasureFence`.
    pub fn new() -> Self {
        Self {
            scope_state: parking_lot::RwLock::new(FenceScopeState::Active),
            locked_doc_ids: parking_lot::RwLock::new(HashSet::new()),
            scope_write_seq: AtomicU64::new(0),
            next_fence_id: AtomicU64::new(1),
            prefix_fences: parking_lot::RwLock::new(Vec::new()),
        }
    }

    /// Checks if two prefixes overlap (i.e. one is a prefix of the other).
    fn prefixes_overlap(p1: &[u8], p2: &[u8]) -> bool {
        p1.starts_with(p2) || p2.starts_with(p1)
    }

    /// Checks if a scope-wide or document-level write block is currently active for `key`.
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

        let fences = self.prefix_fences.read();
        for f in fences.iter() {
            for prefix in &f.prefixes {
                if key.starts_with(prefix) {
                    f.write_seq.fetch_add(1, Ordering::SeqCst);
                    self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
                    return Err(FenceError::CollectionDropping);
                }
            }
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
    ///
    /// Returns `Err(FenceError::Conflict)` if any of the requested prefixes overlap
    /// with an already active scope fence, or internally with each other.
    pub fn acquire_scope_fence_for(
        &self,
        prefixes: Vec<Vec<u8>>,
    ) -> Result<FenFenceGuard<'_>, FenceError> {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return Err(FenceError::CollectionDropping);
        }

        if prefixes.is_empty() {
            return Err(FenceError::Conflict);
        }

        // Check internal overlaps within requested prefixes
        for i in 0..prefixes.len() {
            for j in (i + 1)..prefixes.len() {
                if Self::prefixes_overlap(&prefixes[i], &prefixes[j]) {
                    return Err(FenceError::Conflict);
                }
            }
        }

        let mut fences = self.prefix_fences.write();
        // Check overlaps against existing active prefix fences
        for existing in fences.iter() {
            for ex_p in &existing.prefixes {
                for new_p in &prefixes {
                    if Self::prefixes_overlap(ex_p, new_p) {
                        return Err(FenceError::Conflict);
                    }
                }
            }
        }

        let id = self.next_fence_id.fetch_add(1, Ordering::SeqCst);
        let write_seq = Arc::new(AtomicU64::new(0));

        fences.push(RegisteredPrefixFence {
            id,
            prefixes: prefixes.clone(),
            write_seq: Arc::clone(&write_seq),
        });

        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);

        Ok(FenFenceGuard {
            fence: self,
            id,
            prefixes,
            write_seq,
        })
    }

    /// Helper to release a registered prefix fence by ID.
    fn release_prefix_fence(&self, id: u64) {
        let mut fences = self.prefix_fences.write();
        fences.retain(|f| f.id != id);
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

        let fences = self.prefix_fences.read();
        for f in fences.iter() {
            for doc_id in &doc_ids {
                for prefix in &f.prefixes {
                    if doc_id.starts_with(prefix) {
                        return Err(FenceError::Conflict);
                    }
                }
            }
        }
        drop(fences);

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
    pub fn snapshot_write_seq(&self, prefixes: &[Vec<u8>]) -> u64 {
        let fences = self.prefix_fences.read();
        let mut sum = self.scope_write_seq.load(Ordering::Acquire);
        for f in fences.iter() {
            for ex_p in &f.prefixes {
                for target_p in prefixes {
                    if Self::prefixes_overlap(ex_p, target_p) {
                        sum += f.write_seq.load(Ordering::Acquire);
                        break;
                    }
                }
            }
        }
        sum
    }
}

impl Default for ErasureFence {
    fn default() -> Self {
        Self::new()
    }
}
