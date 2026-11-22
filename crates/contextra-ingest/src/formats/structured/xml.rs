// FILE-CONTEXT
// ZWECK: Extractor-Implementierung für generisches XML Format (P00B-Reader) (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, kein DOCTYPE/Entity, Pfade `root.kind[2].name`, `@attr`.
// STAND: TS:2026-10-31T00:00:00Z

use super::path::{append_index, append_key};
use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::BTreeMap;

#[derive(Debug)]
struct XmlElement {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<XmlChild>,
}

#[derive(Debug)]
enum XmlChild {
    Element(XmlElement),
    Text(String),
}

/// Extrahiert ein generisches XML-Dokument in die `DocumentIr`.
pub fn extract_xml(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    let mut reader = Reader::from_reader(input);
    reader.config_mut().trim_text_start = true;
    reader.config_mut().trim_text_end = true;

    let max_depth = usize::try_from(ctx.limits.max_xml_depth).unwrap_or(256);
    let mut stack: Vec<XmlElement> = Vec::new();
    let mut root_element: Option<XmlElement> = None;
    let mut buf = Vec::new();

    loop {
        ctx.budget.charge_xml_events(1)?;

        let event = reader
            .read_event_into(&mut buf)
            .map_err(|e| IngestError::Corrupt {
                format: Some(FormatId::Xml),
                detail: format!("XML-Parserfehler: {}", e),
            })?;

        match event {
            Event::Start(ref e) => {
                if stack.len() >= max_depth {
                    return Err(IngestError::LimitExceeded {
                        kind: LimitKind::XmlDepth,
                        limit: max_depth as u64,
                        observed: (stack.len() + 1) as u64,
                    });
                }

                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("element")
                    .to_string();

                let mut attributes = Vec::new();
                for attr in e.attributes().flatten() {
                    let key = std::str::from_utf8(attr.key.as_ref())
                        .unwrap_or("attr")
                        .to_string();
                    let val = std::str::from_utf8(&attr.value).unwrap_or("").to_string();
                    attributes.push((key, val));
                }

                stack.push(XmlElement {
                    name,
                    attributes,
                    children: Vec::new(),
                });
            }
            Event::End(_) => {
                if let Some(elem) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(XmlChild::Element(elem));
                    } else {
                        root_element = Some(elem);
                    }
                }
            }
            Event::Empty(ref e) => {
                if stack.len() >= max_depth {
                    return Err(IngestError::LimitExceeded {
                        kind: LimitKind::XmlDepth,
                        limit: max_depth as u64,
                        observed: (stack.len() + 1) as u64,
                    });
                }

                let name = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("element")
                    .to_string();

                let mut attributes = Vec::new();
                for attr in e.attributes().flatten() {
                    let key = std::str::from_utf8(attr.key.as_ref())
                        .unwrap_or("attr")
                        .to_string();
                    let val = std::str::from_utf8(&attr.value).unwrap_or("").to_string();
                    attributes.push((key, val));
                }

                let empty_elem = XmlElement {
                    name,
                    attributes,
                    children: Vec::new(),
                };

                if let Some(parent) = stack.last_mut() {
                    parent.children.push(XmlChild::Element(empty_elem));
                } else {
                    root_element = Some(empty_elem);
                }
            }
            Event::Text(ref e) => {
                let txt = e
                    .unescape()
                    .map(|cow| cow.into_owned())
                    .unwrap_or_else(|_| String::from_utf8_lossy(e.as_ref()).into_owned());
                let trimmed = txt.trim().to_string();
                if !trimmed.is_empty() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(XmlChild::Text(trimmed));
                    }
                }
            }
            Event::CData(ref e) => {
                if let Ok(txt) = std::str::from_utf8(e.as_ref()) {
                    let trimmed = txt.trim().to_string();
                    if !trimmed.is_empty() {
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(XmlChild::Text(trimmed));
                        }
                    }
                }
            }
            Event::Eof => break,
            Event::Comment(_) | Event::PI(_) | Event::Decl(_) | Event::DocType(_) => {
                // Kommentare, PIs, Decls und DOCTYPEs verwerfen (P00B)
            }
        }
        buf.clear();
    }

    let root = root_element.ok_or_else(|| IngestError::Corrupt {
        format: Some(FormatId::Xml),
        detail: "Kein XML-Wurzelelement gefunden".into(),
    })?;

    let mut ir = DocumentIr::new(FormatId::Xml);

    // meta.title suchen
    if root.name == "title" {
        for child in &root.children {
            if let XmlChild::Text(ref t) = child {
                ir.meta.title = Some(t.clone());
                break;
            }
        }
    } else {
        for child in &root.children {
            if let XmlChild::Element(ref child_elem) = child {
                if child_elem.name == "title" {
                    for grand_child in &child_elem.children {
                        if let XmlChild::Text(ref t) = grand_child {
                            ir.meta.title = Some(t.clone());
                            break;
                        }
                    }
                    if ir.meta.title.is_some() {
                        break;
                    }
                }
            }
        }
    }

    emit_element(&root, "", &mut ir, ctx)?;
    Ok(ir)
}

fn emit_element(
    elem: &XmlElement,
    parent_path: &str,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    let current_path = append_key(parent_path, &elem.name);

    // 1. Attribute als @attr ausgeben
    for (attr_name, attr_val) in &elem.attributes {
        let attr_key = format!("@{}", attr_name);
        let attr_path = append_key(&current_path, &attr_key);
        push_record(&attr_path, attr_val, ir, ctx)?;
    }

    // 2. Kind-Element-Häufigkeiten zur Index-Berechnung ermitteln
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
    for child in &elem.children {
        if let XmlChild::Element(ref child_elem) = child {
            *tag_counts.entry(child_elem.name.clone()).or_default() += 1;
        }
    }

    let mut tag_indices: BTreeMap<String, usize> = BTreeMap::new();

    // 3. Kinder in Dokumentreihenfolge ausgeben
    for child in &elem.children {
        match child {
            XmlChild::Text(txt) => {
                push_record(&current_path, txt, ir, ctx)?;
            }
            XmlChild::Element(child_elem) => {
                let total_count = tag_counts.get(&child_elem.name).copied().unwrap_or(0);
                let idx = tag_indices.entry(child_elem.name.clone()).or_default();

                let elem_path = if total_count > 1 {
                    let base_path = append_key(&current_path, &child_elem.name);
                    append_index(&base_path, *idx)
                } else {
                    append_key(&current_path, &child_elem.name)
                };

                *idx += 1;
                emit_element_at_path(child_elem, &elem_path, ir, ctx)?;
            }
        }
    }

    Ok(())
}

fn emit_element_at_path(
    elem: &XmlElement,
    current_path: &str,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    // 1. Attribute als @attr ausgeben
    for (attr_name, attr_val) in &elem.attributes {
        let attr_key = format!("@{}", attr_name);
        let attr_path = append_key(current_path, &attr_key);
        push_record(&attr_path, attr_val, ir, ctx)?;
    }

    // 2. Kind-Element-Häufigkeiten ermitteln
    let mut tag_counts: BTreeMap<String, usize> = BTreeMap::new();
    for child in &elem.children {
        if let XmlChild::Element(ref child_elem) = child {
            *tag_counts.entry(child_elem.name.clone()).or_default() += 1;
        }
    }

    let mut tag_indices: BTreeMap<String, usize> = BTreeMap::new();

    // 3. Kinder in Dokumentreihenfolge ausgeben
    for child in &elem.children {
        match child {
            XmlChild::Text(txt) => {
                push_record(current_path, txt, ir, ctx)?;
            }
            XmlChild::Element(child_elem) => {
                let total_count = tag_counts.get(&child_elem.name).copied().unwrap_or(0);
                let idx = tag_indices.entry(child_elem.name.clone()).or_default();

                let elem_path = if total_count > 1 {
                    let base_path = append_key(current_path, &child_elem.name);
                    append_index(&base_path, *idx)
                } else {
                    append_key(current_path, &child_elem.name)
                };

                *idx += 1;
                emit_element_at_path(child_elem, &elem_path, ir, ctx)?;
            }
        }
    }

    Ok(())
}

fn push_record(
    path: &str,
    value: &str,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    ctx.budget.charge_text(path.len() as u64)?;
    ctx.budget.charge_text(value.len() as u64)?;

    ir.push(
        BlockKind::Record {
            path: path.to_string(),
            value: value.to_string(),
        },
        Locator::default(),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;
    use crate::extractor::ResourceBudget;
    use crate::limits::IngestLimits;
    use crate::registry::ExtractorRegistry;

    fn mock_context(
        limits: IngestLimits,
    ) -> (
        ExtractorRegistry,
        IngestLimits,
        ResourceBudget,
        crate::limits::ExtractPolicy,
    ) {
        let reg = ExtractorRegistry::new_empty();
        let budget = ResourceBudget::new(limits.clone());
        let policy = crate::limits::ExtractPolicy::default();
        (reg, limits, budget, policy)
    }

    #[test]
    fn test_extract_xml_with_attributes_cdata_and_indices() {
        let (reg, limits, budget, policy) = mock_context(IngestLimits::default());
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &reg,
            filename: None,
        };

        let xml = br#"<root id="100">
            <title>XML Demo</title>
            <kind>First</kind>
            <kind>Second</kind>
            <kind><![CDATA[Third CDATA]]></kind>
        </root>"#;

        let ir = extract_xml(xml, &ctx).expect("XML extraction should succeed");

        assert_eq!(ir.meta.title.as_deref(), Some("XML Demo"));
        assert_eq!(ir.blocks.len(), 5);

        let records: Vec<(&str, &str)> = ir
            .blocks
            .iter()
            .map(|b| match &b.kind {
                BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
                _ => panic!("Expected Record"),
            })
            .collect();

        assert_eq!(records[0], ("root.@id", "100"));
        assert_eq!(records[1], ("root.title", "XML Demo"));
        assert_eq!(records[2], ("root.kind[0]", "First"));
        assert_eq!(records[3], ("root.kind[1]", "Second"));
        assert_eq!(records[4], ("root.kind[2]", "Third CDATA"));
    }
}
