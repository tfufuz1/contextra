// FILE-CONTEXT
// STAND: 2026-10-13T22:30:00Z (SESSION: A17)
// ZWECK: Integration tests for mlock explicit status, vault protection reporting, RLIMIT error handling, and strict mode (N11)
// INVARIANTEN: Zero-Panic doctrine; fail-closed behavior in strict mode; CI tolerance without CAP_IPC_LOCK

#![allow(unsafe_code)]

use contextra_sys::mlock::{apply_dontdump, mem_lock_explicit, mem_unlock, MemLockStatus};
use contextra_sys::vault::{LockedRegions, VaultProtectionStatus};

#[test]
fn test_mem_lock_explicit_zero_and_null() {
    assert_eq!(
        mem_lock_explicit(std::ptr::null(), 0),
        MemLockStatus::Locked
    );
    assert_eq!(
        mem_lock_explicit(std::ptr::null(), 1024),
        MemLockStatus::Locked
    );
}

#[test]
fn test_mem_lock_explicit_valid_buffer() {
    let buf = vec![0u8; 2048];
    let ptr = buf.as_ptr();
    let len = buf.len();

    let status = mem_lock_explicit(ptr, len);
    match status {
        MemLockStatus::Locked => {
            assert!(status.is_locked());
            mem_unlock(ptr as usize, len);
        }
        MemLockStatus::NotLocked { errno } => {
            assert!(!status.is_locked());
            if cfg!(any(unix, windows)) {
                assert!(
                    errno.is_some(),
                    "Expected raw OS error code in NotLocked status"
                );
            }
        }
    }
}

#[test]
fn test_vault_protection_status_reporting() {
    let mut tracker = LockedRegions::new();
    assert_eq!(
        tracker.protection_status(),
        VaultProtectionStatus::default()
    );

    let buf = vec![0x42u8; 1024];
    let res = tracker.try_lock_slice(&buf, true);
    assert!(res.is_ok());

    let status = res.as_ref().copied().unwrap_or_default();
    assert_eq!(tracker.protection_status(), status);

    #[cfg(target_os = "linux")]
    if status.mlock_active {
        assert!(
            status.dontdump_applied,
            "When mlock succeeds on Linux, MADV_DONTDUMP should also be applied"
        );
    }

    tracker.unlock_all();
    assert_eq!(
        tracker.protection_status(),
        VaultProtectionStatus::default()
    );
}

#[test]
fn test_strict_mode_disabled_attempt_returns_err() {
    let mut tracker = LockedRegions::with_strict_mode(true);
    let buf = vec![1u8, 2, 3, 4];
    let res = tracker.try_lock_slice(&buf, false);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::PermissionDenied);
}

#[cfg(unix)]
#[test]
fn test_rlimit_memlock_failure_and_strict_mode() {
    let mut old_rl = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: getrlimit retrieves current RLIMIT_MEMLOCK resource limits into old_rl.
    let get_res = unsafe { libc::getrlimit(libc::RLIMIT_MEMLOCK, &mut old_rl) };
    if get_res != 0 {
        return;
    }

    // Temporarily reduce rlim_cur to 0 to simulate RLIMIT_MEMLOCK exhaustion
    let zero_rl = libc::rlimit {
        rlim_cur: 0,
        rlim_max: old_rl.rlim_max,
    };
    // SAFETY: setrlimit updates current process RLIMIT_MEMLOCK limit.
    let set_res = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &zero_rl) };

    let test_buf = vec![0xAAu8; 4096];

    if set_res == 0 {
        // 1. Direct explicit mlock call must return NotLocked with errno
        let status = mem_lock_explicit(test_buf.as_ptr(), test_buf.len());
        assert!(!status.is_locked());
        if let MemLockStatus::NotLocked { errno } = status {
            assert!(
                errno.is_some(),
                "Expected raw_os_error under RLIMIT failure"
            );
        } else {
            panic!("Expected NotLocked under zero RLIMIT_MEMLOCK");
        }

        // 2. Non-strict vault returns Ok with mlock_active = false
        let mut non_strict = LockedRegions::new();
        let non_strict_res = non_strict.try_lock_slice(&test_buf, true);
        assert!(non_strict_res.is_ok());
        let prot = non_strict_res.unwrap();
        assert!(!prot.mlock_active);
        assert!(!prot.dontdump_applied);

        // 3. Strict vault returns Err on mlock failure
        let mut strict = LockedRegions::with_strict_mode(true);
        let strict_res = strict.try_lock_slice(&test_buf, true);
        assert!(
            strict_res.is_err(),
            "Strict mode must fail-closed when mlock fails"
        );

        // SAFETY: restoring original RLIMIT_MEMLOCK limits.
        unsafe {
            libc::setrlimit(libc::RLIMIT_MEMLOCK, &old_rl);
        }
    } else {
        // Fallback check if setrlimit is not allowed in environment
        let mut strict = LockedRegions::with_strict_mode(true);
        let _ = strict.try_lock_slice(&test_buf, true);
    }
}

#[test]
fn test_apply_dontdump_safety() {
    assert!(apply_dontdump(std::ptr::null(), 0));
    let buf = vec![0u8; 512];
    let _ = apply_dontdump(buf.as_ptr(), buf.len());
}
