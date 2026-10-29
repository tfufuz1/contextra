// FILE-CONTEXT
// ZWECK: reStructuredText Extractor (WP M4).
// INVARIANTEN: Zero-Panic, Adornment-Hierarchie, Blöcke für Code, Listen, Tabellen.
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator, Table};

/// Extrahiert reStructuredText (Rst) Dokumente.
#[allow(clippy::indexing_slicing, clippy::cast_possible_truncation)]
pub fn parse_rst(text: &str) -> DocumentIr {
    let mut ir = DocumentIr::new(FormatId::Rst);

    let lines: Vec<&str> = text.lines().collect();
    let mut adornment_levels: Vec<char> = Vec::new();
    let mut idx = 0usize;

    while idx < lines.len() {
        let line = lines.get(idx).copied().unwrap_or("").trim_end();

        if line.trim().is_empty() {
            idx = idx.saturating_add(1);
            continue;
        }

        let trimmed = line.trim();

        // 1. Grid Table (`+---+---+`) or Simple Table (`=== ===`)
        if trimmed.starts_with("+---") || trimmed.starts_with("===") {
            let (table_block, next_idx) = parse_rst_table(&lines, idx);
            if let Some(t) = table_block {
                ir.push(BlockKind::Table(t), Locator::default());
                idx = next_idx;
                continue;
            }
        }

        // 2. Directives (`.. code-block::`, `.. note::`, `.. warning::`, or ignore others)
        if trimmed.starts_with(".. ") {
            if let Some(rest) = trimmed.strip_prefix(".. ") {
                if rest.starts_with("code-block::") || rest.starts_with("code::") {
                    let lang = rest
                        .split("::")
                        .nth(1)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let (code_text, next_idx) =
                        parse_rst_indented_block(&lines, idx.saturating_add(1));
                    ir.push(
                        BlockKind::Code {
                            lang,
                            text: code_text,
                        },
                        Locator::default(),
                    );
                    idx = next_idx;
                    continue;
                } else if rest.starts_with("note::") || rest.starts_with("warning::") {
                    let (note_text, next_idx) =
                        parse_rst_indented_block(&lines, idx.saturating_add(1));
                    ir.push(BlockKind::Quote { text: note_text }, Locator::default());
                    idx = next_idx;
                    continue;
                } else {
                    // Andere Direktiven ignorieren
                    let (_, next_idx) = parse_rst_indented_block(&lines, idx.saturating_add(1));
                    idx = next_idx;
                    continue;
                }
            }
        }

        // 3. Underline Adornment Headings
        if idx.saturating_add(1) < lines.len() {
            let next_line = lines
                .get(idx.saturating_add(1))
                .copied()
                .unwrap_or("")
                .trim();
            if is_adornment_line(next_line) && next_line.chars().count() >= trimmed.chars().count()
            {
                let adorn_char = next_line.chars().next().unwrap_or('=');
                let level = match adornment_levels.iter().position(|&c| c == adorn_char) {
                    Some(pos) => (pos as u8).saturating_add(1),
                    None => {
                        adornment_levels.push(adorn_char);
                        (adornment_levels.len() as u8).min(6)
                    }
                };

                if ir.meta.title.is_none() && level == 1 {
                    ir.meta.title = Some(trimmed.to_string());
                }

                ir.push(
                    BlockKind::Heading {
                        level,
                        text: trimmed.to_string(),
                    },
                    Locator::default(),
                );
                idx = idx.saturating_add(2);
                continue;
            }
        }

        // 4. Literal Block marker `::` at end of line
        if trimmed == "::" || trimmed.ends_with(" ::") {
            let (code_text, next_idx) = parse_rst_indented_block(&lines, idx.saturating_add(1));
            ir.push(
                BlockKind::Code {
                    lang: None,
                    text: code_text,
                },
                Locator::default(),
            );
            idx = next_idx;
            continue;
        }

        // 5. Lists
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let list_text = trimmed[2..].trim().to_string();
            ir.push(
                BlockKind::ListItem {
                    depth: 0,
                    ordered: false,
                    text: list_text,
                },
                Locator::default(),
            );
            idx = idx.saturating_add(1);
            continue;
        }
        if let Some(ordered_text) = strip_ordered_prefix(trimmed) {
            ir.push(
                BlockKind::ListItem {
                    depth: 0,
                    ordered: true,
                    text: ordered_text.to_string(),
                },
                Locator::default(),
            );
            idx = idx.saturating_add(1);
            continue;
        }

        // 6. Standard Paragraph
        ir.push(
            BlockKind::Paragraph {
                text: trimmed.to_string(),
            },
            Locator::default(),
        );
        idx = idx.saturating_add(1);
    }

    ir
}

fn is_adornment_line(line: &str) -> bool {
    if line.len() < 3 {
        return false;
    }
    let first = match line.chars().next() {
        Some(c) => c,
        None => return false,
    };
    matches!(
        first,
        '=' | '-' | '~' | '"' | '\'' | '^' | '#' | '*' | '+' | ':'
    ) && line.chars().all(|c| c == first)
}

fn strip_ordered_prefix(s: &str) -> Option<&str> {
    if let Some(rest) = s.strip_prefix("#. ") {
        return Some(rest.trim());
    }
    let mut parts = s.splitn(2, ". ");
    if let (Some(num), Some(rest)) = (parts.next(), parts.next()) {
        if num
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_alphabetic())
        {
            return Some(rest.trim());
        }
    }
    None
}

fn parse_rst_indented_block(lines: &[&str], start_idx: usize) -> (String, usize) {
    let mut block_lines = Vec::new();
    let mut curr = start_idx;

    // Leerzeilen am Anfang überspringen
    while curr < lines.len() && lines.get(curr).copied().unwrap_or("").trim().is_empty() {
        curr = curr.saturating_add(1);
    }

    while curr < lines.len() {
        let line = lines.get(curr).copied().unwrap_or("");
        if line.trim().is_empty() {
            block_lines.push("");
            curr = curr.saturating_add(1);
            continue;
        }
        if line.starts_with("  ") || line.starts_with('\t') {
            block_lines.push(line.trim());
            curr = curr.saturating_add(1);
        } else {
            break;
        }
    }

    (block_lines.join("\n"), curr)
}

fn parse_rst_table(lines: &[&str], start_idx: usize) -> (Option<Table>, usize) {
    let mut rows = Vec::new();
    let mut curr = start_idx;

    while curr < lines.len() {
        let line = lines.get(curr).copied().unwrap_or("").trim();
        if line.is_empty() {
            break;
        }
        if line.starts_with('+') || line.starts_with('=') {
            curr = curr.saturating_add(1);
            continue;
        }
        if line.starts_with('|') {
            let cells: Vec<String> = line
                .split('|')
                .skip(1)
                .map(|c| c.trim().to_string())
                .collect();
            if !cells.is_empty() {
                rows.push(cells);
            }
        } else {
            let cells: Vec<String> = line.split_whitespace().map(|c| c.to_string()).collect();
            if !cells.is_empty() {
                rows.push(cells);
            }
        }
        curr = curr.saturating_add(1);
    }

    if rows.is_empty() {
        (None, start_idx.saturating_add(1))
    } else {
        (
            Some(Table {
                caption: None,
                header_rows: 1,
                rows,
                origin: (0, 0),
            }),
            curr,
        )
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rst_headings_and_code() {
        let rst = "Titel\n=====\n\nAbsatz.\n\nUnterüberschrift\n----------------\n\n.. code-block:: python\n\n  print('Hello')\n";
        let ir = parse_rst(rst);

        assert_eq!(ir.meta.title.as_deref(), Some("Titel"));
        assert!(ir
            .blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Heading { level: 1, .. })));
        assert!(ir
            .blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Heading { level: 2, .. })));
        assert!(ir.blocks.iter().any(|b| matches!(b.kind, BlockKind::Code { ref lang, .. } if lang.as_deref() == Some("python"))));
    }
}
