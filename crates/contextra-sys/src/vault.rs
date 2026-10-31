// FILE-CONTEXT
// STAND: 2026-10-28T00:00:00Z
// ZWECK: RAII manager for RAM memory-locked sensitive buffer regions and memory zeroization (Ring 0 / SPEC 7.7)
// INVARIANTEN: Automatically releases locked memory regions via mem_unlock on Drop or unlock_all
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7

//! # Modul: Tresor-Speicherverwaltung (`vault`)
//!
//! Dieses Modul stellt RAII-basierte Schutzstrukturen (Resource Acquisition Is Initialization)
//! für sensible kryptografische Puffer im Arbeitsspeicher bereit.
//!
//! ## Ring-Zugehörigkeit und Sicherheitsarchitektur
//! - **Ring 0 (Kern / Unsafe-Insel):** Baut auf [`crate::mlock`] auf und kapselt flüchtiges
//!   Überschreiben (Volatile Memory Zeroization) via `write_volatile`.
//! - **Verwendung im System:** Wird von Kryptografie- und Schlüsselverwaltungsmodulen
//!   (z. B. `contextra-crypto` oder Schlüsselladern in `contextra-store`) eingesetzt.
//! - **Zwei fundamentale Sicherheitsgarantien:**
//!   1. **Kein Swapping:** Speicherbereiche werden im physischen RAM verankert ([`LockedRegions`]).
//!   2. **Zuverlässiges Löschen (Zeroize):** Sensible Datenreste werden vor der Speicherfreigabe
//!      mittels flüchtiger Schreiboperationen (`write_volatile`) mit Nullen überschrieben,
//!      sodass der Compiler das Nullsetzen nicht als "tote Schreiboperation" wegoptimiert (Dead-Store Elimination).

use crate::{mem_lock_with_policy, mem_unlock, MemLockPolicy};

// ANCHOR[contextra-sys:VAULT-LOCKED-REGIONS-001]
/// Kapselt und verfolgt im RAM gesperrte Speicherbereiche (Adressen und Längen) für sensible Schlüsseldaten.
///
/// Beim Verlassen des Gültigkeitsbereichs ([`Drop::drop`]) oder beim expliziten Aufruf von
/// [`unlock_all`](Self::unlock_all) werden alle nachverfolgten Speicherregionen automatisch
/// über das Betriebssystem entsperrt.
#[derive(Debug, Default)]
pub struct LockedRegions {
    regions: Vec<(usize, usize)>,
}

impl LockedRegions {
    // ANCHOR[contextra-sys:VAULT-NEW-001]
    /// Erstellt eine neue, leere Nachverfolgungsinstanz für gesperrte Speicherregionen.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    // ANCHOR[contextra-sys:VAULT-LOCK-SLICE-001]
    /// Versucht einen Byte-Slice im RAM zu sperren, falls `attempt_mlock` wahr ist, unter Verwendung der Standardrichtlinie [`MemLockPolicy::Tolerant`].
    ///
    /// Liefert `true`, wenn die Sperrung aktiv ist, andernfalls `false`.
    pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool {
        self.lock_slice_with_policy(slice, attempt_mlock, MemLockPolicy::Tolerant)
            .unwrap_or(false)
    }

    // ANCHOR[contextra-sys:VAULT-LOCK-SLICE-POLICY-001]
    /// Versucht einen Byte-Slice im RAM gemäß der angegebenen [`MemLockPolicy`] zu sperren, falls `attempt_mlock == true`.
    ///
    /// # Ablauf
    /// 1. Ist `slice.is_empty()`, wird sofort `Ok(true)` zurückgegeben.
    /// 2. Ist `attempt_mlock == false`, wird kein Systemsperraufruf ausgeführt und `Ok(false)` zurückgegeben.
    /// 3. Ermittelt Basisadresse (`ptr`) und Länge (`len`) des Slices.
    /// 4. Ruft [`mem_lock_with_policy`] auf.
    /// 5. Konnte die Region erfolgreich gesperrt werden, wird das Tupel `(ptr as usize, len)` in der internen Liste registriert.
    ///
    /// # Fehlerfälle
    /// Gibt einen [`std::io::Error`] zurück, wenn das Sperren scheitert und `policy` auf [`MemLockPolicy::FailClosed`] steht.
    pub fn lock_slice_with_policy(
        &mut self,
        slice: &[u8],
        attempt_mlock: bool,
        policy: MemLockPolicy,
    ) -> std::io::Result<bool> {
        if slice.is_empty() {
            return Ok(true);
        }
        if !attempt_mlock {
            return Ok(false);
        }

        let ptr = slice.as_ptr();
        let len = slice.len();

        let locked = mem_lock_with_policy(ptr, len, policy)?;
        if locked {
            self.regions.push((ptr as usize, len));
        }
        Ok(locked)
    }

    // ANCHOR[contextra-sys:VAULT-ZEROIZE-001]
    /// Überschreibt einen veränderbaren Byte-Slice sicher mit Nullen unter Verwendung von flüchtigen Schreibzugriffen (`write_volatile`).
    ///
    /// # Warum volatile Writes notwendig sind
    /// Herkömmliche Zuweisungen wie `byte = 0` werden von optimierenden Compilern (LLVM) oft vollständig
    /// eliminiert, wenn der Puffer kurz darauf freigegeben oder nicht mehr gelesen wird (Dead-Store Elimination).
    /// `write_volatile` zwingt den Compiler und Prozessor dazu, den Schreibzugriff tatsächlich im physischen
    /// Speicher auszuführen.
    ///
    /// # Ablauf
    /// Iteriert über jedes Byte des Slices und schreibt atomar/flüchtig eine Null hinein.
    pub fn zeroize_slice(slice: &mut [u8]) {
        for byte in slice.iter_mut() {
            // SAFETY: `byte` ist eine gültige, ausgerichtete mutable Referenz auf ein `u8` im übergebenen Slice. Volatile Schreibzugriffe verhindern das Wegoptimieren durch den Compiler.
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }

    // ANCHOR[contextra-sys:VAULT-UNLOCK-ALL-001]
    /// Entsperrt alle registrierten Speicherregionen und leert die interne Nachverfolgungsliste.
    pub fn unlock_all(&mut self) {
        for (addr, len) in self.regions.drain(..) {
            mem_unlock(addr, len);
        }
    }

    // ANCHOR[contextra-sys:VAULT-LEN-001]
    /// Gibt die Anzahl der aktuell nachverfolgten, gesperrten Regionen zurück.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    // ANCHOR[contextra-sys:VAULT-IS-EMPTY-001]
    /// Gibt `true` zurück, wenn aktuell keine gesperrten Speicherregionen verwaltet werden.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }
}

// ANCHOR[contextra-sys:VAULT-DROP-001]
// Beim Zerstören der Struktur werden alle noch gehaltenen Regionen automatisch entsperrt.
impl Drop for LockedRegions {
    fn drop(&mut self) {
        self.unlock_all();
    }
}

// ANCHOR[contextra-sys:VAULT-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-sys:VAULT-TEST-LIFECYCLE-001]
    // Prüft den gesamten Lebenszyklus von `LockedRegions`: Initialisierung, Sperren, Zählen und manuelles Entsperren.
    #[test]
    fn test_locked_regions_lifecycle() {
        let mut tracker = LockedRegions::new();
        assert!(tracker.is_empty());

        let data = vec![1u8, 2, 3, 4, 5];
        let locked = tracker.lock_slice(&data, true);

        if locked {
            assert_eq!(tracker.len(), 1);
        } else {
            assert_eq!(tracker.len(), 0);
        }

        tracker.unlock_all();
        assert!(tracker.is_empty());
    }

    // ANCHOR[contextra-sys:VAULT-TEST-DISABLED-001]
    // Prüft, dass bei `attempt_mlock = false` kein Sperrversuch stattfindet und die Liste leer bleibt.
    #[test]
    fn test_locked_regions_disabled() {
        let mut tracker = LockedRegions::new();
        let data = vec![1u8, 2, 3, 4, 5];
        let locked = tracker.lock_slice(&data, false);
        assert!(!locked);
        assert!(tracker.is_empty());
    }

    // ANCHOR[contextra-sys:VAULT-TEST-POLICY-001]
    // Prüft das Sperren unter Verwendung der strikten `FailClosed`-Richtlinie.
    #[test]
    fn test_locked_regions_with_fail_closed_policy() {
        let mut tracker = LockedRegions::new();
        let data = vec![1u8, 2, 3, 4, 5];
        let res = tracker.lock_slice_with_policy(&data, true, MemLockPolicy::FailClosed);
        if let Ok(locked) = res {
            if locked {
                assert_eq!(tracker.len(), 1);
            }
        }
    }

    // ANCHOR[contextra-sys:VAULT-TEST-ZEROIZE-001]
    // Prüft, dass `zeroize_slice` alle Bytes eines Puffers zuverlässig auf 0 überschreibt.
    #[test]
    fn test_zeroize_slice() {
        let mut secret = vec![0x42u8; 16];
        LockedRegions::zeroize_slice(&mut secret);
        assert_eq!(secret, vec![0u8; 16]);
    }
}
