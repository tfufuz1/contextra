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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use contextra_types::ContextraError;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ANCHOR[contextra-router:SEM-T01]
    #[test]
    fn test_no_calibration_source() {
        let source = NoCalibrationSource;
        assert_eq!(source.read_calibration(Path::new("any_path")), None);
    }

    // ANCHOR[contextra-router:SEM-T02]
    #[test]
    fn test_std_fs_calibration_source_success_and_missing() {
        let source = StdFsCalibrationSource;
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"{\"test\": 123}").unwrap();
        tmp.flush().unwrap();

        let bytes = source.read_calibration(tmp.path()).unwrap();
        assert_eq!(bytes, b"{\"test\": 123}");

        let non_existent = Path::new("/nonexistent/file/path/12345");
        assert_eq!(source.read_calibration(non_existent), None);
    }

    // ANCHOR[contextra-router:SEM-T03]
    #[test]
    fn test_vfs_calibration_source() {
        struct MockVfs;
        impl contextra_ports::vfs::Vfs for MockVfs {
            fn read(&self, path: &Path) -> Result<Vec<u8>, ContextraError> {
                if path == Path::new("virtual.json") {
                    Ok(b"virtual content".to_vec())
                } else {
                    Err(ContextraError::NotFound(path.display().to_string()))
                }
            }
            fn write_atomic(&self, _path: &Path, _data: &[u8]) -> Result<(), ContextraError> {
                Ok(())
            }
            fn create_dir_all(&self, _path: &Path) -> Result<(), ContextraError> {
                Ok(())
            }
            fn remove_file(&self, _path: &Path) -> Result<(), ContextraError> {
                Ok(())
            }
            fn exists(&self, path: &Path) -> Result<bool, ContextraError> {
                Ok(path == Path::new("virtual.json"))
            }
            fn len(&self, _path: &Path) -> Result<u64, ContextraError> {
                Ok(15)
            }
            fn list_dir(&self, _path: &Path) -> Result<Vec<String>, ContextraError> {
                Ok(vec![])
            }
            fn sync_dir(&self, _path: &Path) -> Result<(), ContextraError> {
                Ok(())
            }
        }

        let mock_vfs: Arc<dyn contextra_ports::vfs::Vfs> = Arc::new(MockVfs);
        let wrapper = VfsCalibrationSource::new(mock_vfs.clone());
        assert_eq!(
            wrapper.read_calibration(Path::new("virtual.json")),
            Some(b"virtual content".to_vec())
        );
        assert_eq!(wrapper.read_calibration(Path::new("other.json")), None);

        assert_eq!(
            mock_vfs.read_calibration(Path::new("virtual.json")),
            Some(b"virtual content".to_vec())
        );
    }
}
