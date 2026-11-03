//! Textanalysatoren für Volltextindexierung (SPEC D-81, TX-07).
//!
//! Unterstützt:
//! - `de/1` (`ANALYZER_DE_1` = 1): UAX #29, NFKC/Umlaut/`ß->ss`-Faltung, Stoppwörter, Kompositazerlegung.
//! - `und/1` (`ANALYZER_UND_1` = 2): Neutraler UAX #29 Fallback/Tokenisierung.

use contextra_crypto::hierarchy::IdxKey;
use contextra_crypto::tag::{tag, TagPreimage};
use contextra_types::{ContextraError, Result, TermTag};
use unicode_segmentation::UnicodeSegmentation;

pub const ANALYZER_DE_1: u8 = 1;
pub const ANALYZER_UND_1: u8 = 2;

pub const ANALYZER_VERSION_1: u16 = 1;

/// Identifikator für die Analyzer-Konfiguration je Collection (D-81).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalyzerId {
    De1,
    Und1,
}

impl AnalyzerId {
    pub const fn id(&self) -> u8 {
        match self {
            Self::De1 => ANALYZER_DE_1,
            Self::Und1 => ANALYZER_UND_1,
        }
    }

    pub const fn version(&self) -> u16 {
        ANALYZER_VERSION_1
    }

    pub const fn lang(&self) -> [u8; 3] {
        match self {
            Self::De1 => *b"de ",
            Self::Und1 => *b"und",
        }
    }

    pub fn from_id(id: u8) -> Result<Self> {
        match id {
            ANALYZER_DE_1 => Ok(Self::De1),
            ANALYZER_UND_1 => Ok(Self::Und1),
            other => Err(ContextraError::InvalidInput(format!(
                "Unbekannte Analyzer-ID: {other} (TX-07: kein stiller Fallback)"
            ))),
        }
    }
}

/// Token mit Text und Position im Dokument (für Phrasensuche, TX-08).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedToken {
    pub term: String,
    pub position: u32,
}

/// Ergebnis einer Dokumentenanalyse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzedDocument {
    pub tokens: Vec<AnalyzedToken>,
    pub token_count: u32,
}

/// Trait für statisch dispatchte Textanalysatoren (SPEC D-81).
pub trait Analyzer: Send + Sync {
    fn analyzer_id(&self) -> AnalyzerId;
    fn analyze(&self, text: &str) -> AnalyzedDocument;
}

/// Deutscher Analyzer `de/1` (SPEC D-81, TX-07).
pub struct GermanAnalyzer {
    compound_splitter: Option<crate::morphology::GermanCompoundSplitter>,
}

impl GermanAnalyzer {
    pub fn new() -> Self {
        let splitter = crate::morphology::GermanCompoundSplitter::new();
        Self {
            compound_splitter: Some(splitter),
        }
    }
}

impl Default for GermanAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Analyzer for GermanAnalyzer {
    fn analyzer_id(&self) -> AnalyzerId {
        AnalyzerId::De1
    }

    fn analyze(&self, text: &str) -> AnalyzedDocument {
        let mut tokens = Vec::new();
        let mut pos = 0u32;

        for word in text.unicode_words() {
            // Normalisierung: lowercase, Umlaute (ae, oe, ue), ß -> ss
            let lower = word.to_lowercase();
            let folded = lower.replace('ß', "ss");
            let norm = crate::morphology::normalize_umlauts(&folded);

            // Stoppwortprüfung
            if crate::morphology::is_german_stopword(&norm) {
                pos = pos.saturating_add(1);
                continue;
            }

            // Falls Dekompoundierung möglich ist:
            if let Some(ref splitter) = self.compound_splitter {
                use crate::morphology::MorphologicalTokenizer;
                let parts = splitter.decompose(&norm);
                if parts.len() > 1 {
                    for part in parts {
                        let part_norm = part.to_lowercase();
                        if !part_norm.is_empty()
                            && !crate::morphology::is_german_stopword(&part_norm)
                        {
                            tokens.push(AnalyzedToken {
                                term: part_norm,
                                position: pos,
                            });
                        }
                    }
                }
            }

            tokens.push(AnalyzedToken {
                term: norm,
                position: pos,
            });

            pos = pos.saturating_add(1);
        }

        AnalyzedDocument {
            token_count: pos,
            tokens,
        }
    }
}

/// Neutraler Analyzer `und/1` (SPEC D-81).
pub struct UndAnalyzer;

impl Analyzer for UndAnalyzer {
    fn analyzer_id(&self) -> AnalyzerId {
        AnalyzerId::Und1
    }

    fn analyze(&self, text: &str) -> AnalyzedDocument {
        let mut tokens = Vec::new();
        let mut pos = 0u32;

        for word in text.unicode_words() {
            let lower = word.to_lowercase();
            tokens.push(AnalyzedToken {
                term: lower,
                position: pos,
            });
            pos = pos.saturating_add(1);
        }

        AnalyzedDocument {
            token_count: pos,
            tokens,
        }
    }
}

/// Berechnet das 16-Byte `TermTag` deterministisch nach SPEC D-81:
/// `term_tag = tag("ctx/term/v1", analyzer_id ‖ lang ‖ norm_term)`.
pub fn compute_term_tag(k_idx: &IdxKey, analyzer: AnalyzerId, norm_term: &str) -> TermTag {
    let mut pre = TagPreimage::new();
    let a_id = [analyzer.id()];
    let lang = analyzer.lang();
    pre.push(b"ctx/term/v1");
    pre.push(&a_id);
    pre.push(&lang);
    pre.push(norm_term.as_bytes());

    let raw = tag(k_idx, &pre);
    TermTag::new(raw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_crypto::secret::SecretKey;

    #[test]
    fn test_german_analyzer_de1() {
        let analyzer = GermanAnalyzer::new();
        assert_eq!(analyzer.analyzer_id(), AnalyzerId::De1);

        let doc = analyzer.analyze("Das Bundesverfassungsgericht entschied über Fußbälle.");
        assert!(doc.token_count > 0);

        let terms: Vec<_> = doc.tokens.iter().map(|t| t.term.as_str()).collect();
        assert!(!terms.contains(&"das"));
        assert!(!terms.contains(&"über"));
        assert!(terms.contains(&"fussbaelle") || terms.contains(&"fussball"));
    }

    #[test]
    fn test_term_tag_determinism() {
        let secret = SecretKey::new([42u8; 32]);
        let key = IdxKey(secret);
        let tag1 = compute_term_tag(&key, AnalyzerId::De1, "bundesgericht");
        let tag2 = compute_term_tag(&key, AnalyzerId::De1, "bundesgericht");
        let tag3 = compute_term_tag(&key, AnalyzerId::Und1, "bundesgericht");

        assert_eq!(tag1, tag2);
        assert_ne!(tag1, tag3);
    }

    #[test]
    fn test_unknown_analyzer_fails_closed() {
        let err = AnalyzerId::from_id(99).unwrap_err();
        assert!(matches!(err, ContextraError::InvalidInput(_)));
    }
}
