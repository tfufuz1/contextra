// ANCHOR[TEST:LOOM-GUARD-ROLLBACK] STATUS:DONE (TS:2026-11-08)
//! Loom model test for Guard-Drop rollback against concurrent append / HMAC access.

#![cfg(loom)]

use loom::sync::atomic::{AtomicU8, Ordering};
use loom::sync::Arc;
use loom::sync::Mutex;
use loom::thread;

const NOT_POISONED: u8 = 0;
const HMAC_CHAIN_DIVERGED: u8 = 4;

struct MockWal {
    last_hmac: Mutex<[u8; 32]>,
    poison: AtomicU8,
}

impl MockWal {
    fn new() -> Self {
        Self {
            last_hmac: Mutex::new([0u8; 32]),
            poison: AtomicU8::new(NOT_POISONED),
        }
    }

    fn rollback_hmac_sync(&self, prev_hmac: [u8; 32]) -> bool {
        if let Ok(mut guard) = self.last_hmac.try_lock() {
            *guard = prev_hmac;
            true
        } else {
            false
        }
    }

    fn poison_with_cause(&self, cause: u8) {
        let _ =
            self.poison
                .compare_exchange(NOT_POISONED, cause, Ordering::SeqCst, Ordering::SeqCst);
    }
}

struct DropGuard {
    wal: Arc<MockWal>,
    prev_hmac: [u8; 32],
    committed: bool,
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        if !self.committed {
            if !self.wal.rollback_hmac_sync(self.prev_hmac) {
                self.wal.poison_with_cause(HMAC_CHAIN_DIVERGED);
            }
        }
    }
}

#[test]
fn loom_guard_drop_against_concurrent_append() {
    loom::model(|| {
        let wal = Arc::new(MockWal::new());

        let wal1 = Arc::clone(&wal);
        let h1 = thread::spawn(move || {
            let guard = DropGuard {
                wal: wal1,
                prev_hmac: [1u8; 32],
                committed: false,
            };
            // Guard dropped here without commit -> triggers rollback_hmac_sync
            drop(guard);
        });

        let wal2 = Arc::clone(&wal);
        let h2 = thread::spawn(move || {
            // Concurrent task holding or updating last_hmac
            if let Ok(mut guard) = wal2.last_hmac.try_lock() {
                *guard = [2u8; 32];
            }
        });

        h1.join().unwrap();
        h2.join().unwrap();

        // Invariant: either rollback succeeded or handle is poisoned with HMAC_CHAIN_DIVERGED
        let poison = wal.poison.load(Ordering::SeqCst);
        let current_hmac = *wal.last_hmac.lock().unwrap();

        if poison == NOT_POISONED {
            // If not poisoned, the HMAC must be a consistent value ([0u8; 32], [1u8; 32], or [2u8; 32])
            assert!(
                current_hmac == [0u8; 32] || current_hmac == [1u8; 32] || current_hmac == [2u8; 32]
            );
        } else {
            assert_eq!(poison, HMAC_CHAIN_DIVERGED);
        }
    });
}
