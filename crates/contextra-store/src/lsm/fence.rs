// FILE-CONTEXT
// STAND: 2026-10-11
// ZWECK: Scope-weite Schreibsperre (ErasureFence) zur Blockierung von Schreibvorgängen während der Purge-Compaction.
// INVARIANTEN: Lock-Hierarchie einhalten, keine Locks über .await, loom-kompatibel.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};

/// Scope-wide write block state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenceScopeState {
    /// Active write access permitted.
    Active,
    /// Collection is currently being dropped / purged. Scope-wide write block active.
    CollectionDropping,
}

/// ErasureFence manages scope-wide write blocks (`CollectionDropping`) and document-level locks.
#[derive(Debug)]
pub struct ErasureFence {
    scope_state: parking_lot::RwLock<FenceScopeState>,
    locked_doc_ids: parking_lot::RwLock<HashSet<Vec<u8>>>,
    scope_write_seq: AtomicU64,
}

impl ErasureFence {
    /// Creates a new inactive `ErasureFence`.
    pub fn new() -> Self {
        Self {
            scope_state: parking_lot::RwLock::new(FenceScopeState::Active),
            locked_doc_ids: parking_lot::RwLock::new(HashSet::new()),
            scope_write_seq: AtomicU64::new(0),
        }
    }

    /// Checks if a scope-wide or document-level write block is currently active for `key`.
    /// Returns `true` if write access is allowed, `false` if blocked by fence.
    pub fn check_write_allowed(&self, key: &[u8]) -> bool {
        let state = *self.scope_state.read();
        if state == FenceScopeState::CollectionDropping {
            return false;
        }

        let locked_docs = self.locked_doc_ids.read();
        if locked_docs.contains(key) {
            return false;
        }

        true
    }

    /// Activates scope-wide write block (`CollectionDropping`).
    pub fn acquire_scope_fence(&self) {
        let mut state = self.scope_state.write();
        *state = FenceScopeState::CollectionDropping;
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Deactivates scope-wide write block back to `Active`.
    pub fn release_scope_fence(&self) {
        let mut state = self.scope_state.write();
        *state = FenceScopeState::Active;
        self.scope_write_seq.fetch_add(1, Ordering::SeqCst);
    }

    /// Locks specific document IDs against writes.
    pub fn lock_documents(&self, doc_ids: &[Vec<u8>]) {
        let mut locked = self.locked_doc_ids.write();
        for id in doc_ids {
            locked.insert(id.clone());
        }
    }

    /// Unlocks specific document IDs.
    pub fn unlock_documents(&self, doc_ids: &[Vec<u8>]) {
        let mut locked = self.locked_doc_ids.write();
        for id in doc_ids {
            locked.remove(id);
        }
    }

    /// Returns current scope write sequence number.
    pub fn scope_write_seq(&self) -> u64 {
        self.scope_write_seq.load(Ordering::Acquire)
    }
}

impl Default for ErasureFence {
    fn default() -> Self {
        Self::new()
    }
}
