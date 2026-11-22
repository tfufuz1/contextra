// FILE-CONTEXT
// ZWECK: OpenDocument Extractor (ODT, ODP und Flat-ODF .fodt/.fodp) (WP M4).
// INVARIANTEN: Zero-Panic, Fail-Closed, Zip-Bomb & Budgetierungsschutz, Verschlüsselungserkennung.
// STAND: TS:2026-10-31T00:00:00Z

pub mod content;
pub mod meta;
pub mod presentation;

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;
use std::io::Read;

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
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        if input.is_empty() {
            return Err(IngestError::Corrupt {
                format: Some(format),
                detail: "Eingabe ist leer".into(),
            });
        }

        // Erkennung Flat-ODF vs. ZIP-Container
        let is_zip = input.starts_with(b"PK\x03\x04");

        if is_zip {
            extract_zip_odf(format, input, ctx)
        } else if is_flat_odf(input) {
            extract_flat_odf(format, input, ctx)
        } else {
            Err(IngestError::Corrupt {
                format: Some(format),
                detail: "Eingabe ist weder ein gültiges ODF-Zip-Archiv noch Flat-ODF XML".into(),
            })
        }
    }
}

fn is_flat_odf(input: &[u8]) -> bool {
    let s = std::str::from_utf8(input).unwrap_or_default().trim_start();
    s.starts_with("<?xml") || s.starts_with("<office:document") || s.starts_with("<office:document-content")
}

fn extract_flat_odf(
    format: FormatId,
    input: &[u8],
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let mut ir = DocumentIr::new(format);

    // Metadaten extrahieren
    if let Ok(meta_data) = meta::parse_meta(input, ctx) {
        ir.meta = meta_data;
    }

    // Inhaltsblöcke extrahieren
    if format == FormatId::Odp {
        presentation::parse_presentation(input, ctx, &mut ir)?;
    } else {
        content::parse_content(format, input, ctx, &mut ir)?;
    }

    if ir.blocks.is_empty() {
        return Err(IngestError::NoExtractableText { format });
    }

    Ok(ir)
}

fn extract_zip_odf(
    format: FormatId,
    input: &[u8],
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let cursor = std::io::Cursor::new(input);
    let mut zip = zip::ZipArchive::new(cursor).map_err(|e| IngestError::Corrupt {
        format: Some(format),
        detail: format!("Ungültiges ZIP-Archiv: {}", e),
    })?;

    let mut mimetype_bytes: Option<Vec<u8>> = None;
    let mut meta_xml_bytes: Option<Vec<u8>> = None;
    let mut content_xml_bytes: Option<Vec<u8>> = None;
    let mut manifest_bytes: Option<Vec<u8>> = None;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| IngestError::Corrupt {
            format: Some(format),
            detail: format!("Fehler beim Lesen des ZIP-Eintrags {}: {}", i, e),
        })?;

        let name = file.name().to_string();
        let uncompressed_size = file.size();

        ctx.budget.charge_decompressed(uncompressed_size)?;

        let mut buf = Vec::with_capacity(uncompressed_size.min(1024 * 1024) as usize);
        file.read_to_end(&mut buf).map_err(|e| IngestError::Corrupt {
            format: Some(format),
            detail: format!("I/O-Fehler beim Lesen von {}: {}", name, e),
        })?;

        match name.as_str() {
            "mimetype" => mimetype_bytes = Some(buf),
            "meta.xml" => meta_xml_bytes = Some(buf),
            "content.xml" => content_xml_bytes = Some(buf),
            "META-INF/manifest.xml" => manifest_bytes = Some(buf),
            _ => {}
        }
    }

    // Mimetype validation
    if let Some(ref mime_raw) = mimetype_bytes {
        let mime_str = std::str::from_utf8(mime_raw).unwrap_or_default().trim();
        if !mime_str.is_empty() && !mime_str.contains("opendocument") && !mime_str.contains("oasis") {
            return Err(IngestError::Corrupt {
                format: Some(format),
                detail: format!("Ungültiger ODF mimetype: {}", mime_str),
            });
        }
    }

    // Verschlüsselungsprüfung
    if let Some(ref manifest) = manifest_bytes {
        if check_manifest_encrypted(manifest) {
            return Err(IngestError::Encrypted {
                format: Some(format),
            });
        }
    }

    let mut ir = DocumentIr::new(format);

    if let Some(ref meta_bytes) = meta_xml_bytes {
        if let Ok(meta_data) = meta::parse_meta(meta_bytes, ctx) {
            ir.meta = meta_data;
        }
    }

    let content_bytes = content_xml_bytes.ok_or_else(|| IngestError::Corrupt {
        format: Some(format),
        detail: "content.xml fehlt im ODF-Archiv".into(),
    })?;

    if format == FormatId::Odp {
        presentation::parse_presentation(&content_bytes, ctx, &mut ir)?;
    } else {
        content::parse_content(format, &content_bytes, ctx, &mut ir)?;
    }

    if ir.blocks.is_empty() {
        return Err(IngestError::NoExtractableText { format });
    }

    Ok(ir)
}

fn check_manifest_encrypted(manifest_bytes: &[u8]) -> bool {
    let s = std::str::from_utf8(manifest_bytes).unwrap_or_default();
    s.contains("manifest:encryption-data") || s.contains("encryption-data")
}
