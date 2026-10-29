// FILE-CONTEXT
// ZWECK: Zusammenführen kleiner Chunks (min_tokens) und Durchsetzung der Überschriften-Klebepflicht (WP M3 / SPEC C-11.7).
// INVARIANTEN: Zero-Panic, Überschrift klebt immer an Folgeblock, Merge nur innerhalb derselben Hauptsektion.
// STAND: TS:2026-10-31T00:00:00Z

use crate::chunking::common::token_count;
use crate::chunking::config::ChunkingConfig;
use crate::chunking::ChunkDraft;
use serde_json::{json, Value};

/// Wendet Zusammenführungsregeln (Heading-Attachment und min_tokens-Merge) auf Drafts an.
pub fn merge_small_chunks(drafts: Vec<ChunkDraft>, hard_limit: usize, cfg: &ChunkingConfig) -> Vec<ChunkDraft> {
    if drafts.is_empty() {
        return Vec::new();
    }

    let adjusted_drafts = attach_headings(drafts, hard_limit);

    if cfg.min_tokens == 0 {
        return adjusted_drafts;
    }

    let mut merged: Vec<ChunkDraft> = Vec::with_capacity(adjusted_drafts.len());

    for draft in adjusted_drafts {
        let draft_tokens = token_count(&draft.text);

        if let Some(last) = merged.last_mut() {
            let last_tokens = token_count(&last.text);

            let same_section = is_same_main_section(last.breadcrumb.as_deref(), draft.breadcrumb.as_deref());

            if same_section
                && (last_tokens < cfg.min_tokens || draft_tokens < cfg.min_tokens)
                && last_tokens.saturating_add(draft_tokens) <= hard_limit
            {
                last.text.push_str("\n\n");
                last.text.push_str(&draft.text);
                last.span = last.span.start..draft.span.end;

                let mut merged_bc_list = Vec::new();

                if let Some(existing_bc_val) = last.extra.get("merged_breadcrumbs").cloned() {
                    if let Some(arr) = existing_bc_val.as_array() {
                        merged_bc_list.extend(arr.iter().cloned());
                    }
                } else if let Some(ref bc) = last.breadcrumb {
                    merged_bc_list.push(json!(bc));
                }

                if let Some(ref d_bc) = draft.breadcrumb {
                    if !merged_bc_list.contains(&json!(d_bc)) {
                        merged_bc_list.push(json!(d_bc));
                    }
                }

                if !merged_bc_list.is_empty() {
                    last.extra.insert("merged_breadcrumbs".into(), Value::Array(merged_bc_list));
                }

                continue;
            }
        }

        merged.push(draft);
    }

    merged
}

/// Stellt sicher, dass Überschriften an mindestens einem Folgeblock kleben.
fn attach_headings(drafts: Vec<ChunkDraft>, hard_limit: usize) -> Vec<ChunkDraft> {
    if drafts.len() <= 1 {
        return drafts;
    }

    let mut result: Vec<ChunkDraft> = Vec::with_capacity(drafts.len());
    let mut pending_headings: Vec<ChunkDraft> = Vec::new();

    for draft in drafts {
        if is_heading_only(&draft.text) {
            pending_headings.push(draft);
        } else if !pending_headings.is_empty() {
            // Überschriften vor den Folgeblock stellen
            let heading_text = pending_headings
                .iter()
                .map(|h| h.text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");

            let combined_text = format!("{}\n\n{}", heading_text, draft.text);
            let combined_tokens = token_count(&combined_text);

            let first_h_span_start = pending_headings.first().map(|h| h.span.start).unwrap_or(draft.span.start);
            let first_bc = pending_headings.first().and_then(|h| h.breadcrumb.clone());
            let first_hl = pending_headings.first().and_then(|h| h.heading_level);

            if combined_tokens <= hard_limit {
                result.push(ChunkDraft {
                    text: combined_text,
                    span: first_h_span_start..draft.span.end,
                    locator: draft.locator,
                    breadcrumb: draft.breadcrumb.or(first_bc),
                    heading_level: draft.heading_level.or(first_hl),
                    extra: draft.extra,
                });
            } else {
                // Falls zusammen zu groß, Heading als eigenen Chunk ausgeben
                for h in pending_headings.drain(..) {
                    result.push(h);
                }
                result.push(draft);
            }

            pending_headings.clear();
        } else {
            result.push(draft);
        }
    }

    if !pending_headings.is_empty() {
        for h in pending_headings {
            result.push(h);
        }
    }

    result
}

fn is_heading_only(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.starts_with('#') && !trimmed.contains("\n\n") && trimmed.lines().count() == 1
}

fn is_same_main_section(bc1: Option<&str>, bc2: Option<&str>) -> bool {
    match (bc1, bc2) {
        (Some(b1), Some(b2)) => {
            let main1 = b1.split('>').next().unwrap_or(b1).trim();
            let main2 = b2.split('>').next().unwrap_or(b2).trim();
            main1 == main2
        }
        (None, None) => true,
        _ => false,
    }
}
