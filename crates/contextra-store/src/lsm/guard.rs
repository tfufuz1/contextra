use crate::memtable::MemTable;
use std::sync::Arc;

/// Proof that `commit_mutex` is currently held by the calling task.
/// Can only be constructed while holding the mutex guard.
pub struct CommitGuard<'a> {
    pub(super) _lock: &'a tokio::sync::MutexGuard<'a, ()>,
}

pub struct LsmState {
    pub(super) memtable: Arc<MemTable>,
    pub(super) immutable_memtables: Vec<Arc<MemTable>>,
}
