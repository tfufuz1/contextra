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
// ZWECK: Sicheres Laden und Stream-Auslesen von OLE/CFB-Dateien mit Ressourcen-Limits (WP M4).
// INVARIANTEN: Zero-Panic, Limit-Prüfung über ResourceBudget, Schleifenzähler gegen Zyklen.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use cfb::{CompoundFile, Entry};
use std::io::{Cursor, Read};

/// Maximale Anzahl durchsuchbarer Verzeichniseinträge zur Vermeidung von Endlosschleifen.
const MAX_DIRECTORY_ENTRIES: usize = 4096;

/// Sumpf-sicherer Wrapper für Compound File Binary (CFB) Subsystem.
pub struct SafeCfbReader<'a> {
    comp: CompoundFile<Cursor<&'a [u8]>>,
    format: FormatId,
}

impl<'a> SafeCfbReader<'a> {
    /// Öffnet eine CFB-Datei aus Byte-Slice. Fängt E/A-Fehler ab und wandelt sie in `IngestError::Corrupt` um.
    pub fn open(input: &'a [u8], format: FormatId) -> Result<Self, IngestError> {
        let cursor = Cursor::new(input);
        let comp = CompoundFile::open(cursor).map_err(|err| IngestError::Corrupt {
            format: Some(format),
            detail: format!("CFB-Header/Struktur ungültig: {}", err),
        })?;

        Ok(Self { comp, format })
    }

    /// Prüft, ob ein Pfad (Stream oder Storage) in der CFB existiert.
    pub fn exists(&self, path: &str) -> bool {
        self.comp.exists(path)
    }

    /// Prüft, ob ein Pfad ein Stream ist.
    pub fn is_stream(&self, path: &str) -> bool {
        self.comp.is_stream(path)
    }

    /// Prüft, ob ein Pfad ein Storage ist.
    pub fn is_storage(&self, path: &str) -> bool {
        self.comp.is_storage(path)
    }

    /// Liest einen Stream vollständig unter Berücksichtigung des Decompress-Budgets.
    pub fn read_stream(
        &mut self,
        path: &str,
        ctx: &ExtractContext<'_>,
    ) -> Result<Vec<u8>, IngestError> {
        if !self.comp.is_stream(path) {
            return Err(IngestError::Corrupt {
                format: Some(self.format),
                detail: format!("Erwarteter Stream '{}' existiert nicht", path),
            });
        }

        let mut stream = self
            .comp
            .open_stream(path)
            .map_err(|err| IngestError::Corrupt {
                format: Some(self.format),
                detail: format!("Fehler beim Öffnen von Stream '{}': {}", path, err),
            })?;

        let stream_len = stream.len();
        ctx.budget.charge_decompressed(stream_len)?;

        let mut buf = Vec::with_capacity(usize::try_from(stream_len).unwrap_or(0));
        let read_bytes = stream
            .read_to_end(&mut buf)
            .map_err(|err| IngestError::Corrupt {
                format: Some(self.format),
                detail: format!("Fehler beim Lesen von Stream '{}': {}", path, err),
            })?;

        if u64::try_from(read_bytes).unwrap_or(0) != stream_len {
            return Err(IngestError::Corrupt {
                format: Some(self.format),
                detail: format!(
                    "Stream '{}' unvollständig gelesen: {} von {} Bytes",
                    path, read_bytes, stream_len
                ),
            });
        }

        Ok(buf)
    }

    /// Listet alle direkten Kinder eines Storages auf (mit Schleifenbegrenzung).
    pub fn list_children(&self, storage_path: &str) -> Result<Vec<Entry>, IngestError> {
        let entries = self
            .comp
            .read_storage(storage_path)
            .map_err(|err| IngestError::Corrupt {
                format: Some(self.format),
                detail: format!("Fehler beim Lesen des Storage '{}': {}", storage_path, err),
            })?;

        let mut result = Vec::new();
        let mut count = 0usize;

        for entry in entries {
            count = count.saturating_add(1);
            if count > MAX_DIRECTORY_ENTRIES {
                return Err(IngestError::LimitExceeded {
                    kind: LimitKind::ArchiveEntries,
                    limit: u64::try_from(MAX_DIRECTORY_ENTRIES).unwrap_or(u64::MAX),
                    observed: u64::try_from(count).unwrap_or(u64::MAX),
                });
            }
            result.push(entry);
        }

        Ok(result)
    }

    /// Rekursiver Walk über alle Einträge in der CFB mit Schleifenbegrenzer gegen Zyklen.
    pub fn walk_all(&self) -> Result<Vec<Entry>, IngestError> {
        let mut result = Vec::new();
        let mut count = 0usize;

        for entry in self.comp.walk() {
            count = count.saturating_add(1);
            if count > MAX_DIRECTORY_ENTRIES {
                return Err(IngestError::LimitExceeded {
                    kind: LimitKind::ArchiveEntries,
                    limit: u64::try_from(MAX_DIRECTORY_ENTRIES).unwrap_or(u64::MAX),
                    observed: u64::try_from(count).unwrap_or(u64::MAX),
                });
            }
            result.push(entry);
        }

        Ok(result)
    }
}
