//! Low-level memory locking abstraction for Unix/Windows target architectures.

/// Locks a memory region in RAM to prevent swapping (best-effort).
/// Returns `true` if locking succeeded, or `false` with a logged warning if it failed.
#[cfg(unix)]
pub fn mem_lock(ptr: *const u8, len: usize) -> bool {
    if len == 0 || ptr.is_null() {
        return true;
    }
    // SAFETY:
    // 1. `ptr` is guaranteed non-null and points to a valid allocated memory region of length `len`.
    // 2. `libc::mlock` takes raw pointer and size to pin pages in physical memory.
    let ret = unsafe { libc::mlock(ptr as *const libc::c_void, len) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        tracing::warn!(
            "mlock() failed for memory region at {:p} (len={}): {}. \
             Sensitive data remains unlocked in memory (swap-risk).",
            ptr,
            len,
            err
        );
        false
    } else {
        true
    }
}

/// Unlocks a previously locked memory region.
#[cfg(unix)]
pub fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }
    // SAFETY:
    // 1. `addr` and `len` originated from a valid memory allocation that was passed to `mem_lock`.
    // 2. `libc::munlock` releases physical memory page pinning.
    unsafe {
        libc::munlock(addr as *mut libc::c_void, len);
    }
}

/// Locks a memory region in RAM on Windows to prevent swapping (best-effort).
/// Returns `true` if locking succeeded, or `false` with a logged warning if it failed.
#[cfg(windows)]
pub fn mem_lock(ptr: *const u8, len: usize) -> bool {
    if len == 0 || ptr.is_null() {
        return true;
    }
    // SAFETY:
    // 1. `ptr` is guaranteed non-null and points to a valid allocated memory region of length `len`.
    // 2. `VirtualLock` locks the specified region into physical RAM.
    let ret = unsafe {
        windows_sys::Win32::System::Memory::VirtualLock(ptr as *const std::ffi::c_void, len)
    };
    if ret == 0 {
        let err = std::io::Error::last_os_error();
        tracing::warn!(
            "VirtualLock() failed for memory region at {:p} (len={}): {}. \
             Sensitive data remains unlocked in memory (swap-risk).",
            ptr,
            len,
            err
        );
        false
    } else {
        true
    }
}

/// Unlocks a previously locked memory region on Windows.
#[cfg(windows)]
pub fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }
    // SAFETY:
    // 1. `addr` and `len` originated from a valid memory allocation passed to `mem_lock`.
    // 2. `VirtualUnlock` releases physical RAM memory lock.
    unsafe {
        windows_sys::Win32::System::Memory::VirtualUnlock(addr as *const std::ffi::c_void, len);
    }
}

/// Fallback implementation for non-Unix/non-Windows platforms.
/// Returns `false` when `len > 0` and `ptr` is non-null to truthfully indicate memory locking is unavailable.
#[cfg(not(any(unix, windows)))]
pub fn mem_lock(ptr: *const u8, len: usize) -> bool {
    if len == 0 || ptr.is_null() {
        return true;
    }
    tracing::warn!(
        "Memory locking is not supported on this target architecture for region at {:p} (len={}). \
         Sensitive data remains unlocked in memory (swap-risk).",
        ptr,
        len
    );
    false
}

/// Fallback no-op memory unlock for unsupported platforms.
#[cfg(not(any(unix, windows)))]
pub fn mem_unlock(_addr: usize, _len: usize) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_lock_null_or_zero_len() {
        assert!(mem_lock(std::ptr::null(), 0));
        mem_unlock(0, 0);
    }

    #[test]
    fn test_mem_lock_valid_slice() {
        let buf = vec![0u8; 1024];
        let ptr = buf.as_ptr();
        let len = buf.len();

        let ok = mem_lock(ptr, len);
        // Note: mlock/VirtualLock might succeed or fail depending on process memory limits.
        // What matters is that it does not panic and returns a boolean.
        if ok {
            mem_unlock(ptr as usize, len);
        }
    }
}
