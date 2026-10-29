// FILE-CONTEXT
// ZWECK: Sicheres Entpacken von ZIP-Archiven unter Ressourcen-Limits (P00B/P04).
// INVARIANTEN: Zero-Panic, LimitExceeded / Corrupt Fehler bei Entkomprimierungsproblemen, kein unsafe.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ResourceBudget;

/// Safe Zip archive reader enforcing resource limits.
#[derive(Debug)]
pub struct SafeArchive<'a> {
    archive: zip::ZipArchive<std::io::Cursor<&'a [u8]>>,
}

impl<'a> SafeArchive<'a> {
    /// Opens a ZIP archive from raw input bytes and checks entry count limits.
    pub fn new(bytes: &'a [u8], _budget: &ResourceBudget) -> Result<Self, IngestError> {
        let cursor = std::io::Cursor::new(bytes);
        let archive = zip::ZipArchive::new(cursor).map_err(|e| IngestError::Corrupt {
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

        let mut content = Vec::with_capacity(usize::try_from(uncompressed_size).unwrap_or(0));
        std::io::Read::read_to_end(&mut file, &mut content).map_err(|e| IngestError::Corrupt {
            format: None,
            detail: format!("Fehler beim Dekomprimieren: {e}"),
        })?;

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
