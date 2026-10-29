// FILE-CONTEXT
// ZWECK: Text-Bereinigung (Sanitization) und Kanonische Text-Repräsentation aus DocumentIr (WP M2).
// INVARIANTEN: Zero-Panic, deterministisch, kontextfreies Block-Rendering, BLAKE3-Hex-Hashing.
// STAND: TS:2026-10-31T00:00:00Z

use crate::ir::{BlockKind, Boundary, DocumentIr, NoteKind};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::ops::Range;

/// Erzeugter kanonischer Text mit exakten Byte-Spans je Block in der `DocumentIr`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CanonicalText {
    pub text: String,
    pub block_spans: Vec<Range<usize>>,
}

/// Entfernt NUL und C0-Steuerzeichen (außer \t und \n), U+FEFF (BOM),
/// Unicode-Tag-Zeichen U+E0000..U+E007F und Bidi-Overrides U+202A..U+202E / U+2066..U+2069.
/// Normalisiert \r\n und \r zu \n. Behält ZWJ/ZWNJ.
/// Liefert gereinigten String und die Anzahl entfernter/normalisierter Zeichen.
pub fn sanitize_text(input: &str) -> (String, u32) {
    let mut clean = String::with_capacity(input.len());
    let mut sanitizations = 0u32;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Zeilenumbruch-Normalisierung \r\n oder \r -> \n
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next(); // \n überspringen
                }
                clean.push('\n');
                sanitizations += 1;
            }
            // Erlaubte Steuerzeichen \n und \t
            '\n' | '\t' => {
                clean.push(c);
            }
            // BOM (U+FEFF)
            '\u{FEFF}' => {
                sanitizations += 1;
            }
            // Unicode Tag-Zeichen U+E0000..U+E007F
            '\u{E0000}'..='\u{E007F}' => {
                sanitizations += 1;
            }
            // Bidi-Overrides U+202A..U+202E und U+2066..U+2069
            '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}' => {
                sanitizations += 1;
            }
            // C0-Steuerzeichen NUL..US (0x00..0x1F) ohne \t (0x09) und \n (0x0A)
            '\u{0000}'..='\u{0008}' | '\u{000B}'..='\u{001F}' | '\u{007F}' => {
                sanitizations += 1;
            }
            // Alle anderen Zeichen (inkl. ZWJ U+200D / ZWNJ U+200C)
            _ => {
                clean.push(c);
            }
        }
    }

    (clean, sanitizations)
}

impl DocumentIr {
    /// Rendering der `DocumentIr` in ein `CanonicalText` Objekt.
    /// Blöcke werden durch "\n\n" getrennt (Trenner gehören zu keinem Block-Span).
    pub fn canonical(&self) -> CanonicalText {
        let mut text = String::new();
        let mut block_spans = Vec::with_capacity(self.blocks.len());

        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                text.push_str("\n\n");
            }

            let start = text.len();
            render_block(&block.kind, &mut text);
            let end = text.len();

            block_spans.push(start..end);
        }

        CanonicalText { text, block_spans }
    }

    /// Erzeugt den BLAKE3-Hex-Hash des kanonischen Textes.
    pub fn canonical_hash(&self) -> String {
        let canon = self.canonical();
        let mut hasher = Hasher::new();
        hasher.update(canon.text.as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

fn render_block(kind: &BlockKind, out: &mut String) {
    match kind {
        BlockKind::Heading { level, text } => {
            let (clean_text, _) = sanitize_text(text);
            let hashes = "#".repeat((*level as usize).clamp(1, 6));
            out.push_str(&hashes);
            out.push(' ');
            out.push_str(&clean_text);
        }
        BlockKind::Paragraph { text } => {
            let (clean_text, _) = sanitize_text(text);
            out.push_str(&clean_text);
        }
        BlockKind::ListItem {
            depth,
            ordered,
            text,
        } => {
            let (clean_text, _) = sanitize_text(text);
            let indent = "  ".repeat(*depth as usize);
            let prefix = if *ordered { "1. " } else { "- " };
            out.push_str(&indent);
            out.push_str(prefix);
            out.push_str(&clean_text);
        }
        BlockKind::Table(table) => {
            if let Some(ref cap) = table.caption {
                let (clean_cap, _) = sanitize_text(cap);
                out.push_str("Table: ");
                out.push_str(&clean_cap);
                out.push('\n');
            }
            if table.rows.is_empty() {
                return;
            }
            // GFM-Pipetabelle
            for (row_idx, row) in table.rows.iter().enumerate() {
                out.push('|');
                for cell in row {
                    out.push(' ');
                    let (clean_cell, _) = sanitize_text(cell);
                    let escaped = clean_cell.replace('|', "\\|").replace('\n', " ");
                    out.push_str(&escaped);
                    out.push_str(" |");
                }
                out.push('\n');

                // Nach Kopfzeile Trennzeile einfügen
                if row_idx == 0 {
                    out.push('|');
                    for _ in row {
                        out.push_str("---|");
                    }
                    out.push('\n');
                }
            }
            // Letzten Zeilenumbruch entfernen falls vorhanden
            if out.ends_with('\n') {
                out.pop();
            }
        }
        BlockKind::Code { lang, text } => {
            let (clean_text, _) = sanitize_text(text);
            out.push_str("```");
            if let Some(l) = lang {
                out.push_str(l.trim());
            }
            out.push('\n');
            out.push_str(&clean_text);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("```");
        }
        BlockKind::Quote { text } => {
            let (clean_text, _) = sanitize_text(text);
            for line in clean_text.lines() {
                out.push_str("> ");
                out.push_str(line);
                out.push('\n');
            }
            if out.ends_with('\n') {
                out.pop();
            }
        }
        BlockKind::Caption { text } => {
            let (clean_text, _) = sanitize_text(text);
            out.push_str(&clean_text);
        }
        BlockKind::Note { kind, text } => {
            let (clean_text, _) = sanitize_text(text);
            let tag = match kind {
                NoteKind::Footnote => "footnote",
                NoteKind::Endnote => "endnote",
                NoteKind::Comment => "comment",
                NoteKind::SpeakerNotes => "speaker_notes",
                NoteKind::Header => "header",
                NoteKind::Footer => "footer",
            };
            out.push('[');
            out.push_str(tag);
            out.push_str("] ");
            out.push_str(&clean_text);
        }
        BlockKind::Boundary(boundary) => match boundary {
            Boundary::Page { number } => {
                out.push_str(&format!("[page {}]", number));
            }
            Boundary::Slide { number, title } => {
                if let Some(t) = title {
                    let (clean_title, _) = sanitize_text(t);
                    out.push_str(&format!("[slide {}: {}]", number, clean_title));
                } else {
                    out.push_str(&format!("[slide {}]", number));
                }
            }
            Boundary::Sheet { name, .. } => {
                let (clean_name, _) = sanitize_text(name);
                out.push_str(&format!("[sheet {}]", clean_name));
            }
            Boundary::Section { title } => {
                if let Some(t) = title {
                    let (clean_title, _) = sanitize_text(t);
                    out.push_str(&format!("[section {}]", clean_title));
                }
            }
            Boundary::Attachment { name, .. } => {
                let (clean_name, _) = sanitize_text(name);
                out.push_str(&format!("[attachment {}]", clean_name));
            }
        },
        BlockKind::Record { path, value } => {
            let (clean_path, _) = sanitize_text(path);
            let (clean_val, _) = sanitize_text(value);
            out.push_str(&clean_path);
            out.push_str(": ");
            out.push_str(&clean_val);
        }
        BlockKind::Message(msg) => {
            let mut fields = Vec::new();
            if !msg.from.is_empty() {
                fields.push(format!("From: {}", msg.from.join(", ")));
            }
            if !msg.to.is_empty() {
                fields.push(format!("To: {}", msg.to.join(", ")));
            }
            if !msg.cc.is_empty() {
                fields.push(format!("Cc: {}", msg.cc.join(", ")));
            }
            if let Some(ref d) = msg.date {
                fields.push(format!("Date: {}", d));
            }
            if let Some(ref s) = msg.subject {
                fields.push(format!("Subject: {}", s));
            }
            let text = fields.join("\n");
            let (clean_text, _) = sanitize_text(&text);
            out.push_str(&clean_text);
        }
        BlockKind::Raw { text } => {
            // Raw = verbatim durchgereichter Text: kanonischer Text == Originaltext.
            out.push_str(text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FormatId;
    use crate::ir::Locator;

    #[test]
    fn test_sanitize_text() {
        let input = "Hello\r\nWorld\r\u{FEFF}Test\u{0000}\tDone";
        let (clean, count) = sanitize_text(input);
        assert_eq!(clean, "Hello\nWorld\nTest\tDone");
        assert_eq!(count, 4);
    }

    #[test]
    fn test_canonical_block_spans_invariant() {
        let mut ir = DocumentIr::new(FormatId::Markdown);
        ir.push(
            BlockKind::Heading {
                level: 1,
                text: "Title".into(),
            },
            Locator::default(),
        );
        ir.push(
            BlockKind::Paragraph {
                text: "Body paragraph".into(),
            },
            Locator::default(),
        );

        let canon = ir.canonical();
        assert_eq!(canon.block_spans.len(), 2);

        let s0 = &canon.text[canon.block_spans[0].clone()];
        let s1 = &canon.text[canon.block_spans[1].clone()];

        assert_eq!(s0, "# Title");
        assert_eq!(s1, "Body paragraph");
        assert_eq!(canon.text, "# Title\n\nBody paragraph");
    }
}
