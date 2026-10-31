//! Splitting-Logik fuer Textbloecke anhand von Absaetzen und Ueberschriften.
//!
//! # Zweck & Architektur
//! Gehoert zur Segmentierungsschicht (Ring 4) in `contextra-ingest`.
//! Zerlegt die hierarchische `DocumentIr` in semantisch zusammenhaengende,
//! budgetierte Einheiten (Chunks) fuer Embeddings und Volltext-Indexierung.
//!
// FILE-CONTEXT
// ZWECK: Aufspaltung übergroßer Blöcke (Code, Tabelle, Absatz, Liste) über dem Hard-Limit (WP M3 / SPEC C-11.7).
// INVARIANTEN: Zero-Panic, UTF-8-sicher, Sprachsynthese von Code-Fences und Tabellenköpfen, Deterministisch.
// STAND: TS:2026-10-31T00:00:00Z

use crate::chunking::common::{token_count, window_split};
use crate::chunking::config::ChunkingConfig;
use super::markdown_parse::ParsedListItem;
use super::sentence::{split_sentences, SentenceSpan};
use crate::chunking::ChunkDraft;
use crate::ir::Locator;
use serde_json::{Map, Value};
use std::ops::Range;

/// Kontextdaten für das Aufspalten übergroßer Blöcke.
#[derive(Debug, Clone)]
// ANCHOR[contextra-ingest:CHK-0230]
// struct `SplitContext` fuer Splitting-Logik fuer Textbloecke anhand von Absaetzen und Ueberschriften.
pub struct SplitContext<'a> {
    pub span: Range<usize>,
    pub locator: Locator,
    pub breadcrumb: Option<String>,
    pub heading_level: Option<u8>,
    pub hard_limit: usize,
    pub cfg: &'a ChunkingConfig,
}

/// Erzeugt Teil-Drafts für einen übergroßen Codeblock.
// ANCHOR[contextra-ingest:CHK-0231]
// fn `split_oversized_code` fuer Splitting-Logik fuer Textbloecke anhand von Absaetzen und Ueberschriften.
pub fn split_oversized_code(
    fence_head: &str,
    fence_tail: &str,
    body_lines: &[&str],
    sctx: &SplitContext<'_>,
) -> Vec<ChunkDraft> {
    let mut drafts = Vec::new();

    let header_prefix = format!("{}\n", fence_head);
    let footer_suffix = format!("\n{}", fence_tail);
    let wrapper_tokens = token_count(&format!("{}{}", header_prefix, footer_suffix));

    let max_body_tokens = sctx.hard_limit.saturating_sub(wrapper_tokens).max(1);

    let window_chars = sctx.hard_limit.saturating_mul(4);
    let overlap_chars = window_chars.saturating_mul(sctx.cfg.overlap_pct as usize) / 100;

    let mut current_lines = Vec::new();
    let mut current_tokens = 0usize;

    let flush_code = |lines: &mut Vec<&str>, target: &mut Vec<ChunkDraft>| {
        if !lines.is_empty() {
            let body_str = lines.join("\n");
            let full_text = format!("{}{}{}", header_prefix, body_str, footer_suffix);

            let mut extra = Map::new();
            extra.insert("synthesized".into(), Value::Bool(true));

            target.push(ChunkDraft {
                text: full_text,
                span: sctx.span.clone(),
                locator: sctx.locator.clone(),
                breadcrumb: sctx.breadcrumb.clone(),
                heading_level: sctx.heading_level,
                extra,
            });

            lines.clear();
        }
    };

    for line in body_lines {
        let line_tok = token_count(line);

        if current_tokens.saturating_add(line_tok) > max_body_tokens && !current_lines.is_empty() {
            flush_code(&mut current_lines, &mut drafts);
            current_tokens = 0;
        }

        if line_tok > max_body_tokens {
            flush_code(&mut current_lines, &mut drafts);
            current_tokens = 0;

            let char_spans = window_split(line, window_chars, overlap_chars);
            if char_spans.is_empty() {
                current_lines.push(line);
                current_tokens = line_tok;
            } else {
                for c_span in char_spans {
                    let w_slice = line.get(c_span).unwrap_or("");
                    let full_text = format!("{}{}{}", header_prefix, w_slice, footer_suffix);
                    let mut extra = Map::new();
                    extra.insert("synthesized".into(), Value::Bool(true));

                    drafts.push(ChunkDraft {
                        text: full_text,
                        span: sctx.span.clone(),
                        locator: sctx.locator.clone(),
                        breadcrumb: sctx.breadcrumb.clone(),
                        heading_level: sctx.heading_level,
                        extra,
                    });
                }
            }
        } else {
            current_lines.push(line);
            current_tokens = current_tokens.saturating_add(line_tok);
        }
    }

    flush_code(&mut current_lines, &mut drafts);
    drafts
}

/// Erzeugt Teil-Drafts für eine übergroße Tabelle.
// ANCHOR[contextra-ingest:CHK-0232]
// fn `split_oversized_table` fuer Splitting-Logik fuer Textbloecke anhand von Absaetzen und Ueberschriften.
pub fn split_oversized_table(
    header_lines: &[String],
    body_lines: &[String],
    sctx: &SplitContext<'_>,
) -> Vec<ChunkDraft> {
    let mut drafts = Vec::new();

    let header_prefix = if header_lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", header_lines.join("\n"))
    };
    let header_tok = token_count(&header_prefix);
    let max_body_tokens = sctx.hard_limit.saturating_sub(header_tok).max(1);

    let window_chars = sctx.hard_limit.saturating_mul(4);
    let overlap_chars = window_chars.saturating_mul(sctx.cfg.overlap_pct as usize) / 100;

    let mut current_rows = Vec::new();
    let mut current_tokens = 0usize;

    let flush_table = |rows: &mut Vec<String>, target: &mut Vec<ChunkDraft>| {
        if !rows.is_empty() {
            let body_str = rows.join("\n");
            let full_text = if header_prefix.is_empty() {
                body_str
            } else {
                format!("{}{}", header_prefix, body_str)
            };

            let mut extra = Map::new();
            extra.insert("synthesized".into(), Value::Bool(true));

            target.push(ChunkDraft {
                text: full_text,
                span: sctx.span.clone(),
                locator: sctx.locator.clone(),
                breadcrumb: sctx.breadcrumb.clone(),
                heading_level: sctx.heading_level,
                extra,
            });

            rows.clear();
        }
    };

    for row in body_lines {
        let row_tok = token_count(row);

        if current_tokens.saturating_add(row_tok) > max_body_tokens && !current_rows.is_empty() {
            flush_table(&mut current_rows, &mut drafts);
            current_tokens = 0;
        }

        if row_tok > max_body_tokens {
            flush_table(&mut current_rows, &mut drafts);
            current_tokens = 0;

            let char_spans = window_split(row, window_chars, overlap_chars);
            if char_spans.is_empty() {
                current_rows.push(row.clone());
                current_tokens = row_tok;
            } else {
                for c_span in char_spans {
                    let w_slice = row.get(c_span).unwrap_or("");
                    let full_text = if header_prefix.is_empty() {
                        w_slice.to_string()
                    } else {
                        format!("{}{}", header_prefix, w_slice)
                    };
                    let mut extra = Map::new();
                    extra.insert("synthesized".into(), Value::Bool(true));

                    drafts.push(ChunkDraft {
                        text: full_text,
                        span: sctx.span.clone(),
                        locator: sctx.locator.clone(),
                        breadcrumb: sctx.breadcrumb.clone(),
                        heading_level: sctx.heading_level,
                        extra,
                    });
                }
            }
        } else {
            current_rows.push(row.clone());
            current_tokens = current_tokens.saturating_add(row_tok);
        }
    }

    flush_table(&mut current_rows, &mut drafts);
    drafts
}

/// Erzeugt Teil-Drafts für eine übergroße Liste an Item-Grenzen.
// ANCHOR[contextra-ingest:CHK-0233]
// fn `split_oversized_list` fuer Splitting-Logik fuer Textbloecke anhand von Absaetzen und Ueberschriften.
pub fn split_oversized_list(
    items: &[ParsedListItem],
    sctx: &SplitContext<'_>,
) -> Vec<ChunkDraft> {
    let mut drafts = Vec::new();
    let mut current_items = Vec::new();
    let mut current_tokens = 0usize;

    let flush_list = |items_buf: &mut Vec<&ParsedListItem>, target: &mut Vec<ChunkDraft>| {
        if !items_buf.is_empty() {
            let combined_text = items_buf
                .iter()
                .map(|item| {
                    let prefix = if item.ordered { "1. " } else { "- " };
                    let indent = "  ".repeat(item.depth as usize);
                    format!("{}{}{}", indent, prefix, item.text)
                })
                .collect::<Vec<_>>()
                .join("\n");

            let start_span = items_buf.first().map(|i| i.span.start).unwrap_or(sctx.span.start);
            let end_span = items_buf.last().map(|i| i.span.end).unwrap_or(sctx.span.end);

            target.push(ChunkDraft {
                text: combined_text,
                span: start_span..end_span,
                locator: sctx.locator.clone(),
                breadcrumb: sctx.breadcrumb.clone(),
                heading_level: sctx.heading_level,
                extra: Map::new(),
            });

            items_buf.clear();
        }
    };

    for item in items {
        let item_str = format!("- {}", item.text);
        let item_tok = token_count(&item_str);

        if current_tokens.saturating_add(item_tok) > sctx.hard_limit && !current_items.is_empty() {
            flush_list(&mut current_items, &mut drafts);
            current_tokens = 0;
        }

        if item_tok > sctx.hard_limit {
            flush_list(&mut current_items, &mut drafts);
            current_tokens = 0;

            let sub_ctx = SplitContext {
                span: item.span.clone(),
                ..sctx.clone()
            };
            let item_drafts = split_oversized_paragraph(&item.text, &sub_ctx);
            drafts.extend(item_drafts);
        } else {
            current_items.push(item);
            current_tokens = current_tokens.saturating_add(item_tok);
        }
    }

    flush_list(&mut current_items, &mut drafts);
    drafts
}

/// Erzeugt Teil-Drafts für einen übergroßen Absatz.
// ANCHOR[contextra-ingest:CHK-0234]
// fn `split_oversized_paragraph` fuer Splitting-Logik fuer Textbloecke anhand von Absaetzen und Ueberschriften.
pub fn split_oversized_paragraph(
    para_text: &str,
    sctx: &SplitContext<'_>,
) -> Vec<ChunkDraft> {
    let mut drafts = Vec::new();
    let sentences = split_sentences(para_text, sctx.span.start);

    if sentences.is_empty() {
        return vec![ChunkDraft {
            text: para_text.to_string(),
            span: sctx.span.clone(),
            locator: sctx.locator.clone(),
            breadcrumb: sctx.breadcrumb.clone(),
            heading_level: sctx.heading_level,
            extra: Map::new(),
        }];
    }

    let window_chars = sctx.hard_limit.saturating_mul(4);
    let overlap_chars = window_chars.saturating_mul(sctx.cfg.overlap_pct as usize) / 100;

    let mut current_sentences = Vec::new();
    let mut current_tokens = 0usize;

    let flush_para = |sents: &mut Vec<SentenceSpan>, target: &mut Vec<ChunkDraft>| {
        if !sents.is_empty() {
            let combined_text: String = sents.iter().map(|s| s.text.as_str()).collect();
            let start_span = sents.first().map(|s| s.span.start).unwrap_or(sctx.span.start);
            let end_span = sents.last().map(|s| s.span.end).unwrap_or(sctx.span.end);

            target.push(ChunkDraft {
                text: combined_text,
                span: start_span..end_span,
                locator: sctx.locator.clone(),
                breadcrumb: sctx.breadcrumb.clone(),
                heading_level: sctx.heading_level,
                extra: Map::new(),
            });
        }
    };

    let mut idx = 0usize;

    while idx < sentences.len() {
        let sent = sentences.get(idx).cloned().unwrap_or(SentenceSpan {
            text: String::new(),
            span: 0..0,
        });

        let sent_tok = token_count(&sent.text);

        if sent_tok > sctx.hard_limit {
            if !current_sentences.is_empty() {
                flush_para(&mut current_sentences, &mut drafts);
                current_sentences.clear();
                current_tokens = 0;
            }

            let char_spans = window_split(&sent.text, window_chars, overlap_chars);
            if char_spans.is_empty() {
                drafts.push(ChunkDraft {
                    text: sent.text.clone(),
                    span: sent.span.clone(),
                    locator: sctx.locator.clone(),
                    breadcrumb: sctx.breadcrumb.clone(),
                    heading_level: sctx.heading_level,
                    extra: Map::new(),
                });
            } else {
                for c_span in char_spans {
                    let w_slice = sent.text.get(c_span.clone()).unwrap_or("");
                    let w_start = sent.span.start.saturating_add(c_span.start);
                    let w_end = sent.span.start.saturating_add(c_span.end);

                    let mut extra = Map::new();
                    if sent_tok > sctx.hard_limit.saturating_mul(2) {
                        extra.insert("oversize".into(), Value::Bool(true));
                    }

                    drafts.push(ChunkDraft {
                        text: w_slice.to_string(),
                        span: w_start..w_end,
                        locator: sctx.locator.clone(),
                        breadcrumb: sctx.breadcrumb.clone(),
                        heading_level: sctx.heading_level,
                        extra,
                    });
                }
            }

            idx = idx.saturating_add(1);
            continue;
        }

        if current_tokens.saturating_add(sent_tok) > sctx.hard_limit && !current_sentences.is_empty() {
            flush_para(&mut current_sentences, &mut drafts);

            let overlap_count = calculate_sentence_overlap(&current_sentences, sctx.cfg.overlap_pct);
            let keep_start = current_sentences.len().saturating_sub(overlap_count);
            current_sentences = current_sentences.get(keep_start..).unwrap_or(&[]).to_vec();

            current_tokens = current_sentences.iter().map(|s| token_count(&s.text)).sum();
        }

        current_sentences.push(sent);
        current_tokens = current_tokens.saturating_add(sent_tok);
        idx = idx.saturating_add(1);
    }

    if !current_sentences.is_empty() {
        flush_para(&mut current_sentences, &mut drafts);
    }

    drafts
}

fn calculate_sentence_overlap(
    sentences: &[SentenceSpan],
    overlap_pct: u32,
) -> usize {
    if sentences.is_empty() || overlap_pct == 0 {
        return 0;
    }

    let total_sents = sentences.len();
    let overlap_count = (total_sents.saturating_mul(overlap_pct as usize) / 100).max(1);
    overlap_count.min(total_sents.saturating_sub(1))
}
