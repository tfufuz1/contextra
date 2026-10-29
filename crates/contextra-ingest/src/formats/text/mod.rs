// FILE-CONTEXT
// ZWECK: Plaintext- & Textdokument-Extractor (Txt, Markdown, Rst, AsciiDoc, Tex, Rtf) (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, DocumentExtractor-Impl.
// STAND: TS:2026-10-31T00:00:00Z

pub mod asciidoc;
pub mod encoding;
pub mod markdown;
pub mod rst;
pub mod rtf;
pub mod tex;
pub mod txt;

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;

#[derive(Debug, Default)]
pub struct TextExtractor;

impl TextExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[
    FormatId::Txt,
    FormatId::Markdown,
    FormatId::Rst,
    FormatId::AsciiDoc,
    FormatId::Tex,
    FormatId::Rtf,
];

impl DocumentExtractor for TextExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        if format == FormatId::Rtf {
            let ir = rtf::parse_rtf(input, ctx)?;
            ctx.budget.charge_text(
                ir.blocks
                    .iter()
                    .map(|b| match &b.kind {
                        crate::ir::BlockKind::Paragraph { text } => text.len(),
                        _ => 0,
                    })
                    .sum::<usize>() as u64,
            )?;
            return Ok(ir);
        }

        let (clean_text, warnings) = encoding::decode_text(input);
        ctx.budget.charge_text(clean_text.len() as u64)?;

        let mut ir = match format {
            FormatId::Txt => txt::parse_txt(&clean_text),
            FormatId::Markdown => markdown::parse_markdown(&clean_text),
            FormatId::Rst => rst::parse_rst(&clean_text),
            FormatId::AsciiDoc => asciidoc::parse_asciidoc(&clean_text),
            FormatId::Tex => tex::parse_tex(&clean_text),
            _ => {
                return Err(IngestError::UnsupportedFormat {
                    format: Some(format),
                    reason: "Format nicht vom TextExtractor unterstützt".into(),
                })
            }
        };

        ir.warnings.extend(warnings);
        Ok(ir)
    }
}
