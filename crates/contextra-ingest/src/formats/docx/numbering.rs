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
// ZWECK: Parsing von word/numbering.xml zur Listenauflösung (aufzählend vs. nummeriert) (WP-ING-04).
// INVARIANTEN: Zero-Panic, deterministische Zuordnung, kein unsafe.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ResourceBudget;
use crate::safe::xml::SafeXmlReader;
use quick_xml::events::Event;
use std::collections::BTreeMap;

/// Zuordnungstabelle für Listenformatierungen.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct NumMap {
    /// Maps numId -> abstractNumId
    num_to_abstract: BTreeMap<String, String>,
    /// Maps (abstractNumId, ilvl) -> numFmt (e.g. "bullet", "decimal")
    fmt_map: BTreeMap<(String, u8), String>,
}

impl NumMap {
    /// Parsen der numbering.xml-Datei.
    pub fn parse(
        bytes: &[u8],
        max_xml_depth: u32,
        budget: &ResourceBudget,
    ) -> Result<Self, IngestError> {
        let mut reader = SafeXmlReader::new(bytes, max_xml_depth);
        let mut num_to_abstract = BTreeMap::new();
        let mut fmt_map = BTreeMap::new();

        let mut current_abstract_id: Option<String> = None;
        let mut current_ilvl: Option<u8> = None;
        let mut current_num_id: Option<String> = None;

        loop {
            match reader.next_event(budget)? {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"abstractNum" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"abstractNumId" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        current_abstract_id = Some(v);
                                    }
                                }
                            }
                        }
                        b"lvl" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"ilvl" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        if let Ok(lvl) = v.parse::<u8>() {
                                            current_ilvl = Some(lvl);
                                        }
                                    }
                                }
                            }
                        }
                        b"numFmt" => {
                            if let (Some(ref abs_id), Some(ilvl)) =
                                (&current_abstract_id, current_ilvl)
                            {
                                for attr in e.attributes().flatten() {
                                    if attr.key.local_name().as_ref() == b"val" {
                                        if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                            fmt_map.insert((abs_id.clone(), ilvl), v);
                                        }
                                    }
                                }
                            }
                        }
                        b"num" => {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"numId" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        current_num_id = Some(v);
                                    }
                                }
                            }
                        }
                        b"abstractNumId" => {
                            if let Some(ref num_id) = current_num_id {
                                for attr in e.attributes().flatten() {
                                    if attr.key.local_name().as_ref() == b"val" {
                                        if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                            num_to_abstract.insert(num_id.clone(), v);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Event::End(ref e) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"abstractNum" => {
                            current_abstract_id = None;
                        }
                        b"lvl" => {
                            current_ilvl = None;
                        }
                        b"num" => {
                            current_num_id = None;
                        }
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(Self {
            num_to_abstract,
            fmt_map,
        })
    }

    /// Bestimmt, ob eine Liste an `numId` und `ilvl` ordered (nummeriert) ist.
    /// Liefert `false` bei "bullet" oder "none", sonst `true`.
    pub fn is_ordered(&self, num_id: &str, ilvl: u8) -> bool {
        if let Some(abs_id) = self.num_to_abstract.get(num_id) {
            if let Some(num_fmt) = self.fmt_map.get(&(abs_id.clone(), ilvl)) {
                return !num_fmt.eq_ignore_ascii_case("bullet")
                    && !num_fmt.eq_ignore_ascii_case("none");
            }
        }
        // Standard: Nummeriert wenn num_id != 0
        num_id != "0"
    }
}
