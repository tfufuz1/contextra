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
// ZWECK: Limits und Extraktions-Policies für contextra-ingest (WP M2).
// INVARIANTEN: Defensiver Startzustand, Zero-Panic, #[non_exhaustive], Serialize/Deserialize.
// STAND: TS:2026-10-31T00:00:00Z

use serde::{Deserialize, Serialize};

/// Konfigurierbare Obergrenzen für Ingestion-Vorgänge (Zip-Bomb-Schutz, Speicher- und Komplexitätsgrenzen).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IngestLimits {
    /// Maximale Quelldateigröße in Bytes (Standard: 16 MiB)
    pub max_document_bytes: u64,
    /// Maximale Anzahl erzeugter Chunks pro Dokument (Standard: 4096)
    pub max_chunks_per_document: u32,
    /// Maximale unkomprimierte Gesamtgröße aus Archiven in Bytes (Standard: 128 MiB)
    pub max_decompressed_bytes: u64,
    /// Maximal zulässiges Kompressionsverhältnis (Standard: 100)
    pub max_compression_ratio: u32,
    /// Maximale Anzahl von Archiveinträgen (Standard: 10_000)
    pub max_archive_entries: u32,
    /// Maximale Verschachtelungstiefe von Dokumenten/Archiven (Standard: 3)
    pub max_nesting_depth: u8,
    /// Maximale Tiefe von XML-Strukturen (Standard: 256)
    pub max_xml_depth: u32,
    /// Maximale Anzahl verarbeiteter XML-Events (Standard: 20_000_000)
    pub max_xml_events: u64,
    /// Maximale Bytes an extrahiertem Kanonischen Text (Standard: 64 MiB)
    pub max_text_bytes: u64,
    /// Maximale Seitenanzahl für Paged Documents (Standard: 5_000)
    pub max_pages: u32,
    /// Maximale Anzahl extrahierter Tabellenzellen (Standard: 5_000_000)
    pub max_table_cells: u64,
}

impl Default for IngestLimits {
    fn default() -> Self {
        Self {
            max_document_bytes: 16 * 1024 * 1024,
            max_chunks_per_document: 4096,
            max_decompressed_bytes: 128 * 1024 * 1024,
            max_compression_ratio: 100,
            max_archive_entries: 10_000,
            max_nesting_depth: 3,
            max_xml_depth: 256,
            max_xml_events: 20_000_000,
            max_text_bytes: 64 * 1024 * 1024,
            max_pages: 5_000,
            max_table_cells: 5_000_000,
        }
    }
}

/// Extraktions-Richtlinien für sensible/versteckte Inhalte und Metadaten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExtractPolicy {
    /// Versteckte Inhalte (Kommentare, ausgeblendeten Text, Redaktionsnotizen) extrahieren? (Standard: false)
    pub include_hidden: bool,
    /// Kommentare im Dokument extrahieren? (Standard: true)
    pub include_comments: bool,
    /// Nachverfolgte Löschungen (Tracked Deletions) einbeziehen? (Standard: false)
    pub include_tracked_deletions: bool,
    /// Kopf- und Fußzeilen extrahieren? (Standard: false)
    pub include_headers_footers: bool,
    /// Personenbezogene Metadaten (Autoren, Letzter Bearbeiter) beibehalten? (Standard: false)
    pub keep_personal_metadata: bool,
}

impl Default for ExtractPolicy {
    fn default() -> Self {
        Self {
            include_hidden: false,
            include_comments: true,
            include_tracked_deletions: false,
            include_headers_footers: false,
            keep_personal_metadata: false,
        }
    }
}
