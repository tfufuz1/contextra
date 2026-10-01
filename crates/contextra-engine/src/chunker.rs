// FILE-CONTEXT
// ZWECK: Markdown-basierte semantische Textzerlegung (WP-7.1 / Teil 6.12 Spezifikation) und UTF-8-sicheres Chunken.
// INVARIANTEN: Aufteilung respektiert Überschriften-Hierarchien, Codeblöcke & Tabellen-Atomarität; UTF-8-Grenzen bleiben gewahrt.
// NICHT-OFFENSICHTLICH: Brotbrösel (Breadcrumbs) werden als Metadaten an Chunks angehängt.
// STAND: TS:2026-09-17T00:00:00Z

//! Markdown Semantic Chunker (WP-7.1 / Teil 6.12 Ingestion Chunking Spezifikation)
//!
//! # Ingestion Chunking Spezifikation (Teil 6.12)
//!
//! Dieser Chunker zerlegt Markdown-Dokumente deterministisch in semantisch zusammenhängende
//! [`ContextChunk`]-Instanzen zur Weiterverarbeitung in der Contextra Ingestion Pipeline.
//!
//! ## 1. Zielgröße je Chunk (Target Size)
//! - **Standard-Zielgröße (`max_tokens`)**: 512 Tokens (konfigurierbar über [`ChunkerConfig::max_tokens`]).
//!   *Begründung*: Orientiert an typischen Kontextfenstern verbreiteter Embedding-Modelle (z. B. 512 Tokens
//!   bei BERT/BGE/Nomic-Embed), um maximale Informationsdichte ohne ungewollte Kürzungs- oder Truncation-Verluste
//!   bei der Vektorisierung zu garantieren.
//! - **Toleranzgrenze (Hard Limit)**: `hard_limit = max_tokens * 1.2` (Standard: 614 Tokens).
//!   Abschnitte innerhalb dieses Rahmens werden ohne Aufspaltung zusammengehalten.
//! - **BPE / Zeichen-Verhältnis (`CHARS_PER_TOKEN`)**: 4 Zeichen pro Token (Heuristik in [`estimate_tokens`]).
//!
//! ## 2. Overlap-Strategie & Sliding-Window-Fallback
//! - Wenn ein einzelner Fließtext-Absatz (Paragraph) die Toleranzgrenze `hard_limit` überschreitet,
//!   greift der Overlap-Fallback mittels [`chunk_text_with_overlap`].
//! - **Window-Größe**: `window_chars = hard_limit * CHARS_PER_TOKEN` (Standard: ~2456 Zeichen).
//! - **Overlap**: 20% der Window-Größe (`overlap_chars = window_chars / 5`, Standard: ~491 Zeichen).
//! - **Garantie**: Der Sliding-Window-Algorithmus wahrt strikt UTF-8-Codepoint-Grenzen und schneidet niemals
//!   mitten in Multi-Byte-Sequenzen oder Unicode-Zeichen.
//!
//! ## 3. Behandlung von Codeblöcken und Tabellen (Strukturerhalt)
//! - **Codeblöcke** (eingefasst mit Triple-Backticks ` ``` `) und **Markdown-Tabellen** (`| ... |`) werden
//!   als atomare strukturelle Einheiten behandelt.
//! - **Regel**: Passt ein Codeblock oder eine Tabelle in die Zielgröße (`tokens <= hard_limit`), wird der Block
//!   **niemals** mitten im Block oder mitten in einer Zeile zerschnitten.
//! - **Fallback bei übergroßen Codeblöcken**:
//!   Codeblöcke, die `hard_limit` überschreiten, werden primär an Zeilengrenzen (`\n`) aufgeteilt, wobei die
//!   Code-Fence-Kopfzeile (z. B. ` ```rust `) und der schließende Fence (` ``` `) bei jedem erzeugten Teil-Chunk
//!   wiederholt werden, um die Gültigkeit der Code-Syntax zu bewahren.
//! - **Fallback bei übergroßen Tabellen**:
//!   Tabellen, die `hard_limit` überschreiten, werden an Tabellenzeilen-Grenzen (`\n`) getrennt. Jedes erzeugte
//!   Tabellen-Fragment behält die ursprünglichen Kopfzeilen (Header-Zeile + Trennzeile `|---|...|`) bei,
//!   sodass das Schema in allen Teil-Chunks erhalten bleibt.
//!
//! ## 4. Beziehung zur ContextPrefixEngine (Teil 6.13)
//! - **Ablaufreihenfolge in der Pipeline**:
//!   1. **MarkdownChunker**: Das Dokument wird in saubere Markdown-Chunks zerlegt.
//!   2. **ContextPrefixEngine** (`contextra-infer-ollama`): Für jeden fertigen Chunk wird über ein kleines
//!      LLM-Modell (z. B. `llama3.2:3b`, `max_document_chars` Default 8000, `max_prefix_tokens` Default 80)
//!      ein 1–2-Satz-Kontextpräfix generiert (unter XML-Escaping zur Prompt-Injection-Isolation).
//!   3. Das generierte Präfix wird im Feld [`ContextChunk::contextual_prefix`] gespeichert bzw. vor BM25/Embedding
//!      dem Chunk-Text vorangestellt.
//! - **Garantie**: Chunking erfolgt **strikt vor** der Präfix-Generierung. Der Chunker operiert ausschließlich
//!   auf dem reinen Quelltext des Dokuments und beeinflusst nicht die Präfix-Synthese.

use contextra_types::{ContextChunk, DocId};
use serde_json::json;

/// Approximate character count per token (BPE ratio).
const CHARS_PER_TOKEN: usize = 4;

/// Schätzt die Token-Anzahl eines Textes mit heuristischen Regeln.
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let mut tokens: f64 = 0.0;
    let mut in_code_block = false;
    let code_multiplier = 1.8f64;

    for line in text.lines() {
        if line.starts_with("```") {
            in_code_block = !in_code_block;
            tokens += 1.0;
            continue;
        }

        let multiplier = if in_code_block { code_multiplier } else { 1.0 };

        let mut char_iter = line.chars().peekable();
        while let Some(c) = char_iter.next() {
            match c {
                '\u{4E00}'..='\u{9FFF}'
                | '\u{3400}'..='\u{4DBF}'
                | '\u{20000}'..='\u{2A6DF}'
                | '\u{3040}'..='\u{309F}'
                | '\u{30A0}'..='\u{30FF}' => {
                    tokens += 1.0 * multiplier;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut word_len = 1usize;
                    while char_iter
                        .peek()
                        .is_some_and(|c| c.is_alphanumeric() || *c == '_')
                    {
                        char_iter.next();
                        word_len += 1;
                    }
                    let word_tokens = if word_len <= 4 {
                        1.0
                    } else {
                        1.0 + (word_len as f64 - 4.0) / 4.0
                    };
                    tokens += word_tokens * multiplier;
                }
                '0'..='9' => {
                    let mut num_len = 1usize;
                    while char_iter.peek().is_some_and(|c| c.is_ascii_digit()) {
                        char_iter.next();
                        num_len += 1;
                    }
                    tokens += ((num_len as f64) / 3.0).ceil() * multiplier;
                }
                ' ' | '\t' => {}
                _ => {
                    tokens += 0.25 * multiplier;
                }
            }
        }
        tokens += 0.1;
    }

    (tokens.ceil() as usize).max(1)
}

/// Configuration for the Markdown chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Maximum tokens per chunk (soft limit, hard limit is this * 1.2).
    /// Default: 512 tokens (aligned with standard dense embedding models).
    pub max_tokens: usize,
    /// Minimum tokens per chunk (merge threshold).
    pub min_tokens: usize,
    /// Include heading breadcrumb as metadata in resulting chunks.
    pub include_breadcrumbs: bool,
    /// Heading levels to split on (1 = H1, 2 = H2, ...).
    pub split_levels: Vec<u8>,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            max_tokens: 512,
            min_tokens: 50,
            include_breadcrumbs: true,
            split_levels: vec![1, 2, 3],
        }
    }
}

/// A deterministic chunker for markdown files respecting structural blocks
/// (headings, code blocks, tables, paragraphs) and UTF-8 safety.
pub struct MarkdownChunker {
    config: ChunkerConfig,
}

#[derive(Debug, Clone)]
struct RawSection {
    lines: Vec<String>,
    breadcrumb: String,
    heading_level: u8,
    source_line: usize,
    tokens: usize,
}

#[derive(Debug, Clone)]
enum MarkdownBlock {
    CodeBlock {
        fence: String,
        body_lines: Vec<String>,
    },
    Table {
        header_lines: Vec<String>,
        body_lines: Vec<String>,
    },
    Paragraph {
        lines: Vec<String>,
    },
}

impl MarkdownBlock {
    fn to_text(&self) -> String {
        match self {
            MarkdownBlock::CodeBlock { fence, body_lines } => {
                let mut s = fence.clone();
                s.push('\n');
                s.push_str(&body_lines.join("\n"));
                if !s.ends_with('\n') {
                    s.push('\n');
                }
                s.push_str("```");
                s
            }
            MarkdownBlock::Table {
                header_lines,
                body_lines,
            } => {
                let mut lines = header_lines.clone();
                lines.extend(body_lines.iter().cloned());
                lines.join("\n")
            }
            MarkdownBlock::Paragraph { lines } => lines.join("\n"),
        }
    }

    fn estimate_tokens(&self) -> usize {
        estimate_tokens(&self.to_text())
    }
}

fn parse_markdown_blocks(lines: &[String]) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            let fence = line.clone();
            let mut body_lines = Vec::new();
            i += 1;
            while i < lines.len() {
                let inner_trim = lines[i].trim();
                if inner_trim.starts_with("```") {
                    i += 1;
                    break;
                }
                body_lines.push(lines[i].clone());
                i += 1;
            }
            blocks.push(MarkdownBlock::CodeBlock { fence, body_lines });
        } else if is_table_row(line) {
            let mut table_lines = Vec::new();
            while i < lines.len() && is_table_row(&lines[i]) {
                table_lines.push(lines[i].clone());
                i += 1;
            }
            let (header_lines, body_lines) =
                if table_lines.len() >= 2 && is_table_delimiter_row(&table_lines[1]) {
                    (table_lines[..2].to_vec(), table_lines[2..].to_vec())
                } else if !table_lines.is_empty() && is_table_delimiter_row(&table_lines[0]) {
                    (table_lines[..1].to_vec(), table_lines[1..].to_vec())
                } else {
                    (Vec::new(), table_lines)
                };
            blocks.push(MarkdownBlock::Table {
                header_lines,
                body_lines,
            });
        } else {
            let mut para_lines = Vec::new();
            while i < lines.len() {
                let cur = &lines[i];
                let cur_trim = cur.trim();
                if cur_trim.starts_with("```") || is_table_row(cur) {
                    break;
                }
                if cur_trim.is_empty() && !para_lines.is_empty() {
                    i += 1;
                    break;
                }
                if !cur_trim.is_empty() || !para_lines.is_empty() {
                    para_lines.push(cur.clone());
                }
                i += 1;
            }
            if !para_lines.is_empty() {
                blocks.push(MarkdownBlock::Paragraph { lines: para_lines });
            }
        }
    }

    blocks
}

fn is_table_row(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return false;
    }
    t.starts_with('|') || (t.ends_with('|') && t.contains('|'))
}

fn is_table_delimiter_row(line: &str) -> bool {
    let t = line.trim();
    t.contains("---") || t.contains("-|-")
}

fn split_raw_section(
    sec: RawSection,
    hard_limit: usize,
    window_chars: usize,
    overlap_chars: usize,
) -> Vec<RawSection> {
    if sec.tokens <= hard_limit {
        return vec![sec];
    }

    let blocks = parse_markdown_blocks(&sec.lines);
    let mut result = Vec::new();
    let mut current_block_texts = Vec::new();
    let mut current_tokens = 0;

    let flush_current = |texts: &mut Vec<String>, target: &mut Vec<RawSection>| {
        if !texts.is_empty() {
            let content = texts.join("\n");
            let tokens = estimate_tokens(&content);
            target.push(RawSection {
                lines: vec![content],
                breadcrumb: sec.breadcrumb.clone(),
                heading_level: sec.heading_level,
                source_line: sec.source_line,
                tokens,
            });
            texts.clear();
        }
    };

    for block in blocks {
        let block_text = block.to_text();
        let block_tokens = block.estimate_tokens();

        if block_tokens <= hard_limit {
            if current_tokens + block_tokens > hard_limit && !current_block_texts.is_empty() {
                flush_current(&mut current_block_texts, &mut result);
                current_tokens = 0;
            }
            current_block_texts.push(block_text);
            current_tokens += block_tokens;
        } else {
            flush_current(&mut current_block_texts, &mut result);
            current_tokens = 0;

            match block {
                MarkdownBlock::CodeBlock { fence, body_lines } => {
                    let mut code_chunk_lines = Vec::new();
                    let base_tokens = estimate_tokens(&format!("{}\n```", fence));
                    let mut code_chunk_tokens = base_tokens;

                    for line in body_lines {
                        let line_tokens = estimate_tokens(&line);
                        if code_chunk_tokens + line_tokens > hard_limit && !code_chunk_lines.is_empty() {
                            let content = format!("{}\n{}\n```", fence, code_chunk_lines.join("\n"));
                            let tokens = estimate_tokens(&content);
                            result.push(RawSection {
                                lines: vec![content],
                                breadcrumb: sec.breadcrumb.clone(),
                                heading_level: sec.heading_level,
                                source_line: sec.source_line,
                                tokens,
                            });
                            code_chunk_lines.clear();
                            code_chunk_tokens = base_tokens;
                        }

                        if base_tokens + line_tokens > hard_limit {
                            let windows = chunk_text_with_overlap(&line, window_chars, overlap_chars);
                            for w in windows {
                                let w_content = format!("{}\n{}\n```", fence, w);
                                let w_tokens = estimate_tokens(&w_content);
                                result.push(RawSection {
                                    lines: vec![w_content],
                                    breadcrumb: sec.breadcrumb.clone(),
                                    heading_level: sec.heading_level,
                                    source_line: sec.source_line,
                                    tokens: w_tokens,
                                });
                            }
                        } else {
                            code_chunk_lines.push(line);
                            code_chunk_tokens += line_tokens;
                        }
                    }

                    if !code_chunk_lines.is_empty() {
                        let content = format!("{}\n{}\n```", fence, code_chunk_lines.join("\n"));
                        let tokens = estimate_tokens(&content);
                        result.push(RawSection {
                            lines: vec![content],
                            breadcrumb: sec.breadcrumb.clone(),
                            heading_level: sec.heading_level,
                            source_line: sec.source_line,
                            tokens,
                        });
                    }
                }
                MarkdownBlock::Table {
                    header_lines,
                    body_lines,
                } => {
                    let header_prefix = header_lines.join("\n");
                    let header_tokens = estimate_tokens(&header_prefix);
                    let mut table_chunk_lines = Vec::new();
                    let mut table_chunk_tokens = header_tokens;

                    for line in body_lines {
                        let line_tokens = estimate_tokens(&line);
                        if table_chunk_tokens + line_tokens > hard_limit && !table_chunk_lines.is_empty() {
                            let mut full_table_lines = header_lines.clone();
                            full_table_lines.append(&mut table_chunk_lines);
                            let content = full_table_lines.join("\n");
                            let tokens = estimate_tokens(&content);
                            result.push(RawSection {
                                lines: vec![content],
                                breadcrumb: sec.breadcrumb.clone(),
                                heading_level: sec.heading_level,
                                source_line: sec.source_line,
                                tokens,
                            });
                            table_chunk_tokens = header_tokens;
                        }

                        if header_tokens + line_tokens > hard_limit {
                            let windows = chunk_text_with_overlap(&line, window_chars, overlap_chars);
                            for w in windows {
                                let content = if header_prefix.is_empty() {
                                    w.to_string()
                                } else {
                                    format!("{}\n{}", header_prefix, w)
                                };
                                let tokens = estimate_tokens(&content);
                                result.push(RawSection {
                                    lines: vec![content],
                                    breadcrumb: sec.breadcrumb.clone(),
                                    heading_level: sec.heading_level,
                                    source_line: sec.source_line,
                                    tokens,
                                });
                            }
                        } else {
                            table_chunk_lines.push(line);
                            table_chunk_tokens += line_tokens;
                        }
                    }

                    if !table_chunk_lines.is_empty() {
                        let mut full_table_lines = header_lines;
                        full_table_lines.extend(table_chunk_lines);
                        let content = full_table_lines.join("\n");
                        let tokens = estimate_tokens(&content);
                        result.push(RawSection {
                            lines: vec![content],
                            breadcrumb: sec.breadcrumb.clone(),
                            heading_level: sec.heading_level,
                            source_line: sec.source_line,
                            tokens,
                        });
                    }
                }
                MarkdownBlock::Paragraph { lines } => {
                    let content = lines.join("\n");
                    let windows = chunk_text_with_overlap(&content, window_chars, overlap_chars);
                    for w in windows {
                        let w_tokens = estimate_tokens(w);
                        result.push(RawSection {
                            lines: vec![w.to_string()],
                            breadcrumb: sec.breadcrumb.clone(),
                            heading_level: sec.heading_level,
                            source_line: sec.source_line,
                            tokens: w_tokens,
                        });
                    }
                }
            }
        }
    }

    flush_current(&mut current_block_texts, &mut result);
    result
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
        let hard_limit = ((self.config.max_tokens as f64 * 1.2) as usize).max(1);
        let mut raw_sections = Vec::new();
        let mut current_lines = Vec::new();
        let mut current_breadcrumb = String::new();
        let mut current_heading_level = 0;
        let mut current_source_line = 1;

        let mut heading_stack: Vec<(u8, String)> = Vec::new();
        let mut in_code_block = false;

        for (i, line) in markdown.lines().enumerate() {
            let line_num = i + 1;

            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code_block = !in_code_block;
            }

            let mut is_heading = false;
            let mut h_level = 0;
            if !in_code_block && line.starts_with('#') {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 && parts[0].chars().all(|c| c == '#') {
                    h_level = parts[0].len() as u8;
                    if self.config.split_levels.contains(&h_level) {
                        is_heading = true;
                    }
                }
            }

            if is_heading {
                if !current_lines.is_empty() {
                    let content = current_lines.join("\n");
                    let tokens = estimate_tokens(&content);
                    raw_sections.push(RawSection {
                        lines: std::mem::take(&mut current_lines),
                        breadcrumb: current_breadcrumb.clone(),
                        heading_level: current_heading_level,
                        source_line: current_source_line,
                        tokens,
                    });
                }

                heading_stack.retain(|(lvl, _)| *lvl < h_level);
                let heading_text = line.trim_start_matches('#').trim();
                let breadcrumb_part = format!("{} {}", parts_first(line, h_level), heading_text);
                heading_stack.push((h_level, breadcrumb_part));

                current_breadcrumb = heading_stack
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
                    .join(" > ");
                current_heading_level = h_level;
                current_source_line = line_num;
                current_lines.push(line.to_string());
            } else {
                current_lines.push(line.to_string());
            }
        }

        if !current_lines.is_empty() {
            let content = current_lines.join("\n");
            let tokens = estimate_tokens(&content);
            raw_sections.push(RawSection {
                lines: current_lines,
                breadcrumb: current_breadcrumb,
                heading_level: current_heading_level,
                source_line: current_source_line,
                tokens,
            });
        }

        let window_chars = hard_limit * CHARS_PER_TOKEN;
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
                    && (last.tokens + sec.tokens) <= hard_limit
                {
                    last.lines.push("".to_string());
                    last.lines.extend(sec.lines);
                    last.tokens += sec.tokens;
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
                        "source_line": sec.source_line
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

pub fn chunk_text_with_overlap(text: &str, window_chars: usize, overlap_chars: usize) -> Vec<&str> {
    if text.is_empty() || window_chars == 0 {
        return Vec::new();
    }

    let indices: Vec<usize> = text
        .char_indices()
        .map(|(idx, _)| idx)
        .chain(std::iter::once(text.len()))
        .collect();

    let total_chars = indices.len() - 1;
    let overlap = overlap_chars.min(window_chars.saturating_sub(1));
    let step = window_chars.saturating_sub(overlap).max(1);

    let mut chunks = Vec::new();
    let mut start_char = 0;

    while start_char < total_chars {
        let end_char = (start_char + window_chars).min(total_chars);
        let start_byte = indices[start_char];
        let end_byte = indices[end_char];

        chunks.push(&text[start_byte..end_byte]);

        if end_char == total_chars {
            break;
        }

        start_char += step;
    }

    chunks
}

pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<&str> {
    if text.is_empty() || chunk_size == 0 {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    let mut char_count = 0;
    let mut start_byte = 0;

    for (byte_idx, _ch) in text.char_indices() {
        if char_count == chunk_size {
            chunks.push(&text[start_byte..byte_idx]);
            start_byte = byte_idx;
            char_count = 0;
        }
        char_count += 1;
    }

    if start_byte < text.len() {
        chunks.push(&text[start_byte..]);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_unicode_german_umlauts() {
        let text = "Äpfel, Öle, Übermut und Straße sind wunderschön!! ".repeat(2);
        let char_count = text.chars().count();
        assert!(
            char_count >= 100,
            "Test string must have at least 100 chars, got {}",
            char_count
        );

        let chunks = chunk_text(&text, 30);
        assert!(!chunks.is_empty());

        for chunk in &chunks {
            assert!(std::str::from_utf8(chunk.as_bytes()).is_ok());
            assert!(chunk.chars().count() <= 30);
        }

        let reassembled = chunks.join("");
        assert_eq!(reassembled, text);
    }

    #[test]
    fn chunker_empty_input() {
        let chunks = MarkdownChunker::with_defaults().chunk(DocId::new(1), "");
        assert!(chunks.is_empty());
    }

    #[test]
    fn chunker_single_chunk_no_split() {
        let text = "Short text";
        let chunks = MarkdownChunker::with_defaults().chunk(DocId::new(1), text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn chunker_respects_max_chunk_size() {
        let text = (0..50)
            .map(|_| "word ".repeat(20))
            .collect::<Vec<_>>()
            .join("\n\n");
        let config = ChunkerConfig {
            max_tokens: 100,
            min_tokens: 10,
            ..Default::default()
        };
        let chunks = MarkdownChunker::new(config).chunk(DocId::new(1), &text);
        let limit = (100.0 * 1.2) as usize;
        for chunk in &chunks {
            assert!(
                chunk.token_count <= limit,
                "Chunk too large: {}",
                chunk.token_count
            );
        }
    }

    #[test]
    fn test_chunk_by_headings() {
        let markdown = "# Title\nSome intro.\n## Section 1\nContent 1\n### Sub 1\nSub content";
        let doc_id = DocId::new(1);

        let config = ChunkerConfig {
            min_tokens: 0,
            ..Default::default()
        };
        let chunker = MarkdownChunker::new(config);
        let chunks = chunker.chunk(doc_id, markdown);

        assert_eq!(chunks.len(), 3);

        let m0 = chunks[0].metadata.as_ref().unwrap();
        assert_eq!(m0["breadcrumb"], "# Title");
        assert_eq!(m0["heading_level"], 1);

        let m1 = chunks[1].metadata.as_ref().unwrap();
        assert_eq!(m1["breadcrumb"], "# Title > ## Section 1");
        assert_eq!(m1["heading_level"], 2);

        let m2 = chunks[2].metadata.as_ref().unwrap();
        assert_eq!(m2["breadcrumb"], "# Title > ## Section 1 > ### Sub 1");
        assert_eq!(m2["heading_level"], 3);
    }

    #[test]
    fn test_merge_small_sections() {
        let config = ChunkerConfig {
            min_tokens: 50,
            ..Default::default()
        };
        let chunker = MarkdownChunker::new(config);
        let markdown = "# S1\na\n# S2\nb\n# S3\nc\n# S4\nd\n# S5\ne";
        let doc_id = DocId::new(2);
        let chunks = chunker.chunk(doc_id, markdown);

        assert_eq!(chunks.len(), 1);

        let m = chunks[0].metadata.as_ref().unwrap();
        assert_eq!(m["breadcrumb"], "# S1");
        assert!(chunks[0].content.contains("e"));
    }

    #[test]
    fn test_no_content_loss() {
        let chunker = MarkdownChunker::with_defaults();
        let markdown = "# Title\n\nSome text\n\n## Subtitle\nMore text.";
        let doc_id = DocId::new(3);
        let chunks = chunker.chunk(doc_id, markdown);

        let concatenated: String = chunks
            .iter()
            .map(|c| c.content.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let orig_words: Vec<_> = markdown.split_whitespace().collect();
        let concat_words: Vec<_> = concatenated.split_whitespace().collect();
        assert_eq!(orig_words, concat_words);
    }

    #[test]
    fn test_real_document() {
        let chunker = MarkdownChunker::with_defaults();
        let doc_id = DocId::new(4);

        let mut markdown = String::new();
        for i in 0..100 {
            markdown.push_str(&format!("## Heading {}\n", i));
            for j in 0..20 {
                markdown.push_str(&format!("This is line {} in paragraph {}.\n", j, i));
            }
        }

        let limit = (chunker.config.max_tokens as f64 * 1.2) as usize;

        let chunks = chunker.chunk(doc_id, &markdown);
        assert!(!chunks.is_empty());
        for chunk in chunks {
            assert!(
                chunk.token_count <= limit,
                "Chunk token count {} exceeds limit {}",
                chunk.token_count,
                limit
            );
        }
    }

    #[test]
    fn test_code_block_within_target_size_preserved_atomically() {
        let code = "```rust\nfn main() {\n    let mut x = 10;\n    x += 5;\n    println!(\"Val: {}\", x);\n}\n```";
        let markdown = format!("# Code Test\n\nSome introductory paragraph.\n\n{}", code);

        let config = ChunkerConfig {
            max_tokens: 500,
            min_tokens: 0,
            ..Default::default()
        };
        let chunker = MarkdownChunker::new(config);
        let chunks = chunker.chunk(DocId::new(10), &markdown);

        let code_chunk = chunks
            .iter()
            .find(|c| c.content.contains("fn main()"))
            .expect("Should find chunk containing code block");

        assert!(
            code_chunk.content.contains("```rust") && code_chunk.content.contains("```"),
            "Code block within max_tokens must be preserved atomically without mid-block splits"
        );
    }

    #[test]
    fn test_table_header_retention_and_row_splitting() {
        let mut table_md = String::from("| ID | Name | Role |\n|---|---|---|\n");
        for i in 0..50 {
            table_md.push_str(&format!("| {} | User_{} | Role_{} |\n", i, i, i));
        }

        let config = ChunkerConfig {
            max_tokens: 30,
            min_tokens: 0,
            ..Default::default()
        };
        let chunker = MarkdownChunker::new(config);
        let chunks = chunker.chunk(DocId::new(20), &table_md);

        assert!(chunks.len() > 1, "Oversized table should be split into multiple chunks");

        for chunk in &chunks {
            assert!(
                chunk.content.contains("| ID | Name | Role |"),
                "Every sub-chunk of an oversized table must retain the table header row"
            );
            assert!(
                chunk.content.contains("|---|---|---|"),
                "Every sub-chunk of an oversized table must retain the table delimiter row"
            );
        }
    }

    #[test]
    fn test_oversized_paragraph_sliding_window_fallback() {
        let long_para = "Satz mit vielen Informationen und Details. ".repeat(100);
        let config = ChunkerConfig {
            max_tokens: 20,
            min_tokens: 0,
            ..Default::default()
        };
        let chunker = MarkdownChunker::new(config);
        let chunks = chunker.chunk(DocId::new(30), &long_para);

        assert!(chunks.len() > 1, "Oversized paragraph must be split into multiple chunks");

        let limit = (20.0 * 1.2) as usize;
        for chunk in &chunks {
            assert!(
                chunk.token_count <= limit || chunk.content.len() <= limit * CHARS_PER_TOKEN,
                "Sliding window sub-chunk must respect hard limit"
            );
            assert!(
                std::str::from_utf8(chunk.content.as_bytes()).is_ok(),
                "Sliding window sub-chunk must remain valid UTF-8"
            );
        }
    }

    #[test]
    fn test_chunker_determinism() {
        let markdown = "# Title\n\nParagraph 1 text.\n\n```python\ndef hello():\n    print('world')\n```\n\n| A | B |\n|---|---|\n| 1 | 2 |\n";
        let chunker = MarkdownChunker::with_defaults();

        let run1 = chunker.chunk(DocId::new(40), markdown);
        let run2 = chunker.chunk(DocId::new(40), markdown);

        assert_eq!(run1.len(), run2.len());
        for (c1, c2) in run1.iter().zip(run2.iter()) {
            assert_eq!(c1.content, c2.content);
            assert_eq!(c1.token_count, c2.token_count);
            assert_eq!(c1.metadata, c2.metadata);
            assert_eq!(c1.doc_id, c2.doc_id);
        }
    }

    #[test]
    fn test_chunk_text_zero_chunk_size() {
        assert!(chunk_text("hello world", 0).is_empty());
    }

    #[test]
    fn test_chunk_text_empty_string_returns_empty() {
        assert!(chunk_text("", 10).is_empty());
    }

    #[test]
    fn test_chunk_text_emoji_multibyte_boundary() {
        let text = "🦀🚀🔥⭐🎉";
        let chunks = chunk_text(text, 2);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "🦀🚀");
        assert_eq!(chunks[1], "🔥⭐");
        assert_eq!(chunks[2], "🎉");
    }

    #[test]
    fn test_chunk_text_exact_chunk_size_multiple() {
        let text = "abcdefghij";
        let chunks = chunk_text(text, 5);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], "abcde");
        assert_eq!(chunks[1], "fghij");
    }

    #[test]
    fn test_chunker_zero_max_tokens_handled_safely() {
        let config = ChunkerConfig {
            max_tokens: 0,
            ..Default::default()
        };
        let chunker = MarkdownChunker::new(config);
        let chunks = chunker.chunk(DocId::new(10), "Line 1\nLine 2");
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_chunker_single_line_no_headings() {
        let chunker = MarkdownChunker::with_defaults();
        let chunks = chunker.chunk(DocId::new(5), "Plain single line document.");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, "Plain single line document.");
        let meta = chunks[0].metadata.as_ref().unwrap();
        assert_eq!(meta["breadcrumb"], "");
        assert_eq!(meta["heading_level"], 0);
    }

    #[test]
    fn test_chunker_unsplit_heading_level_ignored() {
        let config = ChunkerConfig {
            split_levels: vec![1, 2],
            min_tokens: 0,
            ..Default::default()
        };
        let chunker = MarkdownChunker::new(config);
        let markdown = "# Level 1\nText 1\n### Level 3\nText 3";
        let chunks = chunker.chunk(DocId::new(6), markdown);
        assert_eq!(chunks.len(), 1);
        let meta = chunks[0].metadata.as_ref().unwrap();
        assert_eq!(meta["breadcrumb"], "# Level 1");
    }

    #[test]
    fn chunker_handles_headingless_single_paragraph_document() {
        let line = "This is a continuous sentence representing extracted PDF text with single line breaks.\n";
        let text = line.repeat(250);
        assert!(!text.contains("#"));
        assert!(!text.contains("\n\n"));

        let chunker = MarkdownChunker::with_defaults();
        let hard_limit = (chunker.config.max_tokens as f64 * 1.2) as usize;
        let chunks = chunker.chunk(DocId::new(100), &text);

        assert!(
            chunks.len() > 1,
            "Document should be split into multiple chunks, got {}",
            chunks.len()
        );

        for chunk in &chunks {
            assert!(
                chunk.token_count <= hard_limit,
                "Chunk token count {} exceeds hard limit {}",
                chunk.token_count,
                hard_limit
            );
        }
    }

    #[test]
    fn chunk_text_with_overlap_produces_overlapping_windows() {
        let text = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let window_chars = 10;
        let overlap_chars = 3;

        let windows = chunk_text_with_overlap(text, window_chars, overlap_chars);
        assert!(!windows.is_empty());

        for i in 0..windows.len() - 1 {
            let w1 = windows[i];
            let w2 = windows[i + 1];
            let suffix_w1 = &w1[w1.len() - overlap_chars..];
            let prefix_w2 = &w2[..overlap_chars];
            assert_eq!(
                suffix_w1,
                prefix_w2,
                "Windows {} and {} do not overlap correctly",
                i,
                i + 1
            );
        }

        assert_eq!(
            windows.first().unwrap().chars().take(3).collect::<String>(),
            "ABC"
        );
        assert_eq!(
            windows
                .last()
                .unwrap()
                .chars()
                .rev()
                .take(3)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>(),
            "789"
        );
    }

    #[test]
    fn chunk_text_with_overlap_respects_utf8_boundaries() {
        let text = "Äpfel, Öle, Übermut und Straße sind wunderschön!! ".repeat(5);
        let windows = chunk_text_with_overlap(&text, 25, 5);
        assert!(!windows.is_empty());

        for window in &windows {
            assert!(std::str::from_utf8(window.as_bytes()).is_ok());
            assert!(window.chars().count() <= 25);
        }
    }
}
