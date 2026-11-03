//! Memory protection port definitions for vector arenas and sensitive buffers.
//!
//! # SPEC-REF (D-88 (2), D-63, PO-11)
//! In accordance with D-88 (2) and PO-11:
//! - Vector index and arena memory must be locked and guarded against swap/core dump exfiltration.
//! - `contextra-vector` has no direct dependency on `contextra-sys` or `contextra-mvcc`.
//! - Memory protection primitives (`mlock`, `MADV_DONTDUMP`) are decoupled via [`ArenaGuard`].
//! - Implementations are provided by higher-level or platform crates (such as `contextra-engine` using `contextra-sys`).

use crate::Result;
use std::fmt;
use std::sync::Arc;

/// RAII token guard protecting a specific locked memory region.
///
/// Ensures memory unlock or dump-restoration is performed cleanly upon drop.
pub struct MemoryLockGuard {
    _token: Option<Arc<()>>,
}

impl MemoryLockGuard {
    /// Creates a new `MemoryLockGuard`.
    pub fn new(token: Option<Arc<()>>) -> Self {
        Self { _token: token }
    }
}

impl fmt::Debug for MemoryLockGuard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MemoryLockGuard([PROTECTED])")
    }
}

/// Abstract port for locking and hardening process memory against dumping and swapping.
///
/// Follows PO-11 and SPEC D-88 (2).
pub trait ArenaGuard: Send + Sync {
    /// Locks the specified memory slice into physical RAM (`mlock`), preventing swapping to disk.
    ///
    /// # Errors
    /// Returns [`ContextraError`](crate::ContextraError) if memory locking fails or is disallowed by OS limits.
    fn lock_memory(&self, ptr: *const u8, len: usize) -> Result<MemoryLockGuard>;

    /// Excludes the specified memory slice from OS core dumps (`MADV_DONTDUMP`).
    ///
    /// # Errors
    /// Returns [`ContextraError`](crate::ContextraError) if memory advising fails.
    /// # Fehler
    /// Liefert Fehler falls madvise fehlschlägt.
    ///
    /// # Crash
    /// Reiner Speicherschutz im RAM.
    fn protect_against_dumping(&self, ptr: *const u8, len: usize) -> Result<()>;
}

impl<T: ArenaGuard + ?Sized> ArenaGuard for Arc<T> {
    fn lock_memory(&self, ptr: *const u8, len: usize) -> Result<MemoryLockGuard> {
        (**self).lock_memory(ptr, len)
    }

    fn protect_against_dumping(&self, ptr: *const u8, len: usize) -> Result<()> {
        (**self).protect_against_dumping(ptr, len)
    }
}
