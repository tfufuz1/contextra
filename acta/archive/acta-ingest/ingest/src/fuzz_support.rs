//! Modul
//!
//! # Zweck & Architektur
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// Fuzzing-Support-Modul für Ingestion-Fuzzing-Ziele (WP M4).
// Invarianten: Zero-Panic für beliebige Eingabe-Bytes.

use crate::chunking::types::DocId;
use crate::format::FormatId;
use crate::pipeline::{ingest, IngestConfig, IngestRequest};
use crate::registry::ExtractorRegistry;

/// Fuzzing-Einstiegspunkt für alle Ingestion-Extraktoren.
/// Muss für jede beliebige Eingabe panikfrei durchlaufen.
pub fn run(format: FormatId, data: &[u8]) {
    let registry = ExtractorRegistry::with_defaults();
    let req = IngestRequest {
        bytes: data,
        filename: Some("fuzz_sample"),
        mime: None,
        format_override: Some(format),
        doc_id: DocId::new(1),
    };
    let cfg = IngestConfig::default();

    // Ingestion ausführen; Fehler werden ignoriert, Panics sind unzulässig
    let _ = ingest(&req, &cfg, &registry);
}
