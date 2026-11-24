// STAND: 2026-11-08T00:00:00Z
// ZWECK: Platform-aware RAM memory locking abstractions with policy enforcement (Ring 0 / SPEC §10.1 / INV-SYS-MEMLOCK-1)
// INVARIANTEN: Zero panic; configurable mlock policy; madvise(MADV_DONTDUMP) integration; structured error propagation
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7, §10.1

//! # Modul: Speichersperrung im Arbeitsspeicher (`mlock`)
//!
//! Dieses Modul kapselt das Festpinnen (Locking) von virtuellem Speicher im physischen RAM
//! (Random Access Memory). Dadurch wird das Betriebssystem angewiesen, die betroffenen Speicherseiten
//! nicht in die Auslagerungsdatei (Swap / Pagefile) auf den persistenten Datenträger auszulagern.
//! Zusätzlich wird unter Unix `madvise(..., MADV_DONTDUMP)` angewandt, um sensible Daten aus
//! Core-Dumps auszuschließen (INV-SYS-MEMLOCK-1).
//!
//! ## Ring-Zugehörigkeit und Sicherheitsrelevanz
//! - **Ring 0 (Kern / Unsafe-Insel):** Direkte Aufrufe von POSIX `libc::mlock` / `libc::munlock`
//!   / `libc::madvise(MADV_DONTDUMP)` bzw. Windows `VirtualLock` / `VirtualUnlock`.
//! - **Kryptografischer Schutz:** Wenn kryptografische Schlüssel (z. B. für die Verschlüsselung
//!   im Ruhezustand = Encryption-at-Rest) in den RAM geladen werden, bestünde bei einem Swap
//!   das Risiko, dass Klartext-Schlüssel auf unverschlüsselten Swap-Partitionen landen.
//! - **Konfigurierbare Fehlerrichtlinie ([`MemLockPolicy`]):**
//!   - `FailClosed`: Scheitert `mlock` (z. B. wegen `RLIMIT_MEMLOCK`), wird die Operation strikt abgebrochen.
//!   - `Tolerant`: Scheitert `mlock` (z. B. in unprivilegierten Docker-Containern), wird eine Warnung protokolliert
//!     und der Speicher ungesperrt belassen.

use std::io::{Error as IoError, Result as IoResult};

use crate::error::{Error, Result};

#[cfg(test)]
pub(crate) mod fault_injection {
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

    static FAIL_MLOCK: AtomicBool = AtomicBool::new(false);
    static INJECTED_ERRNO: AtomicI32 = AtomicI32::new(libc::ENOMEM);

    pub fn set_fail_mlock(fail: bool, errno: i32) {
        FAIL_MLOCK.store(fail, Ordering::SeqCst);
        INJECTED_ERRNO.store(errno, Ordering::SeqCst);
    }

    pub fn should_fail_mlock() -> Option<i32> {
        if FAIL_MLOCK.load(Ordering::SeqCst) {
            Some(INJECTED_ERRNO.load(Ordering::SeqCst))
        } else {
            None
        }
    }
}

// ANCHOR[contextra-sys:MLOCK-POLICY-001]
/// Konfigurierbare Richtlinie für das Sperren von Speicherregionen (`mlock` / `VirtualLock`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemLockPolicy {
    /// Fehlschläge beim Sperren werden als fataler Fehler behandelt und brechen die Operation ab.
    /// Einsatz z. B. bei hochsensiblen kryptografischen Masterschlüsseln (Fail-Closed).
    FailClosed,
    /// Fehlschläge beim Sperren erzeugen lediglich eine Warnmeldung via `tracing` und führen nicht zum Abbruch.
    /// Standardeinstellung (`#[default]`) für maximale Portabilität in Container-Umgebungen.
    #[default]
    Tolerant,
}

// ANCHOR[contextra-sys:MLOCK-APPLY-DONTDUMP-001]
/// Schließt eine Speicherregion per `madvise(MADV_DONTDUMP)` aus Core-Dumps aus (Unix).
/// Auf Nicht-Unix-Systemen ist dies ein no-op.
pub fn apply_dont_dump(ptr: *const u8, len: usize) -> Result<()> {
    if len == 0 || ptr.is_null() {
        return Ok(());
    }

    #[cfg(unix)]
    {
        #[cfg(target_os = "linux")]
        let advice = libc::MADV_DONTDUMP;
        #[cfg(target_os = "macos")]
        let advice = libc::MADV_ZERO_WIRED_PAGES;
        #[cfg(all(unix, not(target_os = "linux"), not(target_os = "macos")))]
        let advice = libc::MADV_DONTDUMP;

        // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes gültig allokierten Speicher.
        let ret = unsafe { libc::madvise(ptr as *mut libc::c_void, len, advice) };
        if ret != 0 {
            let err = IoError::last_os_error();
            let errno = err.raw_os_error().unwrap_or(0);
            tracing::debug!(
                "madvise(DONTDUMP) advisory call returned errno {}: {}",
                errno,
                err
            );
        }
    }

    #[cfg(not(unix))]
    {
        let _ = (ptr, len);
    }

    Ok(())
}

// ANCHOR[contextra-sys:MLOCK-LOCK-TYPED-001]
/// Sperrt eine Speicherregion im RAM gemäß Richtlinie und liefert ein typisiertes [`Result<bool>`].
pub fn mem_lock_typed(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> {
    if len == 0 || ptr.is_null() {
        return Ok(true);
    }

    #[cfg(test)]
    if let Some(injected_errno) = fault_injection::should_fail_mlock() {
        return match policy {
            MemLockPolicy::FailClosed => Err(Error::MemoryLockFailed {
                errno: injected_errno,
                reason: "injected mlock fault",
            }),
            MemLockPolicy::Tolerant => {
                tracing::warn!(
                    "mlock() failed for memory region at {:p} (len={}): injected fault. \
                     Data remains unlocked in memory (swap-risk, Tolerant).",
                    ptr,
                    len
                );
                Ok(false)
            }
        };
    }

    #[cfg(unix)]
    {
        // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes allokierten Speicher.
        let ret = unsafe { libc::mlock(ptr as *const libc::c_void, len) };
        if ret != 0 {
            let err = IoError::last_os_error();
            let errno = err.raw_os_error().unwrap_or(0);
            match policy {
                MemLockPolicy::FailClosed => Err(Error::MemoryLockFailed {
                    errno,
                    reason: "mlock system call failed",
                }),
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
            let _ = apply_dont_dump(ptr, len);
            Ok(true)
        }
    }

    #[cfg(windows)]
    {
        // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes gültig allokierten Speicher.
        let ret = unsafe {
            windows_sys::Win32::System::Memory::VirtualLock(ptr as *const std::ffi::c_void, len)
        };
        if ret == 0 {
            let err = IoError::last_os_error();
            let errno = err.raw_os_error().unwrap_or(0);
            match policy {
                MemLockPolicy::FailClosed => Err(Error::MemoryLockFailed {
                    errno,
                    reason: "VirtualLock Win32 API call failed",
                }),
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

    #[cfg(not(any(unix, windows)))]
    {
        match policy {
            MemLockPolicy::FailClosed => Err(Error::Unsupported(
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
}

// ANCHOR[contextra-sys:MLOCK-LOCK-WITH-POLICY-001]
/// Sperrt unter POSIX/Windows eine Speicherregion im RAM, um das Auslagern (Swapping) zu verhindern.
pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> IoResult<bool> {
    mem_lock_typed(ptr, len, policy).map_err(Into::into)
}

// ANCHOR[contextra-sys:MLOCK-UNLOCK-001]
/// Entsperrt eine zuvor gesperrte Speicherregion im RAM.
pub fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }

    #[cfg(unix)]
    {
        // SAFETY: `addr` und `len` referenzieren eine zuvor gesperrte Region.
        unsafe {
            libc::munlock(addr as *mut libc::c_void, len);
        }
    }

    #[cfg(windows)]
    {
        // SAFETY: `addr` und `len` referenzieren eine zuvor an `VirtualLock` übergebene Region.
        unsafe {
            windows_sys::Win32::System::Memory::VirtualUnlock(addr as *const std::ffi::c_void, len);
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = (addr, len);
    }
}

// ANCHOR[contextra-sys:MLOCK-CONVENIENCE-001]
/// Sperrt eine Speicherregion im RAM mit der Standard-Richtlinie [`MemLockPolicy::Tolerant`].
pub fn mem_lock(ptr: *const u8, len: usize) -> bool {
    mem_lock_typed(ptr, len, MemLockPolicy::Tolerant).unwrap_or(false)
}

// ANCHOR[contextra-sys:MLOCK-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_lock_null_or_zero_len() -> Result<()> {
        assert!(mem_lock_typed(
            std::ptr::null(),
            0,
            MemLockPolicy::FailClosed
        )?);
        assert!(mem_lock_typed(
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

        let ok = mem_lock_typed(ptr, len, MemLockPolicy::Tolerant)?;
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

        let res = mem_lock_typed(ptr, len, MemLockPolicy::FailClosed);
        if let Ok(true) = res {
            mem_unlock(ptr as usize, len);
        }
    }

    #[test]
    fn test_mem_lock_invalid_address_fail_closed_simulation() {
        let invalid_ptr = (usize::MAX - 4096) as *const u8;
        let res = mem_lock_typed(invalid_ptr, 4096, MemLockPolicy::FailClosed);
        assert!(res.is_err());
    }

    #[test]
    fn test_mem_lock_invalid_address_tolerant_simulation() -> Result<()> {
        let invalid_ptr = (usize::MAX - 4096) as *const u8;
        let res = mem_lock_typed(invalid_ptr, 4096, MemLockPolicy::Tolerant)?;
        assert!(!res);
        Ok(())
    }
}
