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
// ZWECK: Sicheres Entpacken und Lesen von ZIP-Archiven mit Zip-Bomb-Schutz (P00B).
// INVARIANTEN: Zero-Panic, ResourceBudget-Prüfung, keine Unsafe-Islands.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ResourceBudget;
use std::io::{Cursor, Read};
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
