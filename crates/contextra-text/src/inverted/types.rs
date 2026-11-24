//! Typdefinitionen und Konfiguration für den invertierten Index.

use serde::{Deserialize, Serialize};

// ANCHOR[contextra-text:INV-001]
/// Unterstützte Sprachen und Tokenisierungsstrategien für den invertierten Index.
///
/// Steuert, welcher Tokenizer für die Dokumentenindizierung und Suchabfragen verwendet wird.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Language {
    /// Englischer Standard-Tokenizer (Unicode-Wortgrenzen + Stoppwortfilterung).
    #[default]
    English,
    /// Deutscher morphologischer Tokenizer (Komposita-Zerlegung + Umlaut-Normalisierung).
    German,
    /// Benutzerdefinierter Tokenizer — wird über `with_tokenizer()` injiziert.
    Custom,
}

impl Language {
    // ANCHOR[contextra-text:INV-002]
    /// Parst einen ISO-639-1 Sprachcode ("de", "de-AT", "en", "en-US").
    ///
    /// Bewertet nur das primäre Subtag vor dem ersten Bindestrich.
    /// Unbekannte Codes fallen sicher auf `Language::English` zurück.
    pub fn from_iso(code: &str) -> Self {
        let prefix = code.split('-').next().unwrap_or("").to_lowercase();
        match prefix.as_str() {
            "de" => Self::German,
            "en" => Self::English,
            _ => Self::English,
        }
    }
}

// ANCHOR[contextra-text:INV-003]
/// Konsolidierte Metadaten des Textindex zur persistenten Speicherung im Schlüssel `meta:stats`.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TextIndexMetadata {
    /// Gesamtzahl der aktiven Dokumente im Index.
    pub total_docs: u64,
    /// Gesamtzahl aller indizierten Token über alle Dokumente.
    pub total_tokens: u64,
    /// Festkomma-Repräsentation der durchschnittlichen Dokumentlänge multipliziert mit 1000.
    pub avg_doc_len_x1000: u64, // Fixed-point for BM25 caching (FIND-TXT-004)
}

// ANCHOR[contextra-text:INV-004]
/// Gepufferte statistische Änderungen einer uncommitted Transaktion.
#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct StagedStatsChange {
    pub(crate) docs_delta: i64,
    pub(crate) tokens_delta: i64,
}
