//! # Modul: blocks.rs
//!
//! **Zweck:** Blockbasierte Segmentierung mit semantischer Grenzerkennung.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, deterministische Textverarbeitung.

// FILE-CONTEXT
// ZWECK: Zerlegung roher Abschnitte in atomare Markdown-Blöcke (Code, Tabelle, Absatz).
// INVARIANTEN: Erhält Codeblöcke (``` und ~~~ mit Längenregeln) und Tabellen als strukturierte Einheiten.
// NICHT-OFFENSICHTLICH: Isoliert Tabellenköpfe von Datenzeilen für Schemaerhalt. Indexfrei & Panikfrei.
// STAND: TS:2026-10-29T00:00:00Z

use super::estimate::estimate_tokens;

#[derive(Debug, Clone, PartialEq, Eq)]
// ANCHOR[contextra-engine:BLK-001]
/// Repräsentiert einen Code-Block-Begrenzer.
pub(crate) struct CodeFence {
    pub(crate) char_type: char,
    pub(crate) length: usize,
}

// ANCHOR[contextra-engine:BLK-002]
/// Parst eine öffnende Code-Fence-Zeile.
pub(crate) fn parse_opening_fence(line: &str) -> Option<CodeFence> {
    let trimmed = line.trim_start();
    if line.len().saturating_sub(trimmed.len()) > 3 {
        return None;
    }
    if trimmed.starts_with("```") {
        let count = trimmed.chars().take_while(|&c| c == '`').count();
        Some(CodeFence {
            char_type: '`',
            length: count,
        })
    } else if trimmed.starts_with("~~~") {
        let count = trimmed.chars().take_while(|&c| c == '~').count();
        Some(CodeFence {
            char_type: '~',
            length: count,
        })
    } else {
        None
    }
}

// ANCHOR[contextra-engine:BLK-003]
/// Prüft, ob eine Zeile den Code-Block schließt.
pub(crate) fn is_closing_fence(line: &str, active_fence: &CodeFence) -> bool {
    let trimmed = line.trim();
    let count = trimmed
        .chars()
        .take_while(|&c| c == active_fence.char_type)
        .count();
    count >= active_fence.length && count == trimmed.chars().count()
}

#[derive(Debug, Clone)]
// ANCHOR[contextra-engine:BLK-004]
/// Strukturierte Markdown-Blöcke.
pub(crate) enum MarkdownBlock {
    CodeBlock {
        fence: String,
        body_lines: Vec<String>,
        fence_info: CodeFence,
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
    // ANCHOR[contextra-engine:BLK-005]
    /// Konvertiert den Block zurück in Text.
    pub(crate) fn to_text(&self) -> String {
        match self {
            MarkdownBlock::CodeBlock {
                fence,
                body_lines,
                fence_info,
            } => {
                let mut s = fence.clone();
                s.push('\n');
                s.push_str(&body_lines.join("\n"));
                if !s.ends_with('\n') {
                    s.push('\n');
                }
                let closing = std::iter::repeat(fence_info.char_type)
                    .take(fence_info.length)
                    .collect::<String>();
                s.push_str(&closing);
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

    // ANCHOR[contextra-engine:BLK-006]
    /// Schätzt die Token-Anzahl des Blocks ab.
    pub(crate) fn estimate_tokens(&self) -> usize {
        estimate_tokens(&self.to_text())
    }
}

// ANCHOR[contextra-engine:BLK-007]
/// Zerlegt den Text in strukturierte Markdown-Blöcke.
pub(crate) fn parse_markdown_blocks(lines: &[String]) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let Some(line) = lines.get(i) else {
            break;
        };

        if let Some(fence_info) = parse_opening_fence(line) {
            let fence = line.clone();
            let mut body_lines = Vec::new();
            i = i.saturating_add(1);
            while i < lines.len() {
                let Some(cur_line) = lines.get(i) else {
                    break;
                };
                if is_closing_fence(cur_line, &fence_info) {
                    i = i.saturating_add(1);
                    break;
                }
                body_lines.push(cur_line.clone());
                i = i.saturating_add(1);
            }
            blocks.push(MarkdownBlock::CodeBlock {
                fence,
                body_lines,
                fence_info,
            });
        } else if is_table_row(line) {
            let mut table_lines = Vec::new();
            while i < lines.len() {
                let Some(cur_line) = lines.get(i) else {
                    break;
                };
                if !is_table_row(cur_line) {
                    break;
                }
                table_lines.push(cur_line.clone());
                i = i.saturating_add(1);
            }

            let first_row = table_lines.get(0);
            let second_row = table_lines.get(1);

            let (header_lines, body_lines) = if table_lines.len() >= 2
                && second_row.map_or(false, |r| is_table_delimiter_row(r))
            {
                (
                    table_lines.get(..2).map(|s| s.to_vec()).unwrap_or_default(),
                    table_lines.get(2..).map(|s| s.to_vec()).unwrap_or_default(),
                )
            } else if !table_lines.is_empty()
                && first_row.map_or(false, |r| is_table_delimiter_row(r))
            {
                (
                    table_lines.get(..1).map(|s| s.to_vec()).unwrap_or_default(),
                    table_lines.get(1..).map(|s| s.to_vec()).unwrap_or_default(),
                )
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
                let Some(cur) = lines.get(i) else {
                    break;
                };
                if parse_opening_fence(cur).is_some() || is_table_row(cur) {
                    break;
                }
                let cur_trim = cur.trim();
                if cur_trim.is_empty() && !para_lines.is_empty() {
                    i = i.saturating_add(1);
                    break;
                }
                if !cur_trim.is_empty() || !para_lines.is_empty() {
                    para_lines.push(cur.clone());
                }
                i = i.saturating_add(1);
            }
            if !para_lines.is_empty() {
                blocks.push(MarkdownBlock::Paragraph { lines: para_lines });
            }
        }
    }

    blocks
}

// ANCHOR[contextra-engine:BLK-008]
/// Prüft, ob eine Zeile eine Tabellenzeile ist.
pub(crate) fn is_table_row(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return false;
    }
    t.starts_with('|') || (t.ends_with('|') && t.contains('|'))
}

// ANCHOR[contextra-engine:BLK-009]
/// Prüft, ob eine Zeile eine Tabellen-Trennzeile ist.
pub(crate) fn is_table_delimiter_row(line: &str) -> bool {
    let t = line.trim();
    t.contains("---") || t.contains("-|-")
}
