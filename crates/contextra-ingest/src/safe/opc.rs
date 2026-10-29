// FILE-CONTEXT
// ZWECK: Open Packaging Convention (OPC) Paket- & Beziehungsauflösung (P00B/P04).
// INVARIANTEN: Zero-Panic, deterministische Pfadauflösung, Fail-Closed bei ungültigen Bezügen.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ResourceBudget;
use crate::safe::archive::SafeArchive;
use crate::safe::xml::SafeXmlReader;
use quick_xml::events::Event;

/// OPC Relationship mapping ID or Type -> Target path.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct OpcRelationship {
    pub id: String,
    pub rel_type: String,
    pub target: String,
}

/// Helper for OPC package navigation.
#[derive(Debug)]
pub struct OpcPackage<'a> {
    archive: SafeArchive<'a>,
}

impl<'a> OpcPackage<'a> {
    /// Opens an OPC package from bytes.
    pub fn open(bytes: &'a [u8], budget: &ResourceBudget) -> Result<Self, IngestError> {
        let archive = SafeArchive::new(bytes, budget)?;
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
        if target.starts_with('/') {
            return target.trim_start_matches('/').to_string();
        }

        let mut parts: Vec<&str> = base_part_path
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        // Remove file name from base path
        if !parts.is_empty() {
            parts.pop();
        }

        for segment in target.split('/') {
            match segment {
                "" | "." => {}
                ".." => {
                    let _ = parts.pop();
                }
                seg => parts.push(seg),
            }
        }

        parts.join("/")
    }
}
