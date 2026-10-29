// FILE-CONTEXT
// ZWECK: XML-Parser für OpenDocument-Metadaten (meta.xml / office:meta).
// INVARIANTEN: Zero-Panic, Budgetierung, strikte Beachtung der ExtractPolicy (keep_personal_metadata).
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::ir::DocMeta;
use quick_xml::events::Event;
use quick_xml::Reader;

/// Extrahiert Metadaten aus `meta.xml` oder einem `<office:meta>`-Element.
pub fn parse_meta(xml_bytes: &[u8], ctx: &ExtractContext<'_>) -> Result<DocMeta, IngestError> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(false);

    let mut meta = DocMeta::default();
    let mut buf = Vec::new();
    let mut current_tag = Vec::new();

    loop {
        ctx.budget.charge_xml_events(1)?;
        buf.clear();

        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                current_tag = local.as_ref().to_vec();
            }
            Ok(Event::Text(e)) => {
                if current_tag.is_empty() {
                    continue;
                }
                let text_val = e.unescape().map_err(|err| IngestError::Corrupt {
                    format: None,
                    detail: format!("XML Decode-Fehler in Metadaten: {}", err),
                })?;
                let val_str = text_val.trim();
                if val_str.is_empty() {
                    continue;
                }

                match current_tag.as_slice() {
                    b"title" => {
                        meta.title = Some(val_str.to_string());
                    }
                    b"language" => {
                        meta.language = Some(val_str.to_string());
                    }
                    b"creation-date" => {
                        meta.created = Some(val_str.to_string());
                    }
                    b"date" => {
                        meta.modified = Some(val_str.to_string());
                    }
                    b"creator" | b"initial-creator" => {
                        if ctx.policy.keep_personal_metadata {
                            let author_name = val_str.to_string();
                            if !meta.authors.contains(&author_name) {
                                meta.authors.push(author_name);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(IngestError::Corrupt {
                    format: None,
                    detail: format!("Ungültiges Metadaten-XML: {}", e),
                });
            }
            _ => {}
        }
    }

    Ok(meta)
}
