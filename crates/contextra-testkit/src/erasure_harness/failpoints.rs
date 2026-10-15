// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Failpoint-Helfer fuer Fehlerinjektions- und Chaos-Loeschtests.
// INVARIANTEN: Nutzbar ausschliesslich mit Feature 'fault-injection'.

#![cfg(feature = "fault-injection")]

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// Controlled fault injector for erasure and storage fault simulation.
#[derive(Debug, Default, Clone)]
pub struct FailpointHelper {
    fail_io: Arc<AtomicBool>,
    fail_fsync: Arc<AtomicBool>,
    fail_count: Arc<AtomicU32>,
}

impl FailpointHelper {
    /// Creates a new `FailpointHelper`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Triggers I/O failures on subsequent operations.
    pub fn inject_io_error(&self) {
        self.fail_io.store(true, Ordering::SeqCst);
    }

    /// Clears I/O error injection.
    pub fn clear_io_error(&self) {
        self.fail_io.store(false, Ordering::SeqCst);
    }

    /// Returns `true` if I/O failure is currently active.
    pub fn should_fail_io(&self) -> bool {
        self.fail_io.load(Ordering::SeqCst)
    }

    /// Triggers fsync failure.
    pub fn inject_fsync_error(&self) {
        self.fail_fsync.store(true, Ordering::SeqCst);
    }

    /// Clears fsync error injection.
    pub fn clear_fsync_error(&self) {
        self.fail_fsync.store(false, Ordering::SeqCst);
    }

    /// Returns `true` if fsync failure is active.
    pub fn should_fail_fsync(&self) -> bool {
        self.fail_fsync.load(Ordering::SeqCst)
    }

    /// Configures a countdown after which failures trigger.
    pub fn set_fail_after_count(&self, count: u32) {
        self.fail_count.store(count, Ordering::SeqCst);
    }

    /// Decrements failure counter and returns true if count reached 0.
    pub fn check_and_decrement(&self) -> bool {
        loop {
            let current = self.fail_count.load(Ordering::SeqCst);
            if current == 0 {
                return false;
            }
            if current == 1 {
                self.fail_count.store(0, Ordering::SeqCst);
                return true;
            }
            if self
                .fail_count
                .compare_exchange_weak(
                    current,
                    current - 1,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                )
                .is_ok()
            {
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failpoint_helper_io_and_countdown() {
        let helper = FailpointHelper::new();
        assert!(!helper.should_fail_io());

        helper.inject_io_error();
        assert!(helper.should_fail_io());

        helper.clear_io_error();
        assert!(!helper.should_fail_io());

        helper.set_fail_after_count(2);
        assert!(!helper.check_and_decrement()); // count -> 1
        assert!(helper.check_and_decrement());  // count -> 0, triggered
        assert!(!helper.check_and_decrement()); // remaining 0
    }
}
