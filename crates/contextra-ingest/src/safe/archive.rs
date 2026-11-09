//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// FILE-CONTEXT
// ZWECK: Sicheres Entpacken und Lesen von ZIP-Archiven mit Zip-Bomb-Schutz (P00B/P04).
// INVARIANTEN: Zero-Panic, ResourceBudget-Prüfung, keine Unsafe-Islands.
// STAND: TS:2026-11-08T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ResourceBudget;
use crate::limits::IngestLimits;
use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

/// Sichere ZIP-Archiv-Hülle zur lesenden Extraktion von ZIP-Einträgen.
pub struct SafeZipReader<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
}

impl<'a> SafeZipReader<'a> {
    /// Erstellt einen neuen `SafeZipReader` aus Eingabe-Bytes.
    pub fn new(bytes: &'a [u8]) -> Result<Self, IngestError> {
        let cursor = Cursor::new(bytes);
        let archive = ZipArchive::new(cursor).map_err(|e| IngestError::Corrupt {
            format: None,
            detail: format!("Ungültiges ZIP-Archiv: {e}"),
        })?;

        Ok(Self { archive })
    }

    /// Liest den gesamten Inhalt der angegebenen Datei im Archiv und verbucht entkomprimierte Bytes.
    pub fn read_file(
        &mut self,
        name: &str,
        budget: &ResourceBudget,
    ) -> Result<Vec<u8>, IngestError> {
        let mut zip_file = match self.archive.by_name(name) {
            Ok(file) => file,
            Err(zip::result::ZipError::FileNotFound) => {
                return Err(IngestError::Corrupt {
                    format: None,
                    detail: format!("Datei '{name}' im ZIP-Archiv nicht gefunden"),
                });
            }
            Err(e) => {
                return Err(IngestError::Corrupt {
                    format: None,
                    detail: format!("Fehler beim Öffnen von '{name}' im ZIP-Archiv: {e}"),
                });
            }
        };

        let mut result = Vec::new();
        let mut chunk = [0u8; 8192];
        loop {
            let bytes_read = match zip_file.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    return Err(IngestError::Corrupt {
                        format: None,
                        detail: format!("Fehler beim Lesen von '{name}': {e}"),
                    });
                }
            };

            budget.charge_decompressed(bytes_read as u64)?;
            result.extend_from_slice(chunk.get(..bytes_read).ok_or_else(|| {
                IngestError::Corrupt {
                    format: None,
                    detail: "Slicing-Fehler beim Pufferzugriff".into(),
                }
            })?);
        }

        Ok(result)
    }

    /// Prüft, ob eine Datei unter dem Namen im Archiv existiert.
    pub fn contains_file(&mut self, name: &str) -> bool {
        self.archive.by_name(name).is_ok()
    }

    /// Gibt eine Liste aller Dateipfade im Archiv zurück.
    pub fn file_names(&mut self) -> Vec<String> {
        let mut names = Vec::new();
        for i in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index_raw(i) {
                names.push(file.name().to_string());
            }
        }
        names
    }
}

/// Safe Zip archive reader enforcing resource limits (z. B. für OPC / DOCX / PPTX).
#[derive(Debug)]
pub struct SafeArchive<'a> {
    archive: ZipArchive<Cursor<&'a [u8]>>,
}

impl<'a> SafeArchive<'a> {
    /// Opens a ZIP archive from raw input bytes and checks entry count limits.
    pub fn new(bytes: &'a [u8], _budget: &ResourceBudget) -> Result<Self, IngestError> {
        let cursor = Cursor::new(bytes);
        let archive = ZipArchive::new(cursor).map_err(|e| IngestError::Corrupt {
            format: None,
            detail: format!("Ungültiges ZIP-Archiv: {e}"),
        })?;

        let entry_count = u64::try_from(archive.len()).unwrap_or(u64::MAX);
        if entry_count > 10_000 {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::ArchiveEntries,
                limit: 10_000,
                observed: entry_count,
            });
        }

        Ok(Self { archive })
    }

    /// Reads raw bytes for a file entry matching `name` (case-insensitive / path normalized).
    pub fn read_entry(
        &mut self,
        name: &str,
        budget: &ResourceBudget,
    ) -> Result<Vec<u8>, IngestError> {
        // Prevent path traversal
        if name.contains("..") {
            return Err(IngestError::Corrupt {
                format: None,
                detail: format!("Gefährlicher Dateipfad im Archiv: {name}"),
            });
        }

        let normalized_target = name.replace('\\', "/");
        let mut target_index = None;

        for idx in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(idx) {
                let entry_name = file.name().replace('\\', "/");
                if entry_name.eq_ignore_ascii_case(&normalized_target)
                    || entry_name
                        .trim_start_matches('/')
                        .eq_ignore_ascii_case(&normalized_target)
                {
                    target_index = Some(idx);
                    break;
                }
            }
        }

        let index = target_index.ok_or_else(|| IngestError::Corrupt {
            format: None,
            detail: format!("Archiv-Eintrag nicht gefunden: {name}"),
        })?;

        let mut file = self
            .archive
            .by_index(index)
            .map_err(|e| IngestError::Corrupt {
                format: None,
                detail: format!("Fehler beim Lesen des Archiv-Eintrags: {e}"),
            })?;

        let uncompressed_size = file.size();
        budget.charge_decompressed(uncompressed_size)?;

        // DoS-Schutz: Niemals unvalidierte Größen direkt vorab allozieren
        // Max initial capacity auf 64 KiB deckeln, Rest wächst dynamisch mit Leseoperationen
        let initial_cap = (usize::try_from(uncompressed_size).unwrap_or(0)).min(64 * 1024);
        let mut content = Vec::with_capacity(initial_cap);
        let mut chunk = [0u8; 8192];
        loop {
            let n = std::io::Read::read(&mut file, &mut chunk).map_err(|e| IngestError::Corrupt {
                format: None,
                detail: format!("Fehler beim Dekomprimieren: {e}"),
            })?;
            if n == 0 {
                break;
            }
            content.extend_from_slice(chunk.get(..n).ok_or_else(|| IngestError::Corrupt {
                format: None,
                detail: "Puffergrenzenfehler beim Lesen".into(),
            })?);
        }

        Ok(content)
    }

    /// Checks if an entry exists in the archive.
    pub fn has_entry(&mut self, name: &str) -> bool {
        let normalized_target = name.replace('\\', "/");
        for idx in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(idx) {
                let entry_name = file.name().replace('\\', "/");
                if entry_name.eq_ignore_ascii_case(&normalized_target)
                    || entry_name
                        .trim_start_matches('/')
                        .eq_ignore_ascii_case(&normalized_target)
                {
                    return true;
                }
            }
        }
        false
    }

    /// Lists all file paths present in the archive.
    pub fn list_entries(&mut self) -> Vec<String> {
        let mut entries = Vec::new();
        for idx in 0..self.archive.len() {
            if let Ok(file) = self.archive.by_index(idx) {
                entries.push(file.name().to_string());
            }
        }
        entries
    }
}

/// Validiert einen Pfadnamen im Archiv gegen Zip-Slip und Pfadlängenbeschränkungen.
pub fn validate_archive_entry_path(path: &str, max_len: usize) -> Result<(), IngestError> {
    if path.len() > max_len {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::PathLength,
            limit: max_len as u64,
            observed: path.len() as u64,
        });
    }

    // Zip-Slip Schutz: keine relative Pfadnavigation (`..`), keine absoluten Pfade (`/`, `\`, Laufwerksbuchstaben)
    if path.contains("..") {
        return Err(IngestError::Corrupt {
            format: None,
            detail: format!("Zip-Slip-Angriff erkannt (Pfad enthält '..'): {path}"),
        });
    }

    if path.starts_with('/') || path.starts_with('\\') {
        return Err(IngestError::Corrupt {
            format: None,
            detail: format!("Zip-Slip-Angriff erkannt (absoluter Pfad): {path}"),
        });
    }

    // Windows Laufwerksbuchstabe (z. B. "C:")
    if path.as_bytes().get(1) == Some(&b':') {
        return Err(IngestError::Corrupt {
            format: None,
            detail: format!("Zip-Slip-Angriff erkannt (Laufwerkspfad): {path}"),
        });
    }

    Ok(())
}

/// Validiert ein Zip-Archiv vor der Verarbeitung gegen `IngestLimits`.
///
/// Prüft Eintragsanzahl, kumulierte unkomprimierte Bytes, Kompressionsverhältnis,
/// Zip-Slip-Pfade und Pfadlängen, um Zip-Bomben und DoS-Angriffe frühzeitig abzulehnen.
pub fn validate_zip_archive<R: Read + Seek>(
    reader: R,
    limits: &IngestLimits,
) -> Result<(), IngestError> {
    let mut archive = ZipArchive::new(reader).map_err(|e| IngestError::Corrupt {
        format: None,
        detail: format!("Ungültiges Zip-Archiv: {e}"),
    })?;

    let num_entries = u32::try_from(archive.len()).unwrap_or(u32::MAX);
    if num_entries > limits.max_archive_entries {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::ArchiveEntries,
            limit: u64::from(limits.max_archive_entries),
            observed: u64::from(num_entries),
        });
    }

    let mut total_uncompressed: u64 = 0;
    let mut total_compressed: u64 = 0;

    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| IngestError::Corrupt {
            format: None,
            detail: format!("Zip-Eintrag {i} nicht lesbar: {e}"),
        })?;

        // Pfadvalidierung (Zip-Slip & Pfadlänge)
        validate_archive_entry_path(file.name(), limits.max_path_len)?;

        total_uncompressed = total_uncompressed.saturating_add(file.size());
        total_compressed = total_compressed.saturating_add(file.compressed_size());

        if total_uncompressed > limits.max_decompressed_bytes {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::DecompressedBytes,
                limit: limits.max_decompressed_bytes,
                observed: total_uncompressed,
            });
        }
    }

    if total_uncompressed > 0 {
        let compressed_denom = total_compressed.max(1);
        let ratio = total_uncompressed / compressed_denom;
        if ratio > u64::from(limits.max_compression_ratio) {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::CompressionRatio,
                limit: u64::from(limits.max_compression_ratio),
                observed: ratio,
            });
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::field_reassign_with_default
)]
mod tests {
    use super::*;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    #[test]
    fn test_validate_zip_archive_ok() {
        let mut buf = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            let options = FileOptions::default();
            writer.start_file("test.txt", options).unwrap();
            std::io::Write::write_all(&mut writer, b"hello world").unwrap();
            writer.finish().unwrap();
        }

        let limits = IngestLimits::default();
        assert!(validate_zip_archive(Cursor::new(&buf), &limits).is_ok());
    }

    #[test]
    fn test_validate_zip_archive_exceeds_entries() {
        let mut buf = Vec::new();
        {
            let mut writer = ZipWriter::new(Cursor::new(&mut buf));
            let options = FileOptions::default();
            for i in 0..5 {
                writer.start_file(format!("file_{i}.txt"), options).unwrap();
                std::io::Write::write_all(&mut writer, b"x").unwrap();
            }
            writer.finish().unwrap();
        }

        let mut limits = IngestLimits::default();
        limits.max_archive_entries = 3;
        let err = validate_zip_archive(Cursor::new(&buf), &limits).unwrap_err();
        assert!(matches!(
            err,
            IngestError::LimitExceeded {
                kind: LimitKind::ArchiveEntries,
                ..
            }
        ));
    }
}
