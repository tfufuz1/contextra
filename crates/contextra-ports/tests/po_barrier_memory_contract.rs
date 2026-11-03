//! Tests for synchronization barriers (CommitBarrier, PurgeBarrier) and memory protection (ArenaGuard).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_ports::{
    ArenaGuard, BarrierExclusive, BarrierShared, BoxFuture, CommitBarrier, CompactionPermit,
    MemoryLockGuard, PurgeBarrier, PurgePermit, Result,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

struct MockCommitBarrier {
    shared_acquired: AtomicUsize,
    exclusive_acquired: AtomicUsize,
}

impl CommitBarrier for MockCommitBarrier {
    fn shared(&self) -> BoxFuture<'_, BarrierShared> {
        self.shared_acquired.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { BarrierShared::new(None) })
    }

    fn exclusive(&self) -> BoxFuture<'_, BarrierExclusive> {
        self.exclusive_acquired.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { BarrierExclusive::new(None) })
    }
}

struct MockPurgeBarrier {
    purge_waiting: AtomicBool,
    compaction_count: AtomicUsize,
    purge_count: AtomicUsize,
}

impl PurgeBarrier for MockPurgeBarrier {
    fn compaction(&self) -> BoxFuture<'_, CompactionPermit> {
        self.compaction_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { CompactionPermit::new(None) })
    }

    fn purge(&self) -> BoxFuture<'_, PurgePermit> {
        self.purge_count.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { PurgePermit::new(None) })
    }

    fn purge_waiting(&self) -> bool {
        self.purge_waiting.load(Ordering::SeqCst)
    }
}

struct MockArenaGuard {
    lock_calls: AtomicUsize,
    madvise_calls: AtomicUsize,
}

impl ArenaGuard for MockArenaGuard {
    fn lock_memory(&self, _ptr: *const u8, _len: usize) -> Result<MemoryLockGuard> {
        self.lock_calls.fetch_add(1, Ordering::SeqCst);
        Ok(MemoryLockGuard::new(None))
    }

    fn protect_against_dumping(&self, _ptr: *const u8, _len: usize) -> Result<()> {
        self.madvise_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[tokio::test]
async fn test_commit_barrier_contracts() {
    let barrier = MockCommitBarrier {
        shared_acquired: AtomicUsize::new(0),
        exclusive_acquired: AtomicUsize::new(0),
    };
    let dyn_barrier: Arc<dyn CommitBarrier> = Arc::new(barrier);

    let _shared = dyn_barrier.shared().await;
    let _exclusive = dyn_barrier.exclusive().await;

    // Verify debug formatting masks nothing secret but provides clean type descriptor
    assert_eq!(format!("{:?}", _shared), "BarrierShared");
    assert_eq!(format!("{:?}", _exclusive), "BarrierExclusive");
}

#[tokio::test]
async fn test_purge_barrier_contracts() {
    let barrier = MockPurgeBarrier {
        purge_waiting: AtomicBool::new(false),
        compaction_count: AtomicUsize::new(0),
        purge_count: AtomicUsize::new(0),
    };
    let dyn_barrier: Arc<dyn PurgeBarrier> = Arc::new(barrier);

    assert!(!dyn_barrier.purge_waiting());

    let _compaction = dyn_barrier.compaction().await;
    let _purge = dyn_barrier.purge().await;

    assert_eq!(format!("{:?}", _compaction), "CompactionPermit");
    assert_eq!(format!("{:?}", _purge), "PurgePermit");
}

#[test]
fn test_arena_guard_contracts() {
    let guard = MockArenaGuard {
        lock_calls: AtomicUsize::new(0),
        madvise_calls: AtomicUsize::new(0),
    };
    let dyn_guard: Arc<dyn ArenaGuard> = Arc::new(guard);

    let dummy_slice = [0u8; 64];
    let lock_guard = dyn_guard
        .lock_memory(dummy_slice.as_ptr(), dummy_slice.len())
        .expect("lock memory succeeds");
    dyn_guard
        .protect_against_dumping(dummy_slice.as_ptr(), dummy_slice.len())
        .expect("dump protection succeeds");

    assert_eq!(format!("{:?}", lock_guard), "MemoryLockGuard([PROTECTED])");
}
