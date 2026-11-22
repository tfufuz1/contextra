// FILE-CONTEXT
// ZWECK: StructuredChunker für flache Record-Blöcke (JSON, JSONL, XML, YAML, TOML) (WP M4 / WP-ING-15).
// INVARIANTEN: Zero-Panic, deterministisch, Monotone Spans im kanonischen Text.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::common::token_count;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::{ChunkStrategy, FormatId};
use crate::ir::{BlockKind, DocumentIr, Locator};
use serde_json::{json, Map, Value};
use std::ops::Range;

/// Chunker für strukturelle Datenformate (JSON, JSONL, YAML, TOML, XML).
#[derive(Debug, Default)]
pub struct StructuredChunker;

impl IrChunker for StructuredChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Structured
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        let max_tokens = cfg.max_tokens;
        let hard_limit = (max_tokens.saturating_mul(cfg.hard_limit_pct as usize)) / 100;
        let hard_limit = hard_limit.max(10);

        let group_depth = cfg.structured.group_depth as usize;
        let batch_small = cfg.structured.batch_small_records;
        let is_jsonl = ir.format == FormatId::Jsonl;

        // 1. Alle Record-Blöcke sammeln mit Block-Index und Span
        let mut records = Vec::new();
        for (block, span) in ir.blocks.iter().zip(canon.block_spans.iter()) {
            if let BlockKind::Record { path, value } = &block.kind {
                let text = &canon.text[span.clone()];
                let tok_cnt = token_count(text);
                records.push(RecordItem {
                    path: path.clone(),
                    value: value.clone(),
                    span: span.clone(),
                    locator: block.locator.clone(),
                    tokens: tok_cnt,
                });
            }
        }

        if records.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Gruppen auf Initial-Tiefe bilden
        let initial_groups = build_groups(&records, group_depth);

        // 3. Rekursive Rekonstruktion/Teilung großer Gruppen
        let mut final_groups = Vec::new();
        for group in initial_groups {
            split_or_collect_group(group, &records, group_depth, hard_limit, &mut final_groups)?;
        }

        // 4. Bündelung kleiner Gruppen (batch_small_records)
        let mut drafts = Vec::new();

        if batch_small {
            let mut current_batch: Vec<GroupItem> = Vec::new();
            let mut current_tokens = 0usize;

            for grp in final_groups {
                let grp_toks = group_tokens(&grp, &records);

                if !current_batch.is_empty() && current_tokens.saturating_add(grp_toks) > hard_limit
                {
                    // Batch finalisieren
                    emit_batch_draft(&current_batch, &records, canon, is_jsonl, &mut drafts)?;
                    current_batch.clear();
                    current_tokens = 0;
                }

                current_batch.push(grp);
                current_tokens = current_tokens.saturating_add(grp_toks);
            }

            if !current_batch.is_empty() {
                emit_batch_draft(&current_batch, &records, canon, is_jsonl, &mut drafts)?;
            }
        } else {
            for grp in final_groups {
                emit_batch_draft(&[grp], &records, canon, is_jsonl, &mut drafts)?;
            }
        }

        Ok(drafts)
    }
}

#[derive(Debug, Clone)]
struct RecordItem {
    path: String,
    value: String,
    span: Range<usize>,
    locator: Locator,
    tokens: usize,
}

#[derive(Debug, Clone)]
struct GroupItem {
    prefix: String,
    record_indices: Vec<usize>,
}

fn group_tokens(grp: &GroupItem, records: &[RecordItem]) -> usize {
    let mut sum = 0usize;
    for &idx in &grp.record_indices {
        if let Some(rec) = records.get(idx) {
            sum = sum.saturating_add(rec.tokens);
        }
    }
    sum
}

fn build_groups(records: &[RecordItem], depth: usize) -> Vec<GroupItem> {
    let mut groups: Vec<GroupItem> = Vec::new();

    for (rec_idx, rec) in records.iter().enumerate() {
        let prefix = get_path_prefix(&rec.path, depth);

        if let Some(last_grp) = groups.last_mut() {
            if last_grp.prefix == prefix {
                last_grp.record_indices.push(rec_idx);
                continue;
            }
        }

        groups.push(GroupItem {
            prefix,
            record_indices: vec![rec_idx],
        });
    }

    groups
}

fn split_or_collect_group(
    grp: GroupItem,
    records: &[RecordItem],
    current_depth: usize,
    hard_limit: usize,
    out: &mut Vec<GroupItem>,
) -> Result<(), IngestError> {
    let total_toks = group_tokens(&grp, records);

    if total_toks <= hard_limit || grp.record_indices.len() <= 1 {
        // Gruppe passt oder besteht nur aus einem Record
        if total_toks > hard_limit && grp.record_indices.len() == 1 {
            // Einzelner übergroßer Record -> wird später als eigener Oversize-Draft gehandhabt
            out.push(grp);
        } else {
            out.push(grp);
        }
        return Ok(());
    }

    // Rekursiv an nächst tieferer Pfadebene teilen
    let next_depth = current_depth.saturating_add(1);
    let sub_groups = build_groups_with_original_indices(&grp.record_indices, records, next_depth);

    // Prüfen ob Teilung Fortschritt gemacht hat
    if sub_groups.len() <= 1 {
        // Keine tiefere Pfadebene verfügbar -> Aufteilung in einzelne Records
        for &rec_idx in &grp.record_indices {
            if let Some(rec) = records.get(rec_idx) {
                out.push(GroupItem {
                    prefix: rec.path.clone(),
                    record_indices: vec![rec_idx],
                });
            }
        }
    } else {
        for sub_grp in sub_groups {
            split_or_collect_group(sub_grp, records, next_depth, hard_limit, out)?;
        }
    }

    Ok(())
}

fn build_groups_with_original_indices(
    orig_indices: &[usize],
    records: &[RecordItem],
    depth: usize,
) -> Vec<GroupItem> {
    let mut groups: Vec<GroupItem> = Vec::new();

    for &rec_idx in orig_indices {
        if let Some(rec) = records.get(rec_idx) {
            let prefix = get_path_prefix(&rec.path, depth);

            if let Some(last_grp) = groups.last_mut() {
                if last_grp.prefix == prefix {
                    last_grp.record_indices.push(rec_idx);
                    continue;
                }
            }

            groups.push(GroupItem {
                prefix,
                record_indices: vec![rec_idx],
            });
        }
    }

    groups
}

fn emit_batch_draft(
    batch: &[GroupItem],
    records: &[RecordItem],
    canon: &CanonicalText,
    is_jsonl: bool,
    out: &mut Vec<ChunkDraft>,
) -> Result<(), IngestError> {
    if batch.is_empty() {
        return Ok(());
    }

    let mut all_indices = Vec::new();
    let mut prefixes = Vec::new();

    for grp in batch {
        if !grp.prefix.is_empty() {
            prefixes.push(grp.prefix.as_str());
        }
        all_indices.extend(grp.record_indices.iter().copied());
    }

    if all_indices.is_empty() {
        return Ok(());
    }

    let first_rec_idx = match all_indices.first() {
        Some(&idx) => idx,
        None => return Ok(()),
    };
    let last_rec_idx = match all_indices.last() {
        Some(&idx) => idx,
        None => return Ok(()),
    };

    let first_rec = match records.get(first_rec_idx) {
        Some(r) => r,
        None => return Ok(()),
    };
    let last_rec = match records.get(last_rec_idx) {
        Some(r) => r,
        None => return Ok(()),
    };

    // Prüfen ob Einzelrecord oversize ist
    if all_indices.len() == 1 {
        let single_rec = first_rec;
        let single_toks = single_rec.tokens;

        // Wenn Einzelrecord extrem groß ist, mit Fenster teilen
        if single_toks > 2000 {
            emit_oversize_record_windows(single_rec, canon, out)?;
            return Ok(());
        }
    }

    let span = first_rec.span.start..last_rec.span.end;
    let text = match canon.text.get(span.clone()) {
        Some(slice) => slice.to_string(),
        None => return Ok(()),
    };

    let breadcrumb = compute_batch_breadcrumb(&prefixes);

    let mut extra = Map::new();
    if is_jsonl || first_rec.path.starts_with('[') {
        extra.insert(
            "record_range".into(),
            json!({
                "first": first_rec_idx,
                "last": last_rec_idx,
            }),
        );
    }

    out.push(ChunkDraft {
        text,
        span,
        locator: first_rec.locator.clone(),
        breadcrumb,
        heading_level: None,
        extra,
    });

    Ok(())
}

fn emit_oversize_record_windows(
    rec: &RecordItem,
    canon: &CanonicalText,
    out: &mut Vec<ChunkDraft>,
) -> Result<(), IngestError> {
    let window_chars = 1000usize;
    let overlap_chars = 100usize;

    let value_str = &rec.value;
    if value_str.is_empty() {
        let text = canon.text.get(rec.span.clone()).unwrap_or("").to_string();
        out.push(ChunkDraft {
            text,
            span: rec.span.clone(),
            locator: rec.locator.clone(),
            breadcrumb: Some(rec.path.clone()),
            heading_level: None,
            extra: Map::new(),
        });
        return Ok(());
    }

    let windows = crate::chunking::common::window_split(value_str, window_chars, overlap_chars);
    let total_windows = windows.len().max(1);

    for (k, win_span) in windows.into_iter().enumerate() {
        let win_text = value_str.get(win_span).unwrap_or_default();

        let synthesized_text = format!("{}: {}", rec.path, win_text);
        let part_str = format!("{}/{}", k.saturating_add(1), total_windows);

        let mut extra = Map::new();
        extra.insert("part".into(), Value::String(part_str));
        extra.insert("synthesized".into(), Value::Bool(true));

        out.push(ChunkDraft {
            text: synthesized_text,
            span: rec.span.clone(),
            locator: rec.locator.clone(),
            breadcrumb: Some(rec.path.clone()),
            heading_level: None,
            extra,
        });
    }

    Ok(())
}

fn parse_path_segments(path: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0usize;
    let bytes = path.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        match bytes.get(i) {
            Some(&b'.') => {
                if start < i {
                    if let Some(sub) = path.get(start..i) {
                        segments.push(sub);
                    }
                }
                i = i.saturating_add(1);
                start = i;
            }
            Some(&b'[') => {
                if start < i {
                    if let Some(sub) = path.get(start..i) {
                        segments.push(sub);
                    }
                }
                let bracket_start = i;
                while i < bytes.len() && bytes.get(i) != Some(&b']') {
                    i = i.saturating_add(1);
                }
                if i < bytes.len() {
                    i = i.saturating_add(1);
                }
                if let Some(sub) = path.get(bracket_start..i) {
                    segments.push(sub);
                }
                start = i;
                if i < bytes.len() && bytes.get(i) == Some(&b'.') {
                    i = i.saturating_add(1);
                    start = i;
                }
            }
            _ => {
                i = i.saturating_add(1);
            }
        }
    }
    if start < bytes.len() {
        if let Some(sub) = path.get(start..) {
            segments.push(sub);
        }
    }

    segments
}

fn get_path_prefix(path: &str, depth: usize) -> String {
    if depth == 0 {
        return String::new();
    }
    let segs = parse_path_segments(path);
    if segs.is_empty() {
        return path.to_string();
    }

    let take_cnt = depth.min(segs.len());
    let sub_segs = match segs.get(..take_cnt) {
        Some(s) => s,
        None => &segs,
    };

    format_segments(sub_segs)
}

fn format_segments(segments: &[&str]) -> String {
    let mut res = String::new();
    for seg in segments {
        if res.is_empty() || seg.starts_with('[') {
            res.push_str(seg);
        } else {
            res.push('.');
            res.push_str(seg);
        }
    }
    res
}

fn compute_batch_breadcrumb(prefixes: &[&str]) -> Option<String> {
    if prefixes.is_empty() {
        return None;
    }
    if prefixes.len() == 1 {
        return prefixes.first().map(|s| s.to_string());
    }

    // Versuch kunden[3] bis kunden[7] Intervall zu bilden
    let mut base_name: Option<String> = None;
    let mut indices = Vec::new();
    let mut all_match = true;

    for pref in prefixes {
        if let Some(bracket_idx) = pref.find('[') {
            if pref.ends_with(']') {
                let base = match pref.get(..bracket_idx) {
                    Some(b) => b,
                    None => {
                        all_match = false;
                        break;
                    }
                };
                let idx_str =
                    match pref.get(bracket_idx.saturating_add(1)..pref.len().saturating_sub(1)) {
                        Some(s) => s,
                        None => {
                            all_match = false;
                            break;
                        }
                    };

                if let Ok(idx_num) = idx_str.parse::<usize>() {
                    if let Some(ref b) = base_name {
                        if b != base {
                            all_match = false;
                            break;
                        }
                    } else {
                        base_name = Some(base.to_string());
                    }
                    indices.push(idx_num);
                    continue;
                }
            }
        }
        all_match = false;
        break;
    }

    if all_match {
        if let (Some(base), Some(&min_i), Some(&max_i)) =
            (base_name, indices.iter().min(), indices.iter().max())
        {
            if min_i == max_i {
                return Some(format!("{}[{}]", base, min_i));
            } else {
                return Some(format!("{}[{}..{}]", base, min_i, max_i));
            }
        }
    }

    // Fallback: Gemeinsamer Pfad-Präfix
    let first = match prefixes.first() {
        Some(p) => *p,
        None => return None,
    };
    let first_segs = parse_path_segments(first);
    let mut common_segs = Vec::new();

    for (seg_idx, seg) in first_segs.iter().enumerate() {
        let mut match_all = true;
        for p in prefixes.iter().skip(1) {
            let p_segs = parse_path_segments(p);
            if p_segs.get(seg_idx) != Some(seg) {
                match_all = false;
                break;
            }
        }
        if match_all {
            common_segs.push(*seg);
        } else {
            break;
        }
    }

    if common_segs.is_empty() {
        None
    } else {
        Some(format_segments(&common_segs))
    }
}
