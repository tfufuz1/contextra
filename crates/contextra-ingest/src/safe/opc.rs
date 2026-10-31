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
// ZWECK: Open Packaging Convention (OPC) Beziehungsdaten (.rels) & Pfad-Auflösung (P00B).
// INVARIANTEN: Zero-Panic, Determinismus durch BTreeMap, sichere Pfad-Normierung.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ResourceBudget;
use crate::safe::xml::parse_xml;
use std::collections::BTreeMap;

/// Eine einzelne OPC-Beziehung (<Relationship Id="..." Type="..." Target="..." />).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpcRelationship {
    pub id: String,
    pub rel_type: String,
    pub target: String,
    pub target_mode: Option<String>,
}

/// Sammlung aller Beziehungen einer .rels-Datei.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OpcRels {
    pub rels_by_id: BTreeMap<String, OpcRelationship>,
}

impl OpcRels {
    /// Parst .rels-XML-Bytes in eine `OpcRels`-Struktur.
    pub fn parse(xml_bytes: &[u8], budget: &ResourceBudget) -> Result<Self, IngestError> {
        let root = parse_xml(xml_bytes, budget)?;
        let mut rels_by_id = BTreeMap::new();

        for child in root.child_elements() {
            if child.local_name() == "Relationship" {
                if let (Some(id), Some(rel_type), Some(target)) =
                    (child.attr("Id"), child.attr("Type"), child.attr("Target"))
                {
                    let rel = OpcRelationship {
                        id: id.to_string(),
                        rel_type: rel_type.to_string(),
                        target: target.to_string(),
                        target_mode: child.attr("TargetMode").map(|s| s.to_string()),
                    };
                    rels_by_id.insert(id.to_string(), rel);
                }
            }
        }

        Ok(Self { rels_by_id })
    }

    /// Sucht ein Ziel (`Target`) anhand seiner Beziehungs-ID (`rId1` etc.).
    pub fn get_target(&self, id: &str) -> Option<&str> {
        self.rels_by_id.get(id).map(|r| r.target.as_str())
    }

    /// Sucht alle Zielpfade, deren Beziehungs-Typ das angegebene Suffix enthält.
    pub fn find_targets_by_type_suffix(&self, type_suffix: &str) -> Vec<&str> {
        let mut result = Vec::new();
        for rel in self.rels_by_id.values() {
            if rel.rel_type.ends_with(type_suffix) {
                result.push(rel.target.as_str());
            }
        }
        result
    }
}

/// Löst einen relativen OPC-Zielpfad basierend auf dem Pfad des Ursprungsdokuments auf.
pub fn resolve_opc_target(base_file_path: &str, target: &str) -> String {
    let target_clean = target.replace('\\', "/");
    if target_clean.starts_with('/') {
        return target_clean.trim_start_matches('/').to_string();
    }

    let mut parts: Vec<&str> = Vec::new();
    if let Some((dir, _)) = base_file_path.rsplit_once('/') {
        for seg in dir.split('/') {
            if !seg.is_empty() && seg != "." {
                parts.push(seg);
            }
        }
    }

    for seg in target_clean.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            let _ = parts.pop();
        } else {
            parts.push(seg);
        }
    }

    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_opc_target_cases() {
        assert_eq!(
            resolve_opc_target("ppt/presentation.xml", "slides/slide1.xml"),
            "ppt/slides/slide1.xml"
        );
        assert_eq!(
            resolve_opc_target("ppt/slides/slide1.xml", "../notesSlides/notesSlide1.xml"),
            "ppt/notesSlides/notesSlide1.xml"
        );
        assert_eq!(
            resolve_opc_target("ppt/presentation.xml", "/ppt/slides/slide2.xml"),
            "ppt/slides/slide2.xml"
        );
    }
}
