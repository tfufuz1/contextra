//! # Naht für Kalibrierungs-Persistenz (§4.3, Z3)
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Entkopplung des Dateisystem-Zugriffs beim Laden persistierter SLM-Kalibrierungsdaten.
//!
//! **Architektur:**
//! Definiert das lokale Trait [`CalibrationSource`] und drei Implementierungen:
//! - [`NoCalibrationSource`]: Gibt immer `None` zurück (No-Op / In-Memory).
//! - [`StdFsCalibrationSource`]: Einziger Ort für `std::fs`-Zugriffe im Crate für Standard-Dateisysteme.
//! - VFS-Brücke: Ermöglicht das Lesen über `contextra_ports::vfs::Vfs`.

use std::path::Path;
use std::sync::Arc;

// ANCHOR[contextra-router:SEM-001]
/// Trait zur Abstraktion des Lesens persistierter Kalibrierungsdaten.
pub trait CalibrationSource: Send + Sync {
    /// Liest rohe Kalibrierungsbytes von einem gegebenen Pfad.
    ///
    /// Gibt `None` zurück, wenn die Datei nicht existiert oder nicht lesbar ist.
    /// Darf niemals panizieren.
    fn read_calibration(&self, path: &Path) -> Option<Vec<u8>>;
}

// ANCHOR[contextra-router:SEM-002]
/// Deaktiviertes Laden von Kalibrierungsdaten (No-Op).
#[derive(Debug, Clone, Copy, Default)]
pub struct NoCalibrationSource;

impl CalibrationSource for NoCalibrationSource {
    fn read_calibration(&self, _path: &Path) -> Option<Vec<u8>> {
        None
    }
}

// ANCHOR[contextra-router:SEM-003]
/// Standard-Dateisystem-Implementierung über `std::fs`.
///
/// Dies ist der einzige Ort im Crate, an dem synchrones `std::fs::read` verwendet wird.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdFsCalibrationSource;

impl CalibrationSource for StdFsCalibrationSource {
    fn read_calibration(&self, path: &Path) -> Option<Vec<u8>> {
        std::fs::read(path).ok()
    }
}

// ANCHOR[contextra-router:SEM-004]
/// Wrapper für einen virtuellen Dateisystem-Port ([`contextra_ports::vfs::Vfs`]).
#[derive(Clone)]
pub struct VfsCalibrationSource {
    vfs: Arc<dyn contextra_ports::vfs::Vfs>,
}

impl VfsCalibrationSource {
    /// Erzeugt eine neue Kalibrierungsquelle aus einem VFS-Port.
    pub fn new(vfs: Arc<dyn contextra_ports::vfs::Vfs>) -> Self {
        Self { vfs }
    }
}

impl CalibrationSource for VfsCalibrationSource {
    fn read_calibration(&self, path: &Path) -> Option<Vec<u8>> {
        self.vfs.read(path).ok()
    }
}

// Trait-Implementierung für `Arc<dyn contextra_ports::vfs::Vfs>`
impl CalibrationSource for Arc<dyn contextra_ports::vfs::Vfs> {
    fn read_calibration(&self, path: &Path) -> Option<Vec<u8>> {
        self.read(path).ok()
    }
}
