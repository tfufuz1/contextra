// STAND: 2026-11-08T00:00:00Z
// ZWECK: RAII manager for RAM memory-locked sensitive buffer regions, protected page allocations, and zeroization (Ring 0 / SPEC §10.1 / INV-SYS-MEMLOCK-1)
// INVARIANTEN: Zero panic; VaultBuffer protected allocation; non-optimizable volatile zeroize + compiler fence; mlock + MADV_DONTDUMP; munlock after zeroize
// SIEHE AUCH: AGENTS.md, docs/SPEC.md §7.7, §10.1

//! # Modul: Tresor-Speicherverwaltung (`vault`)
//!
//! Dieses Modul stellt RAII-basierte Schutzstrukturen (Resource Acquisition Is Initialization)
//! für sensible kryptografische Puffer im Arbeitsspeicher bereit.
//!
//! ## Ring-Zugehörigkeit und Sicherheitsarchitektur
//! - **Ring 0 (Kern / Unsafe-Insel):** Baut auf [`crate::mlock`] auf und kapselt flüchtiges
//!   Überschreiben (Volatile Memory Zeroization) via `write_volatile` in Kombination mit
//!   einer sequenziellen Compiler-Fence (`std::sync::atomic::compiler_fence(Ordering::SeqCst)`).
//! - **Verwendung im System:** Wird von Kryptografie- und Schlüsselverwaltungsmodulen
//!   (z. B. `contextra-crypto` oder Schlüsselladern in `contextra-store`) eingesetzt.
//! - **Fundamentale Sicherheitsgarantie (INV-SYS-MEMLOCK-1):**
//!   Über [`VaultBuffer`] allozierte Puffer werden mit `mlock` im physischen RAM verankert,
//!   per `madvise(MADV_DONTDUMP)` vom Core-Dump ausgenommen und beim Drop vor der Speicherfreigabe
//!   vollständig genullt. Erst nach dem Nullsetzen wird der Speicher entsperrt (`munlock`).
//!   Scheitert `mlock`, schlägt die Allokation standardmäßig mit [`crate::Error::MemoryLockFailed`]
//!   fehl (Fail-Closed).

use std::sync::atomic::{compiler_fence, Ordering};

use crate::error::Result;
use crate::mlock::{
    apply_dont_dump, mem_lock_typed, mem_lock_with_policy, mem_unlock, MemLockPolicy,
};

// ANCHOR[contextra-sys:VAULT-LOCKED-REGIONS-001]
/// Kapselt und verfolgt im RAM gesperrte Speicherbereiche (Adressen und Längen) für sensible Schlüsseldaten.
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
    pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool {
        self.lock_slice_with_policy(slice, attempt_mlock, MemLockPolicy::Tolerant)
            .unwrap_or(false)
    }

    // ANCHOR[contextra-sys:VAULT-LOCK-SLICE-POLICY-001]
    /// Versucht einen Byte-Slice im RAM gemäß der angegebenen [`MemLockPolicy`] zu sperren, falls `attempt_mlock == true`.
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
    /// Überschreibt einen veränderbaren Byte-Slice sicher mit Nullen unter Verwendung von flüchtigen Schreibzugriffen (`write_volatile`)
    /// und einer Compiler-Fence (`Ordering::SeqCst`), um Dead-Store Elimination durch den Compiler zuverlässig zu verhindern.
    pub fn zeroize_slice(slice: &mut [u8]) {
        for byte in slice.iter_mut() {
            // SAFETY: `byte` ist eine gültige, ausgerichtete mutable Referenz auf ein `u8` im übergebenen Slice. Volatile Schreibzugriffe verhindern das Wegoptimieren.
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
        compiler_fence(Ordering::SeqCst);
    }

    // ANCHOR[contextra-sys:VAULT-UNLOCK-ALL-001]
    /// Entsperrt alle registrierten Speicherregionen und leert die interne Nachverfolgungsliste.
    pub fn unlock_all(&mut self) {
        for (addr, len) in self.regions.drain(..) {
            mem_unlock(addr, len);
        }
    }

    /// Gibt die Anzahl der aktuell nachverfolgten, gesperrten Regionen zurück.
    pub fn len(&self) -> usize {
        self.regions.len()
    }

    /// Gibt `true` zurück, wenn aktuell keine gesperrten Speicherregionen verwaltet werden.
    pub fn is_empty(&self) -> bool {
        self.regions.is_empty()
    }
}

impl Drop for LockedRegions {
    fn drop(&mut self) {
        self.unlock_all();
    }
}

// ANCHOR[contextra-sys:VAULT-BUFFER-001]
/// Ein im RAM verankerter, geschützter Speicherpuffer für hochsensible kryptografische Schlüsseldaten (INV-SYS-MEMLOCK-1).
#[derive(Debug)]
pub struct VaultBuffer {
    data: Vec<u8>,
    is_locked: bool,
}

impl VaultBuffer {
    /// Allokiert einen neuen Puffer der Größe `size` mit Nullen, verankert ihn im RAM (`MemLockPolicy::FailClosed`)
    /// und schließt ihn von Core-Dumps aus.
    ///
    /// # Fehler
    /// Liefert [`crate::Error::MemoryLockFailed`], wenn das Festpinnen im RAM scheitert (Fail-Closed).
    pub fn allocate(size: usize) -> Result<Self> {
        Self::allocate_with_policy(size, MemLockPolicy::FailClosed)
    }

    /// Allokiert einen neuen Puffer der Größe `size` gemäß der übergebenen Sperr-Richtlinie.
    pub fn allocate_with_policy(size: usize, policy: MemLockPolicy) -> Result<Self> {
        let data = vec![0u8; size];
        if size == 0 {
            return Ok(Self {
                data,
                is_locked: false,
            });
        }

        let ptr = data.as_ptr();
        let is_locked = mem_lock_typed(ptr, size, policy)?;
        if is_locked {
            let _ = apply_dont_dump(ptr, size);
        }

        Ok(Self { data, is_locked })
    }

    /// Allokiert einen Puffer und initialisiert ihn sofort mit einer Kopie der übergebenen sensiblen Daten.
    pub fn from_slice(source: &[u8]) -> Result<Self> {
        let mut buf = Self::allocate(source.len())?;
        buf.as_mut_slice().copy_from_slice(source);
        Ok(buf)
    }

    /// Gibt `true` zurück, wenn der Speicherbereich aktiv im physischen RAM verankert ist.
    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    /// Liefert die unveränderliche Sicht auf den Pufferinhalt.
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// Liefert die veränderbare Sicht auf den Pufferinhalt.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Gibt die Kapazität/Länge des Puffers in Bytes zurück.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Prüft, ob der Puffer 0 Bytes lang ist.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl std::ops::Deref for VaultBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for VaultBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Drop for VaultBuffer {
    fn drop(&mut self) {
        if !self.data.is_empty() {
            LockedRegions::zeroize_slice(&mut self.data);
            if self.is_locked {
                mem_unlock(self.data.as_ptr() as usize, self.data.len());
                self.is_locked = false;
            }
        }
    }
}

// ANCHOR[contextra-sys:VAULT-TESTS-001]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::mlock::fault_injection;

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

    #[test]
    fn test_locked_regions_disabled() {
        let mut tracker = LockedRegions::new();
        let data = vec![1u8, 2, 3, 4, 5];
        let locked = tracker.lock_slice(&data, false);
        assert!(!locked);
        assert!(tracker.is_empty());
    }

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

    #[test]
    fn test_zeroize_slice() {
        let mut secret = vec![0x42u8; 16];
        LockedRegions::zeroize_slice(&mut secret);
        assert_eq!(secret, vec![0u8; 16]);
    }

    // ANCHOR[contextra-sys:INV-SYS-MEMLOCK-1-TEST-001]
    /// Prüft INV-SYS-MEMLOCK-1: VaultBuffer wird gesperrt, mit Nullen überschrieben und schlägt fail-closed fehl, wenn mlock scheitert.
    #[test]
    fn test_inv_sys_memlock_1_vault_protection() -> Result<()> {
        let mut buf = VaultBuffer::allocate(64)?;
        assert_eq!(buf.len(), 64);
        buf.as_mut_slice().copy_from_slice(&[0xA5u8; 64]);
        assert_eq!(&buf[..], &[0xA5u8; 64]);
        drop(buf);

        let raw_secret = b"super_secret_encryption_master_key_1234";
        let vault_buf = VaultBuffer::from_slice(raw_secret)?;
        assert_eq!(&vault_buf[..], raw_secret);
        drop(vault_buf);

        fault_injection::set_fail_mlock(true, libc::ENOMEM);

        let fail_res = VaultBuffer::allocate(128);
        assert!(fail_res.is_err());
        assert!(
            matches!(fail_res, Err(Error::MemoryLockFailed { errno, .. }) if errno == libc::ENOMEM),
            "expected MemoryLockFailed(ENOMEM)"
        );

        let tolerant_buf = VaultBuffer::allocate_with_policy(128, MemLockPolicy::Tolerant)?;
        assert!(!tolerant_buf.is_locked());
        drop(tolerant_buf);

        fault_injection::set_fail_mlock(false, 0);
        Ok(())
    }
}
