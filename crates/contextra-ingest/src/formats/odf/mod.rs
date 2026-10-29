// FILE-CONTEXT
// ZWECK: OpenDocument Extractor Stub (Odt, Odp) (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, DocumentExtractor-Impl mit UnsupportedFormat.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;

#[derive(Debug, Default)]
pub struct OdfExtractor;

impl OdfExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[FormatId::Odt, FormatId::Odp];

impl DocumentExtractor for OdfExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        _input: &[u8],
        _ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        Err(IngestError::UnsupportedFormat {
            format: Some(format),
            reason: "noch nicht implementiert".into(),
        })
    }
}
