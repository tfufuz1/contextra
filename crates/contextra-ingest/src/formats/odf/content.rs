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
// ZWECK: XML-Streaming-Parser für ODT-Inhalte (content.xml / office:body) und Flat ODF.
// INVARIANTEN: Zero-Panic, DoS-Schutz für Tabellenwiederholungen, Policy-Filterung für versteckten Text.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::ir::{BlockKind, DocumentIr, Locator, NoteKind, Table};
use quick_xml::events::Event;
use quick_xml::Reader;

/// Maximal zugelassene Wiederholung leerer Tabellenzellen/Zeilen ohne Budgetbelastung.
const MAX_EMPTY_CELL_PADDING: usize = 32;

/// Parsing-Kontext für ODT Content-Streams.
pub fn parse_content(
    format: crate::format::FormatId,
    xml_bytes: &[u8],
    ctx: &ExtractContext<'_>,
    ir: &mut DocumentIr,
) -> Result<(), IngestError> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(false);

    let mut buf = Vec::new();
    let mut text_stack: Vec<String> = Vec::new();
    let mut list_depth: u8 = 0;
    let mut list_ordered_stack: Vec<bool> = Vec::new();

    let mut current_table = Table::default();
    let mut in_header_row = false;
    let mut current_row: Vec<String> = Vec::new();
    let mut row_repeated: u64 = 1;

    let mut in_cell = false;
    let mut cell_spanned: u64 = 1;
    let mut cell_repeated: u64 = 1;
    let mut cell_text = String::new();

    let mut current_note_kind: Option<NoteKind> = None;
    let mut in_note_body = false;
    let mut in_annotation = false;
    let mut note_text = String::new();

    let mut skip_depth: u32 = 0;
    let mut tag_stack: Vec<TagInfo> = Vec::new();

    loop {
        ctx.budget.charge_xml_events(1)?;
        buf.clear();

        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                let tag_name = local.as_ref();

                if skip_depth > 0 {
                    skip_depth = skip_depth.saturating_add(1);
                    tag_stack.push(TagInfo::Other);
                    continue;
                }

                match tag_name {
                    b"h" => {
                        let level = parse_attr_u8(&reader, &e, b"outline-level").unwrap_or(1);
                        text_stack.push(String::new());
                        tag_stack.push(TagInfo::Heading(level));
                    }
                    b"p" => {
                        let is_hidden = check_hidden_attr(&reader, &e);
                        if is_hidden && !ctx.policy.include_hidden {
                            skip_depth = 1;
                            tag_stack.push(TagInfo::Other);
                        } else {
                            text_stack.push(String::new());
                            tag_stack.push(TagInfo::Paragraph);
                        }
                    }
                    b"conditional-text" => {
                        let is_hidden = check_hidden_attr(&reader, &e) || check_condition_attr(&reader, &e);
                        if is_hidden && !ctx.policy.include_hidden {
                            skip_depth = 1;
                            tag_stack.push(TagInfo::Other);
                        } else {
                            tag_stack.push(TagInfo::Other);
                        }
                    }
                    b"list" => {
                        list_depth = list_depth.saturating_add(1);
                        let is_ordered = check_ordered_attr(&reader, &e);
                        list_ordered_stack.push(is_ordered);
                        tag_stack.push(TagInfo::List);
                    }
                    b"list-item" => {
                        tag_stack.push(TagInfo::ListItem);
                    }
                    b"span" => {
                        tag_stack.push(TagInfo::Span);
                    }
                    b"s" => {
                        let count = parse_attr_u64(&reader, &e, b"c").unwrap_or(1).min(1000);
                        let spaces = " ".repeat(count as usize);
                        append_to_current_text(&mut text_stack, &mut cell_text, in_cell, &spaces);
                        tag_stack.push(TagInfo::Other);
                    }
                    b"tab" => {
                        append_to_current_text(&mut text_stack, &mut cell_text, in_cell, "\t");
                        tag_stack.push(TagInfo::Other);
                    }
                    b"line-break" => {
                        append_to_current_text(&mut text_stack, &mut cell_text, in_cell, "\n");
                        tag_stack.push(TagInfo::Other);
                    }
                    b"table" => {
                        current_table = Table::default();
                        tag_stack.push(TagInfo::Table);
                    }
                    b"table-header-rows" => {
                        in_header_row = true;
                        tag_stack.push(TagInfo::TableHeader);
                    }
                    b"table-row" => {
                        current_row.clear();
                        row_repeated = parse_attr_u64(&reader, &e, b"number-rows-repeated").unwrap_or(1);
                        tag_stack.push(TagInfo::TableRow);
                    }
                    b"table-cell" | b"covered-table-cell" => {
                        in_cell = true;
                        cell_spanned = parse_attr_u64(&reader, &e, b"number-columns-spanned").unwrap_or(1);
                        cell_repeated = parse_attr_u64(&reader, &e, b"number-columns-repeated").unwrap_or(1);
                        cell_text.clear();
                        tag_stack.push(TagInfo::TableCell);
                    }
                    b"note" => {
                        let note_class = get_attr_value(&reader, &e, b"note-class");
                        let kind = match note_class.as_deref() {
                            Some("endnote") => NoteKind::Endnote,
                            _ => NoteKind::Footnote,
                        };
                        current_note_kind = Some(kind);
                        note_text.clear();
                        tag_stack.push(TagInfo::Note);
                    }
                    b"note-body" => {
                        in_note_body = true;
                        tag_stack.push(TagInfo::NoteBody);
                    }
                    b"annotation" => {
                        if !ctx.policy.include_comments {
                            skip_depth = 1;
                            tag_stack.push(TagInfo::Other);
                        } else {
                            current_note_kind = Some(NoteKind::Comment);
                            in_annotation = true;
                            note_text.clear();
                            tag_stack.push(TagInfo::Annotation);
                        }
                    }
                    b"deletion" => {
                        if !ctx.policy.include_tracked_deletions {
                            skip_depth = 1;
                            tag_stack.push(TagInfo::Other);
                        } else {
                            tag_stack.push(TagInfo::Deletion);
                        }
                    }
                    b"hidden-paragraph" => {
                        if !ctx.policy.include_hidden {
                            skip_depth = 1;
                            tag_stack.push(TagInfo::Other);
                        } else {
                            text_stack.push(String::new());
                            tag_stack.push(TagInfo::Paragraph);
                        }
                    }
                    _ => {
                        tag_stack.push(TagInfo::Other);
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if skip_depth > 0 {
                    continue;
                }
                let raw_text = e.unescape().map_err(|err| IngestError::Corrupt {
                    format: Some(format),
                    detail: format!("XML Decode-Fehler: {}", err),
                })?;

                ctx.budget.charge_text(raw_text.len() as u64)?;

                if current_note_kind.is_some() {
                    if in_note_body || in_annotation {
                        note_text.push_str(&raw_text);
                    }
                } else if in_cell {
                    cell_text.push_str(&raw_text);
                } else if let Some(top_text) = text_stack.last_mut() {
                    top_text.push_str(&raw_text);
                }
            }
            Ok(Event::End(_)) => {
                if skip_depth > 0 {
                    skip_depth = skip_depth.saturating_sub(1);
                    tag_stack.pop();
                    continue;
                }

                let tag_info = tag_stack.pop().unwrap_or(TagInfo::Other);

                match tag_info {
                    TagInfo::Heading(level) => {
                        if let Some(txt) = text_stack.pop() {
                            let trimmed = txt.trim().to_string();
                            if !trimmed.is_empty() {
                                ir.push(
                                    BlockKind::Heading {
                                        level,
                                        text: trimmed,
                                    },
                                    Locator::default(),
                                );
                            }
                        }
                    }
                    TagInfo::Paragraph => {
                        if let Some(txt) = text_stack.pop() {
                            let trimmed = txt.trim().to_string();
                            if !trimmed.is_empty() {
                                if list_depth > 0 {
                                    let ordered = list_ordered_stack.last().copied().unwrap_or(false);
                                    ir.push(
                                        BlockKind::ListItem {
                                            depth: list_depth,
                                            ordered,
                                            text: trimmed,
                                        },
                                        Locator::default(),
                                    );
                                } else {
                                    ir.push(
                                        BlockKind::Paragraph { text: trimmed },
                                        Locator::default(),
                                    );
                                }
                            }
                        }
                    }
                    TagInfo::List => {
                        list_depth = list_depth.saturating_sub(1);
                        list_ordered_stack.pop();
                    }
                    TagInfo::TableCell => {
                        in_cell = false;
                        let text_val = cell_text.trim().to_string();

                        if text_val.is_empty() {
                            // DoS Schutz: leere Zellen nicht massenhaft materialisieren
                            let eff_rep = cell_repeated.min(MAX_EMPTY_CELL_PADDING as u64);
                            for _ in 0..eff_rep {
                                current_row.push(String::new());
                            }
                        } else {
                            // Nicht-leere Wiederholung über charge_cells buchen
                            ctx.budget.charge_cells(cell_repeated)?;
                            for _ in 0..cell_repeated {
                                current_row.push(text_val.clone());
                            }
                        }

                        // Spanning behandeln
                        if cell_spanned > 1 {
                            let pad = (cell_spanned - 1).min(MAX_EMPTY_CELL_PADDING as u64);
                            for _ in 0..pad {
                                current_row.push(String::new());
                            }
                        }
                        cell_text.clear();
                    }
                    TagInfo::TableRow => {
                        let is_row_empty = current_row.iter().all(|c| c.is_empty());
                        if is_row_empty {
                            let eff_rep = row_repeated.min(1);
                            for _ in 0..eff_rep {
                                current_table.rows.push(current_row.clone());
                            }
                        } else {
                            let cell_count = current_row.len() as u64;
                            let total_cells = row_repeated.saturating_mul(cell_count);
                            ctx.budget.charge_cells(total_cells)?;
                            for _ in 0..row_repeated {
                                current_table.rows.push(current_row.clone());
                            }
                        }
                        if in_header_row {
                            current_table.header_rows = current_table.header_rows.saturating_add(1);
                        }
                    }
                    TagInfo::TableHeader => {
                        in_header_row = false;
                    }
                    TagInfo::Table => {
                        if !current_table.rows.is_empty() {
                            ir.push(BlockKind::Table(current_table.clone()), Locator::default());
                        }
                    }
                    TagInfo::NoteBody => {
                        in_note_body = false;
                    }
                    TagInfo::Note => {
                        if let Some(kind) = current_note_kind.take() {
                            let trimmed = note_text.trim().to_string();
                            if !trimmed.is_empty() {
                                ir.push(BlockKind::Note { kind, text: trimmed }, Locator::default());
                            }
                        }
                    }
                    TagInfo::Annotation => {
                        in_annotation = false;
                        if let Some(kind) = current_note_kind.take() {
                            let trimmed = note_text.trim().to_string();
                            if !trimmed.is_empty() {
                                ir.push(BlockKind::Note { kind, text: trimmed }, Locator::default());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(IngestError::Corrupt {
                    format: Some(format),
                    detail: format!("Ungültiges Content-XML: {}", e),
                });
            }
            _ => {}
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TagInfo {
    Heading(u8),
    Paragraph,
    List,
    ListItem,
    Span,
    Table,
    TableHeader,
    TableRow,
    TableCell,
    Note,
    NoteBody,
    Annotation,
    Deletion,
    Other,
}

fn append_to_current_text(
    text_stack: &mut [String],
    cell_text_mut: &mut String,
    in_cell: bool,
    addition: &str,
) {
    if in_cell {
        cell_text_mut.push_str(addition);
    } else if let Some(top) = text_stack.last_mut() {
        top.push_str(addition);
    }
}

fn parse_attr_u8<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart<'_>,
    name: &[u8],
) -> Option<u8> {
    get_attr_value(reader, e, name)?.parse::<u8>().ok()
}

fn parse_attr_u64<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart<'_>,
    name: &[u8],
) -> Option<u64> {
    get_attr_value(reader, e, name)?.parse::<u64>().ok()
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

fn check_hidden_attr<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart<'_>,
) -> bool {
    for attr in e.attributes().flatten() {
        let key = attr.key.local_name();
        if key.as_ref() == b"display" {
            if let Ok(v) = attr.decode_and_unescape_value(reader.decoder()) {
                if v == "none" || v == "false" {
                    return true;
                }
            }
        }
    }
    false
}

fn check_condition_attr<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart<'_>,
) -> bool {
    if let Some(cond) = get_attr_value(reader, e, b"condition") {
        if cond.to_lowercase().contains("false") || cond.to_lowercase().contains("hidden") {
            return true;
        }
    }
    false
}

fn check_ordered_attr<R: std::io::BufRead>(
    reader: &Reader<R>,
    e: &quick_xml::events::BytesStart<'_>,
) -> bool {
    if let Some(style) = get_attr_value(reader, e, b"style-name") {
        if style.to_lowercase().contains("num") || style.to_lowercase().contains("order") {
            return true;
        }
    }
    false
}
