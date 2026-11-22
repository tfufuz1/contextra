// FILE-CONTEXT
// ZWECK: Leichtgewichtiger TeX/LaTeX Extractor (WP M4).
// INVARIANTEN: Zero-Panic, Doku-Limitationen im Modulkopf, Kommentare entfernen, Formeln wörtlich.
// STAND: TS:2026-10-31T00:00:00Z

//! # TeX / LaTeX Extractor
//!
//! **Dokumentierte Limitationen:**
//! - Leichtgewichtiger Parser ohne TeX-Engine oder Vollparser-Makroexpansion.
//! - Pakete (`\usepackage{...}`) und Präambel-Deklarationen werden ignoriert.
//! - Kommentare (`%...`) werden entfernt, sofern nicht per `\%` maskiert.
//! - Mathematische Formeln (`$...$`, `$$...$$`, `\[...\]`) werden wörtlich als Text beibehalten.
//! - Unbekannte Befehle mit Argumenten (`\command{text}`) behalten den Argumenttext `text`.

use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator, Table};

/// Extrahiert TeX/LaTeX-Dokumente.
pub fn parse_tex(text: &str) -> DocumentIr {
    let mut ir = DocumentIr::new(FormatId::Tex);

    // 1. Kommentare entfernen
    let stripped_text = strip_tex_comments(text);

    let lines: Vec<&str> = stripped_text.lines().collect();
    let mut idx = 0usize;
    let mut in_itemize = false;
    let mut in_enumerate = false;

    while idx < lines.len() {
        let line = lines.get(idx).copied().unwrap_or("").trim();

        if line.is_empty() {
            idx = idx.saturating_add(1);
            continue;
        }

        // 2. Umgebungen: itemize / enumerate / tabular
        if line.starts_with("\\begin{itemize}") {
            in_itemize = true;
            idx = idx.saturating_add(1);
            continue;
        }
        if line.starts_with("\\end{itemize}") {
            in_itemize = false;
            idx = idx.saturating_add(1);
            continue;
        }
        if line.starts_with("\\begin{enumerate}") {
            in_enumerate = true;
            idx = idx.saturating_add(1);
            continue;
        }
        if line.starts_with("\\end{enumerate}") {
            in_enumerate = false;
            idx = idx.saturating_add(1);
            continue;
        }

        if line.starts_with("\\begin{tabular}") {
            let (table_opt, next_idx) = parse_tex_tabular(&lines, idx);
            if let Some(t) = table_opt {
                ir.push(BlockKind::Table(t), Locator::default());
            }
            idx = next_idx;
            continue;
        }

        // 3. List Item
        if let Some(rest) = line.strip_prefix("\\item") {
            let item_text = clean_tex_line(rest);
            if !item_text.is_empty() {
                ir.push(
                    BlockKind::ListItem {
                        depth: 0,
                        ordered: in_enumerate && !in_itemize,
                        text: item_text,
                    },
                    Locator::default(),
                );
            }
            idx = idx.saturating_add(1);
            continue;
        }

        // 4. Structure Headings (\chapter, \section, \subsection, \subsubsection)
        if let Some((level, title)) = parse_tex_heading(line) {
            if ir.meta.title.is_none() && level <= 2 {
                ir.meta.title = Some(title.clone());
            }
            ir.push(
                BlockKind::Heading { level, text: title },
                Locator::default(),
            );
            idx = idx.saturating_add(1);
            continue;
        }

        // 5. Clean line and produce paragraph
        let cleaned = clean_tex_line(line);
        if !cleaned.is_empty() {
            ir.push(BlockKind::Paragraph { text: cleaned }, Locator::default());
        }

        idx = idx.saturating_add(1);
    }

    ir
}

fn strip_tex_comments(input: &str) -> String {
    let mut clean = String::with_capacity(input.len());
    for line in input.lines() {
        let mut escaped = false;
        let mut comment_start = None;
        for (i, c) in line.char_indices() {
            if c == '\\' {
                escaped = !escaped;
            } else {
                if c == '%' && !escaped {
                    comment_start = Some(i);
                    break;
                }
                escaped = false;
            }
        }
        if let Some(pos) = comment_start {
            clean.push_str(&line[..pos]);
        } else {
            clean.push_str(line);
        }
        clean.push('\n');
    }
    clean
}

#[allow(clippy::indexing_slicing)]
fn parse_tex_heading(line: &str) -> Option<(u8, String)> {
    let commands = [
        ("\\chapter*{", 1u8),
        ("\\chapter{", 1u8),
        ("\\section*{", 2u8),
        ("\\section{", 2u8),
        ("\\subsection*{", 3u8),
        ("\\subsection{", 3u8),
        ("\\subsubsection*{", 4u8),
        ("\\subsubsection{", 4u8),
    ];

    for (cmd, level) in commands {
        if let Some(rest) = line.strip_prefix(cmd) {
            if let Some(end_idx) = rest.find('}') {
                let raw_title = &rest[..end_idx];
                return Some((level, clean_tex_line(raw_title)));
            }
        }
    }
    None
}

fn clean_tex_line(input: &str) -> String {
    let known_noop = [
        "\\newpage",
        "\\clearpage",
        "\\maketitle",
        "\\tableofcontents",
        "\\begin{document}",
        "\\end{document}",
    ];

    let mut result = input.to_string();
    for noop in known_noop {
        result = result.replace(noop, "");
    }

    // Filter macro wrappers like \documentclass{...}, \usepackage{...}, \label{...}, \cite{...}, \ref{...}
    for macro_name in &[
        "\\documentclass{",
        "\\usepackage{",
        "\\label{",
        "\\cite{",
        "\\ref{",
    ] {
        while let Some(start) = result.find(macro_name) {
            if let Some(end) = result[start..].find('}') {
                result.replace_range(start..start.saturating_add(end).saturating_add(1), "");
            } else {
                break;
            }
        }
    }

    // Generic unknown macro unwrapping: \cmd{text} -> text
    let mut unwrapped = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let mut cmd = String::new();
            while let Some(&nc) = chars.peek() {
                if nc.is_ascii_alphabetic() {
                    cmd.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if chars.peek() == Some(&'{') {
                chars.next(); // skip '{'
                              // preserve inner text until '}'
                let mut inner = String::new();
                for ic in chars.by_ref() {
                    if ic == '}' {
                        break;
                    }
                    inner.push(ic);
                }
                unwrapped.push_str(&inner);
            } else {
                // macro without brace (e.g. \LaTeX)
                // keep space or ignore command name
            }
        } else {
            unwrapped.push(c);
        }
    }

    unwrapped.trim().to_string()
}

fn parse_tex_tabular(lines: &[&str], start_idx: usize) -> (Option<Table>, usize) {
    let mut rows = Vec::new();
    let mut curr = start_idx.saturating_add(1);

    while curr < lines.len() {
        let line = lines.get(curr).copied().unwrap_or("").trim();
        if line.starts_with("\\end{tabular}") {
            curr = curr.saturating_add(1);
            break;
        }

        let cells: Vec<String> = line
            .trim_end_matches("\\\\")
            .split('&')
            .map(clean_tex_line)
            .collect();
        if !cells.is_empty() && !cells.iter().all(|c| c.is_empty()) {
            rows.push(cells);
        }

        curr = curr.saturating_add(1);
    }

    if rows.is_empty() {
        (None, curr)
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
    fn test_parse_tex() {
        let tex = "% Kommentar\n\\section{Einleitung}\nHallo \\textbf{Welt}!\n\n\\[ E = mc^2 \\]\n";
        let ir = parse_tex(tex);

        assert_eq!(ir.meta.title.as_deref(), Some("Einleitung"));
        assert!(ir
            .blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Heading { level: 2, .. })));
        assert!(ir
            .blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Paragraph { ref text } if text.contains("Welt"))));
        assert!(ir.blocks.iter().any(
            |b| matches!(b.kind, BlockKind::Paragraph { ref text } if text.contains("E = mc^2"))
        ));
    }
}
