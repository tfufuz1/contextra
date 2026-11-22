// FILE-CONTEXT
// ZWECK: ProseChunker Hauptmodul für IR-basiertes span-genaues Chunking (WP M3 / SPEC C-11.7).
// INVARIANTEN: Zero-Panic, deterministisch, span-genau im kanonischen Text, Parität zu Engine-Chunker.
// STAND: TS:2026-10-31T00:00:00Z

pub mod markdown_parse;
pub mod merge;
pub mod sentence;
pub mod split;

use crate::canonical::CanonicalText;
use crate::chunking::common::token_count;
use crate::chunking::config::ChunkingConfig;
use self::markdown_parse::{parse_markdown_blocks, ParsedBlockKind, ParsedListItem};
use self::merge::merge_small_chunks;
use self::split::{
    split_oversized_code, split_oversized_list, split_oversized_paragraph, split_oversized_table, SplitContext,
};
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::ChunkStrategy;
use crate::ir::{BlockKind, DocumentIr, Locator, NoteKind};
use serde_json::{Map, Value};
use std::ops::Range;

#[derive(Debug, Default, Clone)]
pub struct ProseChunker;

impl IrChunker for ProseChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Prose
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        draft_prose(ir, canon, cfg)
    }
}

/// Einheitliche Repräsentation eines Blocks für die Verarbeitungs-Pipeline.
#[derive(Debug, Clone)]
struct IntermediateBlock {
    text: String,
    span: Range<usize>,
    locator: Locator,
    kind: InterKind,
}

#[derive(Debug, Clone)]
enum InterKind {
    YamlFrontMatter,
    Heading { level: u8, text: String },
    Paragraph,
    List { items: Vec<ParsedListItem> },
    Code { fence_head: String, fence_tail: String, body_lines: Vec<String> },
    Table { header_lines: Vec<String>, body_lines: Vec<String> },
    Note { kind_tag: String },
    Other,
}

pub fn draft_prose(
    ir: &DocumentIr,
    canon: &CanonicalText,
    cfg: &ChunkingConfig,
) -> Result<Vec<ChunkDraft>, IngestError> {
    if canon.text.is_empty() {
        return Ok(Vec::new());
    }

    let hard_limit = (cfg.max_tokens.saturating_mul(cfg.hard_limit_pct as usize) / 100).max(1);

    let blocks = collect_intermediate_blocks(ir, canon);
    if blocks.is_empty() {
        return Ok(Vec::new());
    }

    let mut drafts = Vec::new();

    let mut heading_stack: Vec<(u8, String)> = Vec::new();
    let mut current_breadcrumb = String::new();
    let mut current_heading_level = 0u8;

    let mut current_block_drafts = Vec::new();
    let mut current_tokens = 0usize;

    let flush_blocks = |block_drafts: &mut Vec<ChunkDraft>,
                        target: &mut Vec<ChunkDraft>,
                        breadcrumb_opt: Option<String>,
                        heading_level_opt: Option<u8>| {
        if !block_drafts.is_empty() {
            let combined_text = block_drafts
                .iter()
                .map(|d| d.text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n");

            let start_span = block_drafts.first().map(|d| d.span.start).unwrap_or(0);
            let end_span = block_drafts.last().map(|d| d.span.end).unwrap_or(0);
            let loc = block_drafts.first().map(|d| d.locator.clone()).unwrap_or_default();

            target.push(ChunkDraft {
                text: combined_text,
                span: start_span..end_span,
                locator: loc,
                breadcrumb: breadcrumb_opt,
                heading_level: heading_level_opt,
                extra: Map::new(),
            });

            block_drafts.clear();
        }
    };

    let get_bc = |bc: &str| -> Option<String> {
        if bc.is_empty() {
            None
        } else {
            Some(bc.to_string())
        }
    };

    let get_hl = |hl: u8| -> Option<u8> {
        if hl == 0 {
            None
        } else {
            Some(hl)
        }
    };

    for blk in blocks {
        let is_heading = match blk.kind {
            InterKind::Heading { level, .. } => cfg.split_levels.contains(&level),
            _ => false,
        };

        if is_heading {
            if let InterKind::Heading { level, ref text } = blk.kind {
                flush_blocks(
                    &mut current_block_drafts,
                    &mut drafts,
                    get_bc(&current_breadcrumb),
                    get_hl(current_heading_level),
                );
                current_tokens = 0;

                heading_stack.retain(|(lvl, _)| *lvl < level);
                let hashes = "#".repeat(level as usize);
                let bc_part = format!("{} {}", hashes, text);
                heading_stack.push((level, bc_part));

                current_breadcrumb = heading_stack
                    .iter()
                    .map(|(_, b)| b.as_str())
                    .collect::<Vec<_>>()
                    .join(" > ");
                current_heading_level = level;
            }
        }

        if matches!(blk.kind, InterKind::YamlFrontMatter) {
            flush_blocks(
                &mut current_block_drafts,
                &mut drafts,
                get_bc(&current_breadcrumb),
                get_hl(current_heading_level),
            );
            current_tokens = 0;

            let mut extra = Map::new();
            extra.insert("front_matter".into(), Value::Bool(true));

            drafts.push(ChunkDraft {
                text: blk.text,
                span: blk.span,
                locator: blk.locator,
                breadcrumb: None,
                heading_level: None,
                extra,
            });
            continue;
        }

        // Behandlung von Notizen/Fußnoten (eigenständiger Chunk mit Breadcrumb "Notes")
        if let InterKind::Note { ref kind_tag } = blk.kind {
            let blk_tokens = token_count(&blk.text);
            if current_tokens.saturating_add(blk_tokens) <= hard_limit {
                // Passt in aktuellen Chunk
                current_block_drafts.push(ChunkDraft {
                    text: blk.text.clone(),
                    span: blk.span.clone(),
                    locator: blk.locator.clone(),
                    breadcrumb: get_bc(&current_breadcrumb),
                    heading_level: get_hl(current_heading_level),
                    extra: {
                        let mut m = Map::new();
                        m.insert("note_kind".into(), Value::String(kind_tag.clone()));
                        m
                    },
                });
                current_tokens = current_tokens.saturating_add(blk_tokens);
            } else {
                // Eigener Chunk mit Breadcrumb "Notes"
                flush_blocks(
                    &mut current_block_drafts,
                    &mut drafts,
                    get_bc(&current_breadcrumb),
                    get_hl(current_heading_level),
                );
                current_tokens = 0;

                let mut extra = Map::new();
                extra.insert("note_kind".into(), Value::String(kind_tag.clone()));

                drafts.push(ChunkDraft {
                    text: blk.text,
                    span: blk.span,
                    locator: blk.locator,
                    breadcrumb: Some("Notes".to_string()),
                    heading_level: None,
                    extra,
                });
            }
            continue;
        }

        let blk_tokens = token_count(&blk.text);

        if blk_tokens <= hard_limit {
            if current_tokens.saturating_add(blk_tokens) > hard_limit && !current_block_drafts.is_empty() {
                flush_blocks(
                    &mut current_block_drafts,
                    &mut drafts,
                    get_bc(&current_breadcrumb),
                    get_hl(current_heading_level),
                );
                current_tokens = 0;
            }

            current_block_drafts.push(ChunkDraft {
                text: blk.text,
                span: blk.span,
                locator: blk.locator,
                breadcrumb: get_bc(&current_breadcrumb),
                heading_level: get_hl(current_heading_level),
                extra: Map::new(),
            });

            current_tokens = current_tokens.saturating_add(blk_tokens);
        } else {
            flush_blocks(
                &mut current_block_drafts,
                &mut drafts,
                get_bc(&current_breadcrumb),
                get_hl(current_heading_level),
            );
            current_tokens = 0;

            let bc_opt = get_bc(&current_breadcrumb);
            let hl_opt = get_hl(current_heading_level);

            let sctx = SplitContext {
                span: blk.span.clone(),
                locator: blk.locator.clone(),
                breadcrumb: bc_opt,
                heading_level: hl_opt,
                hard_limit,
                cfg,
            };

            match blk.kind {
                InterKind::Code { ref fence_head, ref fence_tail, ref body_lines } => {
                    let body_refs: Vec<&str> = body_lines.iter().map(|s| s.as_str()).collect();
                    let split_drafts = split_oversized_code(
                        fence_head,
                        fence_tail,
                        &body_refs,
                        &sctx,
                    );
                    drafts.extend(split_drafts);
                }
                InterKind::Table { ref header_lines, ref body_lines } => {
                    let split_drafts = split_oversized_table(
                        header_lines,
                        body_lines,
                        &sctx,
                    );
                    drafts.extend(split_drafts);
                }
                InterKind::List { ref items } => {
                    let split_drafts = split_oversized_list(
                        items,
                        &sctx,
                    );
                    drafts.extend(split_drafts);
                }
                _ => {
                    let split_drafts = split_oversized_paragraph(
                        &blk.text,
                        &sctx,
                    );
                    drafts.extend(split_drafts);
                }
            }
        }
    }

    flush_blocks(
        &mut current_block_drafts,
        &mut drafts,
        get_bc(&current_breadcrumb),
        get_hl(current_heading_level),
    );

    let final_drafts = merge_small_chunks(drafts, hard_limit, cfg);

    Ok(final_drafts)
}

fn collect_intermediate_blocks(ir: &DocumentIr, canon: &CanonicalText) -> Vec<IntermediateBlock> {
    let mut results = Vec::new();

    for (idx, block) in ir.blocks.iter().enumerate() {
        let span = canon.block_spans.get(idx).cloned().unwrap_or(0..0);
        let block_str = canon.text.get(span.clone()).unwrap_or("");

        match &block.kind {
            BlockKind::Raw { text } => {
                let parsed = parse_markdown_blocks(text, span.start);
                for p in parsed {
                    let inter_kind = match p.kind {
                        ParsedBlockKind::YamlFrontMatter => InterKind::YamlFrontMatter,
                        ParsedBlockKind::Heading { level, text } => InterKind::Heading { level, text },
                        ParsedBlockKind::FencedCode { fence_char, fence_len, lang, body } => {
                            let head = format!("{}{}", fence_char.to_string().repeat(fence_len), lang.as_deref().unwrap_or(""));
                            let tail = fence_char.to_string().repeat(fence_len);
                            let body_lines = body.lines().map(|s| s.to_string()).collect();
                            InterKind::Code { fence_head: head, fence_tail: tail, body_lines }
                        }
                        ParsedBlockKind::Table { header_lines, body_lines } => {
                            InterKind::Table { header_lines, body_lines }
                        }
                        ParsedBlockKind::Note { label, .. } => InterKind::Note { kind_tag: label },
                        ParsedBlockKind::Paragraph { .. } => InterKind::Paragraph,
                        ParsedBlockKind::List { items } => InterKind::List { items },
                        _ => InterKind::Other,
                    };

                    results.push(IntermediateBlock {
                        text: p.text,
                        span: p.span,
                        locator: block.locator.clone(),
                        kind: inter_kind,
                    });
                }
            }
            BlockKind::Heading { level, text } => {
                results.push(IntermediateBlock {
                    text: block_str.to_string(),
                    span,
                    locator: block.locator.clone(),
                    kind: InterKind::Heading { level: *level, text: text.clone() },
                });
            }
            BlockKind::Paragraph { .. } => {
                results.push(IntermediateBlock {
                    text: block_str.to_string(),
                    span,
                    locator: block.locator.clone(),
                    kind: InterKind::Paragraph,
                });
            }
            BlockKind::ListItem { depth, ordered, text } => {
                let item = ParsedListItem {
                    depth: *depth,
                    ordered: *ordered,
                    text: text.clone(),
                    span: span.clone(),
                };
                results.push(IntermediateBlock {
                    text: block_str.to_string(),
                    span,
                    locator: block.locator.clone(),
                    kind: InterKind::List { items: vec![item] },
                });
            }
            BlockKind::Code { lang, text } => {
                let head = format!("```{}", lang.as_deref().unwrap_or(""));
                let tail = "```".to_string();
                let body_lines = text.lines().map(|s| s.to_string()).collect();

                results.push(IntermediateBlock {
                    text: block_str.to_string(),
                    span,
                    locator: block.locator.clone(),
                    kind: InterKind::Code { fence_head: head, fence_tail: tail, body_lines },
                });
            }
            BlockKind::Table(t) => {
                let mut header_lines = Vec::new();
                let mut body_lines = Vec::new();

                if !t.rows.is_empty() {
                    let mut row_strs = Vec::new();
                    for row in &t.rows {
                        row_strs.push(format!("| {} |", row.join(" | ")));
                    }

                    if let Some(first_row_str) = row_strs.first() {
                        header_lines.push(first_row_str.clone());
                        let col_count = t.rows.first().map(|r| r.len()).unwrap_or(1);
                        let delim = format!("|{}|", "---|".repeat(col_count));
                        header_lines.push(delim);
                        body_lines.extend(row_strs.into_iter().skip(1));
                    }
                }

                results.push(IntermediateBlock {
                    text: block_str.to_string(),
                    span,
                    locator: block.locator.clone(),
                    kind: InterKind::Table { header_lines, body_lines },
                });
            }
            BlockKind::Note { kind, .. } => {
                let tag = match kind {
                    NoteKind::Footnote => "footnote",
                    NoteKind::Endnote => "endnote",
                    NoteKind::Comment => "comment",
                    NoteKind::SpeakerNotes => "speaker_notes",
                    NoteKind::Header => "header",
                    NoteKind::Footer => "footer",
                };

                results.push(IntermediateBlock {
                    text: block_str.to_string(),
                    span,
                    locator: block.locator.clone(),
                    kind: InterKind::Note { kind_tag: tag.to_string() },
                });
            }
            _ => {
                results.push(IntermediateBlock {
                    text: block_str.to_string(),
                    span,
                    locator: block.locator.clone(),
                    kind: InterKind::Other,
                });
            }
        }
    }

    results
}
