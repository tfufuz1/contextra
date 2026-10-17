// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Trait FaultInjector und NoFault Standard-Injektor fuer I/O-Fehlerinjektion in Durability-Pfade.
// INVARIANTEN: Zero-Cost in Produktionspfaden via Inlining und Generics (kein dyn Dispatch), Zero-Panic.

use std::path::Path;

/// Trait for injecting I/O fault hooks into durable filesystem operations.
///
/// Implementations can intercept file writes, file flushes (`fsync`), atomic renames,
/// and directory flushes to simulate storage faults (e.g., `ENOSPC` disk full, fsync failure).
///
/// Default implementations return `Ok(())` (no-op).
pub trait FaultInjector: Send + Sync {
    /// Hook executed prior to or during a file write operation.
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }

    /// Hook executed prior to a file sync/flush (`fsync`) operation.
    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        Ok(())
    }

    /// Hook executed prior to an atomic rename operation.
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }

    /// Hook executed prior to a directory sync/flush (`fsync`) operation.
    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

/// Zero-cost, no-op fault injector for production paths.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoFault;

impl FaultInjector for NoFault {
    #[inline(always)]
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

impl<T: FaultInjector + ?Sized> FaultInjector for &T {
    #[inline]
    fn on_write(&self, path: Option<&Path>, bytes: usize) -> std::io::Result<()> {
        (**self).on_write(path, bytes)
    }

    #[inline]
    fn on_fsync(&self, path: Option<&Path>) -> std::io::Result<()> {
        (**self).on_fsync(path)
    }

    #[inline]
    fn on_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        (**self).on_rename(from, to)
    }

    #[inline]
    fn on_dir_fsync(&self, dir: &Path) -> std::io::Result<()> {
        (**self).on_dir_fsync(dir)
    }
}
