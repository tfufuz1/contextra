//! Low-level memory locking abstraction for Unix/Windows target architectures.

/// Detailed status outcome for memory locking operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemLockStatus {
    /// Memory region was successfully locked into RAM.
    Locked,
    /// Memory region could not be locked into RAM, with optional OS errno or OS error code.
    NotLocked {
        /// The raw OS error code (e.g. `errno` on Unix or `GetLastError` on Windows) if available.
        errno: Option<i32>,
    },
}

impl MemLockStatus {
    /// Returns `true` if memory was successfully locked.
    pub fn is_locked(&self) -> bool {
        matches!(self, MemLockStatus::Locked)
    }
}

/// Locks a memory region in RAM to prevent swapping (best-effort) and returns explicit status.
#[cfg(unix)]
pub fn mem_lock_explicit(ptr: *const u8, len: usize) -> MemLockStatus {
    if len == 0 || ptr.is_null() {
        return MemLockStatus::Locked;
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
        MemLockStatus::NotLocked {
            errno: err.raw_os_error(),
        }
    } else {
        MemLockStatus::Locked
    }
}

/// Locks a memory region in RAM to prevent swapping (best-effort).
/// Returns `true` if locking succeeded, or `false` with a logged warning if it failed.
pub fn mem_lock(ptr: *const u8, len: usize) -> bool {
    mem_lock_explicit(ptr, len).is_locked()
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

/// Locks a memory region in RAM on Windows to prevent swapping (best-effort) and returns explicit status.
#[cfg(windows)]
pub fn mem_lock_explicit(ptr: *const u8, len: usize) -> MemLockStatus {
    if len == 0 || ptr.is_null() {
        return MemLockStatus::Locked;
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
        MemLockStatus::NotLocked {
            errno: err.raw_os_error(),
        }
    } else {
        MemLockStatus::Locked
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
/// Returns `NotLocked { errno: None }` when `len > 0` and `ptr` is non-null to truthfully indicate memory locking is unavailable.
#[cfg(not(any(unix, windows)))]
pub fn mem_lock_explicit(ptr: *const u8, len: usize) -> MemLockStatus {
    if len == 0 || ptr.is_null() {
        return MemLockStatus::Locked;
    }
    tracing::warn!(
        "Memory locking is not supported on this target architecture for region at {:p} (len={}). \
         Sensitive data remains unlocked in memory (swap-risk).",
        ptr,
        len
    );
    MemLockStatus::NotLocked { errno: None }
}

/// Fallback no-op memory unlock for unsupported platforms.
#[cfg(not(any(unix, windows)))]
pub fn mem_unlock(_addr: usize, _len: usize) {}

/// Applies `MADV_DONTDUMP` to exclude the memory region from core dumps on Linux targets.
/// Returns `true` if advice succeeded or slice is empty/null, `false` otherwise.
///
/// # Security Limitations
/// `MADV_DONTDUMP` only excludes memory from core dumps (`gcore`, panics).
/// It does **NOT** protect memory against swapping, `fork()` child processes, `ptrace` inspection,
/// or direct reading via `/proc/<pid>/mem`.
#[cfg(target_os = "linux")]
pub fn apply_dontdump(ptr: *const u8, len: usize) -> bool {
    if len == 0 || ptr.is_null() {
        return true;
    }
    let page_size = 4096;
    let addr = ptr as usize;
    let page_mask = page_size - 1;
    let aligned_addr = addr & !page_mask;
    let offset = addr & page_mask;
    let aligned_len = (len + offset + page_mask) & !page_mask;

    // SAFETY:
    // 1. `ptr` is non-null and points to a valid allocated memory region of length `len`.
    // 2. `aligned_addr` and `aligned_len` enclose `[ptr, ptr + len)` on page boundaries.
    // 3. `libc::madvise` with `MADV_DONTDUMP` instructs Linux kernel to exclude memory pages from core dumps.
    let ret = unsafe {
        libc::madvise(
            aligned_addr as *mut libc::c_void,
            aligned_len,
            libc::MADV_DONTDUMP,
        )
    };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        tracing::warn!(
            "madvise(MADV_DONTDUMP) failed for region at {:p} (len={}): {}",
            ptr,
            len,
            err
        );
        false
    } else {
        true
    }
}

/// Fallback implementation of `apply_dontdump` for non-Linux target platforms.
#[cfg(not(target_os = "linux"))]
pub fn apply_dontdump(ptr: *const u8, len: usize) -> bool {
    if len == 0 || ptr.is_null() {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_lock_null_or_zero_len() {
        assert_eq!(
            mem_lock_explicit(std::ptr::null(), 0),
            MemLockStatus::Locked
        );
        assert!(mem_lock(std::ptr::null(), 0));
        mem_unlock(0, 0);
    }

    #[test]
    fn test_mem_lock_valid_slice() {
        let buf = vec![0u8; 1024];
        let ptr = buf.as_ptr();
        let len = buf.len();

        let status = mem_lock_explicit(ptr, len);
        if status.is_locked() {
            mem_unlock(ptr as usize, len);
        }
    }

    #[test]
    fn test_apply_dontdump_null_or_zero_len() {
        assert!(apply_dontdump(std::ptr::null(), 0));
    }
}
