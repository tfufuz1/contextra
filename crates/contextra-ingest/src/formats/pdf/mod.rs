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
// ZWECK: Robustes PDF-Extraktionsmodul mit lopdf (WP ING-08).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, catch_unwind Schutz, keine unsafe operations, Fail-Closed.
// LIMITATION: Unsichtbarer Text (Render-Mode 3, weiß-auf-weiß) wird mit lopdf-Textextraktion nicht unterschieden;
//             kein Anspruch auf Erkennung.
// STAND: TS:2026-10-31T00:00:00Z

pub mod text_processing;

use std::collections::{BTreeMap, BTreeSet};
use std::panic::{catch_unwind, AssertUnwindSafe};

use lopdf::decode_text_string;

use crate::error::{IngestError, LimitKind};
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::{
    BlockKind, Boundary, DocMeta, DocumentIr, ExtractWarning, Locator, NoteKind, WarningCode,
};
use text_processing::{
    analyze_headers_footers, clean_page_text, join_end_of_line_hyphens, normalize_line_digits,
    reconstruct_paragraphs,
};

/// PDF Extractor basierend auf `lopdf`.
#[derive(Debug, Default)]
pub struct PdfExtractor;

impl PdfExtractor {
    /// Erzeugt eine neue `PdfExtractor`-Instanz.
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[FormatId::Pdf];

impl DocumentExtractor for PdfExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        // 1. Eingabegröße vorab prüfen
        if u64::try_from(input.len()).unwrap_or(u64::MAX) > ctx.limits.max_document_bytes {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::DocumentBytes,
                limit: ctx.limits.max_document_bytes,
                observed: u64::try_from(input.len()).unwrap_or(u64::MAX),
            });
        }

        // 2. Dokument laden mit Panik-Isolierung gegen defekte PDF-Strukturen
        let mut doc = match catch_unwind(AssertUnwindSafe(|| lopdf::Document::load_mem(input))) {
            Ok(Ok(d)) => d,
            Ok(Err(e)) => {
                return Err(IngestError::Corrupt {
                    format: Some(format),
                    detail: format!("lopdf Parsing-Fehler: {e}"),
                });
            }
            Err(_) => {
                return Err(IngestError::Corrupt {
                    format: Some(format),
                    detail: "lopdf Panik beim Laden des Dokuments abgefangen".into(),
                });
            }
        };

        // 3. Verschlüsselung handhaben
        if doc.is_encrypted() {
            // Versuch mit leerem Benutzerpasswort
            let decrypt_res = catch_unwind(AssertUnwindSafe(|| doc.decrypt("")));
            match decrypt_res {
                Ok(Ok(())) => {}
                _ => {
                    return Err(IngestError::Encrypted {
                        format: Some(format),
                    });
                }
            }
        }

        let mut ir = DocumentIr::new(format);

        // 4. Metadaten extrahieren
        extract_metadata(&doc, ctx, &mut ir.meta);

        // 5. Seiten-Baum abfragen & Seitenanzahl prüfen
        let pages = doc.get_pages();
        let total_pages = pages.len();
        ir.meta.page_count = Some(u32::try_from(total_pages).unwrap_or(u32::MAX));

        if u64::try_from(total_pages).unwrap_or(u64::MAX) > u64::from(ctx.limits.max_pages) {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::Pages,
                limit: u64::from(ctx.limits.max_pages),
                observed: u64::try_from(total_pages).unwrap_or(u64::MAX),
            });
        }

        if total_pages == 0 {
            return Err(IngestError::NoExtractableText { format });
        }

        // Budget für Seiten verbuchen
        ctx.budget
            .charge_pages(u64::try_from(total_pages).unwrap_or(u64::MAX))?;

        // 6. Text pro Seite extrahieren und vorverarbeiten
        let mut raw_page_lines: BTreeMap<u32, Vec<String>> = BTreeMap::new();
        let mut page_text_extracted = false;

        for &page_num in pages.keys() {
            let page_text_res = catch_unwind(AssertUnwindSafe(|| doc.extract_text(&[page_num])));

            match page_text_res {
                Ok(Ok(raw_text)) => {
                    let cleaned = clean_page_text(&raw_text);
                    let hyphen_joined = join_end_of_line_hyphens(&cleaned);

                    let lines: Vec<String> = hyphen_joined
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty())
                        .collect();

                    if !lines.is_empty() {
                        page_text_extracted = true;
                    }

                    raw_page_lines.insert(page_num, lines);
                }
                _ => {
                    ir.warnings.push(ExtractWarning {
                        code: WarningCode::FeatureLimited,
                        detail: format!("Textextraktion auf Seite {page_num} fehlgeschlagen"),
                    });
                    raw_page_lines.insert(page_num, Vec::new());
                }
            }
        }

        // 7. Wiederkehrende Kopf-/Fußzeilen analysieren
        let all_page_lines: Vec<Vec<String>> = raw_page_lines.values().cloned().collect();
        let hf_analysis = analyze_headers_footers(&all_page_lines);

        let header_norms: BTreeSet<String> = hf_analysis.detected_headers.into_iter().collect();
        let footer_norms: BTreeSet<String> = hf_analysis.detected_footers.into_iter().collect();

        // 8. Lesezeichen / Outlines extrahieren
        let headings_by_page = extract_outlines(&doc);

        // Header / Footer bereits einmalig ausgegeben Tracken
        let mut emitted_headers = BTreeSet::new();
        let mut emitted_footers = BTreeSet::new();

        let mut total_text_bytes_extracted: u64 = 0;

        // 9. Blöcke pro Seite aufbauen
        for (&page_num, lines) in &raw_page_lines {
            // Seiten-Grenzmarkierung einfügen
            ir.push(
                BlockKind::Boundary(Boundary::Page { number: page_num }),
                Locator {
                    page: Some(page_num),
                    ..Default::default()
                },
            );

            // Lesezeichen/Heading-Blöcke am Anfang der Seite platzieren
            if let Some(headings) = headings_by_page.get(&page_num) {
                for (level, title) in headings {
                    ir.push(
                        BlockKind::Heading {
                            level: *level,
                            text: title.clone(),
                        },
                        Locator {
                            page: Some(page_num),
                            ..Default::default()
                        },
                    );
                }
            }

            let mut page_filtered_lines = Vec::new();

            for (idx, line) in lines.iter().enumerate() {
                let norm = normalize_line_digits(line);
                let is_top = idx == 0;
                let is_bottom = idx + 1 == lines.len();

                let is_header = is_top && header_norms.contains(&norm);
                let is_footer = is_bottom && footer_norms.contains(&norm);

                if is_header {
                    if ctx.policy.include_headers_footers && !emitted_headers.contains(line) {
                        emitted_headers.insert(line.clone());
                        ir.push(
                            BlockKind::Note {
                                kind: NoteKind::Header,
                                text: line.clone(),
                            },
                            Locator {
                                page: Some(page_num),
                                ..Default::default()
                            },
                        );
                    }
                    continue;
                }

                if is_footer {
                    if ctx.policy.include_headers_footers && !emitted_footers.contains(line) {
                        emitted_footers.insert(line.clone());
                        ir.push(
                            BlockKind::Note {
                                kind: NoteKind::Footer,
                                text: line.clone(),
                            },
                            Locator {
                                page: Some(page_num),
                                ..Default::default()
                            },
                        );
                    }
                    continue;
                }

                page_filtered_lines.push(line.as_str());
            }

            if page_filtered_lines.is_empty() {
                ir.warnings.push(ExtractWarning {
                    code: WarningCode::FeatureLimited,
                    detail: format!("Seite {page_num} enthält keinen extrahierbaren Text"),
                });
            } else {
                let page_joined = page_filtered_lines.join("\n");
                let paragraphs = reconstruct_paragraphs(&page_joined);

                for para in paragraphs {
                    let para_len = u64::try_from(para.len()).unwrap_or(u64::MAX);
                    ctx.budget.charge_text(para_len)?;
                    total_text_bytes_extracted =
                        total_text_bytes_extracted.saturating_add(para_len);

                    ir.push(
                        BlockKind::Paragraph { text: para },
                        Locator {
                            page: Some(page_num),
                            ..Default::default()
                        },
                    );
                }
            }
        }

        // 10. Kein extrahierbarer Text im gesamten Dokument
        let non_boundary_blocks = ir
            .blocks
            .iter()
            .any(|b| !matches!(b.kind, BlockKind::Boundary(_)));
        if !page_text_extracted || !non_boundary_blocks {
            return Err(IngestError::NoExtractableText { format });
        }

        Ok(ir)
    }
}

/// Extrahiert Metadaten aus dem `/Info`-Dictionary des PDF-Dokuments.
fn extract_metadata(doc: &lopdf::Document, ctx: &ExtractContext<'_>, meta: &mut DocMeta) {
    if let Ok(info_obj) = doc.trailer.get(b"Info") {
        if let Ok((_, lopdf::Object::Dictionary(dict))) = doc.dereference(info_obj) {
            if let Ok(title_obj) = dict.get(b"Title") {
                if let Ok(title) = decode_text_string(title_obj) {
                    meta.title = Some(title);
                }
            }

            if ctx.policy.keep_personal_metadata {
                if let Ok(author_obj) = dict.get(b"Author") {
                    if let Ok(author) = decode_text_string(author_obj) {
                        meta.authors.push(author);
                    }
                }
                if let Ok(creator_obj) = dict.get(b"Creator") {
                    if let Ok(creator) = decode_text_string(creator_obj) {
                        meta.extra.insert("Creator".into(), creator);
                    }
                }
            }
        }
    }
}

/// Extrahiert Outlines/Lesezeichen aus dem Inhaltsverzeichnis des PDF-Dokuments.
fn extract_outlines(doc: &lopdf::Document) -> BTreeMap<u32, Vec<(u8, String)>> {
    let mut map: BTreeMap<u32, Vec<(u8, String)>> = BTreeMap::new();

    let toc_res = catch_unwind(AssertUnwindSafe(|| doc.get_toc()));
    if let Ok(Ok(toc)) = toc_res {
        for entry in toc.toc {
            let page = u32::try_from(entry.page).unwrap_or(1);
            let level = u8::try_from(entry.level).unwrap_or(1).clamp(1, 6);
            map.entry(page).or_default().push((level, entry.title));
        }
    }

    map
}
