// FILE-CONTEXT
// ZWECK: OLE/CFB Extractor Subsystem (Doc, Ppt, Msg) (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, DocumentExtractor-Impl mit SafeCfbReader.
// STAND: TS:2026-10-31T00:00:00Z

pub mod cfb_reader;
pub mod doc;
pub mod msg;
pub mod ppt;

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;
use cfb_reader::SafeCfbReader;

#[derive(Debug, Default)]
pub struct OleExtractor;

impl OleExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[FormatId::Doc, FormatId::Ppt, FormatId::Msg];

impl DocumentExtractor for OleExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        let mut reader = SafeCfbReader::open(input, format)?;
        match format {
            FormatId::Doc => doc::extract_doc(&mut reader, ctx),
            FormatId::Ppt => ppt::extract_ppt(&mut reader, ctx),
            FormatId::Msg => msg::extract_msg(&mut reader, ctx),
            _ => Err(IngestError::UnsupportedFormat {
                format: Some(format),
                reason: format!(
                    "Format {:?} wird vom OleExtractor nicht unterstützt",
                    format
                ),
            }),
        }
    }
}
