// FILE-CONTEXT
// ZWECK: Eigenständiger Markdown-Blockparser mit exakten Byte-Offsets ohne Fremdabhängigkeiten (WP M3 / SPEC C-11.7).
// INVARIANTEN: Zero-Panic, UTF-8-sicher, exakte Byte-Spans relativ zur Eingabe, Deterministisch.
// STAND: TS:2026-10-31T00:00:00Z

use crate::ir::Locator;
use std::ops::Range;

/// Klassifikation von geparsten Markdown-Blöcken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedBlockKind {
    YamlFrontMatter,
    Heading {
        level: u8,
        text: String,
    },
    FencedCode {
        fence_char: char,
        fence_len: usize,
        lang: Option<String>,
        body: String,
    },
    IndentedCode {
        body: String,
    },
    Table {
        header_lines: Vec<String>,
        body_lines: Vec<String>,
    },
    List {
        items: Vec<ParsedListItem>,
    },
    Blockquote {
        text: String,
    },
    HtmlBlock {
        text: String,
    },
    Note {
        label: String,
        text: String,
    },
    ThematicBreak,
    Paragraph {
        text: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedListItem {
    pub depth: u8,
    pub ordered: bool,
    pub text: String,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMarkdownBlock {
    pub kind: ParsedBlockKind,
    pub text: String,
    pub span: Range<usize>,
    pub locator: Locator,
}

#[derive(Debug, Clone, Copy)]
struct LineInfo<'a> {
    content: &'a str,
    start: usize,
    end: usize,
}

/// Zerlegt einen Markdown-Text in Blöcke mit exakten Byte-Spans.
pub fn parse_markdown_blocks(full_text: &str, base_offset: usize) -> Vec<ParsedMarkdownBlock> {
    if full_text.is_empty() {
        return Vec::new();
    }

    let lines = collect_lines(full_text);
    if lines.is_empty() {
        return Vec::new();
    }

    let mut blocks = Vec::new();
    let mut i = 0usize;

    if let Some((front_block, next_idx)) = try_parse_yaml_front_matter(&lines, full_text, base_offset) {
        blocks.push(front_block);
        i = next_idx;
    }

    while i < lines.len() {
        let line = lines.get(i).copied().unwrap_or(LineInfo {
            content: "",
            start: 0,
            end: 0,
        });

        let trimmed = line.content.trim();

        if trimmed.is_empty() {
            i = i.saturating_add(1);
            continue;
        }

        if let Some((code_block, next_idx)) = try_parse_fenced_code(&lines, i, full_text, base_offset) {
            blocks.push(code_block);
            i = next_idx;
            continue;
        }

        if let Some(atx) = parse_atx_heading(line, full_text, base_offset) {
            blocks.push(atx);
            i = i.saturating_add(1);
            continue;
        }

        if let Some((setext, next_idx)) = try_parse_setext_heading(&lines, i, full_text, base_offset) {
            blocks.push(setext);
            i = next_idx;
            continue;
        }

        if let Some((table_block, next_idx)) = try_parse_table(&lines, i, full_text, base_offset) {
            blocks.push(table_block);
            i = next_idx;
            continue;
        }

        if let Some((note_block, next_idx)) = try_parse_note(&lines, i, full_text, base_offset) {
            blocks.push(note_block);
            i = next_idx;
            continue;
        }

        if is_thematic_break(trimmed) {
            blocks.push(ParsedMarkdownBlock {
                kind: ParsedBlockKind::ThematicBreak,
                text: line.content.to_string(),
                span: (base_offset + line.start)..(base_offset + line.end),
                locator: Locator::default(),
            });
            i = i.saturating_add(1);
            continue;
        }

        if trimmed.starts_with('>') {
            let (quote_block, next_idx) = parse_blockquote(&lines, i, full_text, base_offset);
            blocks.push(quote_block);
            i = next_idx;
            continue;
        }

        if is_list_item_start(line.content) {
            let (list_block, next_idx) = parse_list(&lines, i, full_text, base_offset);
            blocks.push(list_block);
            i = next_idx;
            continue;
        }

        if is_indented_code(line.content) {
            let (indented_block, next_idx) = parse_indented_code(&lines, i, full_text, base_offset);
            blocks.push(indented_block);
            i = next_idx;
            continue;
        }

        if trimmed.starts_with('<') && (trimmed.ends_with('>') || trimmed.starts_with("<!--")) {
            let (html_block, next_idx) = parse_html_block(&lines, i, full_text, base_offset);
            blocks.push(html_block);
            i = next_idx;
            continue;
        }

        let (para_block, next_idx) = parse_paragraph(&lines, i, full_text, base_offset);
        blocks.push(para_block);
        i = next_idx;
    }

    blocks
}

fn collect_lines(text: &str) -> Vec<LineInfo<'_>> {
    let mut lines = Vec::new();
    let mut start = 0usize;

    for (idx, ch) in text.char_indices() {
        if ch == '\n' {
            let line_content = text.get(start..idx).unwrap_or("");
            lines.push(LineInfo {
                content: line_content,
                start,
                end: idx,
            });
            start = idx.saturating_add(1);
        }
    }

    if start <= text.len() {
        let line_content = text.get(start..text.len()).unwrap_or("");
        lines.push(LineInfo {
            content: line_content,
            start,
            end: text.len(),
        });
    }

    lines
}

fn try_parse_yaml_front_matter<'a>(
    lines: &[LineInfo<'a>],
    full_text: &'a str,
    base_offset: usize,
) -> Option<(ParsedMarkdownBlock, usize)> {
    if lines.is_empty() {
        return None;
    }
    let first = lines.first()?;
    if first.content.trim() != "---" {
        return None;
    }

    let mut i = 1usize;
    while i < lines.len() {
        let line = lines.get(i)?;
        let t = line.content.trim();
        if t == "---" || t == "..." {
            let end_byte = line.end;
            let slice = full_text.get(first.start..end_byte).unwrap_or("");
            let block = ParsedMarkdownBlock {
                kind: ParsedBlockKind::YamlFrontMatter,
                text: slice.to_string(),
                span: (base_offset + first.start)..(base_offset + end_byte),
                locator: Locator::default(),
            };
            return Some((block, i.saturating_add(1)));
        }
        i = i.saturating_add(1);
    }

    None
}

fn try_parse_fenced_code<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> Option<(ParsedMarkdownBlock, usize)> {
    let line = lines.get(start_idx)?;
    let trimmed = line.content.trim_start();
    let indent = line.content.len().saturating_sub(trimmed.len());
    if indent > 3 {
        return None;
    }

    let fence_char = if trimmed.starts_with("```") {
        '`'
    } else if trimmed.starts_with("~~~") {
        '~'
    } else {
        return None;
    };

    let fence_len = trimmed.chars().take_while(|&c| c == fence_char).count();
    if fence_len < 3 {
        return None;
    }

    let rest = trimmed.get(fence_len..).unwrap_or("").trim();
    let lang = if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    };

    let mut body_lines = Vec::new();
    let mut i = start_idx.saturating_add(1);
    let mut closed = false;
    let mut end_byte = line.end;

    while i < lines.len() {
        let cur = lines.get(i)?;
        end_byte = cur.end;
        let cur_trim = cur.content.trim();

        if cur_trim.starts_with(fence_char) {
            let cur_fence_len = cur_trim.chars().take_while(|&c| c == fence_char).count();
            if cur_fence_len >= fence_len && cur_trim.get(cur_fence_len..).unwrap_or("").trim().is_empty() {
                closed = true;
                i = i.saturating_add(1);
                break;
            }
        }

        body_lines.push(cur.content);
        i = i.saturating_add(1);
    }

    let _ = closed;
    let full_span_str = full_text.get(line.start..end_byte).unwrap_or("");
    let body_text = body_lines.join("\n");

    Some((
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::FencedCode {
                fence_char,
                fence_len,
                lang,
                body: body_text,
            },
            text: full_span_str.to_string(),
            span: (base_offset + line.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    ))
}

fn parse_atx_heading(line: LineInfo<'_>, full_text: &str, base_offset: usize) -> Option<ParsedMarkdownBlock> {
    let trimmed = line.content.trim_start();
    let indent = line.content.len().saturating_sub(trimmed.len());
    if indent > 3 || !trimmed.starts_with('#') {
        return None;
    }

    let hashes = trimmed.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }

    let rest = trimmed.get(hashes..).unwrap_or("");
    if !rest.is_empty() && !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }

    let heading_text = rest.trim().trim_end_matches('#').trim().to_string();
    let slice = full_text.get(line.start..line.end).unwrap_or("");

    Some(ParsedMarkdownBlock {
        kind: ParsedBlockKind::Heading {
            level: u8::try_from(hashes.clamp(1, 6)).unwrap_or(1),
            text: heading_text,
        },
        text: slice.to_string(),
        span: (base_offset + line.start)..(base_offset + line.end),
        locator: Locator::default(),
    })
}

fn try_parse_setext_heading<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> Option<(ParsedMarkdownBlock, usize)> {
    let first = lines.get(start_idx)?;
    let second = lines.get(start_idx.saturating_add(1))?;

    let text_trim = first.content.trim();
    if text_trim.is_empty() || text_trim.starts_with('#') {
        return None;
    }

    let underline_trim = second.content.trim();
    if underline_trim.is_empty() {
        return None;
    }

    let level = if underline_trim.chars().all(|c| c == '=') && underline_trim.len() >= 2 {
        1u8
    } else if underline_trim.chars().all(|c| c == '-') && underline_trim.len() >= 2 {
        2u8
    } else {
        return None;
    };

    let slice = full_text.get(first.start..second.end).unwrap_or("");
    Some((
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::Heading {
                level,
                text: text_trim.to_string(),
            },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + second.end),
            locator: Locator::default(),
        },
        start_idx.saturating_add(2),
    ))
}

fn try_parse_table<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> Option<(ParsedMarkdownBlock, usize)> {
    let first = lines.get(start_idx)?;
    if !is_table_row(first.content) {
        return None;
    }

    let second = lines.get(start_idx.saturating_add(1))?;
    if !is_table_delimiter_row(second.content) {
        return None;
    }

    let mut i = start_idx.saturating_add(2);
    let mut end_byte = second.end;

    while i < lines.len() {
        let cur = lines.get(i)?;
        if !is_table_row(cur.content) {
            break;
        }
        end_byte = cur.end;
        i = i.saturating_add(1);
    }

    let slice = full_text.get(first.start..end_byte).unwrap_or("");
    let table_lines: Vec<String> = slice.lines().map(|l| l.to_string()).collect();

    let (header_lines, body_lines) = if table_lines.len() >= 2 {
        (
            table_lines.get(..2).unwrap_or(&[]).to_vec(),
            table_lines.get(2..).unwrap_or(&[]).to_vec(),
        )
    } else {
        (Vec::new(), table_lines)
    };

    Some((
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::Table {
                header_lines,
                body_lines,
            },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    ))
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
    (t.contains("---") || t.contains("-|-")) && t.contains('|')
}

fn try_parse_note<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> Option<(ParsedMarkdownBlock, usize)> {
    let first = lines.get(start_idx)?;
    let trimmed = first.content.trim();
    if !trimmed.starts_with("[^") && !trimmed.starts_with("[note") {
        return None;
    }

    let colon_pos = trimmed.find("]: ")?;
    let label = trimmed.get(1..colon_pos).unwrap_or("note").to_string();
    let mut note_text = trimmed.get(colon_pos + 3..).unwrap_or("").to_string();

    let mut i = start_idx.saturating_add(1);
    let mut end_byte = first.end;

    while i < lines.len() {
        let cur = lines.get(i)?;
        if cur.content.starts_with("  ") || cur.content.starts_with('\t') {
            note_text.push('\n');
            note_text.push_str(cur.content.trim());
            end_byte = cur.end;
            i = i.saturating_add(1);
        } else {
            break;
        }
    }

    let slice = full_text.get(first.start..end_byte).unwrap_or("");
    Some((
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::Note { label, text: note_text },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    ))
}

fn is_thematic_break(line: &str) -> bool {
    let t = line.trim();
    if t.len() < 3 {
        return false;
    }
    let chars: Vec<char> = t.chars().filter(|&c| !c.is_whitespace()).collect();
    if chars.len() < 3 {
        return false;
    }
    let first = chars.first().copied().unwrap_or(' ');
    (first == '-' || first == '*' || first == '_') && chars.iter().all(|&c| c == first)
}

fn parse_blockquote<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> (ParsedMarkdownBlock, usize) {
    let first = lines.get(start_idx).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
    let mut i = start_idx;
    let mut end_byte = first.end;
    let mut body_lines = Vec::new();

    while i < lines.len() {
        let cur = lines.get(i).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
        let trimmed = cur.content.trim();
        if trimmed.starts_with('>') {
            let inner = trimmed.trim_start_matches('>').trim_start();
            body_lines.push(inner.to_string());
            end_byte = cur.end;
            i = i.saturating_add(1);
        } else {
            break;
        }
    }

    let slice = full_text.get(first.start..end_byte).unwrap_or("");
    (
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::Blockquote {
                text: body_lines.join("\n"),
            },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    )
}

fn is_list_item_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return true;
    }
    let mut parts = trimmed.splitn(2, ". ");
    if let Some(first) = parts.next() {
        if first.chars().all(|c| c.is_ascii_digit()) && parts.next().is_some() {
            return true;
        }
    }
    false
}

fn parse_list<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> (ParsedMarkdownBlock, usize) {
    let first = lines.get(start_idx).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
    let mut items = Vec::new();
    let mut i = start_idx;
    let mut end_byte = first.end;

    while i < lines.len() {
        let cur = lines.get(i).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
        if is_list_item_start(cur.content) {
            let trimmed = cur.content.trim_start();
            let indent = cur.content.len().saturating_sub(trimmed.len());
            let depth = u8::try_from((indent / 2).min(255)).unwrap_or(0);
            let ordered = trimmed.chars().next().is_some_and(|c| c.is_ascii_digit());

            let text = if ordered {
                trimmed.split_once(". ").map(|x| x.1).unwrap_or("").to_string()
            } else {
                trimmed.get(2..).unwrap_or("").to_string()
            };

            end_byte = cur.end;
            items.push(ParsedListItem {
                depth,
                ordered,
                text,
                span: (base_offset + cur.start)..(base_offset + cur.end),
            });
            i = i.saturating_add(1);
        } else if cur.content.starts_with("  ") && !items.is_empty() {
            if let Some(last) = items.last_mut() {
                last.text.push('\n');
                last.text.push_str(cur.content.trim());
                last.span.end = base_offset + cur.end;
            }
            end_byte = cur.end;
            i = i.saturating_add(1);
        } else {
            break;
        }
    }

    let slice = full_text.get(first.start..end_byte).unwrap_or("");
    (
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::List { items },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    )
}

fn is_indented_code(line: &str) -> bool {
    line.starts_with("    ") || line.starts_with('\t')
}

fn parse_indented_code<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> (ParsedMarkdownBlock, usize) {
    let first = lines.get(start_idx).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
    let mut i = start_idx;
    let mut end_byte = first.end;
    let mut code_lines = Vec::new();

    while i < lines.len() {
        let cur = lines.get(i).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
        if is_indented_code(cur.content) || cur.content.trim().is_empty() {
            code_lines.push(cur.content.to_string());
            end_byte = cur.end;
            i = i.saturating_add(1);
        } else {
            break;
        }
    }

    let slice = full_text.get(first.start..end_byte).unwrap_or("");
    (
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::IndentedCode {
                body: code_lines.join("\n"),
            },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    )
}

fn parse_html_block<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> (ParsedMarkdownBlock, usize) {
    let first = lines.get(start_idx).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
    let mut i = start_idx.saturating_add(1);
    let mut end_byte = first.end;

    while i < lines.len() {
        let cur = lines.get(i).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
        if cur.content.trim().is_empty() {
            break;
        }
        end_byte = cur.end;
        i = i.saturating_add(1);
    }

    let slice = full_text.get(first.start..end_byte).unwrap_or("");
    (
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::HtmlBlock {
                text: slice.to_string(),
            },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    )
}

fn parse_paragraph<'a>(
    lines: &[LineInfo<'a>],
    start_idx: usize,
    full_text: &'a str,
    base_offset: usize,
) -> (ParsedMarkdownBlock, usize) {
    let first = lines.get(start_idx).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
    let mut i = start_idx;
    let mut end_byte = first.end;
    let mut para_lines = Vec::new();

    while i < lines.len() {
        let cur = lines.get(i).copied().unwrap_or(LineInfo { content: "", start: 0, end: 0 });
        let trimmed = cur.content.trim();

        if (trimmed.is_empty()
            || cur.content.trim_start().starts_with("```")
            || cur.content.trim_start().starts_with("~~~")
            || is_table_row(cur.content)
            || parse_atx_heading(cur, full_text, base_offset).is_some()
            || is_thematic_break(trimmed)
            || is_list_item_start(cur.content))
            && !para_lines.is_empty()
        {
            break;
        }

        para_lines.push(cur.content);
        end_byte = cur.end;
        i = i.saturating_add(1);
    }

    let slice = full_text.get(first.start..end_byte).unwrap_or("");
    (
        ParsedMarkdownBlock {
            kind: ParsedBlockKind::Paragraph {
                text: para_lines.join("\n"),
            },
            text: slice.to_string(),
            span: (base_offset + first.start)..(base_offset + end_byte),
            locator: Locator::default(),
        },
        i,
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic, clippy::get_first)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_atx_heading_span() {
        let input = "# Title\n\nParagraph text here.";
        let blocks = parse_markdown_blocks(input, 0);
        assert_eq!(blocks.len(), 2);
        assert_eq!(input.get(blocks[0].span.clone()).unwrap(), "# Title");
        assert_eq!(input.get(blocks[1].span.clone()).unwrap(), "Paragraph text here.");
    }

    #[test]
    fn test_heading_in_fenced_code_is_not_heading() {
        let input = "```rust\n# Not a heading\n```";
        let blocks = parse_markdown_blocks(input, 0);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(blocks.first().unwrap().kind, ParsedBlockKind::FencedCode { .. }));
    }
}
