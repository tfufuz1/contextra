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
// ZWECK: Word DOCX Extractor (Docx) (WP-ING-04).
// INVARIANTEN: FILE-CONTEXT Kopf, DocumentExtractor-Impl, Zero-Panic, Fail-Closed bei Beschädigungen.
// STAND: TS:2026-10-31T00:00:00Z

pub mod document;
pub mod notes;
pub mod numbering;
pub mod styles;

use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::{BlockKind, DocMeta, DocumentIr, ExtractWarning, Locator, WarningCode};
use crate::safe::opc::OpcPackage;
use crate::safe::xml::SafeXmlReader;
use quick_xml::events::Event;

#[derive(Debug, Default)]
pub struct DocxExtractor;

impl DocxExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[FormatId::Docx];

impl DocumentExtractor for DocxExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        let mut package = OpcPackage::open(input, ctx.budget)?;

        let mut ir = DocumentIr::new(format);

        // 1. Core Metadata
        if package.has_part("docProps/core.xml") {
            if let Ok(bytes) = package.read_part("docProps/core.xml", ctx.budget) {
                if let Ok(meta) = parse_core_properties(&bytes, ctx) {
                    ir.meta = meta;
                }
            }
        }

        // 2. Parse Relationships
        let rels =
            package.parse_relationships("_rels/.rels", ctx.limits.max_xml_depth, ctx.budget)?;
        let mut main_part = "word/document.xml".to_string();
        for rel in rels {
            if rel.rel_type.ends_with("/officeDocument") {
                main_part = rel.target.trim_start_matches('/').to_string();
            }
        }

        let doc_rels_path = format!(
            "{}/_rels/{}.rels",
            get_parent_dir(&main_part),
            get_filename(&main_part)
        );
        let doc_rels =
            package.parse_relationships(&doc_rels_path, ctx.limits.max_xml_depth, ctx.budget)?;

        let mut styles_part = None;
        let mut numbering_part = None;
        let mut footnotes_part = None;
        let mut endnotes_part = None;
        let mut comments_part = None;
        let mut header_parts = Vec::new();
        let mut footer_parts = Vec::new();

        for rel in doc_rels {
            let target = OpcPackage::resolve_target(&main_part, &rel.target);
            if rel.rel_type.ends_with("/styles") {
                styles_part = Some(target);
            } else if rel.rel_type.ends_with("/numbering") {
                numbering_part = Some(target);
            } else if rel.rel_type.ends_with("/footnotes") {
                footnotes_part = Some(target);
            } else if rel.rel_type.ends_with("/endnotes") {
                endnotes_part = Some(target);
            } else if rel.rel_type.ends_with("/comments") {
                comments_part = Some(target);
            } else if rel.rel_type.ends_with("/header") {
                header_parts.push(target);
            } else if rel.rel_type.ends_with("/footer") {
                footer_parts.push(target);
            }
        }

        // 3. Parse Styles
        let styles = if let Some(ref path) = styles_part {
            if package.has_part(path) {
                let bytes = package.read_part(path, ctx.budget)?;
                styles::StyleMap::parse(&bytes, ctx.limits.max_xml_depth, ctx.budget)?
            } else {
                styles::StyleMap::default()
            }
        } else {
            styles::StyleMap::default()
        };

        // 4. Parse Numbering
        let numbering = if let Some(ref path) = numbering_part {
            if package.has_part(path) {
                let bytes = package.read_part(path, ctx.budget)?;
                numbering::NumMap::parse(&bytes, ctx.limits.max_xml_depth, ctx.budget)?
            } else {
                numbering::NumMap::default()
            }
        } else {
            numbering::NumMap::default()
        };

        // 5. Parse Main Document
        if !package.has_part(&main_part) {
            return Err(IngestError::Corrupt {
                format: Some(format),
                detail: format!("Hauptdokument-Part {main_part} nicht im Archiv"),
            });
        }

        let main_bytes = package.read_part(&main_part, ctx.budget)?;
        let parse_res =
            document::parse_document_xml(&main_bytes, &styles, &numbering, &mut ir, ctx)?;

        // 6. Parse Footnotes & Endnotes
        if let Some(ref path) = footnotes_part {
            if package.has_part(path) {
                let bytes = package.read_part(path, ctx.budget)?;
                let fn_notes = notes::parse_footnotes_or_endnotes(
                    &bytes,
                    false,
                    ctx.limits.max_xml_depth,
                    ctx.budget,
                )?;
                for note in fn_notes {
                    ir.push(
                        BlockKind::Note {
                            kind: note.kind,
                            text: note.text,
                        },
                        Locator::default(),
                    );
                }
            }
        }

        if let Some(ref path) = endnotes_part {
            if package.has_part(path) {
                let bytes = package.read_part(path, ctx.budget)?;
                let en_notes = notes::parse_footnotes_or_endnotes(
                    &bytes,
                    true,
                    ctx.limits.max_xml_depth,
                    ctx.budget,
                )?;
                for note in en_notes {
                    ir.push(
                        BlockKind::Note {
                            kind: note.kind,
                            text: note.text,
                        },
                        Locator::default(),
                    );
                }
            }
        }

        // 7. Parse Comments
        if let Some(ref path) = comments_part {
            if package.has_part(path) {
                let bytes = package.read_part(path, ctx.budget)?;
                let comm_notes = notes::parse_comments(&bytes, ctx)?;
                for note in comm_notes {
                    ir.push(
                        BlockKind::Note {
                            kind: note.kind,
                            text: note.text,
                        },
                        Locator::default(),
                    );
                }
            }
        }

        // 8. Parse Headers & Footers
        for path in header_parts {
            if package.has_part(&path) {
                let bytes = package.read_part(&path, ctx.budget)?;
                if let Some(hf_note) = notes::parse_header_footer(&bytes, false, ctx)? {
                    ir.push(
                        BlockKind::Note {
                            kind: hf_note.kind,
                            text: hf_note.text,
                        },
                        Locator::default(),
                    );
                }
            }
        }

        for path in footer_parts {
            if package.has_part(&path) {
                let bytes = package.read_part(&path, ctx.budget)?;
                if let Some(hf_note) = notes::parse_header_footer(&bytes, true, ctx)? {
                    ir.push(
                        BlockKind::Note {
                            kind: hf_note.kind,
                            text: hf_note.text,
                        },
                        Locator::default(),
                    );
                }
            }
        }

        // 9. Warnings
        if parse_res.hidden_content_skipped {
            ir.warnings.push(ExtractWarning {
                code: WarningCode::HiddenContentSkipped,
                detail: "Ausgeblendeter Text (w:vanish) wurde übersprungen".into(),
            });
        }

        if parse_res.embedded_object_count > 0 {
            ir.warnings.push(ExtractWarning {
                code: WarningCode::EmbeddedObjectSkipped,
                detail: format!(
                    "{} eingebettete Objekte/Bilder übersprungen",
                    parse_res.embedded_object_count
                ),
            });
        }

        if ir.blocks.is_empty() {
            return Err(IngestError::NoExtractableText { format });
        }

        Ok(ir)
    }
}

fn parse_core_properties(bytes: &[u8], ctx: &ExtractContext<'_>) -> Result<DocMeta, IngestError> {
    let mut reader = SafeXmlReader::new(bytes, ctx.limits.max_xml_depth);
    let mut meta = DocMeta::default();

    let mut current_tag = String::new();

    loop {
        match reader.next_event(ctx.budget)? {
            Event::Start(ref e) => {
                let local = e.local_name();
                current_tag = String::from_utf8_lossy(local.as_ref()).to_string();
            }
            Event::Text(ref e) => {
                if let Ok(txt) = e.unescape() {
                    let trimmed = txt.trim().to_string();
                    if !trimmed.is_empty() {
                        match current_tag.as_str() {
                            "title" => meta.title = Some(trimmed),
                            "language" => meta.language = Some(trimmed),
                            "creator" => {
                                if ctx.policy.keep_personal_metadata {
                                    meta.authors.push(trimmed);
                                }
                            }
                            "created" => meta.created = Some(trimmed),
                            "modified" => meta.modified = Some(trimmed),
                            _ => {}
                        }
                    }
                }
            }
            Event::End(_) => {
                current_tag.clear();
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(meta)
}

fn get_parent_dir(path: &str) -> &str {
    match path.rfind('/') {
        Some(idx) => &path[..idx],
        None => "",
    }
}

fn get_filename(path: &str) -> &str {
    match path.rfind('/') {
        Some(idx) => &path[idx.saturating_add(1)..],
        None => path,
    }
}
