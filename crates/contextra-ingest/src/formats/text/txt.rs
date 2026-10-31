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
// ZWECK: Plaintext Extractor (Txt und Log-Dateien) (WP M4).
// INVARIANTEN: Zero-Panic, Absätze an Leerzeilen => Paragraph (kein Raw).
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator};

/// Extrahiert Plaintext/Log-Dateien in Absätze an Leerzeilen.
pub fn parse_txt(text: &str) -> DocumentIr {
    let mut ir = DocumentIr::new(FormatId::Txt);

    let mut current_paragraph = Vec::new();

    for line in text.lines() {
        if line.trim().is_empty() {
            if !current_paragraph.is_empty() {
                let paragraph_text = current_paragraph.join("\n");
                ir.push(
                    BlockKind::Paragraph {
                        text: paragraph_text,
                    },
                    Locator::default(),
                );
                current_paragraph.clear();
            }
        } else {
            current_paragraph.push(line);
        }
    }

    if !current_paragraph.is_empty() {
        let paragraph_text = current_paragraph.join("\n");
        ir.push(
            BlockKind::Paragraph {
                text: paragraph_text,
            },
            Locator::default(),
        );
    }

    ir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_txt_paragraphs() {
        let text = "First line\nSecond line\n\nSecond paragraph\n# Log line";
        let ir = parse_txt(text);

        assert_eq!(ir.blocks.len(), 2);
        assert_eq!(
            ir.blocks.first().map(|b| &b.kind),
            Some(&BlockKind::Paragraph {
                text: "First line\nSecond line".into()
            })
        );
        assert_eq!(
            ir.blocks.get(1).map(|b| &b.kind),
            Some(&BlockKind::Paragraph {
                text: "Second paragraph\n# Log line".into()
            })
        );
    }
}
