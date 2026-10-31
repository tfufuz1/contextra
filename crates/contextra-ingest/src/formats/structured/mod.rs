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
// ZWECK: Extractor für strukturierte Formate (Json, Jsonl, Xml, Yaml, Toml) (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, DocumentExtractor-Impl für 5 strukturierte Formate.
// STAND: TS:2026-10-31T00:00:00Z

pub mod json;
pub mod path;
pub mod toml;
pub mod xml;
pub mod yaml;

use crate::canonical::sanitize_text;
use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr};

#[derive(Debug, Default)]
pub struct StructuredExtractor;

impl StructuredExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[
    FormatId::Json,
    FormatId::Jsonl,
    FormatId::Xml,
    FormatId::Yaml,
    FormatId::Toml,
];

impl DocumentExtractor for StructuredExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        let mut ir = match format {
            FormatId::Json => json::extract_json(input, ctx)?,
            FormatId::Jsonl => json::extract_jsonl(input, ctx)?,
            FormatId::Xml => xml::extract_xml(input, ctx)?,
            FormatId::Yaml => yaml::extract_yaml(input, ctx)?,
            FormatId::Toml => toml::extract_toml(input, ctx)?,
            other => {
                return Err(IngestError::UnsupportedFormat {
                    format: Some(other),
                    reason: "Format nicht vom StructuredExtractor unterstützt".into(),
                });
            }
        };

        // Alle Datensätze (Pfad und Wert) sowie Titel durch sanitize_text führen
        if let Some(ref mut title) = ir.meta.title {
            let (clean_title, _) = sanitize_text(title);
            *title = clean_title;
        }

        for block in &mut ir.blocks {
            if let BlockKind::Record {
                ref mut path,
                ref mut value,
            } = block.kind
            {
                let (clean_path, _) = sanitize_text(path);
                let (clean_val, _) = sanitize_text(value);
                *path = clean_path;
                *value = clean_val;
            }
        }

        Ok(ir)
    }
}
