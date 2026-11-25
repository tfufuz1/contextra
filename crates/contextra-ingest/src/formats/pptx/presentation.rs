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
// ZWECK: Parser für PPTX-Präsentationsstruktur (ppt/presentation.xml) und Slide-Reihenfolge.
// INVARIANTEN: Zero-Panic, strikte Befolgung von p:sldIdLst, keine Unsafe-Islands.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ResourceBudget;
use crate::safe::opc::{resolve_opc_target, OpcRels};
use crate::safe::xml::parse_xml;

/// Metadaten einer Folie aus presentation.xml.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlideRef {
    pub r_id: String,
    pub slide_path: String,
    pub is_hidden: bool,
}

/// Parst ppt/presentation.xml und liest die exakte Folienreihenfolge aus p:sldIdLst.
pub fn parse_presentation_slides(
    pres_xml_bytes: &[u8],
    pres_rels: &OpcRels,
    budget: &ResourceBudget,
) -> Result<Vec<SlideRef>, IngestError> {
    let root = parse_xml(pres_xml_bytes, budget)?;
    let mut slides = Vec::new();

    // Suche p:sldIdLst unter p:presentation
    if let Some(sld_id_lst) = root.find_first_child("sldIdLst") {
        for child in sld_id_lst.child_elements() {
            if child.local_name() == "sldId" {
                let rel_id = child
                    .attr("r:id")
                    .or_else(|| child.attr("id"))
                    .unwrap_or("");
                if !rel_id.is_empty() {
                    if let Some(target) = pres_rels.get_target(rel_id) {
                        let slide_path = resolve_opc_target("ppt/presentation.xml", target);
                        let is_hidden =
                            child.attr("show").is_some_and(|v| v == "0" || v == "false");
                        slides.push(SlideRef {
                            r_id: rel_id.to_string(),
                            slide_path,
                            is_hidden,
                        });
                    }
                }
            }
        }
    }

    Ok(slides)
}
