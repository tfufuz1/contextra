// FILE-CONTEXT
// ZWECK: Aufspaltung übergroßer Abschnitte unter Erhalt atomarer Blöcke und Syntax-Fences.
// INVARIANTEN: Codeblock-Fences und Tabellenköpfe werden bei Aufspaltung in Teil-Chunks wiederholt.
// NICHT-OFFENSICHTLICH: Verwendet Sliding-Window-Fallback für übergroße Absätze.
// STAND: TS:2026-10-29T00:00:00Z

use super::blocks::{parse_markdown_blocks, MarkdownBlock};
use super::estimate::estimate_tokens;
use super::sections::RawSection;
use super::windows::chunk_text_with_overlap;

pub(crate) fn split_raw_section(
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
                source_line_end: sec.source_line_end,
                start_byte: sec.start_byte,
                end_byte: sec.end_byte,
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
                MarkdownBlock::CodeBlock {
                    fence,
                    body_lines,
                    fence_info,
                } => {
                    let mut code_chunk_lines = Vec::new();
                    let closing_fence = std::iter::repeat(fence_info.char_type)
                        .take(fence_info.length)
                        .collect::<String>();
                    let base_tokens = estimate_tokens(&format!("{}\n{}", fence, closing_fence));
                    let mut code_chunk_tokens = base_tokens;

                    for line in body_lines {
                        let line_tokens = estimate_tokens(&line);
                        if code_chunk_tokens + line_tokens > hard_limit
                            && !code_chunk_lines.is_empty()
                        {
                            let content = format!(
                                "{}\n{}\n{}",
                                fence,
                                code_chunk_lines.join("\n"),
                                closing_fence
                            );
                            let tokens = estimate_tokens(&content);
                            result.push(RawSection {
                                lines: vec![content],
                                breadcrumb: sec.breadcrumb.clone(),
                                heading_level: sec.heading_level,
                                source_line: sec.source_line,
                                source_line_end: sec.source_line_end,
                                start_byte: sec.start_byte,
                                end_byte: sec.end_byte,
                                tokens,
                            });
                            code_chunk_lines.clear();
                            code_chunk_tokens = base_tokens;
                        }

                        if base_tokens + line_tokens > hard_limit {
                            let windows =
                                chunk_text_with_overlap(&line, window_chars, overlap_chars);
                            for w in windows {
                                let w_content = format!("{}\n{}\n{}", fence, w, closing_fence);
                                let w_tokens = estimate_tokens(&w_content);
                                result.push(RawSection {
                                    lines: vec![w_content],
                                    breadcrumb: sec.breadcrumb.clone(),
                                    heading_level: sec.heading_level,
                                    source_line: sec.source_line,
                                    source_line_end: sec.source_line_end,
                                    start_byte: sec.start_byte,
                                    end_byte: sec.end_byte,
                                    tokens: w_tokens,
                                });
                            }
                        } else {
                            code_chunk_lines.push(line);
                            code_chunk_tokens += line_tokens;
                        }
                    }

                    if !code_chunk_lines.is_empty() {
                        let content = format!(
                            "{}\n{}\n{}",
                            fence,
                            code_chunk_lines.join("\n"),
                            closing_fence
                        );
                        let tokens = estimate_tokens(&content);
                        result.push(RawSection {
                            lines: vec![content],
                            breadcrumb: sec.breadcrumb.clone(),
                            heading_level: sec.heading_level,
                            source_line: sec.source_line,
                            source_line_end: sec.source_line_end,
                            start_byte: sec.start_byte,
                            end_byte: sec.end_byte,
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
                        if table_chunk_tokens + line_tokens > hard_limit
                            && !table_chunk_lines.is_empty()
                        {
                            let mut full_table_lines = header_lines.clone();
                            full_table_lines.append(&mut table_chunk_lines);
                            let content = full_table_lines.join("\n");
                            let tokens = estimate_tokens(&content);
                            result.push(RawSection {
                                lines: vec![content],
                                breadcrumb: sec.breadcrumb.clone(),
                                heading_level: sec.heading_level,
                                source_line: sec.source_line,
                                source_line_end: sec.source_line_end,
                                start_byte: sec.start_byte,
                                end_byte: sec.end_byte,
                                tokens,
                            });
                            table_chunk_tokens = header_tokens;
                        }

                        if header_tokens + line_tokens > hard_limit {
                            let windows =
                                chunk_text_with_overlap(&line, window_chars, overlap_chars);
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
                                    source_line_end: sec.source_line_end,
                                    start_byte: sec.start_byte,
                                    end_byte: sec.end_byte,
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
                            source_line_end: sec.source_line_end,
                            start_byte: sec.start_byte,
                            end_byte: sec.end_byte,
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
                            source_line_end: sec.source_line_end,
                            start_byte: sec.start_byte,
                            end_byte: sec.end_byte,
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
