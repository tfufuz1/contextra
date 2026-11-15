//! Modul
//!
//! # Zweck & Architektur
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// E-Mail Extractor (Eml und Mbox) (WP M4).
// Invarianten: DocumentExtractor-Impl mit Zero-Panic Garantien.

pub mod eml;
pub mod html;
pub mod mbox;

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;

#[derive(Debug, Default)]
pub struct EmailExtractor;

impl EmailExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[FormatId::Eml, FormatId::Mbox];

impl DocumentExtractor for EmailExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        _input: &[u8],
        _ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        match format {
            FormatId::Eml => eml::extract_eml(_input, _ctx, None),
            FormatId::Mbox => mbox::extract_mbox(_input, _ctx),
            _ => Err(IngestError::UnsupportedFormat {
                format: Some(format),
                reason: "E-Mail Extractor unterstützt nur Eml und Mbox".into(),
            }),
        }
    }
}
