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
// ZWECK: Parsing von word/document.xml für Absätze, Überschriften, Listen, Tabellen & Textboxen (WP-ING-04).
// INVARIANTEN: Zero-Panic, LimitExceeded bei Budget-Überschreitung, AlternateContent ohne Dopplung.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::formats::docx::numbering::NumMap;
use crate::formats::docx::styles::StyleMap;
use crate::ir::{BlockKind, DocumentIr, Locator, Table};
use crate::safe::xml::SafeXmlReader;
use quick_xml::events::Event;

/// Resultat des Parsevorgangs von document.xml.
#[derive(Debug, Clone, Default)]
pub struct DocumentParseResult {
    pub hidden_content_skipped: bool,
    pub embedded_object_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VMerge {
    Restart,
    Continue,
}

#[derive(Debug, Clone, Default)]
struct RawCell {
    text: String,
    grid_span: usize,
    v_merge: Option<VMerge>,
}

#[derive(Debug, Clone, Default)]
struct RawRow {
    is_header: bool,
    cells: Vec<RawCell>,
}

/// Parst die Hauptdokument-XML (word/document.xml).
pub fn parse_document_xml(
    bytes: &[u8],
    styles: &StyleMap,
    numbering: &NumMap,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentParseResult, IngestError> {
    let mut reader = SafeXmlReader::new(bytes, ctx.limits.max_xml_depth);
    let mut result = DocumentParseResult::default();

    // Stack or state tracking
    let mut ignore_instr = false;
    let mut in_del = false;
    let mut in_alternate_content = false;
    let mut choice_processed = false;
    let mut skip_branch = false;

    // Current paragraph state
    let mut in_p = false;
    let mut p_style_id: Option<String> = None;
    let mut p_outline_lvl: Option<u8> = None;
    let mut p_num_id: Option<String> = None;
    let mut p_ilvl: Option<u8> = None;
    let mut p_vanish = false;
    let mut p_text = String::new();

    // Current run state
    let mut in_r = false;
    let mut r_vanish = false;

    // Current table state (support nesting via stack or simple state)
    let mut table_stack: Vec<Vec<RawRow>> = Vec::new();
    let mut cell_stack: Vec<RawCell> = Vec::new();

    loop {
        match reader.next_event(ctx.budget)? {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let local_name = e.local_name();
                let name = local_name.as_ref();

                // 1. Alternate Content Handling: choose ONLY Choice, skip Fallback
                if name == b"AlternateContent" {
                    in_alternate_content = true;
                    choice_processed = false;
                    skip_branch = false;
                } else if name == b"Choice" && in_alternate_content {
                    skip_branch = false;
                } else if name == b"Fallback" && in_alternate_content {
                    skip_branch = choice_processed;
                }

                if skip_branch {
                    continue;
                }

                match name {
                    b"tbl" => {
                        table_stack.push(Vec::new());
                    }
                    b"tr" => {
                        if let Some(tbl) = table_stack.last_mut() {
                            tbl.push(RawRow::default());
                        }
                    }
                    b"tblHeader" => {
                        if let Some(tbl) = table_stack.last_mut() {
                            if let Some(row) = tbl.last_mut() {
                                row.is_header = true;
                            }
                        }
                    }
                    b"tc" => {
                        cell_stack.push(RawCell {
                            grid_span: 1,
                            ..Default::default()
                        });
                    }
                    b"gridSpan" => {
                        if let Some(cell) = cell_stack.last_mut() {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        if let Ok(span) = v.parse::<usize>() {
                                            cell.grid_span = span.max(1);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"vMerge" => {
                        if let Some(cell) = cell_stack.last_mut() {
                            let mut vm = VMerge::Continue;
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        if v == "restart" {
                                            vm = VMerge::Restart;
                                        }
                                    }
                                }
                            }
                            cell.v_merge = Some(vm);
                        }
                    }
                    b"p" => {
                        if table_stack.is_empty() {
                            in_p = true;
                            p_style_id = None;
                            p_outline_lvl = None;
                            p_num_id = None;
                            p_ilvl = None;
                            p_vanish = false;
                            p_text.clear();
                        }
                    }
                    b"pStyle" => {
                        if in_p {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        p_style_id = Some(v);
                                    }
                                }
                            }
                        }
                    }
                    b"outlineLvl" => {
                        if in_p {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        if let Ok(lvl) = v.parse::<u8>() {
                                            p_outline_lvl = Some(lvl);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"numPr" => {}
                    b"numId" => {
                        if in_p {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        p_num_id = Some(v);
                                    }
                                }
                            }
                        }
                    }
                    b"ilvl" => {
                        if in_p {
                            for attr in e.attributes().flatten() {
                                if attr.key.local_name().as_ref() == b"val" {
                                    if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                        if let Ok(lvl) = v.parse::<u8>() {
                                            p_ilvl = Some(lvl);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    b"r" => {
                        in_r = true;
                        r_vanish = false;
                    }
                    b"vanish" => {
                        if in_r {
                            r_vanish = true;
                        } else if in_p {
                            p_vanish = true;
                        }
                    }
                    b"instrText" => {
                        ignore_instr = true;
                    }
                    b"del" => {
                        in_del = true;
                    }
                    b"br" => {
                        let mut is_page = false;
                        for attr in e.attributes().flatten() {
                            if attr.key.local_name().as_ref() == b"type" {
                                if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                    if v == "page" {
                                        is_page = true;
                                    }
                                }
                            }
                        }
                        if !is_page {
                            append_run_text(
                                "\n",
                                in_r && r_vanish || p_vanish,
                                in_del,
                                &mut p_text,
                                &mut cell_stack,
                                &mut result,
                                ctx,
                            )?;
                        }
                    }
                    b"tab" => {
                        append_run_text(
                            "\t",
                            in_r && r_vanish || p_vanish,
                            in_del,
                            &mut p_text,
                            &mut cell_stack,
                            &mut result,
                            ctx,
                        )?;
                    }
                    b"noBreakHyphen" => {
                        append_run_text(
                            "-",
                            in_r && r_vanish || p_vanish,
                            in_del,
                            &mut p_text,
                            &mut cell_stack,
                            &mut result,
                            ctx,
                        )?;
                    }
                    b"drawing" | b"object" => {
                        result.embedded_object_count =
                            result.embedded_object_count.saturating_add(1);
                    }
                    b"docPr" => {
                        // Image alt text caption
                        let mut descr = String::new();
                        let mut title = String::new();
                        for attr in e.attributes().flatten() {
                            let k = attr.key.local_name();
                            if k.as_ref() == b"descr" {
                                if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                    descr = v;
                                }
                            } else if k.as_ref() == b"title" {
                                if let Ok(v) = String::from_utf8(attr.value.to_vec()) {
                                    title = v;
                                }
                            }
                        }
                        let caption_text = if !descr.trim().is_empty() {
                            descr.trim().to_string()
                        } else {
                            title.trim().to_string()
                        };
                        if !caption_text.is_empty() {
                            ir.push(
                                BlockKind::Caption { text: caption_text },
                                Locator::default(),
                            );
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(ref e) => {
                if skip_branch || ignore_instr {
                    continue;
                }
                if let Ok(txt) = e.unescape() {
                    append_run_text(
                        &txt,
                        in_r && r_vanish || p_vanish,
                        in_del,
                        &mut p_text,
                        &mut cell_stack,
                        &mut result,
                        ctx,
                    )?;
                }
            }
            Event::End(ref e) => {
                let local_name = e.local_name();
                let name = local_name.as_ref();

                if name == b"Choice" && in_alternate_content {
                    choice_processed = true;
                    skip_branch = true;
                } else if name == b"AlternateContent" {
                    in_alternate_content = false;
                    choice_processed = false;
                    skip_branch = false;
                }

                if skip_branch {
                    continue;
                }

                match name {
                    b"r" => {
                        in_r = false;
                        r_vanish = false;
                    }
                    b"instrText" => {
                        ignore_instr = false;
                    }
                    b"del" => {
                        in_del = false;
                    }
                    b"p" => {
                        if table_stack.is_empty() {
                            in_p = false;
                            emit_paragraph(
                                &p_text,
                                p_style_id.as_deref(),
                                p_outline_lvl,
                                p_num_id.as_deref(),
                                p_ilvl,
                                styles,
                                numbering,
                                ir,
                            );
                            p_text.clear();
                        }
                    }
                    b"tc" => {
                        if let Some(cell) = cell_stack.pop() {
                            if let Some(tbl) = table_stack.last_mut() {
                                if let Some(row) = tbl.last_mut() {
                                    ctx.budget.charge_cells(1)?;
                                    row.cells.push(cell);
                                }
                            }
                        }
                    }
                    b"tbl" => {
                        if let Some(raw_rows) = table_stack.pop() {
                            if table_stack.is_empty() {
                                emit_table(raw_rows, ir);
                            } else if let Some(parent_cell) = cell_stack.last_mut() {
                                // Nested table rendered as text into parent cell
                                for row in raw_rows {
                                    let row_txt = row
                                        .cells
                                        .iter()
                                        .map(|c| c.text.trim())
                                        .collect::<Vec<_>>()
                                        .join(" | ");
                                    if !row_txt.is_empty() {
                                        parent_cell.text.push_str(&row_txt);
                                        parent_cell.text.push('\n');
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(result)
}

fn append_run_text(
    txt: &str,
    is_hidden: bool,
    is_deleted: bool,
    p_text: &mut String,
    cell_stack: &mut [RawCell],
    result: &mut DocumentParseResult,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    if is_hidden && !ctx.policy.include_hidden {
        result.hidden_content_skipped = true;
        return Ok(());
    }

    if is_deleted && !ctx.policy.include_tracked_deletions {
        return Ok(());
    }

    ctx.budget
        .charge_text(u64::try_from(txt.len()).unwrap_or(u64::MAX))?;

    if let Some(cell) = cell_stack.last_mut() {
        cell.text.push_str(txt);
    } else {
        p_text.push_str(txt);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn emit_paragraph(
    text: &str,
    style_id: Option<&str>,
    outline_lvl: Option<u8>,
    num_id: Option<&str>,
    ilvl: Option<u8>,
    styles: &StyleMap,
    numbering: &NumMap,
    ir: &mut DocumentIr,
) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    // 1. Heading check
    let heading_level = outline_lvl
        .map(|l| l.saturating_add(1))
        .or_else(|| style_id.and_then(|sid| styles.heading_level(sid)));

    if let Some(level) = heading_level {
        ir.push(
            BlockKind::Heading {
                level: level.clamp(1, 6),
                text: trimmed.to_string(),
            },
            Locator::default(),
        );
        return;
    }

    // 2. List check
    if let Some(nid) = num_id {
        if nid != "0" {
            let depth = ilvl.unwrap_or(0);
            let ordered = numbering.is_ordered(nid, depth);
            ir.push(
                BlockKind::ListItem {
                    depth,
                    ordered,
                    text: trimmed.to_string(),
                },
                Locator::default(),
            );
            return;
        }
    }

    // 3. Regular Paragraph
    ir.push(
        BlockKind::Paragraph {
            text: trimmed.to_string(),
        },
        Locator::default(),
    );
}

fn emit_table(raw_rows: Vec<RawRow>, ir: &mut DocumentIr) {
    if raw_rows.is_empty() {
        return;
    }

    let mut header_rows = 0u8;
    for row in &raw_rows {
        if row.is_header {
            header_rows = header_rows.saturating_add(1);
        } else {
            break;
        }
    }

    // Compute max columns
    let mut max_cols = 0;
    for row in &raw_rows {
        let row_span: usize = row.cells.iter().map(|c| c.grid_span.max(1)).sum();
        max_cols = max_cols.max(row_span);
    }

    if max_cols == 0 {
        return;
    }

    let row_count = raw_rows.len();
    let mut matrix = vec![vec![String::new(); max_cols]; row_count];
    let mut active_vmerge = vec![String::new(); max_cols];

    for (r_idx, raw_row) in raw_rows.into_iter().enumerate() {
        let mut c_idx = 0;
        for cell in raw_row.cells {
            // Find next unpopulated column or apply vMerge
            while c_idx < max_cols
                && cell.v_merge == Some(VMerge::Continue)
                && matrix
                    .get(r_idx)
                    .and_then(|r| r.get(c_idx))
                    .is_some_and(|s| !s.is_empty())
            {
                c_idx += 1;
            }

            if c_idx >= max_cols {
                break;
            }

            let span = cell.grid_span.max(1);
            let cell_text = cell.text.trim().to_string();

            match cell.v_merge {
                Some(VMerge::Restart) => {
                    if let Some(v) = active_vmerge.get_mut(c_idx) {
                        *v = cell_text.clone();
                    }
                    for offset in 0..span {
                        if let Some(row) = matrix.get_mut(r_idx) {
                            if let Some(col) = row.get_mut(c_idx + offset) {
                                *col = cell_text.clone();
                            }
                        }
                    }
                }
                Some(VMerge::Continue) => {
                    let inherited = active_vmerge.get(c_idx).cloned().unwrap_or_default();
                    let text_to_use = if !cell_text.is_empty() {
                        cell_text
                    } else {
                        inherited
                    };
                    for offset in 0..span {
                        if let Some(row) = matrix.get_mut(r_idx) {
                            if let Some(col) = row.get_mut(c_idx + offset) {
                                *col = text_to_use.clone();
                            }
                        }
                    }
                }
                None => {
                    if let Some(v) = active_vmerge.get_mut(c_idx) {
                        v.clear();
                    }
                    for offset in 0..span {
                        if let Some(row) = matrix.get_mut(r_idx) {
                            if let Some(col) = row.get_mut(c_idx + offset) {
                                *col = cell_text.clone();
                            }
                        }
                    }
                }
            }

            c_idx += span;
        }
    }

    ir.push(
        BlockKind::Table(Table {
            caption: None,
            header_rows,
            rows: matrix,
            origin: (0, 0),
        }),
        Locator::default(),
    );
}
