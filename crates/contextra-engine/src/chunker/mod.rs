//! # Modul: mod.rs
//!
//! **Zweck:** Dokument-Segmentierung und Chunker-Subsystem für Fließtexte und Abschnitte.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, deterministische Textverarbeitung.

// FILE-CONTEXT
// ZWECK: Hauptmodul des Markdown-Chunkers (WP-7.1 / Teil 6.12 Ingestion Chunking Spezifikation).
// INVARIANTEN: Re-exportiert exakt alle bisherigen öffentlichen Typen und Funktionen.
// NICHT-OFFENSICHTLICH: Zerlegt Dokumente deterministisch; Brotkrumen und Spans als Metadaten angehängt. Indexfrei & Panikfrei.
// STAND: TS:2026-10-29T00:00:00Z

//! Markdown Semantic Chunker (WP-7.1 / Teil 6.12 Ingestion Chunking Spezifikation)

mod blocks;
mod config;
mod estimate;
mod sections;
mod split;
mod windows;

pub use config::ChunkerConfig;
pub use estimate::estimate_tokens;
pub use windows::{chunk_text, chunk_text_with_overlap};

use contextra_types::{ContextChunk, DocId};
use serde_json::json;

use blocks::{is_closing_fence, is_table_row, parse_opening_fence, CodeFence};
use estimate::CHARS_PER_TOKEN;
use sections::RawSection;
use split::split_raw_section;

/// A deterministic chunker for markdown files respecting structural blocks
/// (headings, code blocks, tables, paragraphs) and UTF-8 safety.
pub struct MarkdownChunker {
    config: ChunkerConfig,
}

#[derive(Debug)]
struct LineInfo<'a> {
    line: &'a str,
    line_number: usize,
    start_byte: usize,
    end_byte: usize,
}

fn scan_lines(text: &str) -> Vec<LineInfo<'_>> {
    let mut lines = Vec::new();
    let mut line_num = 1;
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut line_start = 0;
    let mut i = 0;

    while i < len {
        let b = bytes.get(i).copied().unwrap_or(0);
        if b == b'\n' {
            let content_end =
                if i > line_start && bytes.get(i.saturating_sub(1)).copied() == Some(b'\r') {
                    i.saturating_sub(1)
                } else {
                    i
                };
            if let Some(line_slice) = text.get(line_start..content_end) {
                lines.push(LineInfo {
                    line: line_slice,
                    line_number: line_num,
                    start_byte: line_start,
                    end_byte: content_end,
                });
            }
            line_num = line_num.saturating_add(1);
            i = i.saturating_add(1);
            line_start = i;
        } else if b == b'\r' {
            if bytes.get(i.saturating_add(1)).copied() == Some(b'\n') {
                if let Some(line_slice) = text.get(line_start..i) {
                    lines.push(LineInfo {
                        line: line_slice,
                        line_number: line_num,
                        start_byte: line_start,
                        end_byte: i,
                    });
                }
                line_num = line_num.saturating_add(1);
                i = i.saturating_add(2);
                line_start = i;
            } else {
                if let Some(line_slice) = text.get(line_start..i) {
                    lines.push(LineInfo {
                        line: line_slice,
                        line_number: line_num,
                        start_byte: line_start,
                        end_byte: i,
                    });
                }
                line_num = line_num.saturating_add(1);
                i = i.saturating_add(1);
                line_start = i;
            }
        } else {
            i = i.saturating_add(1);
        }
    }

    if line_start < len {
        if let Some(line_slice) = text.get(line_start..len) {
            lines.push(LineInfo {
                line: line_slice,
                line_number: line_num,
                start_byte: line_start,
                end_byte: len,
            });
        }
    }

    lines
}

fn is_setext_h1(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.chars().all(|c| c == '=')
}

fn is_setext_h2(line: &str) -> bool {
    let trimmed = line.trim();
    !trimmed.is_empty() && trimmed.chars().all(|c| c == '-')
}

impl MarkdownChunker {
    /// Creates a new MarkdownChunker with the given configuration.
    pub fn new(config: ChunkerConfig) -> Self {
        Self { config }
    }

    /// Creates a new MarkdownChunker with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ChunkerConfig::default())
    }

    /// Chunks a Markdown document into semantically coherent pieces.
    pub fn chunk(&self, doc_id: DocId, markdown: &str) -> Vec<ContextChunk> {
        if markdown.is_empty() {
            return Vec::new();
        }

        let hard_limit = self
            .config
            .max_tokens
            .saturating_add(self.config.max_tokens / 5)
            .max(1);

        let lines = scan_lines(markdown);
        let mut raw_sections = Vec::new();

        let mut current_lines: Vec<String> = Vec::new();
        let mut current_breadcrumb = String::new();
        let mut current_heading_level = 0;
        let mut current_source_line = 1;
        let mut current_source_line_end = 1;
        let mut current_start_byte = 0;
        let mut current_end_byte = 0;

        let mut heading_stack: Vec<(u8, String)> = Vec::new();
        let mut active_fence: Option<CodeFence> = None;

        let mut i = 0;
        while i < lines.len() {
            let Some(info) = lines.get(i) else {
                break;
            };

            let is_in_fence_at_start = active_fence.is_some();
            if let Some(ref active) = active_fence {
                if is_closing_fence(info.line, active) {
                    active_fence = None;
                }
            } else if let Some(fence_info) = parse_opening_fence(info.line) {
                active_fence = Some(fence_info);
            }

            let mut is_setext = false;
            let mut setext_level = 0;
            if !is_in_fence_at_start && (is_setext_h1(info.line) || is_setext_h2(info.line)) {
                if !current_lines.is_empty() {
                    let prev_line_str = current_lines.last().map(|s| s.as_str()).unwrap_or("");
                    let prev_trimmed = prev_line_str.trim();
                    if !prev_trimmed.is_empty()
                        && !prev_trimmed.starts_with('#')
                        && !is_table_row(prev_line_str)
                        && parse_opening_fence(prev_line_str).is_none()
                    {
                        setext_level = if is_setext_h1(info.line) { 1 } else { 2 };
                        if self.config.split_levels.contains(&setext_level) {
                            is_setext = true;
                        }
                    }
                }
            }

            if is_setext {
                let Some(heading_line) = current_lines.pop() else {
                    i = i.saturating_add(1);
                    continue;
                };
                let heading_text = heading_line.trim().to_string();

                if !current_lines.is_empty() {
                    let prev_info = if i >= 2 {
                        lines.get(i.saturating_sub(2))
                    } else {
                        lines.first()
                    };
                    let prev_section_line_end = prev_info.map_or(1, |l| l.line_number);
                    let prev_section_end_byte = prev_info.map_or(0, |l| l.end_byte);

                    let content = current_lines.join("\n");
                    let tokens = estimate_tokens(&content);
                    raw_sections.push(RawSection {
                        lines: std::mem::take(&mut current_lines),
                        breadcrumb: current_breadcrumb.clone(),
                        heading_level: current_heading_level,
                        source_line: current_source_line,
                        source_line_end: prev_section_line_end,
                        start_byte: current_start_byte,
                        end_byte: prev_section_end_byte,
                        tokens,
                    });
                }

                heading_stack.retain(|(lvl, _)| *lvl < setext_level);
                let breadcrumb_part = format!("{} {}", parts_first("", setext_level), heading_text);
                heading_stack.push((setext_level, breadcrumb_part));

                current_breadcrumb = heading_stack
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" > ");
                current_heading_level = setext_level;
                current_source_line = info.line_number.saturating_sub(1);
                current_start_byte = lines.get(i.saturating_sub(1)).map_or(0, |l| l.start_byte);

                current_lines.push(heading_line);
                current_lines.push(info.line.to_string());
                current_source_line_end = info.line_number;
                current_end_byte = info.end_byte;

                i = i.saturating_add(1);
                continue;
            }

            let mut is_atx = false;
            let mut h_level = 0;
            if !is_in_fence_at_start && info.line.starts_with('#') {
                let parts: Vec<&str> = info.line.splitn(2, ' ').collect();
                if parts.len() == 2 && parts.get(0).map_or(false, |p| p.chars().all(|c| c == '#')) {
                    h_level = parts.get(0).map_or(0, |p| p.len() as u8);
                    if self.config.split_levels.contains(&h_level) {
                        is_atx = true;
                    }
                }
            }

            if is_atx {
                if !current_lines.is_empty() {
                    let content = current_lines.join("\n");
                    let tokens = estimate_tokens(&content);
                    raw_sections.push(RawSection {
                        lines: std::mem::take(&mut current_lines),
                        breadcrumb: current_breadcrumb.clone(),
                        heading_level: current_heading_level,
                        source_line: current_source_line,
                        source_line_end: current_source_line_end,
                        start_byte: current_start_byte,
                        end_byte: current_end_byte,
                        tokens,
                    });
                }

                heading_stack.retain(|(lvl, _)| *lvl < h_level);
                let heading_text = info.line.trim_start_matches('#').trim();
                let breadcrumb_part =
                    format!("{} {}", parts_first(info.line, h_level), heading_text);
                heading_stack.push((h_level, breadcrumb_part));

                current_breadcrumb = heading_stack
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" > ");
                current_heading_level = h_level;
                current_source_line = info.line_number;
                current_start_byte = info.start_byte;

                current_lines.push(info.line.to_string());
                current_source_line_end = info.line_number;
                current_end_byte = info.end_byte;
            } else {
                if current_lines.is_empty() {
                    current_source_line = info.line_number;
                    current_start_byte = info.start_byte;
                }
                current_lines.push(info.line.to_string());
                current_source_line_end = info.line_number;
                current_end_byte = info.end_byte;
            }

            i = i.saturating_add(1);
        }

        if !current_lines.is_empty() {
            let content = current_lines.join("\n");
            let tokens = estimate_tokens(&content);
            raw_sections.push(RawSection {
                lines: current_lines,
                breadcrumb: current_breadcrumb,
                heading_level: current_heading_level,
                source_line: current_source_line,
                source_line_end: current_source_line_end,
                start_byte: current_start_byte,
                end_byte: current_end_byte,
                tokens,
            });
        }

        let window_chars = hard_limit.saturating_mul(CHARS_PER_TOKEN);
        let overlap_chars = window_chars / 5;

        let mut limited_sections = Vec::new();
        for sec in raw_sections {
            let split_secs = split_raw_section(sec, hard_limit, window_chars, overlap_chars);
            limited_sections.extend(split_secs);
        }

        let mut final_sections: Vec<RawSection> = Vec::new();
        for sec in limited_sections {
            if let Some(last) = final_sections.last_mut() {
                if (last.tokens < self.config.min_tokens || sec.tokens < self.config.min_tokens)
                    && (last.tokens.saturating_add(sec.tokens)) <= hard_limit
                {
                    last.lines.push("".to_string());
                    last.lines.extend(sec.lines);
                    last.tokens = last.tokens.saturating_add(sec.tokens);
                    last.source_line_end = sec.source_line_end;
                    last.end_byte = sec.end_byte;
                    continue;
                }
            }
            final_sections.push(sec);
        }

        final_sections
            .into_iter()
            .map(|sec| {
                let metadata = if self.config.include_breadcrumbs {
                    Some(json!({
                        "breadcrumb": sec.breadcrumb,
                        "heading_level": sec.heading_level,
                        "source_line": sec.source_line,
                        "source_line_end": sec.source_line_end,
                        "source_span": {
                            "start": sec.start_byte,
                            "end": sec.end_byte
                        }
                    }))
                } else {
                    None
                };

                let content = sec.lines.join("\n");
                let token_count = estimate_tokens(&content);

                ContextChunk {
                    doc_id,
                    content,
                    relevance: 1.0,
                    token_count,
                    metadata,
                    contextual_prefix: None,
                    links: Vec::new(),
                }
            })
            .collect()
    }
}

fn parts_first(_line: &str, h_level: u8) -> String {
    let mut s = String::new();
    for _ in 0..h_level {
        s.push('#');
    }
    s
}

#[cfg(test)]
mod unit_tests;
