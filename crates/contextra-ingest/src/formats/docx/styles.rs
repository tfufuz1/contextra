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
// ZWECK: Parsing von word/styles.xml zur Erkennung von Überschriftenhierarchien (WP-ING-04).
// INVARIANTEN: Zero-Panic, deterministische Style-Vererbung (max. 16 Ebenen), kein unsafe.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ResourceBudget;
use crate::safe::xml::SafeXmlReader;
use quick_xml::events::Event;
use std::collections::BTreeMap;

/// Metadaten eines Absatzstils in styles.xml.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct StyleInfo {
    pub style_id: String,
    pub name: String,
    pub based_on: Option<String>,
    pub outline_lvl: Option<u8>,
}

/// Sammlung aller Stile in styles.xml mit Vererbungsauswertung.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct StyleMap {
    styles: BTreeMap<String, StyleInfo>,
}

impl StyleMap {
    /// Parsen der styles.xml-Datei.
    pub fn parse(
        bytes: &[u8],
        max_xml_depth: u32,
        budget: &ResourceBudget,
    ) -> Result<Self, IngestError> {
        let mut reader = SafeXmlReader::new(bytes, max_xml_depth);
        let mut styles = BTreeMap::new();

        let mut current_style: Option<StyleInfo> = None;
        let mut in_p_pr = false;

        loop {
            match reader.next_event(budget)? {
                Event::Start(ref e) | Event::Empty(ref e) => {
                    let local_name = e.local_name();
                    match local_name.as_ref() {
                        b"style" => {
                            let mut style_id = String::new();
                            let mut style_type = String::new();

                            for attr in e.attributes().flatten() {
                                let val = match String::from_utf8(attr.value.to_vec()) {
                                    Ok(v) => v,
                                    Err(_) => continue,
                                };
                                match attr.key.local_name().as_ref() {
                                    b"styleId" => style_id = val,
                                    b"type" => style_type = val,
                                    _ => {}
                                }
                            }

                            if style_type.is_empty() || style_type == "paragraph" {
                                current_style = Some(StyleInfo {
                                    style_id,
                                    ..Default::default()
                                });
                            }
                        }
                        b"pPr" => {
                            if current_style.is_some() {
                                in_p_pr = true;
                            }
                        }
                        b"name" => {
                            if let Some(ref mut style) = current_style {
                                for attr in e.attributes().flatten() {
                                    if attr.key.local_name().as_ref() == b"val" {
                                        if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                            style.name = v;
                                        }
                                    }
                                }
                            }
                        }
                        b"basedOn" => {
                            if let Some(ref mut style) = current_style {
                                for attr in e.attributes().flatten() {
                                    if attr.key.local_name().as_ref() == b"val" {
                                        if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                            style.based_on = Some(v);
                                        }
                                    }
                                }
                            }
                        }
                        b"outlineLvl" => {
                            if in_p_pr {
                                if let Some(ref mut style) = current_style {
                                    for attr in e.attributes().flatten() {
                                        if attr.key.local_name().as_ref() == b"val" {
                                            if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                                if let Ok(lvl) = v.parse::<u8>() {
                                                    style.outline_lvl = Some(lvl);
                                                }
                                            }
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
                        b"style" => {
                            if let Some(style) = current_style.take() {
                                if !style.style_id.is_empty() {
                                    styles.insert(style.style_id.clone(), style);
                                }
                            }
                        }
                        b"pPr" => {
                            in_p_pr = false;
                        }
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(Self { styles })
    }

    /// Ermittelt die Überschriftenebene (1..=9) für eine Stil-ID oder liefert `None`.
    pub fn heading_level(&self, style_id: &str) -> Option<u8> {
        let mut curr_id = style_id.to_string();
        let mut depth = 0u8;

        while depth < 16 {
            depth = depth.saturating_add(1);

            let style = match self.styles.get(&curr_id) {
                Some(s) => s,
                None => {
                    // Fallback: Direkte Überprüfung der ID, falls Style nicht in map
                    return match_heading_string(&curr_id);
                }
            };

            // 1. Direkte outlineLvl im Stil
            if let Some(lvl) = style.outline_lvl {
                if lvl <= 8 {
                    return Some(lvl.saturating_add(1));
                }
            }

            // 2. Namensvergleich (style_id oder name)
            if let Some(lvl) =
                match_heading_string(&style.style_id).or_else(|| match_heading_string(&style.name))
            {
                return Some(lvl);
            }

            // 3. Vererbungskette folgen
            match style.based_on {
                Some(ref parent) => curr_id = parent.clone(),
                None => break,
            }
        }

        match_heading_string(style_id)
    }
}

/// Prüft, ob ein String ein Heading-Muster wie "heading 1", "Überschrift 1", "Title" etc. ist.
pub fn match_heading_string(s: &str) -> Option<u8> {
    let lower = s.to_lowercase();
    let trimmed = lower.trim();

    if trimmed == "title" || trimmed == "titel" {
        return Some(1);
    }
    if trimmed == "subtitle" || trimmed == "untertitel" {
        return Some(2);
    }

    if let Some(num_str) = trimmed
        .strip_prefix("heading ")
        .or_else(|| trimmed.strip_prefix("heading"))
        .or_else(|| trimmed.strip_prefix("überschrift "))
        .or_else(|| trimmed.strip_prefix("überschrift"))
    {
        if let Ok(lvl) = num_str.trim().parse::<u8>() {
            if (1..=9).contains(&lvl) {
                return Some(lvl);
            }
        }
    }

    None
}
