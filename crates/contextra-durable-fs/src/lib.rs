#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

//! # Dauerhafte Dateisystem-Operationen und synchrone Disk-Flushes (`contextra-durable-fs`)
//!
//! Zweck: Dateisystem-Implementierung [`real_vfs::RealVfs`] mit dauerhaften Schreib-
//! und Austauschoperationen gemäß Spezifikation §8.5 (DF-01 bis DF-12).
//!
//! Ring-Zugehörigkeit: Ring 1 (Dateisystem-Durability; in `Cargo.toml` als `ring = "1"` annotiert).
//!
//! Bereitgestellte Module:
//! - [`real_vfs`]: Produktions-Implementierung des kanonischen [`contextra_ports::vfs::Vfs`]-Traits.
//! - [`atomic`]: Atomare Austausch- und Schreiboperationen (`write_atomic`, `atomic_replace`, `clean_tmp`).
//! - [`dir_lock`]: Exklusive Verzeichnissperre (`LOCK`-Datei) mit Schutz vor unsicheren Netzdateisystemen.
//! - [`probe`]: Startprobe zur Validierung von Plattformstufen und Dateisystemgarantien.
//! - [`scrub`]: Sicheres Überschreiben vor dem Unlink nur auf rotierenden Medien ohne Copy-on-Write.

use std::path::Path;

// ANCHOR[contextra-durable-fs:LIB-SCRUB-RECEIPT-001]
/// Quittung über das sichere Überschreiben (Scrubbing) einer Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Gesamtanzahl der vor dem Löschen mit Nullen überschriebenen Bytes.
    pub bytes_scrubbed: u64,
}

// ANCHOR[contextra-durable-fs:LIB-FAULT-INJECTOR-001]
/// Trait für deterministische Fehlerinjektion in Persistenzpfaden (intern für scrub-Kompatibilität).
pub trait FaultInjector: Send + Sync {
    /// Hook vor oder während eines Schreibvorgangs.
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        Ok(())
    }
    /// Hook vor dem Hardware-Flush einer Datei.
    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        Ok(())
    }
    /// Hook vor einer atomaren Umbenennungsoperation.
    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        Ok(())
    }
    /// Hook vor dem Hardware-Flush eines Verzeichnisses.
    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        Ok(())
    }
}

// ANCHOR[contextra-durable-fs:LIB-NO-FAULT-001]
/// Null-Kosten-Standardinjektor für Produktivumgebungen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NoFault;

impl FaultInjector for NoFault {}

impl<T: FaultInjector + ?Sized> FaultInjector for &T {
    fn on_write(&self, path: Option<&Path>, bytes: usize) -> std::io::Result<()> {
        (**self).on_write(path, bytes)
    }
    fn on_fsync(&self, path: Option<&Path>) -> std::io::Result<()> {
        (**self).on_fsync(path)
    }
    fn on_rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        (**self).on_rename(from, to)
    }
    fn on_dir_fsync(&self, dir: &Path) -> std::io::Result<()> {
        (**self).on_dir_fsync(dir)
    }
}

// ANCHOR[contextra-durable-fs:LIB-DURABLE-REMOVE-001]
/// Entfernt eine Datei physisch und synchronisiert das übergeordnete Verzeichnis unter Verwendung eines Injektors (DF-09).
pub fn durable_remove_with_injector<F: FaultInjector>(
    path: &Path,
    injector: &F,
) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "target path has no parent directory",
        )
    })?;

    std::fs::remove_file(path)?;
    injector.on_dir_fsync(parent)?;
    contextra_sys::flush_dir(parent)?;
    Ok(())
}

/// Entfernt eine Datei dauerhaft und flusht das übergeordnete Verzeichnis (DF-09).
pub fn durable_remove(path: &Path) -> std::io::Result<()> {
    durable_remove_with_injector(path, &NoFault)
}

/// Reales Virtuelles Dateisystem (`RealVfs`) als primäre Produktionsimplementierung.
pub mod real_vfs;

/// Atomare Austausch- und Schreiboperationen (`write_atomic`, `atomic_replace`, `clean_tmp`, DF-02).
pub mod atomic;

/// Exklusive Verzeichnissperre (`LOCK`-Datei) gemäß SPEC D-50 und DF-05.
pub mod dir_lock;

/// Startprobe zur Validierung von Plattform- und Dateisystem-Garantien (DF-06).
pub mod probe;

/// Sicheres Überschreiben (Scrubbing) und Schutzmechanismen vor dem Löschen (DF-07).
pub mod scrub;

// Re-Exports für primäre Komponenten
pub use atomic::{atomic_replace, clean_tmp, write_atomic};
pub use dir_lock::{DirLock, LockMode, DIR_LOCKED_MSG};
pub use probe::{
    free_space, probe_dir, verdict, DetectedFs, FsProbe, ProbeVerdict, Tier, UnsupportedFilesystem,
};
pub use real_vfs::{RealFile, RealVfs};
pub use scrub::{scrub_before_unlink, scrub_if_safe, ScrubOutcome, ScrubPolicy, ScrubRefusal};
