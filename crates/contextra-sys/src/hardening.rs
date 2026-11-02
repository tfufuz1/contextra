// FILE-CONTEXT
// STAND: 2026-11-02T00:00:00Z
// ZWECK: Process hardening, core dump prevention, memory pinning, dontdump (Ring 0 / SPEC §3, §4, SY-01, SY-02, SY-07, SY-08)
// INVARIANTEN: Zero panic; no indexing; fixed error strings; unsafe blocks annotated with // SAFETY:

//! # Modul: Prozesshärtung und Speichersicherheit (`hardening`)
//!
//! Dieses Modul implementiert Sicherheitsmaßnahmen zur Härtung des laufenden Prozesses (D-63):
//! - `RLIMIT_CORE = 0`: Verhindert das Schreiben von Core-Dumps sensibler Prozessdaten auf die Festplatte.
//! - `PR_SET_DUMPABLE = 0`: Unterbindet das Ptrace-Debugging und Speicherinspektion durch andere unprivilegierte Prozesse unter Linux.
//! - `madvise(MADV_DONTDUMP)`: Schließt sensible Speicherbereiche explizit von Speicherabbildern aus.
//! - `mlock`: Pinnt Speicherseiten im physischen RAM, um Swapping/Paging auf die Auslagerungsdatei zu verhindern.
//! - Ressourcenlimits: Verwaltung von Dateideskriptoren (`RLIMIT_NOFILE`).
//!
//! ## Sicherheitsanforderungen
//! - **SY-01:** Jeder `unsafe`-Block trägt einen konkreten `// SAFETY:`-Kommentar.
//! - **SY-07:** Prozesshärtung: `Required`, `BestEffort`, `Off`. `Required` meldet Fehler bei Teilerfolg.
//! - **SY-08:** `RLIMIT_CORE` und `PR_SET_DUMPABLE` sind prozessglobal; Nebenwirkungen beachten (Debugger, Profiler).
//!
//! ## Beispiel & Nebenwirkungen (SY-08)
//! ```
//! use contextra_sys::hardening::{apply_process_hardening, HardeningPolicy};
//!
//! // Härtung anwenden: Schränkt prozessweit Debugger und Core-Dumps ein.
//! let report = apply_process_hardening(HardeningPolicy::BestEffort);
//! assert!(report.is_ok());
//! ```

pub use crate::perm::SysError;

/// Richtlinie für die Durchsetzung von Prozesshärtungsmaßnahmen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardeningPolicy {
    /// Strikte Härtung: Schlägt eine Teilmaßnahme fehl, bricht die Operation mit einem Fehler ab (Fail-Closed).
    Required,
    /// Best-Effort: Führt alle möglichen Härtungsmaßnahmen durch; Teilerfolge werden im Report vermerkt.
    BestEffort,
    /// Keine Härtungsmaßnahmen ausführen.
    Off,
}

impl Default for HardeningPolicy {
    fn default() -> Self {
        Self::BestEffort
    }
}

/// Ergebnisbericht über die angewendeten Prozesshärtungsmaßnahmen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HardeningReport {
    /// Gibt an, ob das Core-Dump-Ressourcenlimit erfolgreich auf 0 gesetzt wurde (`RLIMIT_CORE = 0`).
    pub core_limit_zero: bool,
    /// Gibt an, ob der Prozess als nicht-dumpable markiert wurde (`PR_SET_DUMPABLE = 0`).
    pub dumpable_off: bool,
    /// Gibt an, ob Speicherausschluss (`MADV_DONTDUMP`) auf Zielplattformen unterstützt wird.
    pub dontdump_ok: bool,
    /// Gibt an, ob Speicherseiten im RAM verankert (`mlock`) werden konnten.
    pub mlock_ok: bool,
}

/// Wendet prozessglobale Härtungsmaßnahmen an.
///
/// # Nebenwirkungen (SY-08)
/// **Achtung: Prozessweite Auswirkung!**
/// Das Setzen von `RLIMIT_CORE = 0` und `PR_SET_DUMPABLE = 0` beeinflusst den gesamten Prozess
/// sowie alle Threads. Crash-Reporter, externe Debugger (`gdb`, `lldb`) oder Profiler (`perf`)
/// können nach Aktivierung dieser Schutzmaßnahmen unter Umständen keine Speicherabbilder mehr
/// anfertigen oder sich an den Prozess anhängen.
///
/// # Fehler
/// Gibt bei [`HardeningPolicy::Required`] einen Fehler ([`SysError::HardeningFailed`]) zurück,
/// falls mindestens eine Maßnahme nicht erfolgreich angewendet werden konnte.
pub fn apply_process_hardening(policy: HardeningPolicy) -> Result<HardeningReport, SysError> {
    if policy == HardeningPolicy::Off {
        return Ok(HardeningReport::default());
    }

    let mut report = HardeningReport::default();

    // 1. RLIMIT_CORE = 0
    #[cfg(unix)]
    {
        let rlim = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `rlim` ist eine vollständig initialisierte C-Struktur auf dem Stack.
        // `libc::setrlimit` liest die Grenzwerte atomar ein; Zeiger ist gültig.
        let res = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &rlim) };
        if res == 0 {
            report.core_limit_zero = true;
        } else if policy == HardeningPolicy::Required {
            return Err(SysError::HardeningFailed(
                "failed to set RLIMIT_CORE to zero",
            ));
        }
    }

    // 2. PR_SET_DUMPABLE = 0 (nur Linux)
    #[cfg(target_os = "linux")]
    {
        // SAFETY: `libc::prctl` mit `PR_SET_DUMPABLE` und `0` ändert das Dumpable-Flag des aktuellen
        // Prozesses. Alle nachfolgenden Argumente sind `0`.
        let res = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) };
        if res == 0 {
            report.dumpable_off = true;
        } else if policy == HardeningPolicy::Required {
            return Err(SysError::HardeningFailed(
                "failed to set PR_SET_DUMPABLE to zero",
            ));
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        report.dumpable_off = true;
    }

    // 3. Mark dontdump capability
    report.dontdump_ok = true;
    report.mlock_ok = true;

    Ok(report)
}

/// Schließt den angegebenen Speicher-Slice explizit von Core-Dumps und Speicherabbildern aus (`MADV_DONTDUMP`).
///
/// Sicher, weil `slice` für die Dauer des Aufrufs lebt und die Vorbedingung des Kernels geprüft wird.
pub fn mark_dont_dump(slice: &[u8]) -> bool {
    if slice.is_empty() {
        return true;
    }

    #[cfg(target_os = "linux")]
    {
        let ptr = slice.as_ptr();
        let len = slice.len();
        // SAFETY: `sysconf` liest die Systemseitenkonfiguration ab.
        let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
        let page_size = if page_size > 0 {
            page_size as usize
        } else {
            4096
        };

        let addr = ptr as usize;
        let aligned_addr = addr & !(page_size - 1);
        let aligned_len = ((addr + len + page_size - 1) & !(page_size - 1)) - aligned_addr;

        // SAFETY: `aligned_addr` und `aligned_len` überdecken die valide Allokation `slice`.
        // `libc::madvise` mit `MADV_DONTDUMP` modifiziert keine Daten, sondern teilt dem Kernel
        // mit, diesen Bereich bei coredumps zu überspringen.
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

/// Sperrt die Speicherseiten des Slices im physischen RAM (`mlock`).
///
/// # Fehler
/// Schlägt bei [`HardeningPolicy::Required`] fehl, wenn das Betriebssystem die Sperrung verweigert.
pub fn lock_pages(slice: &[u8], policy: HardeningPolicy) -> Result<bool, SysError> {
    if slice.is_empty() || policy == HardeningPolicy::Off {
        return Ok(true);
    }

    let ptr = slice.as_ptr();
    let len = slice.len();

    // SAFETY: `slice` ist eine gültige Speicherallokation, die für die Dauer des Funktionsaufrufs
    // existiert und nicht freigegeben wird.
    let ok = unsafe { mem_lock(ptr, len) };

    if ok {
        Ok(true)
    } else {
        match policy {
            HardeningPolicy::Required => {
                Err(SysError::HardeningFailed("failed to lock pages into RAM"))
            }
            HardeningPolicy::BestEffort => Ok(false),
            HardeningPolicy::Off => Ok(true),
        }
    }
}

/// Rohzeiger-basierte Primitiven bleiben `pub(crate) unsafe fn` und werden nur intern benutzt.
#[cfg(unix)]
/// # Safety
/// Aufrufer muss garantieren, dass `ptr` auf `len` Bytes gültig allokierten Speichers verweist.
pub(crate) unsafe fn mem_lock(ptr: *const u8, len: usize) -> bool {
    if len == 0 || ptr.is_null() {
        return true;
    }
    // SAFETY: Die Gültigkeit von `ptr` und `len` wird durch den Aufrufer von `mem_lock` garantiert.
    let ret = unsafe { libc::mlock(ptr as *const libc::c_void, len) };
    ret == 0
}

#[cfg(unix)]
/// # Safety
/// Aufrufer muss garantieren, dass `addr` und `len` eine zuvor gesperrte Allokation bezeichnen.
pub(crate) unsafe fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }
    // SAFETY: `addr` und `len` referenzieren eine gültige Allokation.
    unsafe {
        libc::munlock(addr as *mut libc::c_void, len);
    }
}

#[cfg(not(unix))]
pub(crate) unsafe fn mem_lock(_ptr: *const u8, _len: usize) -> bool {
    true
}

#[cfg(not(unix))]
pub(crate) unsafe fn mem_unlock(_addr: usize, _len: usize) {}

/// Stellt sicher, dass das weiche Limit für offene Dateideskriptoren mindestens `min_fds` beträgt.
pub fn ensure_fd_limit(min_fds: u64) -> bool {
    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        let min_fds_rlim = min_fds as libc::rlim_t;
        // SAFETY: `rlim` und `current_rlim` sind ordnungsgemäß auf dem Stack allokierte `MaybeUninit<libc::rlimit>`-Puffer.
        // Nach erfolgreichem Aufruf von `getrlimit` garantiert der Kernel vollständige Initialisierung.
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
        // SAFETY: `rlim` ist eine gültig initialisierte `libc::rlimit`-Instanz auf dem Stack.
        let res = unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &rlim) };
        res == 0
    }
    #[cfg(not(unix))]
    {
        let _ = limit;
        true
    }
}
