// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: RAII manager for RAM memory-locked sensitive buffer regions and memory zeroization (Ring 0 / SPEC 7.7)
// INVARIANTEN: Automatically releases locked memory regions via mem_unlock on Drop or unlock_all
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! RAM memory-locking region manager for sensitive vault buffers.

use crate::{mem_lock_with_policy, mem_unlock, MemLockPolicy};

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

    /// Attempts to lock a byte slice in RAM if `attempt_mlock` is `true`, using default [`MemLockPolicy::Tolerant`].
    pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool {
        self.lock_slice_with_policy(slice, attempt_mlock, MemLockPolicy::Tolerant)
            .unwrap_or(false)
    }

    /// Attempts to lock a byte slice in RAM according to `policy` if `attempt_mlock` is `true`.
    ///
    /// # Errors
    /// Returns an [`std::io::Error`] if locking fails under [`MemLockPolicy::FailClosed`].
    pub fn lock_slice_with_policy(
        &mut self,
        slice: &[u8],
        attempt_mlock: bool,
        policy: MemLockPolicy,
    ) -> std::io::Result<bool> {
        if slice.is_empty() {
            return Ok(true);
        }
        if !attempt_mlock {
            return Ok(false);
        }

        let ptr = slice.as_ptr();
        let len = slice.len();

        let locked = mem_lock_with_policy(ptr, len, policy)?;
        if locked {
            self.regions.push((ptr as usize, len));
        }
        Ok(locked)
    }

    /// Safely zeroizes mutable slice memory using volatile writes to prevent compiler dead-store elimination.
    pub fn zeroize_slice(slice: &mut [u8]) {
        for byte in slice.iter_mut() {
            // SAFETY: `byte` is a valid, aligned mutable reference to a u8 in `slice`.
            // Volatile write guarantees write is not optimized away by compiler passes.
            // SAFETY: Volatile write guarantees write is not optimized away by compiler passes.
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
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
        let locked = tracker.lock_slice(&data, true);

        if locked {
            assert_eq!(tracker.len(), 1);
        } else {
            assert_eq!(tracker.len(), 0);
        }

        tracker.unlock_all();
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_locked_regions_disabled() {
        let mut tracker = LockedRegions::new();
        let data = vec![1u8, 2, 3, 4, 5];
        let locked = tracker.lock_slice(&data, false);
        assert!(!locked);
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_locked_regions_with_fail_closed_policy() {
        let mut tracker = LockedRegions::new();
        let data = vec![1u8, 2, 3, 4, 5];
        let res = tracker.lock_slice_with_policy(&data, true, MemLockPolicy::FailClosed);
        if let Ok(locked) = res {
            if locked {
                assert_eq!(tracker.len(), 1);
            }
        }
    }

    #[test]
    fn test_zeroize_slice() {
        let mut secret = vec![0x42u8; 16];
        LockedRegions::zeroize_slice(&mut secret);
        assert_eq!(secret, vec![0u8; 16]);
    }
}
