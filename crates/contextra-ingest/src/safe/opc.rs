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
use crate::safe::archive::SafeArchive;
use crate::safe::xml::{parse_xml, SafeXmlReader};
use quick_xml::events::Event;
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

/// Helper for OPC package navigation.
pub struct OpcPackage<'a> {
    archive: SafeArchive<'a>,
}

impl<'a> OpcPackage<'a> {
    /// Opens an OPC package from bytes.
    pub fn open(bytes: &'a [u8], _budget: &ResourceBudget) -> Result<Self, IngestError> {
        let archive = SafeArchive::new(bytes)?;
        Ok(Self { archive })
    }

    /// Checks if a part exists in the package.
    pub fn has_part(&mut self, path: &str) -> bool {
        self.archive.has_entry(path)
    }

    /// Reads raw bytes for a part path.
    pub fn read_part(
        &mut self,
        path: &str,
        budget: &ResourceBudget,
    ) -> Result<Vec<u8>, IngestError> {
        self.archive.read_entry(path, budget)
    }

    /// Parses relationships XML at `rels_path` (e.g., `_rels/.rels` or `word/_rels/document.xml.rels`).
    pub fn parse_relationships(
        &mut self,
        rels_path: &str,
        max_xml_depth: u32,
        budget: &ResourceBudget,
    ) -> Result<Vec<OpcRelationship>, IngestError> {
        if !self.has_part(rels_path) {
            return Ok(Vec::new());
        }

        let bytes = self.read_part(rels_path, budget)?;
        let mut reader = SafeXmlReader::new(&bytes, max_xml_depth);
        let mut rels = Vec::new();

        loop {
            match reader.next_event(budget)? {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    if e.local_name().as_ref() == b"Relationship" {
                        let mut id = String::new();
                        let mut rel_type = String::new();
                        let mut target = String::new();

                        for attr in e.attributes().flatten() {
                            let val = match String::from_utf8(attr.value.to_vec()) {
                                Ok(v) => v,
                                Err(_) => continue,
                            };
                            match attr.key.local_name().as_ref() {
                                b"Id" => id = val,
                                b"Type" => rel_type = val,
                                b"Target" => target = val,
                                _ => {}
                            }
                        }

                        if !rel_type.is_empty() && !target.is_empty() {
                            rels.push(OpcRelationship {
                                id,
                                rel_type,
                                target,
                                target_mode: None,
                            });
                        }
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(rels)
    }

    /// Finds main document part path from `_rels/.rels`. Defaults to `word/document.xml`.
    pub fn main_document_part(
        &mut self,
        max_xml_depth: u32,
        budget: &ResourceBudget,
    ) -> Result<String, IngestError> {
        let rels = self.parse_relationships("_rels/.rels", max_xml_depth, budget)?;
        for rel in rels {
            if rel.rel_type.ends_with("/officeDocument") {
                let target = rel.target.trim_start_matches('/');
                return Ok(target.to_string());
            }
        }

        if self.has_part("word/document.xml") {
            Ok("word/document.xml".to_string())
        } else {
            Err(IngestError::Corrupt {
                format: None,
                detail: "Hauptdokument-Part (document.xml) nicht gefunden".into(),
            })
        }
    }

    /// Resolves target path for a relationship relative to a base part directory.
    pub fn resolve_target(base_part_path: &str, target: &str) -> String {
        resolve_opc_target(base_part_path, target)
    }
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
