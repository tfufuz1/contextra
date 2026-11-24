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
// ZWECK: Markdown Extractor mit Raw-Identitätsinvariante und Titel-Extraktion (WP M4).
// INVARIANTEN: Ein BlockKind::Raw, kanonischer Text byte-identisch zum Eingabetext.
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator};

/// Extrahiert Markdown als einzelnen `BlockKind::Raw` Block.
pub fn parse_markdown(text: &str) -> DocumentIr {
    let mut ir = DocumentIr::new(FormatId::Markdown);

    ir.meta.title = extract_title(text);

    ir.push(
        BlockKind::Raw {
            text: text.to_string(),
        },
        Locator::default(),
    );

    ir
}

#[allow(clippy::indexing_slicing)]
fn extract_title(text: &str) -> Option<String> {
    // 1. YAML Front Matter title check
    if text.starts_with("---\n") || text.starts_with("---\r\n") {
        let rest = if let Some(r) = text.strip_prefix("---\r\n") {
            r
        } else {
            text.strip_prefix("---\n").unwrap_or(text)
        };

        if let Some(end_idx) = rest.find("\n---") {
            let front_matter = &rest[..end_idx];
            for line in front_matter.lines() {
                let trimmed = line.trim();
                if let Some(val) = trimmed.strip_prefix("title:") {
                    let title = val.trim().trim_matches('"').trim_matches('\'').trim();
                    if !title.is_empty() {
                        return Some(title.to_string());
                    }
                }
            }
        }
    }

    // 2. Erster ATX H1 (# Title)
    let lines: Vec<&str> = text.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(h1_text) = trimmed.strip_prefix("# ") {
            let title = h1_text.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }

        // 3. Setext H1 (Title followed by ===)
        if i.saturating_add(1) < lines.len() {
            let next_line = lines.get(i.saturating_add(1)).copied().unwrap_or("").trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && next_line.len() >= 3
                && next_line.chars().all(|c| c == '=')
            {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_frontmatter_title() {
        let md = "---\ntitle: \"Mein Dokument\"\nauthor: Jules\n---\n\n# Nicht verwendet\n\nInhalt";
        let ir = parse_markdown(md);
        assert_eq!(ir.meta.title.as_deref(), Some("Mein Dokument"));
        assert_eq!(ir.blocks.len(), 1);
        if let Some(BlockKind::Raw { text }) = ir.blocks.first().map(|b| &b.kind) {
            assert_eq!(text, md);
        } else {
            panic!("Expected Raw block");
        }
    }

    #[test]
    fn test_parse_markdown_atx_title() {
        let md = "# Titelzeile\n\nErster Absatz.";
        let ir = parse_markdown(md);
        assert_eq!(ir.meta.title.as_deref(), Some("Titelzeile"));
    }
}
