// FILE-CONTEXT
// ZWECK: Seiten- und Folien-Verarbeitung für PagedChunker (WP-ING-14).
// INVARIANTEN: Zero-Panic, Zitierbarkeit über page/slide locators, monotone Spans.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::common::{token_count, window_split};
use crate::chunking::config::ChunkingConfig;
use crate::chunking::ChunkDraft;
use crate::error::IngestError;
use crate::ir::{BlockKind, Boundary, DocumentIr, NoteKind};
use serde_json::{Map, Value};
use std::ops::Range;

#[derive(Debug, Clone)]
struct PageGroup {
    is_slide: bool,
    start_number: u32,
    end_number: u32,
    header: Option<String>,
    footer: Option<String>,
    body_block_indices: Vec<usize>,
    speaker_note_indices: Vec<usize>,
    breadcrumb: Option<String>,
    heading_level: Option<u8>,
    cross_page_join: bool,
}

/// Verarbeitet das Dokument seiten- bzw. folienbasiert.
pub fn process_paged_document(
    ir: &DocumentIr,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
) -> Result<Vec<ChunkDraft>, IngestError> {
    if ir.blocks.is_empty() {
        return Ok(Vec::new());
    }

    let hard_limit = (cfg.max_tokens.saturating_mul(cfg.hard_limit_pct as usize)) / 100;
    let hard_limit = hard_limit.max(1);

    // 1. Erkennen, ob Präsentation (Slide) oder PDF (Page)
    let is_presentation = ir.blocks.iter().any(|b| {
        matches!(b.kind, BlockKind::Boundary(Boundary::Slide { .. })) || b.locator.slide.is_some()
    });

    // 2. Blöcke in initiale Seiten/Folien-Gruppen aufteilen
    let mut raw_groups = collect_initial_groups(ir, cfg, is_presentation);

    // Leere Seiten/Folien ohne echten Text herausfiltern
    raw_groups.retain(|g| !is_group_empty(g, canon));

    if raw_groups.is_empty() {
        return Ok(Vec::new());
    }

    // 3. Optional: join_cross_page_paragraphs
    if cfg.paged.join_cross_page_paragraphs && !is_presentation {
        raw_groups = apply_cross_page_join(raw_groups, ir, canon);
    }

    // 4. Optional: merge_small_pages
    let groups = if cfg.paged.merge_small_pages {
        apply_merge_small_pages(raw_groups, canon, cfg.min_tokens, hard_limit)
    } else {
        raw_groups
    };

    // 5. Chunks aus den Gruppen generieren
    let mut drafts = Vec::new();

    for group in groups {
        generate_drafts_for_group(group, ir, canon, cfg, hard_limit, &mut drafts);
    }

    Ok(drafts)
}

fn collect_initial_groups(
    ir: &DocumentIr,
    cfg: &ChunkingConfig,
    is_presentation: bool,
) -> Vec<PageGroup> {
    let mut groups: Vec<PageGroup> = Vec::new();
    let mut current_is_slide = is_presentation;
    let mut current_number = 1u32;
    let mut current_title: Option<String> = None;
    let mut current_header: Option<String> = None;
    let mut current_footer: Option<String> = None;
    let mut current_body_indices = Vec::new();
    let mut current_speaker_indices = Vec::new();

    let mut heading_stack: Vec<(u8, String)> = Vec::new();
    let mut last_heading_level: Option<u8> = None;

    for (idx, block) in ir.blocks.iter().enumerate() {
        match &block.kind {
            BlockKind::Boundary(boundary) => {
                if !current_body_indices.is_empty()
                    || !current_speaker_indices.is_empty()
                    || current_header.is_some()
                    || current_footer.is_some()
                {
                    let bc = build_breadcrumb(
                        current_is_slide,
                        current_number,
                        current_title.as_deref(),
                        &heading_stack,
                        cfg.include_breadcrumbs,
                    );
                    groups.push(PageGroup {
                        is_slide: current_is_slide,
                        start_number: current_number,
                        end_number: current_number,
                        header: current_header.take(),
                        footer: current_footer.take(),
                        body_block_indices: std::mem::take(&mut current_body_indices),
                        speaker_note_indices: std::mem::take(&mut current_speaker_indices),
                        breadcrumb: bc,
                        heading_level: last_heading_level,
                        cross_page_join: false,
                    });
                }

                match boundary {
                    Boundary::Page { number } => {
                        current_is_slide = false;
                        current_number = *number;
                        current_title = None;
                    }
                    Boundary::Slide { number, title } => {
                        current_is_slide = true;
                        current_number = *number;
                        current_title = title.clone();
                    }
                    _ => {}
                }
            }
            BlockKind::Heading { level, text } => {
                last_heading_level = Some(*level);
                let hashes = "#".repeat((*level as usize).clamp(1, 6));
                let fmt_text = format!("{} {}", hashes, text);
                heading_stack.retain(|(l, _)| *l < *level);
                heading_stack.push((*level, fmt_text));
                current_body_indices.push(idx);
            }
            BlockKind::Note { kind, text } => match kind {
                NoteKind::Header => {
                    current_header = Some(text.clone());
                }
                NoteKind::Footer => {
                    current_footer = Some(text.clone());
                }
                NoteKind::SpeakerNotes => {
                    if cfg.paged.notes_with_slide {
                        current_body_indices.push(idx);
                    } else {
                        current_speaker_indices.push(idx);
                    }
                }
                _ => {
                    current_body_indices.push(idx);
                }
            },
            _ => {
                current_body_indices.push(idx);
            }
        }
    }

    if !current_body_indices.is_empty()
        || !current_speaker_indices.is_empty()
        || current_header.is_some()
        || current_footer.is_some()
    {
        let bc = build_breadcrumb(
            current_is_slide,
            current_number,
            current_title.as_deref(),
            &heading_stack,
            cfg.include_breadcrumbs,
        );
        groups.push(PageGroup {
            is_slide: current_is_slide,
            start_number: current_number,
            end_number: current_number,
            header: current_header,
            footer: current_footer,
            body_block_indices: current_body_indices,
            speaker_note_indices: current_speaker_indices,
            breadcrumb: bc,
            heading_level: last_heading_level,
            cross_page_join: false,
        });
    }

    groups
}

fn build_breadcrumb(
    is_slide: bool,
    number: u32,
    title: Option<&str>,
    heading_stack: &[(u8, String)],
    include_breadcrumbs: bool,
) -> Option<String> {
    if !include_breadcrumbs {
        return None;
    }
    let mut parts = Vec::new();
    if is_slide {
        if let Some(t) = title {
            parts.push(format!("Folie {}: {}", number, t));
        } else {
            parts.push(format!("Folie {}", number));
        }
    }
    for (_, text) in heading_stack {
        parts.push(text.clone());
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" > "))
    }
}

fn apply_cross_page_join(
    groups: Vec<PageGroup>,
    ir: &DocumentIr,
    canon: &CanonicalText,
) -> Vec<PageGroup> {
    if groups.len() < 2 {
        return groups;
    }

    let mut merged = Vec::with_capacity(groups.len());
    let mut i = 0;

    while i < groups.len() {
        let mut curr = match groups.get(i) {
            Some(g) => g.clone(),
            None => break,
        };

        while i + 1 < groups.len() {
            let next_g = match groups.get(i + 1) {
                Some(g) => g,
                None => break,
            };

            if should_join_pages(&curr, next_g, ir, canon) {
                curr.end_number = next_g.end_number;
                curr.body_block_indices
                    .extend_from_slice(&next_g.body_block_indices);
                curr.speaker_note_indices
                    .extend_from_slice(&next_g.speaker_note_indices);
                if curr.footer.is_none() {
                    curr.footer = next_g.footer.clone();
                }
                curr.cross_page_join = true;
                i += 1;
            } else {
                break;
            }
        }

        merged.push(curr);
        i += 1;
    }

    merged
}

fn should_join_pages(
    g1: &PageGroup,
    g2: &PageGroup,
    ir: &DocumentIr,
    canon: &CanonicalText,
) -> bool {
    if g1.is_slide || g2.is_slide {
        return false;
    }

    let last_block_idx = match g1.body_block_indices.last() {
        Some(&idx) => idx,
        None => return false,
    };

    let first_block_idx = match g2.body_block_indices.first() {
        Some(&idx) => idx,
        None => return false,
    };

    let last_block = match ir.blocks.get(last_block_idx) {
        Some(b) => b,
        None => return false,
    };

    let first_block = match ir.blocks.get(first_block_idx) {
        Some(b) => b,
        None => return false,
    };

    let last_text = match get_block_text(&last_block.kind, last_block_idx, canon) {
        Some(t) => t,
        None => return false,
    };

    let first_text = match get_block_text(&first_block.kind, first_block_idx, canon) {
        Some(t) => t,
        None => return false,
    };

    let last_trimmed = last_text.trim_end();
    let last_char = match last_trimmed.chars().last() {
        Some(c) => c,
        None => return false,
    };

    if is_sentence_terminal(last_char) {
        return false;
    }

    let first_trimmed = first_text.trim_start();
    let first_char = match first_trimmed.chars().next() {
        Some(c) => c,
        None => return false,
    };

    first_char.is_lowercase()
}

fn is_sentence_terminal(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | ':' | '；' | '。' | '！' | '？')
}

fn get_block_text<'a>(
    kind: &'a BlockKind,
    idx: usize,
    canon: &'a CanonicalText,
) -> Option<&'a str> {
    match kind {
        BlockKind::Paragraph { text } | BlockKind::Quote { text } | BlockKind::Raw { text } => {
            Some(text.as_str())
        }
        _ => canon
            .block_spans
            .get(idx)
            .and_then(|span| canon.text.get(span.clone())),
    }
}

fn is_group_empty(group: &PageGroup, canon: &CanonicalText) -> bool {
    if !group.speaker_note_indices.is_empty() || group.header.is_some() || group.footer.is_some() {
        return false;
    }
    for &idx in &group.body_block_indices {
        if let Some(span) = canon.block_spans.get(idx) {
            if let Some(text) = canon.text.get(span.clone()) {
                if !text.trim().is_empty() {
                    return false;
                }
            }
        }
    }
    true
}

fn apply_merge_small_pages(
    groups: Vec<PageGroup>,
    canon: &CanonicalText,
    _min_tokens: usize,
    hard_limit: usize,
) -> Vec<PageGroup> {
    if groups.len() < 2 {
        return groups;
    }

    let mut merged = Vec::with_capacity(groups.len());
    let mut i = 0;

    while i < groups.len() {
        let mut curr = match groups.get(i) {
            Some(g) => g.clone(),
            None => break,
        };

        let mut curr_tokens = calc_group_tokens(&curr, canon);

        while i + 1 < groups.len() {
            let next_g = match groups.get(i + 1) {
                Some(g) => g,
                None => break,
            };

            if curr.is_slide != next_g.is_slide {
                break;
            }

            let next_tokens = calc_group_tokens(next_g, canon);

            if curr_tokens.saturating_add(next_tokens) <= hard_limit {
                curr.end_number = next_g.end_number;
                curr.body_block_indices
                    .extend_from_slice(&next_g.body_block_indices);
                curr.speaker_note_indices
                    .extend_from_slice(&next_g.speaker_note_indices);
                if curr.footer.is_none() {
                    curr.footer = next_g.footer.clone();
                }
                curr_tokens = curr_tokens.saturating_add(next_tokens);
                i += 1;
            } else {
                break;
            }
        }

        merged.push(curr);
        i += 1;
    }

    merged
}

fn calc_group_tokens(group: &PageGroup, canon: &CanonicalText) -> usize {
    let mut total: usize = 0;
    for &idx in &group.body_block_indices {
        if let Some(span) = canon.block_spans.get(idx) {
            if let Some(text) = canon.text.get(span.clone()) {
                total = total.saturating_add(token_count(text));
            }
        }
    }
    total
}

fn generate_drafts_for_group(
    group: PageGroup,
    ir: &DocumentIr,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
    hard_limit: usize,
    drafts: &mut Vec<ChunkDraft>,
) {
    if group.body_block_indices.is_empty() && group.speaker_note_indices.is_empty() {
        return;
    }

    let effective_hard_limit = if group.cross_page_join {
        (hard_limit.saturating_mul(3)) / 2
    } else {
        hard_limit
    };

    // Body block text verarbeiten
    if !group.body_block_indices.is_empty() {
        let total_tok = calc_group_tokens(&group, canon);

        if total_tok <= effective_hard_limit {
            if let Some((span, text)) = get_span_and_text(&group.body_block_indices, canon) {
                if !text.trim().is_empty() {
                    let mut extra = Map::new();
                    if let Some(ref h) = group.header {
                        extra.insert("header".into(), Value::String(h.clone()));
                    }
                    if let Some(ref f) = group.footer {
                        extra.insert("footer".into(), Value::String(f.clone()));
                    }
                    if group.cross_page_join {
                        extra.insert("cross_page_join".into(), Value::Bool(true));
                    }

                    let locator =
                        make_locator(group.is_slide, group.start_number, group.end_number);

                    drafts.push(ChunkDraft {
                        text,
                        span,
                        locator,
                        breadcrumb: group.breadcrumb.clone(),
                        heading_level: group.heading_level,
                        extra,
                    });
                }
            }
        } else {
            split_large_group(&group, ir, canon, cfg, effective_hard_limit, drafts);
        }
    }

    // Separate speaker notes (falls notes_with_slide = false)
    if !group.speaker_note_indices.is_empty() {
        for &sn_idx in &group.speaker_note_indices {
            if let Some(span) = canon.block_spans.get(sn_idx) {
                if let Some(text) = canon.text.get(span.clone()) {
                    if !text.trim().is_empty() {
                        let mut extra = Map::new();
                        extra.insert("speaker_notes".into(), Value::Bool(true));
                        let locator =
                            make_locator(group.is_slide, group.start_number, group.end_number);

                        drafts.push(ChunkDraft {
                            text: text.to_string(),
                            span: span.clone(),
                            locator,
                            breadcrumb: group.breadcrumb.clone(),
                            heading_level: group.heading_level,
                            extra,
                        });
                    }
                }
            }
        }
    }
}

fn make_locator(is_slide: bool, start: u32, end: u32) -> crate::ir::Locator {
    let mut loc = crate::ir::Locator::default();
    if is_slide {
        loc.slide = Some(start);
        loc.slide_end = Some(end);
    } else {
        loc.page = Some(start);
        loc.page_end = Some(end);
    }
    loc
}

fn get_span_and_text(indices: &[usize], canon: &CanonicalText) -> Option<(Range<usize>, String)> {
    let first_idx = *indices.first()?;
    let last_idx = *indices.last()?;

    let start_span = canon.block_spans.get(first_idx)?;
    let end_span = canon.block_spans.get(last_idx)?;
    let full_span = start_span.start..end_span.end;

    let text = canon.text.get(full_span.clone())?.to_string();
    Some((full_span, text))
}

fn split_large_group(
    group: &PageGroup,
    ir: &DocumentIr,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
    hard_limit: usize,
    drafts: &mut Vec<ChunkDraft>,
) {
    let mut raw_parts: Vec<(Range<usize>, String)> = Vec::new();

    let mut current_batch = Vec::new();
    let mut current_tok = 0usize;

    for &idx in &group.body_block_indices {
        let block = match ir.blocks.get(idx) {
            Some(b) => b,
            None => continue,
        };

        let block_span = match canon.block_spans.get(idx) {
            Some(s) => s.clone(),
            None => continue,
        };

        let block_text = match canon.text.get(block_span.clone()) {
            Some(t) => t,
            None => continue,
        };

        let b_tok = token_count(block_text);

        if b_tok > hard_limit {
            if !current_batch.is_empty() {
                if let Some(pt) = get_span_and_text(&current_batch, canon) {
                    raw_parts.push(pt);
                }
                current_batch.clear();
                current_tok = 0;
            }

            match &block.kind {
                BlockKind::Table(table) => {
                    split_table_block(table, block_span, canon, hard_limit, &mut raw_parts);
                }
                _ => {
                    split_paragraph_block(block_text, block_span, cfg, hard_limit, &mut raw_parts);
                }
            }
        } else if current_tok.saturating_add(b_tok) <= hard_limit {
            current_batch.push(idx);
            current_tok = current_tok.saturating_add(b_tok);
        } else {
            if !current_batch.is_empty() {
                if let Some(pt) = get_span_and_text(&current_batch, canon) {
                    raw_parts.push(pt);
                }
                current_batch.clear();
            }
            current_batch.push(idx);
            current_tok = b_tok;
        }
    }

    if !current_batch.is_empty() {
        if let Some(pt) = get_span_and_text(&current_batch, canon) {
            raw_parts.push(pt);
        }
    }

    let total_parts = raw_parts.len();

    for (part_idx, (span, text)) in raw_parts.into_iter().enumerate() {
        if text.trim().is_empty() {
            continue;
        }
        let part_num = part_idx.saturating_add(1);
        let part_str = format!("{}/{}", part_num, total_parts);

        let mut extra = Map::new();
        extra.insert("part".into(), Value::String(part_str));
        if let Some(ref h) = group.header {
            extra.insert("header".into(), Value::String(h.clone()));
        }
        if let Some(ref f) = group.footer {
            extra.insert("footer".into(), Value::String(f.clone()));
        }
        if group.cross_page_join {
            extra.insert("cross_page_join".into(), Value::Bool(true));
        }

        let locator = make_locator(group.is_slide, group.start_number, group.end_number);

        drafts.push(ChunkDraft {
            text,
            span,
            locator,
            breadcrumb: group.breadcrumb.clone(),
            heading_level: group.heading_level,
            extra,
        });
    }
}

fn split_table_block(
    table: &crate::ir::Table,
    block_span: Range<usize>,
    canon: &CanonicalText,
    hard_limit: usize,
    out: &mut Vec<(Range<usize>, String)>,
) {
    if table.rows.is_empty() {
        if let Some(text) = canon.text.get(block_span.clone()) {
            out.push((block_span, text.to_string()));
        }
        return;
    }

    let header_rows_cnt = if table.header_rows == 0 && !table.rows.is_empty() {
        1
    } else {
        table.header_rows as usize
    };

    let header_rows_cnt = header_rows_cnt.min(table.rows.len());

    let header_slice = table.rows.get(..header_rows_cnt).unwrap_or(&[]);
    let mut header_str = String::new();

    if let Some(ref cap) = table.caption {
        header_str.push_str("Table: ");
        header_str.push_str(cap);
        header_str.push('\n');
    }

    for (row_idx, row) in header_slice.iter().enumerate() {
        header_str.push('|');
        for cell in row {
            header_str.push(' ');
            let escaped = cell.replace('|', "\\|").replace('\n', " ");
            header_str.push_str(&escaped);
            header_str.push_str(" |");
        }
        header_str.push('\n');

        if row_idx == 0 {
            header_str.push('|');
            for _ in row {
                header_str.push_str("---|");
            }
            header_str.push('\n');
        }
    }

    let data_rows = table.rows.get(header_rows_cnt..).unwrap_or(&[]);

    if data_rows.is_empty() {
        if let Some(text) = canon.text.get(block_span.clone()) {
            out.push((block_span, text.to_string()));
        }
        return;
    }

    let mut current_data_str = String::new();
    let mut current_rows_cnt = 0;

    for row in data_rows {
        let mut row_str = String::new();
        row_str.push('|');
        for cell in row {
            row_str.push(' ');
            let escaped = cell.replace('|', "\\|").replace('\n', " ");
            row_str.push_str(&escaped);
            row_str.push_str(" |");
        }
        row_str.push('\n');

        let row_tok = token_count(&row_str);
        let curr_tok = token_count(&header_str).saturating_add(token_count(&current_data_str));

        if curr_tok.saturating_add(row_tok) <= hard_limit || current_rows_cnt == 0 {
            current_data_str.push_str(&row_str);
            current_rows_cnt += 1;
        } else {
            let data_trimmed = current_data_str.trim_end();
            let sub_span = find_sub_span(canon, block_span.clone(), data_trimmed);
            let full_chunk_text = format!("{}{}", header_str, data_trimmed);
            out.push((sub_span, full_chunk_text));

            current_data_str.clear();
            current_data_str.push_str(&row_str);
            current_rows_cnt = 1;
        }
    }

    if current_rows_cnt > 0 {
        let data_trimmed = current_data_str.trim_end();
        let sub_span = find_sub_span(canon, block_span, data_trimmed);
        let full_chunk_text = format!("{}{}", header_str, data_trimmed);
        out.push((sub_span, full_chunk_text));
    }
}

fn split_paragraph_block(
    text: &str,
    block_span: Range<usize>,
    cfg: &ChunkingConfig,
    hard_limit: usize,
    out: &mut Vec<(Range<usize>, String)>,
) {
    if text.is_empty() {
        return;
    }

    let target_tokens = cfg.max_tokens.min(hard_limit);
    let target_chars = target_tokens.saturating_mul(4).max(1);
    let overlap_chars = (target_chars.saturating_mul(cfg.overlap_pct as usize)) / 100;

    let relative_spans = window_split(text, target_chars, overlap_chars);

    if relative_spans.is_empty() {
        if let Some(slice_str) = text.get(0..text.len()) {
            out.push((block_span, slice_str.to_string()));
        }
        return;
    }

    let block_start = block_span.start;

    for rel_span in relative_spans {
        if let Some(slice_str) = text.get(rel_span.clone()) {
            if !slice_str.trim().is_empty() {
                let abs_start = block_start.saturating_add(rel_span.start);
                let abs_end = block_start.saturating_add(rel_span.end);
                out.push((abs_start..abs_end, slice_str.to_string()));
            }
        }
    }
}

fn find_sub_span(canon: &CanonicalText, parent_span: Range<usize>, sub: &str) -> Range<usize> {
    if sub.is_empty() {
        return parent_span.start..parent_span.start;
    }

    if let Some(parent_str) = canon.text.get(parent_span.clone()) {
        if let Some(rel_pos) = parent_str.find(sub) {
            let start = parent_span.start.saturating_add(rel_pos);
            let end = start.saturating_add(sub.len());
            return start..end;
        }
    }

    if let Some(global_pos) = canon.text.find(sub) {
        let end = global_pos.saturating_add(sub.len());
        return global_pos..end;
    }

    parent_span
}
