// FILE-CONTEXT
// ZWECK: Tabular Chunking Strategy Implementation (WP-ING-13 / P00A / P06).
// INVARIANTEN: Zero-Panic, deterministisch, Spans im kanonischen Text, Fail-Closed.
// STAND: TS:2026-10-31T00:00:00Z

pub mod helpers;

use self::helpers::{col_to_letter, format_a1_range};
use crate::canonical::CanonicalText;
use crate::chunking::common::{token_count, window_split};
use crate::chunking::config::{ChunkingConfig, TabularMode};
use crate::chunking::ChunkDraft;
use crate::error::IngestError;
use crate::ir::{BlockKind, Boundary, DocumentIr, Locator, Table};
use serde_json::{json, Map, Value};
use std::ops::Range;

struct ValidatedRow {
    data_row_num: usize,
    orig_table_row_idx: usize,
    cells: Vec<String>,
}

struct TableContext<'a> {
    sheet_name: &'a str,
    table: &'a Table,
    table_index_in_sheet: u32,
    active_cols: &'a [usize],
    active_headers: &'a [String],
    row_byte_spans: &'a [Range<usize>],
    block_span: &'a Range<usize>,
}

pub fn draft_tabular(
    ir: &DocumentIr,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
) -> Result<Vec<ChunkDraft>, IngestError> {
    let hard_limit = if cfg.hard_limit_pct == 0 {
        cfg.max_tokens
    } else {
        cfg.max_tokens.saturating_mul(cfg.hard_limit_pct as usize) / 100
    };

    let mut drafts = Vec::new();
    let mut current_sheet_name = String::from("Sheet1");
    let mut sheet_table_count = 0u32;

    for (block_idx, block) in ir.blocks.iter().enumerate() {
        match &block.kind {
            BlockKind::Boundary(Boundary::Sheet { name, .. }) => {
                if !name.trim().is_empty() {
                    current_sheet_name = name.clone();
                }
                sheet_table_count = 0;
            }
            BlockKind::Table(table) => {
                sheet_table_count = sheet_table_count.saturating_add(1);
                let block_span = canon.block_spans.get(block_idx).cloned().unwrap_or(0..0);

                let sheet_name = block
                    .locator
                    .sheet
                    .as_ref()
                    .unwrap_or(&current_sheet_name)
                    .clone();

                let table_drafts = process_table(
                    table,
                    &sheet_name,
                    sheet_table_count,
                    &block_span,
                    canon,
                    cfg,
                    hard_limit,
                );

                drafts.extend(table_drafts);
            }
            _ => {}
        }
    }

    // Merge mini-sheets if total tokens < min_tokens (Requirement 2)
    let merged_drafts = maybe_merge_mini_sheets(drafts, cfg, hard_limit);

    Ok(merged_drafts)
}

fn process_table(
    table: &Table,
    sheet_name: &str,
    table_index_in_sheet: u32,
    block_span: &Range<usize>,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
    hard_limit: usize,
) -> Vec<ChunkDraft> {
    if table.rows.is_empty() {
        return Vec::new();
    }

    let orig_col_count = table.rows.iter().map(|r| r.len()).max().unwrap_or(0);

    if orig_col_count == 0 {
        return Vec::new();
    }

    // Determine raw headers
    let raw_headers: Vec<String> = (0..orig_col_count)
        .map(|c| {
            if table.header_rows > 0 {
                let mut parts = Vec::new();
                for r in 0..(table.header_rows as usize).min(table.rows.len()) {
                    if let Some(cell) = table.rows.get(r).and_then(|row| row.get(c)) {
                        let clean = cell.trim();
                        if !clean.is_empty() {
                            parts.push(clean);
                        }
                    }
                }
                if !parts.is_empty() {
                    parts.join(" ")
                } else {
                    let c_u32 = u32::try_from(c).unwrap_or(u32::MAX);
                    format!("Spalte {}", col_to_letter(c_u32))
                }
            } else {
                let c_u32 = u32::try_from(c).unwrap_or(u32::MAX);
                format!("Spalte {}", col_to_letter(c_u32))
            }
        })
        .collect();

    let data_start_row_idx = (table.header_rows as usize).min(table.rows.len());

    // Active columns
    let active_cols: Vec<usize> = if cfg.tabular.drop_empty_columns {
        (0..orig_col_count)
            .filter(|&c| {
                for r in data_start_row_idx..table.rows.len() {
                    if let Some(cell) = table.rows.get(r).and_then(|row| row.get(c)) {
                        if !cell.trim().is_empty() {
                            return true;
                        }
                    }
                }
                false
            })
            .collect()
    } else {
        (0..orig_col_count).collect()
    };

    if active_cols.is_empty() {
        return Vec::new();
    }

    let active_headers: Vec<String> = active_cols
        .iter()
        .filter_map(|&c| raw_headers.get(c).cloned())
        .collect();

    // Valid data rows
    let mut valid_data_rows = Vec::new();
    let mut data_row_counter = 0usize;

    for orig_row_idx in data_start_row_idx..table.rows.len() {
        data_row_counter = data_row_counter.saturating_add(1);
        let row_cells = table.rows.get(orig_row_idx);

        let extracted_cells: Vec<String> = active_cols
            .iter()
            .map(|&c| {
                row_cells
                    .and_then(|r| r.get(c))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default()
            })
            .collect();

        let is_empty = extracted_cells.iter().all(|c| c.is_empty());
        if cfg.tabular.skip_empty_rows && is_empty {
            continue;
        }

        valid_data_rows.push(ValidatedRow {
            data_row_num: data_row_counter,
            orig_table_row_idx: orig_row_idx,
            cells: extracted_cells,
        });
    }

    if valid_data_rows.is_empty() {
        return Vec::new();
    }

    // Determine Mode
    let mode = match cfg.tabular.mode {
        TabularMode::RowGroups => TabularMode::RowGroups,
        TabularMode::RecordRows => TabularMode::RecordRows,
        TabularMode::Auto => {
            let total_chars: usize = valid_data_rows
                .iter()
                .flat_map(|r| &r.cells)
                .map(|c| c.chars().count())
                .sum();
            let total_cells = valid_data_rows.len().saturating_mul(active_cols.len());
            let avg_cell_len = if total_cells > 0 {
                total_chars / total_cells
            } else {
                0
            };

            if active_cols.len() >= (cfg.tabular.wide_table_cols as usize) || avg_cell_len > 40 {
                TabularMode::RecordRows
            } else {
                TabularMode::RowGroups
            }
        }
    };

    // Calculate row byte spans in canon.text
    let row_byte_spans = compute_row_byte_spans(table, block_span, canon);

    let ctx = TableContext {
        sheet_name,
        table,
        table_index_in_sheet,
        active_cols: &active_cols,
        active_headers: &active_headers,
        row_byte_spans: &row_byte_spans,
        block_span,
    };

    let mut chunks = Vec::new();

    // First check: Small table (<= hard_limit) -> single chunk
    let valid_refs: Vec<&ValidatedRow> = valid_data_rows.iter().collect();
    let full_draft_text = format_row_group(
        sheet_name,
        table.caption.as_deref(),
        &active_headers,
        &valid_refs,
        mode,
    );

    if token_count(&full_draft_text) <= hard_limit {
        if let (Some(first_row), Some(last_row)) = (valid_data_rows.first(), valid_data_rows.last())
        {
            let span = get_span_for_rows(
                first_row.orig_table_row_idx,
                last_row.orig_table_row_idx,
                &row_byte_spans,
                block_span,
            );

            let draft = build_draft(
                full_draft_text,
                span,
                &ctx,
                first_row.data_row_num,
                last_row.data_row_num,
            );
            chunks.push(draft);
            return chunks;
        }
    }

    // Larger table: group rows sequentially
    let mut current_group: Vec<&ValidatedRow> = Vec::new();

    for row in &valid_data_rows {
        let mut candidate_group = current_group.clone();
        candidate_group.push(row);

        let candidate_text = format_row_group(
            sheet_name,
            table.caption.as_deref(),
            &active_headers,
            &candidate_group,
            mode,
        );

        if token_count(&candidate_text) <= hard_limit {
            current_group.push(row);
        } else if !current_group.is_empty() {
            // Flush current group
            if let (Some(first_row), Some(last_row)) = (current_group.first(), current_group.last())
            {
                let text = format_row_group(
                    sheet_name,
                    table.caption.as_deref(),
                    &active_headers,
                    &current_group,
                    mode,
                );
                let span = get_span_for_rows(
                    first_row.orig_table_row_idx,
                    last_row.orig_table_row_idx,
                    &row_byte_spans,
                    block_span,
                );

                chunks.push(build_draft(
                    text,
                    span,
                    &ctx,
                    first_row.data_row_num,
                    last_row.data_row_num,
                ));
            }

            current_group = vec![row];
            let single_text = format_row_group(
                sheet_name,
                table.caption.as_deref(),
                &active_headers,
                &current_group,
                mode,
            );
            if token_count(&single_text) > hard_limit {
                // Oversized row
                let oversized_chunks = handle_oversized_row(row, &ctx, hard_limit, mode);
                chunks.extend(oversized_chunks);
                current_group.clear();
            }
        } else {
            // Row alone exceeds hard_limit
            let oversized_chunks = handle_oversized_row(row, &ctx, hard_limit, mode);
            chunks.extend(oversized_chunks);
        }
    }

    if !current_group.is_empty() {
        if let (Some(first_row), Some(last_row)) = (current_group.first(), current_group.last()) {
            let text = format_row_group(
                sheet_name,
                table.caption.as_deref(),
                &active_headers,
                &current_group,
                mode,
            );
            let span = get_span_for_rows(
                first_row.orig_table_row_idx,
                last_row.orig_table_row_idx,
                &row_byte_spans,
                block_span,
            );

            chunks.push(build_draft(
                text,
                span,
                &ctx,
                first_row.data_row_num,
                last_row.data_row_num,
            ));
        }
    }

    chunks
}

fn format_row_group(
    sheet_name: &str,
    caption: Option<&str>,
    active_headers: &[String],
    rows: &[&ValidatedRow],
    mode: TabularMode,
) -> String {
    if rows.is_empty() {
        return String::new();
    }

    let first_num = rows.first().map(|r| r.data_row_num).unwrap_or(1);
    let last_num = rows.last().map(|r| r.data_row_num).unwrap_or(1);

    let mut out = String::new();
    out.push_str(&format!(
        "Sheet: {} (Zeilen {}–{})\n",
        sheet_name, first_num, last_num
    ));

    if let Some(cap) = caption {
        out.push_str("Table: ");
        out.push_str(cap);
        out.push('\n');
    }

    match mode {
        TabularMode::RowGroups | TabularMode::Auto => {
            // GFM Table
            let escaped_headers: Vec<String> = active_headers
                .iter()
                .map(|h| h.replace('|', "\\|").replace('\n', " "))
                .collect();
            out.push('|');
            for h in &escaped_headers {
                out.push(' ');
                out.push_str(h);
                out.push_str(" |");
            }
            out.push('\n');

            out.push('|');
            for _ in active_headers {
                out.push_str("---|");
            }
            out.push('\n');

            for row in rows {
                out.push('|');
                for cell in &row.cells {
                    out.push(' ');
                    let escaped = cell.replace('|', "\\|").replace('\n', " ");
                    out.push_str(&escaped);
                    out.push_str(" |");
                }
                out.push('\n');
            }
        }
        TabularMode::RecordRows => {
            for row in rows {
                let mut pairs = Vec::new();
                for (hdr, cell) in active_headers.iter().zip(row.cells.iter()) {
                    let clean_val = cell.trim();
                    if !clean_val.is_empty() {
                        let escaped_val = clean_val.replace('\n', " ");
                        pairs.push(format!("{}: {}", hdr, escaped_val));
                    }
                }
                out.push_str(&pairs.join("; "));
                out.push('\n');
            }
        }
    }

    if out.ends_with('\n') {
        out.pop();
    }

    out
}

fn handle_oversized_row(
    row: &ValidatedRow,
    ctx: &TableContext<'_>,
    hard_limit: usize,
    mode: TabularMode,
) -> Vec<ChunkDraft> {
    let mut chunks = Vec::new();
    let span = get_span_for_rows(
        row.orig_table_row_idx,
        row.orig_table_row_idx,
        ctx.row_byte_spans,
        ctx.block_span,
    );

    // Check if any single cell is gigantic
    let has_gigantic_cell = row
        .cells
        .iter()
        .any(|c| c.chars().count() > hard_limit.saturating_mul(4));

    if has_gigantic_cell {
        let window_chars = hard_limit.saturating_mul(2).clamp(100, 2000);
        let overlap_chars = window_chars / 5;

        for (col_idx, (hdr, cell)) in ctx.active_headers.iter().zip(row.cells.iter()).enumerate() {
            if cell.chars().count() > window_chars {
                let slices = window_split(cell, window_chars, overlap_chars);
                let active_sub_cols = vec![ctx.active_cols.get(col_idx).copied().unwrap_or(0)];
                let active_sub_headers = vec![hdr.clone()];

                let sub_ctx = TableContext {
                    sheet_name: ctx.sheet_name,
                    table: ctx.table,
                    table_index_in_sheet: ctx.table_index_in_sheet,
                    active_cols: &active_sub_cols,
                    active_headers: &active_sub_headers,
                    row_byte_spans: ctx.row_byte_spans,
                    block_span: ctx.block_span,
                };

                for slice_span in slices {
                    let cell_slice = cell.get(slice_span).unwrap_or("");
                    let text = format!(
                        "Sheet: {} (Zeilen {}–{})\n[Column: {}]\n{}",
                        ctx.sheet_name, row.data_row_num, row.data_row_num, hdr, cell_slice
                    );

                    chunks.push(build_draft(
                        text,
                        span.clone(),
                        &sub_ctx,
                        row.data_row_num,
                        row.data_row_num,
                    ));
                }
            }
        }
        if !chunks.is_empty() {
            return chunks;
        }
    }

    // Split row by column subsets
    let mut col_start = 0;
    while col_start < ctx.active_headers.len() {
        let mut col_end = col_start.saturating_add(1);
        while col_end <= ctx.active_headers.len() {
            let sub_headers = ctx.active_headers.get(col_start..col_end).unwrap_or(&[]);
            let sub_cells: Vec<String> = row.cells.get(col_start..col_end).unwrap_or(&[]).to_vec();
            let sub_row = ValidatedRow {
                data_row_num: row.data_row_num,
                orig_table_row_idx: row.orig_table_row_idx,
                cells: sub_cells,
            };

            let text = format_row_group(
                ctx.sheet_name,
                ctx.table.caption.as_deref(),
                sub_headers,
                &[&sub_row],
                mode,
            );

            if token_count(&text) > hard_limit && col_end > col_start.saturating_add(1) {
                col_end = col_end.saturating_sub(1);
                break;
            }
            if col_end == ctx.active_headers.len() {
                break;
            }
            col_end = col_end.saturating_add(1);
        }

        let sub_cols = ctx
            .active_cols
            .get(col_start..col_end)
            .unwrap_or(&[])
            .to_vec();
        let sub_headers = ctx
            .active_headers
            .get(col_start..col_end)
            .unwrap_or(&[])
            .to_vec();
        let sub_cells = row.cells.get(col_start..col_end).unwrap_or(&[]).to_vec();
        let sub_row = ValidatedRow {
            data_row_num: row.data_row_num,
            orig_table_row_idx: row.orig_table_row_idx,
            cells: sub_cells,
        };

        let sub_ctx = TableContext {
            sheet_name: ctx.sheet_name,
            table: ctx.table,
            table_index_in_sheet: ctx.table_index_in_sheet,
            active_cols: &sub_cols,
            active_headers: &sub_headers,
            row_byte_spans: ctx.row_byte_spans,
            block_span: ctx.block_span,
        };

        let text = format_row_group(
            ctx.sheet_name,
            ctx.table.caption.as_deref(),
            &sub_headers,
            &[&sub_row],
            mode,
        );

        chunks.push(build_draft(
            text,
            span.clone(),
            &sub_ctx,
            row.data_row_num,
            row.data_row_num,
        ));

        col_start = col_end;
    }

    chunks
}

fn build_draft(
    text: String,
    span: Range<usize>,
    ctx: &TableContext<'_>,
    first_data_row_num: usize,
    last_data_row_num: usize,
) -> ChunkDraft {
    let locator = Locator {
        sheet: Some(ctx.sheet_name.to_string()),
        range: Some(format_a1_range(
            ctx.table.origin,
            ctx.table.header_rows,
            ctx.active_cols,
            first_data_row_num,
            last_data_row_num,
        )),
        ..Default::default()
    };

    let caption_or_table = ctx
        .table
        .caption
        .clone()
        .unwrap_or_else(|| format!("Tabelle {}", ctx.table_index_in_sheet));
    let breadcrumb = format!("{} > {}", ctx.sheet_name, caption_or_table);

    let mut extra = Map::new();
    extra.insert(
        "rows".into(),
        json!({
            "first": first_data_row_num,
            "last": last_data_row_num,
        }),
    );
    extra.insert("header".into(), json!(ctx.active_headers));
    extra.insert("synthesized".into(), Value::Bool(true));

    ChunkDraft {
        text,
        span,
        locator,
        breadcrumb: Some(breadcrumb),
        heading_level: None,
        extra,
    }
}

fn compute_row_byte_spans(
    table: &Table,
    block_span: &Range<usize>,
    canon: &CanonicalText,
) -> Vec<Range<usize>> {
    let mut spans = Vec::with_capacity(table.rows.len());
    let block_str = match canon.text.get(block_span.clone()) {
        Some(s) => s,
        None => return spans,
    };

    let mut lines = Vec::new();
    let mut cursor = 0usize;

    for line in block_str.split('\n') {
        let line_len = line.len();
        let end = cursor.saturating_add(line_len);
        lines.push(cursor..end);
        cursor = end.saturating_add(1); // \n offset
    }

    let mut line_idx = 0usize;
    if table.caption.is_some() {
        line_idx = line_idx.saturating_add(1);
    }

    for table_row_idx in 0..table.rows.len() {
        if let Some(line_range) = lines.get(line_idx) {
            let abs_start = block_span.start.saturating_add(line_range.start);
            let abs_end = block_span.start.saturating_add(line_range.end);
            spans.push(abs_start..abs_end);
        } else {
            spans.push(block_span.clone());
        }

        line_idx = line_idx.saturating_add(1);
        if table_row_idx == 0 && !table.rows.is_empty() {
            // separator row line
            line_idx = line_idx.saturating_add(1);
        }
    }

    spans
}

fn get_span_for_rows(
    first_orig_idx: usize,
    last_orig_idx: usize,
    row_spans: &[Range<usize>],
    block_span: &Range<usize>,
) -> Range<usize> {
    let start = row_spans
        .get(first_orig_idx)
        .map(|r| r.start)
        .unwrap_or(block_span.start);
    let end = row_spans
        .get(last_orig_idx)
        .map(|r| r.end)
        .unwrap_or(block_span.end);

    let start = start.clamp(block_span.start, block_span.end);
    let end = end.clamp(start, block_span.end);

    start..end
}

fn maybe_merge_mini_sheets(
    drafts: Vec<ChunkDraft>,
    cfg: &ChunkingConfig,
    hard_limit: usize,
) -> Vec<ChunkDraft> {
    if drafts.len() <= 1 || cfg.min_tokens == 0 {
        return drafts;
    }

    // Group drafts by sheet name
    let mut sheet_groups: Vec<(String, Vec<ChunkDraft>)> = Vec::new();
    for draft in drafts {
        let sheet_name = draft
            .locator
            .sheet
            .clone()
            .unwrap_or_else(|| "Sheet1".into());

        if let Some((last_name, group)) = sheet_groups.last_mut() {
            if last_name == &sheet_name {
                group.push(draft);
                continue;
            }
        }
        sheet_groups.push((sheet_name, vec![draft]));
    }

    let mut final_drafts = Vec::new();
    let mut i = 0;

    while i < sheet_groups.len() {
        if let Some((curr_sheet_name, curr_chunks)) = sheet_groups.get(i) {
            if curr_chunks.len() == 1 {
                if let Some(curr_draft) = curr_chunks.first() {
                    let curr_tokens = token_count(&curr_draft.text);
                    if curr_tokens < cfg.min_tokens && i + 1 < sheet_groups.len() {
                        if let Some((next_sheet_name, next_chunks)) = sheet_groups.get(i + 1) {
                            if next_chunks.len() == 1 {
                                if let Some(next_draft) = next_chunks.first() {
                                    let combined_text =
                                        format!("{}\n\n{}", curr_draft.text, next_draft.text);
                                    if token_count(&combined_text) <= hard_limit {
                                        let merged_span =
                                            curr_draft.span.start..next_draft.span.end;
                                        let mut merged_draft = curr_draft.clone();
                                        merged_draft.text = combined_text;
                                        merged_draft.span = merged_span;
                                        merged_draft.extra.insert(
                                            "sheets".into(),
                                            json!([curr_sheet_name, next_sheet_name]),
                                        );

                                        final_drafts.push(merged_draft);
                                        i += 2;
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            final_drafts.extend(curr_chunks.clone());
            i += 1;
        } else {
            break;
        }
    }

    final_drafts
}
