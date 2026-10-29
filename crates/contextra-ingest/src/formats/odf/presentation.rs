// FILE-CONTEXT
// ZWECK: XML-Streaming-Parser für ODP-Präsentationen (draw:page, Titel, Notizen, versteckte Folien).
// INVARIANTEN: Zero-Panic, Slide-Boundary Invarianten, Policy-Beachtung für versteckte Folien.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::ir::{BlockKind, Boundary, DocumentIr, Locator, NoteKind};
use quick_xml::events::Event;
use quick_xml::Reader;

/// Extrahiert Folien, Titel, Inhalte und Sprechernotizen aus ODP-Content XML.
pub fn parse_presentation(
    xml_bytes: &[u8],
    ctx: &ExtractContext<'_>,
    ir: &mut DocumentIr,
) -> Result<(), IngestError> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(false);
    reader.config_mut().expand_empty_elements(true);

    let mut buf = Vec::new();
    let mut slide_number: u32 = 0;
    let mut skip_page = false;
    let mut skip_depth: u32 = 0;

    let mut in_notes = false;
    let mut notes_text = String::new();

    let mut current_frame_class = String::new();
    let mut current_text = String::new();
    let mut page_title: Option<String> = None;

    let mut page_blocks: Vec<(BlockKind, Locator)> = Vec::new();

    loop {
        ctx.budget.charge_xml_events(1)?;
        buf.clear();

        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                let tag_name = local.as_ref();

                if skip_depth > 0 {
                    skip_depth = skip_depth.saturating_add(1);
                    continue;
                }

                match tag_name {
                    b"page" => {
                        slide_number = slide_number.saturating_add(1);

                        let is_hidden = is_page_hidden(&reader, &e);
                        if is_hidden && !ctx.policy.include_hidden {
                            skip_page = true;
                            skip_depth = 1;
                            continue;
                        }

                        skip_page = false;
                        page_title = None;
                        page_blocks.clear();
                    }
                    b"notes" => {
                        in_notes = true;
                        notes_text.clear();
                    }
                    b"frame" => {
                        current_frame_class =
                            get_attr_value(&reader, &e, b"class").unwrap_or_default();
                    }
                    b"p" | b"h" => {
                        current_text.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if skip_depth > 0 || skip_page {
                    continue;
                }
                let raw = e.unescape().map_err(|err| IngestError::Corrupt {
                    format: Some(crate::format::FormatId::Odp),
                    detail: format!("XML Decode-Fehler in ODP: {}", err),
                })?;

                ctx.budget.charge_text(raw.len() as u64)?;

                if in_notes {
                    notes_text.push_str(&raw);
                } else {
                    current_text.push_str(&raw);
                }
            }
            Ok(Event::End(e)) => {
                let local = e.local_name();
                let tag_name = local.as_ref();

                if skip_depth > 0 {
                    skip_depth = skip_depth.saturating_sub(1);
                    if skip_depth == 0 {
                        skip_page = false;
                    }
                    continue;
                }

                match tag_name {
                    b"p" | b"h" => {
                        let trimmed = current_text.trim().to_string();
                        if !trimmed.is_empty() {
                            if (current_frame_class == "title" || current_frame_class == "header")
                                && page_title.is_none()
                            {
                                page_title = Some(trimmed.clone());
                            } else if in_notes {
                                // Sprechernotizen sammeln
                            } else {
                                let kind = if tag_name == b"h" {
                                    BlockKind::Heading {
                                        level: 1,
                                        text: trimmed,
                                    }
                                } else {
                                    BlockKind::Paragraph { text: trimmed }
                                };
                                let loc = Locator {
                                    slide: Some(slide_number),
                                    ..Default::default()
                                };
                                page_blocks.push((kind, loc));
                            }
                        }
                        current_text.clear();
                    }
                    b"notes" => {
                        in_notes = false;
                        let trimmed = notes_text.trim().to_string();
                        if !trimmed.is_empty() {
                            let loc = Locator {
                                slide: Some(slide_number),
                                ..Default::default()
                            };
                            page_blocks.push((
                                BlockKind::Note {
                                    kind: NoteKind::SpeakerNotes,
                                    text: trimmed,
                                },
                                loc,
                            ));
                        }
                        notes_text.clear();
                    }
                    b"page" => {
                        if !skip_page {
                            // Boundary-Block für Folie ausgeben
                            let slide_boundary = BlockKind::Boundary(Boundary::Slide {
                                number: slide_number,
                                title: page_title.take(),
                            });
                            let loc = Locator {
                                slide: Some(slide_number),
                                ..Default::default()
                            };
                            ir.push(slide_boundary, loc);

                            // Blöcke der Folie übernehmen
                            for (kind, block_loc) in page_blocks.drain(..) {
                                ir.push(kind, block_loc);
                            }
                        }
                        skip_page = false;
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(IngestError::Corrupt {
                    format: Some(crate::format::FormatId::Odp),
                    detail: format!("Ungültiges ODP-XML: {}", e),
                });
            }
            _ => {}
        }
    }

    Ok(())
}

fn is_page_hidden<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart<'_>,
) -> bool {
    for attr in e.attributes().flatten() {
        let key = attr.key.local_name();
        if key.as_ref() == b"visibility" || key.as_ref() == b"display" {
            if let Ok(val) = attr.decode_and_unescape_value(reader.decoder()) {
                if val == "hidden" || val == "none" || val == "false" {
                    return true;
                }
            }
        }
    }
    false
}

fn get_attr_value<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart<'_>,
    name: &[u8],
) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == name {
            if let Ok(v) = attr.decode_and_unescape_value(reader.decoder()) {
                return Some(v.to_string());
            }
        }
    }
    None
}
