//! Minimaler Umwelt-Vertrag (`contract`) für `contextra-sys` (Ring 0 / E0).
//!
//! Definiert die minimalen Abstraktions-Traits für speicherbezogene und haltbare
//! Systemoperationen, damit Konsumenten und Test-Harnesses ohne physische E/A oder
//! Plattform-Sonderrechte arbeiten können (R2/R3-Anforderung).

use crate::error::Result;
use crate::mlock::MemLockPolicy;

/// Abstraktion für Memory-Locking im Arbeitsspeicher.
pub trait MemoryLocker: Send + Sync {
    /// Sperrt eine Speicherregion im RAM.
    fn lock(&self, ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool>;

    /// Entsperrt eine zuvor gesperrte Speicherregion.
    fn unlock(&self, addr: usize, len: usize);
}

/// Referenzimplementierung eines Memory-Lockers ohne echte E/A / ohne Syscalls (No-op).
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopMemoryLocker;

impl MemoryLocker for NoopMemoryLocker {
    fn lock(&self, _ptr: *const u8, _len: usize, _policy: MemLockPolicy) -> Result<bool> {
        Ok(true)
    }

    fn unlock(&self, _addr: usize, _len: usize) {}
}

/// Standard-Implementierung über die nativen OS-Syscalls von `contextra-sys`.
#[derive(Debug, Default, Clone, Copy)]
pub struct OsMemoryLocker;

impl MemoryLocker for OsMemoryLocker {
    fn lock(&self, ptr: *const u8, len: usize, policy: MemLockPolicy) -> Result<bool> {
        crate::mlock::mem_lock_typed(ptr, len, policy)
    }

    fn unlock(&self, addr: usize, len: usize) {
        crate::mlock::mem_unlock(addr, len);
    }
}
