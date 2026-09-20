//! RAM memory-locking region manager for sensitive vault buffers.

use crate::{mem_lock, mem_unlock};

/// Encapsulates locked memory regions (addresses and lengths) for sensitive vault buffers.
/// Automatically releases (unlocks) locked regions when [`unlock_all`](Self::unlock_all) or [`Drop::drop`] is called.
#[derive(Debug, Default)]
pub struct LockedRegions {
    regions: Vec<(usize, usize)>,
}

impl LockedRegions {
    /// Creates a new empty tracker for locked memory regions.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Attempts to lock a byte slice in RAM (best-effort) if `attempt_mlock` is `true`.
    ///
    /// If locking succeeds, the region is tracked and will be unlocked when [`unlock_all`](Self::unlock_all)
    /// or [`Drop::drop`] is called.
    pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) {
        if !attempt_mlock || slice.is_empty() {
            return;
        }

        let ptr = slice.as_ptr();
        let len = slice.len();

        if mem_lock(ptr, len) {
            self.regions.push((ptr as usize, len));
        }
    }

    /// Unlocks all tracked memory regions and clears the tracked list.
    pub fn unlock_all(&mut self) {
        for (addr, len) in self.regions.drain(..) {
            mem_unlock(addr, len);
        }
    }

    /// Returns the number of currently tracked locked regions.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Returns `true` if no locked regions are tracked.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }
}

impl Drop for LockedRegions {
    fn drop(&mut self) {
        self.unlock_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locked_regions_lifecycle() {
        let mut tracker = LockedRegions::new();
        assert!(tracker.is_empty());

        let data = vec![1u8, 2, 3, 4, 5];
        tracker.lock_slice(&data, true);

        #[cfg(target_os = "linux")]
        assert_eq!(tracker.len(), 1);

        tracker.unlock_all();
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_locked_regions_disabled() {
        let mut tracker = LockedRegions::new();
        let data = vec![1u8, 2, 3, 4, 5];
        tracker.lock_slice(&data, false);
        assert!(tracker.is_empty());
    }
}
