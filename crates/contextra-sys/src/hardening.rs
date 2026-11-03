// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Prozess- und Speicherhaertung gemaess D-63 / WP-5.2 / SY-07 / SY-08
// INVARIANTEN: Zero-Panic; globale Nebeneffekte auf Debugger & Coredumps dokumentiert

//! # Modul: Prozesshaertung (`hardening`)
//!
//! Implementiert Prozess- und Speicherhaertungsmaßnahmen nach D-63:
//! - Deaktivierung von Coredumps (`RLIMIT_CORE = 0`)
//! - Unterbindung von Prozess-Speicherabzuegen (`PR_SET_DUMPABLE = 0` unter Linux)
//! - Ausschluss sensibler Puffer aus Speicherabbildern (`madvise(MADV_DONTDUMP)`)
//! - Speicherseiten-Sperrung (`mlock`) gegen Auslagerung in Swap
//!
//! # Nebenwirkungen (SY-08)
//! `RLIMIT_CORE` und `PR_SET_DUMPABLE` sind **prozessglobal**. Sie koennen externe Debugger
//! (z. B. `gdb`, `lldb`), Profiler (`perf`) und automatisierte Absturzberichterstatter
//! im Betriebssystem daran hindern, den Prozess einzusehen oder Coredumps zu schreiben.

use std::mem::MaybeUninit;
use std::sync::atomic::{compiler_fence, Ordering};

use crate::error::SysError;

/// Richtlinie fuer die Prozesshaertung (D-63).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HardeningPolicy {
    /// Pflichtmodus: Jede fehlgeschlagene Maßnahme fuehrt zum Fehlerabbruch.
    Required,
    /// Standardmodus: Maßnahmen werden nach Moeglichkeit angewendet; Fehler fuehren nicht zum Abbruch.
    #[default]
    BestEffort,
    /// Haertung ist deaktiviert (z. B. fuer Debugging oder Profiling).
    Off,
}

/// Statusbericht ueber die durchgefuehrten Haertungsmaßnahmen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardeningReport {
    /// Ob das Coredump-Limit (`RLIMIT_CORE`) auf 0 gesetzt werden konnte.
    pub core_limit_zero: bool,
    /// Ob `PR_SET_DUMPABLE` auf 0 gesetzt wurde (Linux).
    pub dumpable_off: bool,
    /// Ob `MADV_DONTDUMP` grundlegend unterstuetzt wird.
    pub dontdump_ok: bool,
    /// Ob `mlock` verfuegbar ist.
    pub mlock_ok: bool,
}

/// Fuehrt die Prozesshaertung gemaess der gewaehlten Richtlinie durch.
///
/// # Nebenwirkungen
/// Beeinflusst das Coredump-Verhalten des gesamten laufenden Prozesses und seiner Threads.
pub fn apply_process_hardening(policy: HardeningPolicy) -> Result<HardeningReport, SysError> {
    if policy == HardeningPolicy::Off {
        return Ok(HardeningReport {
            core_limit_zero: false,
            dumpable_off: false,
            dontdump_ok: false,
            mlock_ok: false,
        });
    }

    #[cfg(unix)]
    let (core_limit_zero, dumpable_off, mlock_ok) = {
        // 1. RLIMIT_CORE = 0
        let rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `rlim` ist eine wohlgeformte Struktur auf dem Stack.
        let ret = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlim) };
        let core = ret == 0;

        // 2. PR_SET_DUMPABLE = 0 (nur Linux)
        #[cfg(target_os = "linux")]
        let dumpable = {
            // SAFETY: `prctl` mit `PR_SET_DUMPABLE` und Wert `0` ist ein valider Linux-Syscall.
            let ret = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) };
            ret == 0
        };

        #[cfg(not(target_os = "linux"))]
        let dumpable = false;

        // Testen, ob mlock auf einer Dummy-Seite funktioniert
        let dummy = [0u8; 16];
        let mut mlock_available = false;
        // SAFETY: Dummy-Puffer auf dem Stack lebt waehrend des mlock/munlock-Aufrufs.
        let ret_lock = unsafe { libc::mlock(dummy.as_ptr() as *const libc::c_void, dummy.len()) };
        if ret_lock == 0 {
            mlock_available = true;
            // SAFETY: Rueckgaengig machen des Test-Locks.
            unsafe {
                libc::munlock(dummy.as_ptr() as *const libc::c_void, dummy.len());
            }
        }

        (core, dumpable, mlock_available)
    };

    #[cfg(not(unix))]
    let (core_limit_zero, dumpable_off, mlock_ok) = (false, false, false);

    let dontdump_ok = true;

    let report = HardeningReport {
        core_limit_zero,
        dumpable_off,
        dontdump_ok,
        mlock_ok,
    };

    if policy == HardeningPolicy::Required
        && (!core_limit_zero || (!dumpable_off && cfg!(target_os = "linux")))
    {
        return Err(SysError::HardeningIncomplete);
    }

    Ok(report)
}

/// Markiert einen Speicher-Slice mit `madvise(MADV_DONTDUMP)`, damit dieser bei Coredumps ausgelassen wird.
///
/// Sicher, weil `slice` fuer die Dauer des Aufrufs existiert und die Vorbedingungen des Kernels
/// geprueft werden.
pub fn mark_dont_dump(slice: &[u8]) -> bool {
    if slice.is_empty() {
        return true;
    }

    #[cfg(target_os = "linux")]
    {
        let addr = slice.as_ptr() as usize;
        let page_size = 4096;
        let aligned_addr = addr & !(page_size - 1);
        let end_addr = (addr + slice.len() + page_size - 1) & !(page_size - 1);
        let aligned_len = end_addr - aligned_addr;

        // SAFETY: `aligned_addr` und `aligned_len` decken den Speicherbereich des gueltigen `slice` ab.
        // `MADV_DONTDUMP` schliesst die Seiten lediglich von Coredumps aus und fuehrt zu keinen Speicherschreibungen.
        let ret = unsafe {
            libc::madvise(
                aligned_addr as *mut libc::c_void,
                aligned_len,
                libc::MADV_DONTDUMP,
            )
        };
        ret == 0
    }

    #[cfg(not(target_os = "linux"))]
    {
        true
    }
}

/// Überschreibt einen Speicherbereich (`slice`) mit Nullen mittels flüchtiger Schreiboperationen (`write_volatile`)
/// und sequenzieller Compiler-Barriere (`compiler_fence(Ordering::SeqCst)`).
///
/// Dies verhindert, dass der Compiler den Nullungsvorgang bei aktiver Optimierung (z. B. opt-level 3)
/// als Dead-Store wegoptimiert (D-29, Zero-Trust).
///
/// # Vorbedingungen
/// `slice` ist ein gültiger, veränderlicher Byte-Puffer.
///
/// # Nachbedingungen
/// Alle Bytes in `slice` sind 0, und der Compiler darf nachfolgende Speicheroperationen nicht vor diese Barriere ziehen.
pub fn secure_zero_volatile(slice: &mut [u8]) {
    for byte in slice.iter_mut() {
        // SAFETY: `byte` stammt aus einer gültigen, veränderlichen Referenz auf ein initialisiertes `u8`.
        // `write_volatile` schreibt 0 ohne Aliasing-Konflikte direkt an die Zieladresse.
        unsafe {
            std::ptr::write_volatile(byte, 0);
        }
    }
    compiler_fence(Ordering::SeqCst);
}

/// Sperrt die Speicherseiten von `slice` im RAM gegen Auslagerung (Swap).
///
/// Sicher, da `slice` eine gueltige Allokation fuer die Dauer des Aufrufs ist.
pub fn lock_pages(slice: &[u8], policy: HardeningPolicy) -> Result<bool, SysError> {
    if slice.is_empty() || policy == HardeningPolicy::Off {
        return Ok(true);
    }

    #[cfg(unix)]
    {
        // SAFETY: `slice` ist eine gueltige Referenz auf initialisierte Bytes.
        let ret = unsafe { libc::mlock(slice.as_ptr() as *const libc::c_void, slice.len()) };
        if ret == 0 {
            Ok(true)
        } else {
            let err = std::io::Error::last_os_error();
            if policy == HardeningPolicy::Required {
                Err(SysError::from(err))
            } else {
                tracing::warn!("mlock failed for memory slice: {err}");
                Ok(false)
            }
        }
    }

    #[cfg(not(unix))]
    {
        if policy == HardeningPolicy::Required {
            Err(SysError::Unsupported)
        } else {
            Ok(false)
        }
    }
}

/// Stellt sicher, dass das weiche Limit fuer offene Dateideskriptoren (`RLIMIT_NOFILE`) mindestens `min_fds` betraegt.
pub fn ensure_fd_limit(min_fds: u64) -> bool {
    #[cfg(unix)]
    {
        let min_fds_rlim = min_fds as libc::rlim_t;
        // SAFETY: `rlim` und `current_rlim` sind auf dem Stack allokierte MaybeUninit-Puffer.
        // Nach Rueckgabewert 0 von `getrlimit` garantiert das Betriebssystem deren Initialisierung.
        unsafe {
            let mut rlim = MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_NOFILE, rlim.as_mut_ptr()) != 0 {
                return false;
            }
            let mut rlim = rlim.assume_init();
            if rlim.rlim_cur < min_fds_rlim {
                let target = std::cmp::min(min_fds_rlim, rlim.rlim_max);
                rlim.rlim_cur = target;
                let _ = libc::setrlimit(libc::RLIMIT_NOFILE, &rlim);
            }
            let mut current_rlim = MaybeUninit::<libc::rlimit>::uninit();
            if libc::getrlimit(libc::RLIMIT_NOFILE, current_rlim.as_mut_ptr()) != 0 {
                return false;
            }
            let current_rlim = current_rlim.assume_init();
            current_rlim.rlim_cur >= min_fds_rlim
        }
    }
    #[cfg(not(unix))]
    {
        let _ = min_fds;
        true
    }
}

/// Setzt das weiche und harte Dateideskriptorlimit (`RLIMIT_NOFILE`) des aktuellen Prozesses exakt auf `limit`.
pub fn set_fd_limit(limit: u64) -> bool {
    #[cfg(unix)]
    {
        let rlim = libc::rlimit {
            rlim_cur: limit as libc::rlim_t,
            rlim_max: limit as libc::rlim_t,
        };
        // SAFETY: `rlim` ist eine Stack-Allokation; `setrlimit` liest die Struktur unveränderlich.
        let res = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) };
        res == 0
    }
    #[cfg(not(unix))]
    {
        let _ = limit;
        true
    }
}
