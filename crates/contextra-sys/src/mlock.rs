// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: Platform-aware RAM memory locking abstractions with policy enforcement (Ring 0 / SPEC 7.7)
// INVARIANTEN: Zero panic; configurable mlock policy (FailClosed vs Tolerant); typed io::Error propagation
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! # Modul: Speichersperrung im Arbeitsspeicher (`mlock`)
//!
//! Dieses Modul kapselt das Festpinnen (Locking) von virtuellem Speicher im physischen RAM
//! (Random Access Memory). Dadurch wird das Betriebssystem angewiesen, die betroffenen Speicherseiten
//! nicht in die Auslagerungsdatei (Swap / Pagefile) auf den persistenten Datenträger auszulagern.
//!
//! ## Ring-Zugehörigkeit und Sicherheitsrelevanz
//! - **Ring 0 (Kern / Unsafe-Insel):** Direkte Aufrufe von POSIX `libc::mlock` / `libc::munlock`
//!   bzw. Windows `VirtualLock` / `VirtualUnlock`.
//! - **Kryptografischer Schutz:** Wenn kryptografische Schlüssel (z. B. für die Verschlüsselung
//!   im Ruhezustand = Encryption-at-Rest) in den RAM geladen werden, bestünde bei einem Swap
//!   das Risiko, dass Klartext-Schlüssel auf unverschlüsselten Swap-Partitionen landen.
//! - **Konfigurierbare Fehlerrichtlinie ([`MemLockPolicy`]):**
//!   - `FailClosed`: Scheitert `mlock` (z. B. wegen `RLIMIT_MEMLOCK`), wird die Operation strikt abgebrochen.
//!   - `Tolerant`: Scheitert `mlock` (z. B. in unprivilegierten Docker-Containern), wird eine Warnung protokolliert
//!     und der Speicher ungesperrt belassen.

use std::io::{Error, Result};

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

// ANCHOR[contextra-sys:MLOCK-LOCK-UNIX-001]
/// Sperrt unter Unix eine Speicherregion im RAM, um das Auslagern (Swapping) zu verhindern.
///
/// # Ablauf
/// 1. Prüft, ob `len == 0` oder `ptr.is_null()` ist: In diesem Fall ist nichts zu tun (`Ok(true)`).
/// 2. Ruft den POSIX-Systemaufruf `libc::mlock` mit dem Speicherzeiger und der Länge auf.
/// 3. War der Systemaufruf erfolgreich (`ret == 0`), wird `Ok(true)` zurückgegeben.
/// 4. Scheitert der Systemaufruf (`ret != 0`):
///    - Unter [`MemLockPolicy::FailClosed`] wird ein typisierter [`std::io::Error`] erzeugt und zurückgegeben.
///    - Unter [`MemLockPolicy::Tolerant`] wird eine Warnung protokolliert und `Ok(false)` zurückgegeben.
///
/// # Vorbedingungen
/// - `ptr` muss auf einen gültigen, allokierten Speicherbereich von mindestens `len` Bytes zeigen.
///
/// # Fehlerfälle
/// - Gibt bei unzureichenden Rechten (`EPERM`) oder überschrittenem Limit (`ENOMEM`) einen Fehler zurück,
///   sofern `policy` auf [`MemLockPolicy::FailClosed`] steht.
#[cfg(unix)]
pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> {
    if len == 0 || ptr.is_null() {
        return Ok(true);
    }
    // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes gültig allokierten Speicher. `libc::mlock` pinnt Seiten im RAM.
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

// ANCHOR[contextra-sys:MLOCK-UNLOCK-UNIX-001]
/// Entsperrt eine zuvor unter Unix gesperrte Speicherregion im RAM.
///
/// # Vorbedingungen
/// - `addr` und `len` müssen exakt einer zuvor über [`mem_lock_with_policy`] gesperrten Region entsprechen.
#[cfg(unix)]
pub fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }
    // SAFETY: `addr` und `len` referenzieren eine gültige Speicherallokation, die zuvor an `mem_lock` übergeben wurde.
    unsafe {
        libc::munlock(addr as *mut libc::c_void, len);
    }
}

// ANCHOR[contextra-sys:MLOCK-LOCK-WIN32-001]
/// Sperrt unter Windows eine Speicherregion im RAM gemäß der übergebenen [`MemLockPolicy`].
///
/// # Ablauf
/// 1. Behandelt Nullzeiger oder Nulllänge sofort mit `Ok(true)`.
/// 2. Ruft die Win32-API-Funktion `VirtualLock` auf.
/// 3. Gibt bei Erfolg `Ok(true)` zurück.
/// 4. Scheitert `VirtualLock`:
///    - Bei [`MemLockPolicy::FailClosed`] Rückgabe eines Fehlers.
///    - Bei [`MemLockPolicy::Tolerant`] Warnung via `tracing` und Rückgabe von `Ok(false)`.
#[cfg(windows)]
pub fn mem_lock_with_policy(ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> {
    if len == 0 || ptr.is_null() {
        return Ok(true);
    }
    // SAFETY: `ptr` ist nicht null und verweist auf `len` Bytes gültig allokierten Speicher. `VirtualLock` pinnt Seiten im physischen RAM.
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

// ANCHOR[contextra-sys:MLOCK-UNLOCK-WIN32-001]
/// Entsperrt eine zuvor unter Windows gesperrte Speicherregion im RAM über `VirtualUnlock`.
#[cfg(windows)]
pub fn mem_unlock(addr: usize, len: usize) {
    if len == 0 || addr == 0 {
        return;
    }
    // SAFETY: `addr` und `len` referenzieren eine gültige Speicherallokation, die zuvor an `VirtualLock` übergeben wurde.
    unsafe {
        windows_sys::Win32::System::Memory::VirtualUnlock(addr as *const std::ffi::c_void, len);
    }
}

// ANCHOR[contextra-sys:MLOCK-FALLBACK-001]
/// Fallback-Implementierung für Zielarchitekturen, die weder Unix noch Windows unterstützen.
///
/// # Fehlerfälle
/// Liefert bei [`MemLockPolicy::FailClosed`] und `len > 0` stets einen `Unsupported`-Fehler.
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

// ANCHOR[contextra-sys:MLOCK-UNLOCK-FALLBACK-001]
/// Fallback No-Op zum Entsperren von Speicher auf nicht unterstützten Plattformen.
#[cfg(not(any(unix, windows)))]
pub fn mem_unlock(_addr: usize, _len: usize) {}

// ANCHOR[contextra-sys:MLOCK-CONVENIENCE-001]
/// Sperrt eine Speicherregion im RAM mit der Standard-Richtlinie [`MemLockPolicy::Tolerant`].
///
/// Gibt `true` zurück, falls das Sperren erfolgreich war, oder `false`, falls das Betriebssystem
/// das Sperren verweigert hat (z. B. wegen Ressourcenbeschränkungen).
pub fn mem_lock(ptr: *const u8, len: usize) -> bool {
    mem_lock_with_policy(ptr, len, MemLockPolicy::Tolerant).unwrap_or(false)
}

// ANCHOR[contextra-sys:MLOCK-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-sys:MLOCK-TEST-NULL-001]
    // Prüft, dass Nullzeiger und Nulllänge von allen Policies ohne Fehler und ohne Systemaufruf akzeptiert werden.
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

    // ANCHOR[contextra-sys:MLOCK-TEST-TOLERANT-001]
    // Prüft das Sperren eines gültigen Puffers im toleranten Modus und anschließendes Entsperren.
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

    // ANCHOR[contextra-sys:MLOCK-TEST-FAILCLOSED-OK-001]
    // Prüft das Verhalten im FailClosed-Modus für einen regulären Speicherpuffer.
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

    // ANCHOR[contextra-sys:MLOCK-TEST-SIM-FAILCLOSED-001]
    // Simuliert einen Fehler durch Übergabe einer ungemappten Adresse im FailClosed-Modus und erwartet Err.
    #[test]
    fn test_mem_lock_invalid_address_fail_closed_simulation() {
        // Pass a dangling non-null pointer with unmapped address to simulate mlock failure.
        let invalid_ptr = (usize::MAX - 4096) as *const u8;
        let res = mem_lock_with_policy(invalid_ptr, 4096, MemLockPolicy::FailClosed);
        assert!(res.is_err());
    }

    // ANCHOR[contextra-sys:MLOCK-TEST-SIM-TOLERANT-001]
    // Simuliert einen Fehler durch ungültige Adresse im Tolerant-Modus und prüft, dass false ohne Err zurückkehrt.
    #[test]
    fn test_mem_lock_invalid_address_tolerant_simulation() -> Result<()> {
        let invalid_ptr = (usize::MAX - 4096) as *const u8;
        let res = mem_lock_with_policy(invalid_ptr, 4096, MemLockPolicy::Tolerant)?;
        assert!(!res);
        Ok(())
    }
}
