// FILE-CONTEXT
// ZWECK: Zwischenrepräsentation (DocumentIr) für extrahierte Dokumente (WP M2).
// INVARIANTEN: Flache Blockliste, Anhang-Einbettung, keine std::collections::{HashMap,HashSet} (BTreeMap/Vec).
// STAND: TS:2026-10-31T00:00:00Z

use crate::format::FormatId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Zwischenrepräsentation eines extrahierten Dokuments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DocumentIr {
    pub format: FormatId,
    pub meta: DocMeta,
    pub blocks: Vec<Block>,
    pub warnings: Vec<ExtractWarning>,
}

/// Metadaten des Dokuments aus der Quelle (ISO-8601 Strings, niemals Systemzeit).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DocMeta {
    pub title: Option<String>,
    pub language: Option<String>,
    pub authors: Vec<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub page_count: Option<u32>,
    pub extra: BTreeMap<String, String>,
}

/// Ein struktureller Inhaltsblock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Block {
    pub kind: BlockKind,
    pub locator: Locator,
}

/// Arten von Inhaltsblöcken in der Zwischenrepräsentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BlockKind {
    Heading {
        level: u8,
        text: String,
    },
    Paragraph {
        text: String,
    },
    ListItem {
        depth: u8,
        ordered: bool,
        text: String,
    },
    Table(Table),
    Code {
        lang: Option<String>,
        text: String,
    },
    Quote {
        text: String,
    },
    Caption {
        text: String,
    },
    Note {
        kind: NoteKind,
        text: String,
    },
    Boundary(Boundary),
    Record {
        path: String,
        value: String,
    },
    Message(MessageHeader),
    Raw {
        text: String,
    },
}

/// Tabellenstruktur in der Zwischenrepräsentation.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Table {
    pub caption: Option<String>,
    pub header_rows: u8,
    pub rows: Vec<Vec<String>>,
    pub origin: (u32, u32),
}

/// Arten von Anmerkungen/Notizen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NoteKind {
    Footnote,
    Endnote,
    Comment,
    SpeakerNotes,
    Header,
    Footer,
}

/// Strukturelle Grenzen innerhalb von Dokumenten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Boundary {
    Page { number: u32 },
    Slide { number: u32, title: Option<String> },
    Sheet { name: String, index: u32 },
    Section { title: Option<String> },
    Attachment { name: String, mime: Option<String> },
}

/// Header-Daten von E-Mail-Nachrichten.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MessageHeader {
    pub subject: Option<String>,
    pub from: Vec<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub date: Option<String>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
}

/// Lokationsangabe eines Blocks im Quelldokument.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Locator {
    pub page: Option<u32>,
    pub page_end: Option<u32>,
    pub slide: Option<u32>,
    pub slide_end: Option<u32>,
    pub sheet: Option<String>,
    pub range: Option<String>,
    pub message: Option<u32>,
    pub path: Option<String>,
    pub attachment: Option<String>,
}

/// Warnungen, die während der Extraktion auftreten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExtractWarning {
    pub code: WarningCode,
    pub detail: String,
}

/// Klassifikation von Extraktionswarnungen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WarningCode {
    HiddenContentSkipped,
    TextSanitized,
    EncodingGuessed,
    AttachmentSkipped,
    EmbeddedObjectSkipped,
    FeatureLimited,
}

impl DocumentIr {
    /// Erstellt eine neue leere `DocumentIr` für ein gegebenes Format.
    pub fn new(format: FormatId) -> Self {
        Self {
            format,
            meta: DocMeta::default(),
            blocks: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Fügt einen neuen Block mit gegebenem Locator hinzu.
    pub fn push(&mut self, kind: BlockKind, locator: Locator) {
        self.blocks.push(Block { kind, locator });
    }

    /// Bettet ein verschachteltes Dokument (z. B. Anhang) flach ein.
    pub fn append_nested(&mut self, name: &str, mime: Option<&str>, mut nested: DocumentIr) {
        // Boundary-Marker für Anhang einfügen
        self.push(
            BlockKind::Boundary(Boundary::Attachment {
                name: name.to_string(),
                mime: mime.map(|s| s.to_string()),
            }),
            Locator::default(),
        );

        // Blöcke übernehmen und attachment-Locator setzen
        for mut block in nested.blocks {
            block.locator.attachment = Some(name.to_string());
            self.blocks.push(block);
        }

        // Warnungen übernehmen
        self.warnings.append(&mut nested.warnings);
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]
    use super::*;

    #[test]
    fn test_document_ir_append_nested() {
        let mut parent = DocumentIr::new(FormatId::Eml);
        parent.push(
            BlockKind::Paragraph {
                text: "Parent body".into(),
            },
            Locator::default(),
        );

        let mut child = DocumentIr::new(FormatId::Txt);
        child.push(
            BlockKind::Paragraph {
                text: "Attachment body".into(),
            },
            Locator::default(),
        );

        parent.append_nested("notes.txt", Some("text/plain"), child);

        assert_eq!(parent.blocks.len(), 3);
        assert!(matches!(
            parent.blocks[1].kind,
            BlockKind::Boundary(Boundary::Attachment { .. })
        ));
        assert_eq!(
            parent.blocks[2].locator.attachment.as_deref(),
            Some("notes.txt")
        );
    }
}
