// FILE-CONTEXT
// ZWECK: Webdokument-Extractor (Html, Xhtml, Epub) (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, DocumentExtractor-Impl.
// STAND: TS:2026-10-31T00:00:00Z

pub mod encoding;
pub mod epub;
pub mod html;

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;

#[derive(Debug, Default)]
pub struct WebExtractor;

impl WebExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[FormatId::Html, FormatId::Xhtml, FormatId::Epub];

impl DocumentExtractor for WebExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        match format {
            FormatId::Html | FormatId::Xhtml => {
                let decoded = encoding::decode_html_bytes(input);
                html::extract_html(&decoded.text, format, ctx, decoded.is_guessed)
            }
            FormatId::Epub => epub::extract_epub(input, ctx),
            _ => Err(IngestError::UnsupportedFormat {
                format: Some(format),
                reason: "Nicht unterstütztes Format in WebExtractor".into(),
            }),
        }
    }
}
