// FILE-CONTEXT
// ZWECK: Parsing von Fußnoten, Endnoten, Kommentaren, Kopf- & Fußzeilen (WP-ING-04).
// INVARIANTEN: Zero-Panic, Policy-Prüfung vor Extraktion von Kommentaren/Kopfzeilen, kein unsafe.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::{ExtractContext, ResourceBudget};
use crate::ir::NoteKind;
use crate::safe::xml::SafeXmlReader;
use quick_xml::events::Event;

/// Extrahierte Notiz aus Fußnoten, Endnoten, Kommentaren, Kopf-/Fußzeilen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedNote {
    pub kind: NoteKind,
    pub text: String,
}

/// Parst eine Footnotes- oder Endnotes-XML-Datei.
pub fn parse_footnotes_or_endnotes(
    bytes: &[u8],
    is_endnote: bool,
    max_xml_depth: u32,
    budget: &ResourceBudget,
) -> Result<Vec<ExtractedNote>, IngestError> {
    let mut reader = SafeXmlReader::new(bytes, max_xml_depth);
    let mut notes = Vec::new();

    let note_tag = if is_endnote {
        b"endnote".as_slice()
    } else {
        b"footnote".as_slice()
    };
    let note_kind = if is_endnote {
        NoteKind::Endnote
    } else {
        NoteKind::Footnote
    };

    let mut current_type: Option<String> = None;
    let mut in_note = false;
    let mut current_text = String::new();

    loop {
        match reader.next_event(budget)? {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == note_tag {
                    in_note = true;
                    current_text.clear();
                    current_type = None;

                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"type" {
                            if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                current_type = Some(v);
                            }
                        }
                    }
                } else if in_note && local_name.as_ref() == b"t" {
                    // text event next
                } else if in_note && local_name.as_ref() == b"br" {
                    current_text.push('\n');
                } else if in_note && local_name.as_ref() == b"tab" {
                    current_text.push('\t');
                }
            }
            Event::Text(ref e) => {
                if in_note {
                    if let Ok(txt) = e.unescape() {
                        current_text.push_str(&txt);
                    }
                }
            }
            Event::End(ref e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == note_tag {
                    in_note = false;
                    let is_sep = current_type
                        .as_deref()
                        .is_some_and(|t| t == "separator" || t == "continuationSeparator");

                    let trimmed = current_text.trim();
                    if !is_sep && !trimmed.is_empty() {
                        notes.push(ExtractedNote {
                            kind: note_kind,
                            text: trimmed.to_string(),
                        });
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(notes)
}

/// Parst word/comments.xml wenn `policy.include_comments` true ist.
pub fn parse_comments(
    bytes: &[u8],
    ctx: &ExtractContext<'_>,
) -> Result<Vec<ExtractedNote>, IngestError> {
    if !ctx.policy.include_comments {
        return Ok(Vec::new());
    }

    let mut reader = SafeXmlReader::new(bytes, ctx.limits.max_xml_depth);
    let mut notes = Vec::new();
    let mut in_comment = false;
    let mut current_text = String::new();

    loop {
        match reader.next_event(ctx.budget)? {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"comment" {
                    in_comment = true;
                    current_text.clear();
                } else if in_comment && local_name.as_ref() == b"br" {
                    current_text.push('\n');
                } else if in_comment && local_name.as_ref() == b"tab" {
                    current_text.push('\t');
                }
            }
            Event::Text(ref e) => {
                if in_comment {
                    if let Ok(txt) = e.unescape() {
                        current_text.push_str(&txt);
                    }
                }
            }
            Event::End(ref e) => {
                if e.local_name().as_ref() == b"comment" {
                    in_comment = false;
                    let trimmed = current_text.trim();
                    if !trimmed.is_empty() {
                        notes.push(ExtractedNote {
                            kind: NoteKind::Comment,
                            text: trimmed.to_string(),
                        });
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(notes)
}

/// Parst eine Header/Footer-XML-Datei wenn `policy.include_headers_footers` true ist.
pub fn parse_header_footer(
    bytes: &[u8],
    is_footer: bool,
    ctx: &ExtractContext<'_>,
) -> Result<Option<ExtractedNote>, IngestError> {
    if !ctx.policy.include_headers_footers {
        return Ok(None);
    }

    let mut reader = SafeXmlReader::new(bytes, ctx.limits.max_xml_depth);
    let mut text = String::new();

    loop {
        match reader.next_event(ctx.budget)? {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"br" {
                    text.push('\n');
                } else if local_name.as_ref() == b"tab" {
                    text.push('\t');
                }
            }
            Event::Text(ref e) => {
                if let Ok(txt) = e.unescape() {
                    text.push_str(&txt);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    let trimmed = text.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        let kind = if is_footer {
            NoteKind::Footer
        } else {
            NoteKind::Header
        };
        Ok(Some(ExtractedNote {
            kind,
            text: trimmed.to_string(),
        }))
    }
}
