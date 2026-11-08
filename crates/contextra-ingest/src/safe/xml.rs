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
// ZWECK: Sicheres XML-Parsing mit Event-Budgetierung, XML-Baumstrukturen und SafeXmlReader (P00B/P04).
// INVARIANTEN: Zero-Panic, ResourceBudget-Prüfung, BTreeMap für Determinismus, XXE-Schutz.
// STAND: TS:2026-11-08T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ResourceBudget;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;
use std::io::Cursor;

/// Safe XML reader enforcing XML depth limits, event budget, and XXE protection.
pub struct SafeXmlReader<'a> {
    reader: Reader<&'a [u8]>,
    depth: u32,
    max_depth: u32,
}

impl<'a> SafeXmlReader<'a> {
    /// Creates a new `SafeXmlReader` from XML byte slice.
    pub fn new(bytes: &'a [u8], max_depth: u32) -> Self {
        let mut reader = Reader::from_reader(bytes);
        reader.config_mut().trim_text_start = false;
        reader.config_mut().trim_text_end = false;
        reader.config_mut().expand_empty_elements = true;

        Self {
            reader,
            depth: 0,
            max_depth,
        }
    }

    /// Advances to the next XML event while charging the resource budget and enforcing depth limits.
    pub fn next_event(&mut self, budget: &ResourceBudget) -> Result<Event<'static>, IngestError> {
        budget.charge_xml_events(1)?;

        let mut buf = Vec::new();
        let event = self
            .reader
            .read_event_into(&mut buf)
            .map_err(|e| IngestError::Corrupt {
                format: None,
                detail: format!("XML Parse-Fehler: {e}"),
            })?;

        match &event {
            Event::Start(_) => {
                self.depth = self.depth.saturating_add(1);
                if self.depth > self.max_depth {
                    return Err(IngestError::LimitExceeded {
                        kind: LimitKind::XmlDepth,
                        limit: u64::from(self.max_depth),
                        observed: u64::from(self.depth),
                    });
                }
            }
            Event::End(_) => {
                self.depth = self.depth.saturating_sub(1);
            }
            Event::DocType(dt) => {
                // Reject DTD external entity declarations or entity expansion (XXE protection)
                let dt_bytes: &[u8] = dt.as_ref();
                if dt_bytes
                    .windows(6)
                    .any(|w| w.eq_ignore_ascii_case(b"ENTITY"))
                    || dt_bytes
                        .windows(6)
                        .any(|w| w.eq_ignore_ascii_case(b"SYSTEM"))
                    || dt_bytes
                        .windows(6)
                        .any(|w| w.eq_ignore_ascii_case(b"PUBLIC"))
                {
                    return Err(IngestError::Corrupt {
                        format: None,
                        detail: "XXE-Sicherheitsverstoß: Externe DTDs/Entitäten sind verboten"
                            .into(),
                    });
                }
            }
            _ => {}
        }

        Ok(event.into_owned())
    }
}

/// Ein Knoten in einem geparsten XML-Baum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlNode {
    Element(XmlElement),
    Text(String),
}

/// Ein XML-Element mit Name, Attributen und Kind-Knoten.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct XmlElement {
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub children: Vec<XmlNode>,
}

impl XmlElement {
    /// Gibt den lokalen Elementnamen ohne Präfix zurück (z. B. "sp" für "p:sp").
    pub fn local_name(&self) -> &str {
        match self.name.rsplit_once(':') {
            Some((_, local)) => local,
            None => &self.name,
        }
    }

    /// Liest ein Attribut nach vergleichendem Namen (exakt oder lokaler Name).
    pub fn attr(&self, key: &str) -> Option<&str> {
        if let Some(val) = self.attributes.get(key) {
            return Some(val.as_str());
        }
        for (k, v) in &self.attributes {
            let local_k = match k.rsplit_once(':') {
                Some((_, l)) => l,
                None => k.as_str(),
            };
            if local_k == key {
                return Some(v.as_str());
            }
        }
        None
    }

    /// Gibt eine Liste aller untergeordneten `XmlElement`-Kinder zurück.
    pub fn child_elements(&self) -> Vec<&XmlElement> {
        let mut list = Vec::new();
        for child in &self.children {
            if let XmlNode::Element(elem) = child {
                list.push(elem);
            }
        }
        list
    }

    /// Sucht untergeordnete Elemente mit passendem lokalen oder qualifizierten Namen.
    pub fn find_children<'a>(&'a self, name: &'a str) -> Vec<&'a XmlElement> {
        let mut list = Vec::new();
        for child in &self.children {
            if let XmlNode::Element(elem) = child {
                if elem.name == name || elem.local_name() == name {
                    list.push(elem);
                }
            }
        }
        list
    }

    /// Sucht das erste untergeordnete Element mit passendem Namen.
    pub fn find_first_child(&self, name: &str) -> Option<&XmlElement> {
        for child in &self.children {
            if let XmlNode::Element(elem) = child {
                if elem.name == name || elem.local_name() == name {
                    return Some(elem);
                }
            }
        }
        None
    }

    /// Sammelt sämtlichen Text aus allen Kindknoten rekursiv.
    pub fn collect_text(&self) -> String {
        let mut buf = String::new();
        self.collect_text_into(&mut buf);
        buf
    }

    fn collect_text_into(&self, buf: &mut String) {
        for child in &self.children {
            match child {
                XmlNode::Text(t) => buf.push_str(t),
                XmlNode::Element(e) => e.collect_text_into(buf),
            }
        }
    }
}

/// Liest ein XML-Dokument aus Bytes sicher in ein Wurzel-`XmlElement` ein.
pub fn parse_xml(bytes: &[u8], budget: &ResourceBudget) -> Result<XmlElement, IngestError> {
    let mut reader = Reader::from_reader(Cursor::new(bytes));
    reader.config_mut().trim_text_start = false;
    reader.config_mut().trim_text_end = false;

    let mut buf = Vec::new();
    let mut stack: Vec<XmlElement> = Vec::new();
    let mut root: Option<XmlElement> = None;

    loop {
        buf.clear();
        budget.charge_xml_events(1)?;

        let event = match reader.read_event_into(&mut buf) {
            Ok(e) => e,
            Err(e) => {
                return Err(IngestError::Corrupt {
                    format: None,
                    detail: format!("XML-Syntaxfehler: {e}"),
                });
            }
        };

        match event {
            Event::Start(e) => {
                let name = match String::from_utf8(e.name().as_ref().to_vec()) {
                    Ok(n) => n,
                    Err(_) => {
                        return Err(IngestError::Corrupt {
                            format: None,
                            detail: "Ungültige UTF-8-Zeichenkette in XML-Tag".into(),
                        });
                    }
                };

                let mut attributes = BTreeMap::new();
                for attr_res in e.attributes() {
                    let attr = attr_res.map_err(|err| IngestError::Corrupt {
                        format: None,
                        detail: format!("Fehlerhafte XML-Attribute: {err}"),
                    })?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = attr
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(|err| IngestError::Corrupt {
                            format: None,
                            detail: format!("Fehlerhafte XML-Attributwerte: {err}"),
                        })?
                        .to_string();
                    attributes.insert(key, value);
                }

                let element = XmlElement {
                    name,
                    attributes,
                    children: Vec::new(),
                };

                stack.push(element);
            }
            Event::Empty(e) => {
                let name = match String::from_utf8(e.name().as_ref().to_vec()) {
                    Ok(n) => n,
                    Err(_) => {
                        return Err(IngestError::Corrupt {
                            format: None,
                            detail: "Ungültige UTF-8-Zeichenkette in XML-Tag".into(),
                        });
                    }
                };

                let mut attributes = BTreeMap::new();
                for attr_res in e.attributes() {
                    let attr = attr_res.map_err(|err| IngestError::Corrupt {
                        format: None,
                        detail: format!("Fehlerhafte XML-Attribute: {err}"),
                    })?;
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = attr
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(|err| IngestError::Corrupt {
                            format: None,
                            detail: format!("Fehlerhafte XML-Attributwerte: {err}"),
                        })?
                        .to_string();
                    attributes.insert(key, value);
                }

                let element = XmlElement {
                    name,
                    attributes,
                    children: Vec::new(),
                };

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Element(element));
                } else if root.is_none() {
                    root = Some(element);
                }
            }
            Event::Text(e) => {
                let text = match e.unescape() {
                    Ok(t) => t.to_string(),
                    Err(_) => String::from_utf8_lossy(e.as_ref()).to_string(),
                };
                if !text.is_empty() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(XmlNode::Text(text));
                    }
                }
            }
            Event::CData(e) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                if !text.is_empty() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(XmlNode::Text(text));
                    }
                }
            }
            Event::End(_) => {
                let elem = match stack.pop() {
                    Some(e) => e,
                    None => {
                        return Err(IngestError::Corrupt {
                            format: None,
                            detail: "Unerwartetes schließendes XML-Tag".into(),
                        });
                    }
                };

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlNode::Element(elem));
                } else {
                    root = Some(elem);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    root.ok_or_else(|| IngestError::Corrupt {
        format: None,
        detail: "Leeres oder fehlerhaftes XML-Dokument".into(),
    })
}
