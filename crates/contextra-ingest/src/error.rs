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
// ZWECK: Fehler-Typen und Limit-Kategorien für contextra-ingest (WP M2).
// INVARIANTEN: Zero-Panic, #[non_exhaustive], deutschsprachige thiserror-Meldungen, Mapping zu ContextraError.
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::FormatId;
use contextra_types::ContextraError;
use serde::{Deserialize, Serialize};

/// Kategorien von Ressourceneinschränkungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum LimitKind {
    /// Maximale Dateigröße in Bytes
    DocumentBytes,
    /// Maximale unkomprimierte Bytes (Zip-Bomb-Schutz)
    DecompressedBytes,
    /// Maximales Kompressionsverhältnis (z. B. 100:1)
    CompressionRatio,
    /// Maximale Anzahl von Einträgen im Archiv
    ArchiveEntries,
    /// Maximale Verschachtelungstiefe von Dokumenten/Archiven
    NestingDepth,
    /// Maximale XML-Strukturtiefe
    XmlDepth,
    /// Maximale Anzahl von XML-Parser-Events
    XmlEvents,
    /// Maximale Gesamt-Textgröße in Bytes
    TextBytes,
    /// Maximale Anzahl von Seiten (PDF, PPTX)
    Pages,
    /// Maximale Anzahl von Tabellenzellen
    TableCells,
    /// Maximale Anzahl erzeugter Chunks
    Chunks,
}

/// Einheitlicher Fehler-Enum für alle Ingestion- und Chunking-Vorgänge.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum IngestError {
    #[error("Nicht unterstütztes Dokumentformat {format:?}: {reason}")]
    UnsupportedFormat {
        format: Option<FormatId>,
        reason: String,
    },

    #[error("Dokument ist verschlüsselt oder passwortgeschützt ({format:?})")]
    Encrypted { format: Option<FormatId> },

    #[error("Dokument ist beschädigt oder ungültig ({format:?}): {detail}")]
    Corrupt {
        format: Option<FormatId>,
        detail: String,
    },

    #[error("Ressourcenlimit überschritten ({kind:?}): beobachtet {observed}, Limit ist {limit}")]
    LimitExceeded {
        kind: LimitKind,
        limit: u64,
        observed: u64,
    },

    #[error("Kein extrahierbarer Text im Dokument ({format:?}) gefunden")]
    NoExtractableText { format: FormatId },

    #[error("Unerwarteter Parser-Fehler/Panik bei Format {format:?}")]
    ParserPanic { format: FormatId },

    #[error("Fehler bei der Textzerlegung (Chunking): {detail}")]
    Chunking { detail: String },
}

impl From<IngestError> for ContextraError {
    fn from(err: IngestError) -> Self {
        match err {
            IngestError::LimitExceeded {
                kind,
                limit,
                observed,
            } => ContextraError::limit_exceeded(
                usize::try_from(limit).unwrap_or(usize::MAX),
                format!("Ingestion limit {:?} exceeded: observed {}", kind, observed),
            ),
            IngestError::ParserPanic { format } => {
                ContextraError::Internal(format!("Parser panic during ingestion of {:?}", format))
            }
            other => ContextraError::InvalidInput(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingest_error_conversion() {
        let limit_err = IngestError::LimitExceeded {
            kind: LimitKind::DocumentBytes,
            limit: 16 * 1024 * 1024,
            observed: 20 * 1024 * 1024,
        };
        let ce: ContextraError = limit_err.into();
        assert!(matches!(ce, ContextraError::LimitExceeded { .. }));

        let panic_err = IngestError::ParserPanic {
            format: FormatId::Pdf,
        };
        let ce2: ContextraError = panic_err.into();
        assert!(matches!(ce2, ContextraError::Internal(_)));

        let corrupt_err = IngestError::Corrupt {
            format: Some(FormatId::Docx),
            detail: "Bad header".into(),
        };
        let ce3: ContextraError = corrupt_err.into();
        assert!(matches!(ce3, ContextraError::InvalidInput(_)));
    }
}
