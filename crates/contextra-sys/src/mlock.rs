// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Platform-aware RAM memory locking abstractions with policy enforcement (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; configurable mlock policy (FailClosed vs Tolerant); typed io::Error propagation
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! Low-level RAM memory locking abstraction for Unix/Windows target architectures.

use std::io::{Error, Result};

/// Configurable policy for memory locking operations (`mlock` / `VirtualLock`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemLockPolicy {
    /// Memory locking failures are treated as fatal errors (e.g. for cryptographic key material).
    FailClosed,
    /// Memory locking failures are logged as warnings and non-fatal (e.g. for performance caches).
    #[default]
    Tolerant,
}

/// Locks a memory region in RAM to prevent swapping according to the given [`MemLockPolicy`].
///
/// # Errors
/// Returns an [`std::io::Error`] if memory locking fails and `policy` is [`MemLockPolicy::FailClosed`].
#[cfg(unix)]
pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> {
    if len == 0 || ptr.is_null() {
        return Ok(true);
    }
    // SAFETY: `ptr` is non-null and points to `len` bytes of allocated memory. `libc::mlock` pins pages in RAM.
    let ret = unsafe { libc::mlock(ptr as *const libc::c_void, len) };
    if ret != 0 {
        let err = Error::last_os_error();
        match policy {
            MemLockPolicy::FailClosed => Err(Error::new(
                err.kind(),
                format!(
                    "mlock() failed for memory region at {:p} (len={}): {} (FailClosed)",
                    ptr, len, err
                ),
            )),
            MemLockPolicy::Tolerant => {
                tracing::warn!(
                    "mlock() failed for memory region at {:p} (len={}): {}. \
                     Data remains unlocked in memory (swap-risk, Tolerant).",
                    ptr,
                    len,
                    err
                );
                Ok(false)
            }
        }
    } else {
        Ok(true)
    }
}

/// Unlocks a previously locked memory region on Unix.
#[cfg(unix)]
pub fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }
    // SAFETY: `addr` and `len` refer to a valid memory allocation passed to `mem_lock`.
    unsafe {
        libc::munlock(addr as *mut libc::c_void, len);
    }
}

/// Locks a memory region in RAM on Windows according to the given [`MemLockPolicy`].
///
/// # Errors
/// Returns an [`std::io::Error`] if memory locking fails and `policy` is [`MemLockPolicy::FailClosed`].
#[cfg(windows)]
pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> {
    if len == 0 || ptr.is_null() {
        return Ok(true);
    }
    // SAFETY: `ptr` is non-null and points to `len` bytes of memory. `VirtualLock` pins pages in physical RAM.
    let ret = unsafe {
        windows_sys::Win32::System::Memory::VirtualLock(ptr as *const std::ffi::c_void, len)
    };
    if ret == 0 {
        let err = Error::last_os_error();
        match policy {
            MemLockPolicy::FailClosed => Err(Error::new(
                err.kind(),
                format!(
                    "VirtualLock() failed for region at {:p} (len={}): {} (FailClosed)",
                    ptr, len, err
                ),
            )),
            MemLockPolicy::Tolerant => {
                tracing::warn!(
                    "VirtualLock() failed for memory region at {:p} (len={}): {}. \
                     Data remains unlocked in memory (swap-risk, Tolerant).",
                    ptr,
                    len,
                    err
                );
                Ok(false)
            }
        }
    } else {
        Ok(true)
    }
}

/// Unlocks a previously locked memory region on Windows.
#[cfg(windows)]
pub fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }
    // SAFETY: `addr` and `len` refer to a valid memory allocation passed to `mem_lock`.
    unsafe {
        windows_sys::Win32::System::Memory::VirtualUnlock(addr as *const std::ffi::c_void, len);
    }
}

/// Fallback implementation for non-Unix/non-Windows platforms.
///
/// # Errors
/// Returns an error if `policy` is [`MemLockPolicy::FailClosed`] and `len > 0`.
#[cfg(not(any(unix, windows)))]
pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> {
    if len == 0 || ptr.is_null() {
        return Ok(true);
    }
    match policy {
        MemLockPolicy::FailClosed => Err(Error::new(
            std::io::ErrorKind::Unsupported,
            "Memory locking is not supported on this target architecture (FailClosed)",
        )),
        MemLockPolicy::Tolerant => {
            tracing::warn!(
                "Memory locking is not supported on this target architecture for region at {:p} (len={}).",
                ptr,
                len
            );
            Ok(false)
        }
    }
}

/// Fallback no-op memory unlock for unsupported platforms.
#[cfg(not(any(unix, windows)))]
pub fn mem_unlock(_addr: usize, _len: usize) {}

/// Locks a memory region in RAM using default [`MemLockPolicy::Tolerant`] behavior.
/// Returns `true` if locking succeeded, or `false` if it failed.
pub fn mem_lock(ptr: *const u8, len: usize) -> bool {
    mem_lock_with_policy(ptr, len, MemLockPolicy::Tolerant).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_lock_null_or_zero_len() -> Result<()> {
        assert!(mem_lock_with_policy(
            std::ptr::null(),
            0,
            MemLockPolicy::FailClosed
        )?);
        assert!(mem_lock_with_policy(
            std::ptr::null(),
            0,
            MemLockPolicy::Tolerant
        )?);
        assert!(mem_lock(std::ptr::null(), 0));
        mem_unlock(0, 0);
        Ok(())
    }

    #[test]
    fn test_mem_lock_valid_slice_tolerant() -> Result<()> {
        let buf = vec![0u8; 1024];
        let ptr = buf.as_ptr();
        let len = buf.len();

        let ok = mem_lock_with_policy(ptr, len, MemLockPolicy::Tolerant)?;
        if ok {
            mem_unlock(ptr as usize, len);
        }
        Ok(())
    }

    #[test]
    fn test_mem_lock_valid_slice_fail_closed_or_ok() {
        let buf = vec![0u8; 1024];
        let ptr = buf.as_ptr();
        let len = buf.len();

        let res = mem_lock_with_policy(ptr, len, MemLockPolicy::FailClosed);
        if let Ok(true) = res {
            mem_unlock(ptr as usize, len);
        }
    }

    #[test]
    fn test_mem_lock_invalid_address_fail_closed_simulation() {
        // Pass a dangling non-null pointer with unmapped address to simulate mlock failure.
        let invalid_ptr = (usize::MAX - 4096) as *const u8;
        let res = mem_lock_with_policy(invalid_ptr, 4096, MemLockPolicy::FailClosed);
        assert!(res.is_err());
    }

    #[test]
    fn test_mem_lock_invalid_address_tolerant_simulation() -> Result<()> {
        let invalid_ptr = (usize::MAX - 4096) as *const u8;
        let res = mem_lock_with_policy(invalid_ptr, 4096, MemLockPolicy::Tolerant)?;
        assert!(!res);
        Ok(())
    }
}
