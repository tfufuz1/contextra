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
// ZWECK: PowerPoint PresentationML Extractor (PPTX, PPSX, POTX, PPTM) (WP-ING-05).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, ResourceBudget-Absicherung, BTreeMap.
// STAND: TS:2026-10-31T00:00:00Z

pub mod presentation;
pub mod slide;

use crate::canonical::sanitize_text;
use crate::error::IngestError;
use crate::extractor::{DocumentExtractor, ExtractContext};
use crate::format::FormatId;
use crate::ir::DocumentIr;
use crate::safe::archive::SafeZipReader;
use crate::safe::opc::OpcRels;
use crate::safe::xml::parse_xml;

#[derive(Debug, Default)]
pub struct PptxExtractor;

impl PptxExtractor {
    pub fn new() -> Self {
        Self
    }
}

const FORMATS: &[FormatId] = &[FormatId::Pptx];

impl DocumentExtractor for PptxExtractor {
    fn formats(&self) -> &'static [FormatId] {
        FORMATS
    }

    fn extract(
        &self,
        format: FormatId,
        input: &[u8],
        ctx: &ExtractContext<'_>,
    ) -> Result<DocumentIr, IngestError> {
        let mut zip = SafeZipReader::new(input)?;
        let mut ir = DocumentIr::new(format);

        // Core Metadaten (docProps/core.xml) extrahieren
        extract_core_metadata(&mut zip, ctx, &mut ir);

        // Beziehungen von ppt/presentation.xml lesen
        let pres_rels = if zip.contains_file("ppt/_rels/presentation.xml.rels") {
            let rels_bytes = zip.read_file("ppt/_rels/presentation.xml.rels", ctx.budget)?;
            OpcRels::parse(&rels_bytes, ctx.budget)?
        } else {
            OpcRels::default()
        };

        // Folienreihenfolge aus ppt/presentation.xml (p:sldIdLst) lesen
        let pres_bytes = zip.read_file("ppt/presentation.xml", ctx.budget)?;
        let slides = presentation::parse_presentation_slides(&pres_bytes, &pres_rels, ctx.budget)?;

        #[allow(clippy::cast_possible_truncation)]
        let total_slides = slides.len() as u32;
        ir.meta.page_count = Some(total_slides);

        for (idx, slide_ref) in slides.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let slide_num = (idx as u32).saturating_add(1);

            // Seitenbudget prüfen
            ctx.budget.charge_pages(1)?;

            // Folieninhalte extrahieren
            let slide_res = slide::parse_slide_content(
                &mut zip,
                &slide_ref.slide_path,
                slide_num,
                slide_ref.is_hidden,
                ctx.policy,
                ctx.budget,
            )?;

            for (kind, locator) in slide_res.blocks {
                ir.push(kind, locator);
            }
            ir.warnings.extend(slide_res.warnings);
        }

        Ok(ir)
    }
}

fn extract_core_metadata(
    zip: &mut SafeZipReader<'_>,
    ctx: &ExtractContext<'_>,
    ir: &mut DocumentIr,
) {
    if zip.contains_file("docProps/core.xml") {
        if let Ok(xml_bytes) = zip.read_file("docProps/core.xml", ctx.budget) {
            if let Ok(root) = parse_xml(&xml_bytes, ctx.budget) {
                if let Some(title_elem) = root.find_first_child("title") {
                    let title_text = title_elem.collect_text();
                    if !title_text.trim().is_empty() {
                        let (clean, _) = sanitize_text(title_text.trim());
                        ir.meta.title = Some(clean);
                    }
                }
                if ctx.policy.keep_personal_metadata {
                    if let Some(creator_elem) = root.find_first_child("creator") {
                        let creator_text = creator_elem.collect_text();
                        if !creator_text.trim().is_empty() {
                            let (clean, _) = sanitize_text(creator_text.trim());
                            ir.meta.authors.push(clean);
                        }
                    }
                }
            }
        }
    }
}
