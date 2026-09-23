//! Low-level memory locking abstraction for Unix/Windows target architectures.

/// Locks a memory region in RAM to prevent swapping (best-effort).
/// Returns `true` if locking succeeded, or `false` with a logged warning if it failed.
#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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

#[cfg(not(target_os = "linux"))]
pub fn mem_lock(_ptr: *const u8, _len: usize) -> bool {
    true
}

#[cfg(not(target_os = "linux"))]
pub fn mem_unlock(_addr: usize, _len: usize) {}
