use crate::memtable::MemTable;
use std::sync::Arc;
use tokio::sync::OwnedMutexGuard;

/// Proof that `commit_mutex` is currently held by the calling task.
/// Can only be constructed while holding the mutex guard.
pub(super) struct CommitGuard<'a> {
    _lock: &'a tokio::sync::MutexGuard<'a, ()>,
}

pub(super) struct LsmState {
    pub(super) memtable: Arc<MemTable>,
    pub(super) immutable_memtables: Vec<Arc<MemTable>>,
}
