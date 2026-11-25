//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// FILE-CONTEXT
// ZWECK: AsciiDoc Extractor (WP M4).
// INVARIANTEN: Zero-Panic, '=' Überschriften, '----'/'....' Code-Blöcke, '|===' Tabellen.
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator, Table};

/// Extrahiert AsciiDoc Dokumente.
#[allow(clippy::indexing_slicing)]
pub fn parse_asciidoc(text: &str) -> DocumentIr {
    let mut ir = DocumentIr::new(FormatId::AsciiDoc);

    let lines: Vec<&str> = text.lines().collect();
    let mut idx = 0usize;
    let mut pending_lang: Option<String> = None;

    while idx < lines.len() {
        let line = lines.get(idx).copied().unwrap_or("").trim();

        if line.is_empty() {
            idx = idx.saturating_add(1);
            continue;
        }

        // 1. Source attribute `[source,lang]`
        if line.starts_with("[source") && line.ends_with(']') {
            let inner = &line[1..line.len().saturating_sub(1)];
            if let Some(lang) = inner.split(',').nth(1) {
                pending_lang = Some(lang.trim().to_string());
            }
            idx = idx.saturating_add(1);
            continue;
        }

        // 2. Listing block `----` or Literal block `....`
        if line == "----" || line == "...." {
            let delimiter = line;
            let mut code_lines = Vec::new();
            idx = idx.saturating_add(1);

            while idx < lines.len() {
                let l = lines.get(idx).copied().unwrap_or("");
                if l.trim() == delimiter {
                    idx = idx.saturating_add(1);
                    break;
                }
                code_lines.push(l);
                idx = idx.saturating_add(1);
            }

            ir.push(
                BlockKind::Code {
                    lang: pending_lang.take(),
                    text: code_lines.join("\n"),
                },
                Locator::default(),
            );
            continue;
        }

        // 3. Table `|===`
        if line == "|===" {
            let mut table_rows = Vec::new();
            idx = idx.saturating_add(1);

            while idx < lines.len() {
                let l = lines.get(idx).copied().unwrap_or("").trim();
                if l == "|===" {
                    idx = idx.saturating_add(1);
                    break;
                }
                if l.starts_with('|') {
                    let cells: Vec<String> =
                        l.split('|').skip(1).map(|c| c.trim().to_string()).collect();
                    if !cells.is_empty() {
                        table_rows.push(cells);
                    }
                }
                idx = idx.saturating_add(1);
            }

            if !table_rows.is_empty() {
                ir.push(
                    BlockKind::Table(Table {
                        caption: None,
                        header_rows: 1,
                        rows: table_rows,
                        origin: (0, 0),
                    }),
                    Locator::default(),
                );
            }
            continue;
        }

        // 4. Headings (`=` bis `======`)
        if line.starts_with('=') {
            let eq_count = line.chars().take_while(|&c| c == '=').count();
            if (1..=6).contains(&eq_count) {
                let title = line[eq_count..].trim().to_string();
                #[allow(clippy::cast_possible_truncation)]
                let level = eq_count as u8;

                if ir.meta.title.is_none() && level == 1 {
                    ir.meta.title = Some(title.clone());
                }

                ir.push(
                    BlockKind::Heading { level, text: title },
                    Locator::default(),
                );
                idx = idx.saturating_add(1);
                continue;
            }
        }

        // 5. Admonitions (`NOTE:`, `WARNING:`, `TIP:`, `IMPORTANT:`, `CAUTION:`)
        if let Some(rest) = strip_admonition(line) {
            ir.push(
                BlockKind::Quote {
                    text: rest.to_string(),
                },
                Locator::default(),
            );
            idx = idx.saturating_add(1);
            continue;
        }

        // 6. Lists
        if line.starts_with("* ") || line.starts_with("** ") || line.starts_with("- ") {
            let text = line
                .trim_start_matches('*')
                .trim_start_matches('-')
                .trim()
                .to_string();
            ir.push(
                BlockKind::ListItem {
                    depth: 0,
                    ordered: false,
                    text,
                },
                Locator::default(),
            );
            idx = idx.saturating_add(1);
            continue;
        }
        if line.starts_with(". ") || line.starts_with(".. ") {
            let text = line.trim_start_matches('.').trim().to_string();
            ir.push(
                BlockKind::ListItem {
                    depth: 0,
                    ordered: true,
                    text,
                },
                Locator::default(),
            );
            idx = idx.saturating_add(1);
            continue;
        }

        // 7. Standard Paragraph
        ir.push(
            BlockKind::Paragraph {
                text: line.to_string(),
            },
            Locator::default(),
        );
        idx = idx.saturating_add(1);
    }

    ir
}

fn strip_admonition(line: &str) -> Option<&str> {
    for prefix in &["NOTE:", "WARNING:", "TIP:", "IMPORTANT:", "CAUTION:"] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some(rest.trim());
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asciidoc() {
        let adoc = "= Document Title\n\n== Section 1\n\nNOTE: Important notice.\n\n[source,rust]\n----\nfn main() {}\n----\n";
        let ir = parse_asciidoc(adoc);

        assert_eq!(ir.meta.title.as_deref(), Some("Document Title"));
        assert!(ir
            .blocks
            .iter()
            .any(|b| matches!(b.kind, BlockKind::Quote { .. })));
        assert!(ir.blocks.iter().any(|b| matches!(b.kind, BlockKind::Code { ref lang, .. } if lang.as_deref() == Some("rust"))));
    }
}
