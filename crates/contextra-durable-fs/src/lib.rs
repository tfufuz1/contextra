#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

//! # Dauerhafte Dateisystem-Operationen und synchrone Disk-Flushes (`contextra-durable-fs`)
//!
//! Zweck: Dateisystem-Implementierung `RealVfs` mit dauerhaften Schreib- und Austauschoperationen.
//!
//! Ring: Ring 1 (Dateisystem-Durability).
//! Erlaubte Abhängigkeiten laut SPEC §5.3: `contextra-types`, `contextra-ports`, `contextra-sys`.
//! Normative Spezifikation: SPEC §8.5 (DF-01 bis DF-12), D-04, D-50, D-55, D-56, D-83, P14.
//!
//! # Direkte Nutzung von `std::fs` (SPEC D-04)
//! Gemäß SPEC D-04 ist dieses Crate (`contextra-durable-fs`) zusammen mit `contextra-sys`
//! die einzige legitime Stelle im gesamten Produktions-Workspace, an der direkte Dateisystem-I/O
//! (`std::fs`) ausgeführt werden darf. Alle anderen Schichten (Store, Vektor, Text, Engine)
//! interagieren ausschließlich über den abstrahierten Port `contextra_ports::vfs::Vfs`.
//!
//! Bereitgestellte Module:
//! - [`real_vfs`]: Produktions-Implementierung des kanonischen [`contextra_ports::vfs::Vfs`]-Traits.
//! - [`atomic`]: Atomare Austausch- und Schreiboperationen (`write_atomic`, `clean_tmp`).
//! - [`dir_lock`]: Exklusive Verzeichnissperre (`LOCK`-Datei) mit Schutz vor unsicheren Netzdateisystemen.
//! - [`probe`]: Startprobe zur Validierung von Plattformstufen und Dateisystemgarantien.
//! - [`scrub`]: Sicheres Überschreiben vor dem Unlink nur auf rotierenden Medien ohne Copy-on-Write.

use std::path::Path;

/// Quittung über das sichere Überschreiben (Scrubbing) einer Datei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrubReceipt {
    /// Gesamtanzahl der vor dem Löschen mit Nullen überschriebenen Bytes.
    pub bytes_scrubbed: u64,
}

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

/// Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit Standardinjektor [`NoFault`].
pub fn durable_remove(path: &Path) -> std::io::Result<()> {
    durable_remove_with_injector(path, &NoFault)
}

/// Entfernt eine Datei dauerhaft und flusht das Elternverzeichnis mit benutzerdefiniertem [`FaultInjector`].
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

/// Reales Virtuelles Dateisystem (`RealVfs`) als primäre Produktionsimplementierung.
pub mod real_vfs;

/// Atomare Austausch- und Schreiboperationen (`write_atomic`, `clean_tmp`, DF-02).
pub mod atomic;

/// Exklusive Verzeichnissperre (`LOCK`-Datei) gemäß SPEC D-50 und DF-05.
pub mod dir_lock;

/// Startprobe zur Validierung von Plattform- und Dateisystem-Garantien (DF-06).
pub mod probe;

/// Sicheres Überschreiben (Scrubbing) und Schutzmechanismen vor dem Löschen (DF-07).
pub mod scrub;

// Re-Exports für primäre Komponenten
pub use atomic::{clean_tmp, write_atomic};
/// Ersetzt den Inhalt der Datei unter `path` atomar mit `bytes` (DF-02).
pub fn atomic_replace(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    atomic::write_atomic(path, bytes)
}
pub use dir_lock::{DirLock, DIR_LOCKED_MSG};
pub use probe::{
    free_space, probe_dir, verdict, DetectedFs, FsProbe, ProbeVerdict, Tier, UnsupportedFilesystem,
};
pub use real_vfs::{RealFile, RealVfs};
pub use scrub::{scrub_if_safe, ScrubOutcome, ScrubPolicy, ScrubRefusal};
