// FILE-CONTEXT
// ZWECK: Fuzzing-Support-Modul für Ingestion-Fuzzing-Ziele (WP M4).
// INVARIANTEN: Zero-Panic für beliebige Eingabe-Bytes.
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::FormatId;
use crate::pipeline::{ingest, IngestConfig, IngestRequest};
use crate::registry::ExtractorRegistry;
use contextra_types::DocId;

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
