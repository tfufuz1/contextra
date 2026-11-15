//! Lokale schmale Dateisystem-Schnittstelle (Dateinaht nach B08).
//!
//! Kapselt die tatsächlich genutzten VFS-Operationen (atomic write, read).
//! Die optionale Bindung an externe Ports erfolgt in `src/compat/`.

use std::path::Path;

/// Lokale minimale VFS-Schnittstelle für `acta-checkpoint`.
pub trait CheckpointVfs: Send + Sync {
    /// Schreibt Daten atomar an den angegebenen Pfad.
    fn write_atomic(&self, path: &Path, data: &[u8]) -> std::io::Result<()>;

    /// Liest den gesamten Dateiinhalt an dem angegebenen Pfad.
    fn read(&self, path: &Path) -> std::io::Result<Vec<u8>>;
}
