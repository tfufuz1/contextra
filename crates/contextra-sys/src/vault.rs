// FILE-CONTEXT
// STAND: 2026-10-13T22:30:00Z (SESSION: A17)
// ZWECK: RAII manager for RAM memory-locked sensitive buffer regions (Ring 0)
// INVARIANTEN: Automatically releases locked memory regions via mem_unlock on Drop or unlock_all
// NICHT-OFFENSICHTLICH: Best-effort locking by default; strict mode returns Err if mlock fails.
// SIEHE AUCH: AGENTS.md, docs/audits/contextra-sys_AUDIT_2026-09-27.md

//! RAM memory-locking region manager for sensitive vault buffers.

use crate::{mem_unlock, mlock};

/// Observed protection status for a vaulted memory region.
///
/// # Security Limitations & Honest Disclosure
/// These status flags report **observed** operating system responses, NOT physical security guarantees.
/// - `mlock_active`: Indicates `mlock(2)` or `VirtualLock` returned success. This prevents OS paging/swapping
///   under normal operating conditions. It does **NOT** guarantee immunity against hibernation dumps, cold-boot attacks,
///   or kernel memory analysis.
/// - `dontdump_applied`: Indicates `madvise(MADV_DONTDUMP)` returned success. This excludes the memory region from
///   process core dumps (e.g. `gcore` or panic dumps). It does **NOT** protect memory against OS swapping, `fork()` child process
///   memory copying, `ptrace` debugging/inspection, or process memory reading by privileged users (e.g., `/proc/<pid>/mem`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VaultProtectionStatus {
    /// Whether `mlock` or `VirtualLock` was successfully applied to the memory region.
    pub mlock_active: bool,
    /// Whether `MADV_DONTDUMP` was successfully applied to the memory region.
    pub dontdump_applied: bool,
}

/// Encapsulates locked memory regions (addresses and lengths) for sensitive vault buffers.
/// Automatically releases (unlocks) locked regions when [`unlock_all`](Self::unlock_all) or [`Drop::drop`] is called.
#[derive(Debug, Default)]
pub struct LockedRegions {
    regions: Vec<(usize, usize)>,
    strict_mode: bool,
    last_status: VaultProtectionStatus,
}

impl LockedRegions {
    /// Creates a new empty tracker for locked memory regions in default (non-strict) mode.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            strict_mode: false,
            last_status: VaultProtectionStatus::default(),
        }
    }

    /// Creates a new empty tracker for locked memory regions with the specified strict mode flag.
    ///
    /// In strict mode (`strict_mode = true`), attempting to lock non-empty key material returns an [`Err`]
    /// if `mlock` fails, preventing unencrypted or unlocked secret processing.
    pub fn with_strict_mode(strict_mode: bool) -> Self {
        Self {
            regions: Vec::new(),
            strict_mode,
            last_status: VaultProtectionStatus::default(),
        }
    }

    /// Returns `true` if strict mode is enabled.
    pub fn is_strict(&self) -> bool {
        self.strict_mode
    }

    /// Returns the last observed protection status for tracked memory regions.
    pub fn protection_status(&self) -> VaultProtectionStatus {
        if self.regions.is_empty() {
            VaultProtectionStatus::default()
        } else {
            self.last_status
        }
    }

    /// Attempts to lock a byte slice in RAM with explicit status and error reporting.
    ///
    /// If `attempt_mlock` is `true`, invokes `mem_lock_explicit` and applies `MADV_DONTDUMP` (where supported).
    /// Returns [`Ok(VaultProtectionStatus)`] with the observed protection status.
    /// In strict mode (`strict_mode = true`), if memory locking fails for a non-empty slice, returns an [`Err(std::io::Error)`].
    pub fn try_lock_slice(
        &mut self,
        slice: &[u8],
        attempt_mlock: bool,
    ) -> Result<VaultProtectionStatus, std::io::Error> {
        if slice.is_empty() {
            let status = VaultProtectionStatus {
                mlock_active: true,
                dontdump_applied: true,
            };
            return Ok(status);
        }

        if !attempt_mlock {
            let status = VaultProtectionStatus {
                mlock_active: false,
                dontdump_applied: false,
            };
            if self.strict_mode {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Memory locking is disabled but strict mode requires active mlock",
                ));
            }
            return Ok(status);
        }

        let ptr = slice.as_ptr();
        let len = slice.len();

        let lock_status = mlock::mem_lock_explicit(ptr, len);
        if lock_status.is_locked() {
            self.regions.push((ptr as usize, len));
            let dontdump = mlock::apply_dontdump(ptr, len);
            let prot = VaultProtectionStatus {
                mlock_active: true,
                dontdump_applied: dontdump,
            };
            self.last_status = prot;
            Ok(prot)
        } else {
            let errno_opt = match lock_status {
                mlock::MemLockStatus::NotLocked { errno } => errno,
                _ => None,
            };
            let prot = VaultProtectionStatus {
                mlock_active: false,
                dontdump_applied: false,
            };
            self.last_status = prot;

            if self.strict_mode {
                let err = match errno_opt {
                    Some(code) => std::io::Error::from_raw_os_error(code),
                    None => std::io::Error::other(
                        "Memory locking failed on unsupported target in strict mode",
                    ),
                };
                Err(err)
            } else {
                Ok(prot)
            }
        }
    }

    /// Attempts to lock a byte slice in RAM (best-effort) if `attempt_mlock` is `true`.
    ///
    /// Returns `true` if the slice is successfully memory-locked (or if `slice` is empty).
    /// Returns `false` if `attempt_mlock` is `false` (for non-empty slices) or if `mem_lock` failed.
    ///
    /// If locking succeeds, the region is tracked and will be unlocked when [`unlock_all`](Self::unlock_all)
    /// or [`Drop::drop`] is called.
    pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool {
        match self.try_lock_slice(slice, attempt_mlock) {
            Ok(status) => status.mlock_active,
            Err(_) => false,
        }
    }

    /// Unlocks all tracked memory regions and clears the tracked list.
    pub fn unlock_all(&mut self) {
        for (addr, len) in self.regions.drain(..) {
            mem_unlock(addr, len);
        }
        self.last_status = VaultProtectionStatus::default();
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
        assert!(!tracker.is_strict());

        let data = vec![1u8, 2, 3, 4, 5];
        let locked = tracker.lock_slice(&data, true);

        #[cfg(any(unix, windows))]
        {
            if locked {
                assert_eq!(tracker.len(), 1);
                let status = tracker.protection_status();
                assert!(status.mlock_active);
            } else {
                assert_eq!(tracker.len(), 0);
            }
        }

        tracker.unlock_all();
        assert!(tracker.is_empty());
        assert_eq!(
            tracker.protection_status(),
            VaultProtectionStatus::default()
        );
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
    fn test_strict_mode_disabled_lock_attempt() {
        let mut tracker = LockedRegions::with_strict_mode(true);
        assert!(tracker.is_strict());
        let data = vec![1u8, 2, 3, 4, 5];
        let res = tracker.try_lock_slice(&data, false);
        assert!(res.is_err());
    }
}
