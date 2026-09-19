use serde::{Deserialize, Serialize};

/// Supported tokenizer languages for BM25 text indexing.
///
/// Controls which tokenizer is used for document indexing and query processing.
/// Use `Language::from_iso()` to parse ISO-639-1 language codes.
///
/// **Important**: Language is configured explicitly — namespace names do NOT
/// determine the tokenizer language.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Language {
    /// English tokenizer (default, whitespace-based + stopword filtering).
    #[default]
    English,
    /// German morphology tokenizer (compound splitting + umlaut normalization).
    German,
    /// Custom tokenizer — override via `with_tokenizer()`.
    Custom,
}

impl Language {
    /// Parses an ISO-639-1 language code ("de", "de-AT", "en", "en-US").
    ///
    /// Only the primary subtag (before the first '-') is evaluated.
    /// Unknown codes fall back to `Language::English` (safe default).
    pub fn from_iso(code: &str) -> Self {
        let prefix = code.split('-').next().unwrap_or("").to_lowercase();
        match prefix.as_str() {
            "de" => Self::German,
            "en" => Self::English,
            _ => Self::English,
        }
    }
}

/// Consolidated metadata for the text index.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TextIndexMetadata {
    pub total_docs: u64,
    pub total_tokens: u64,
    pub avg_doc_len_x1000: u64, // Fixed-point for BM25 caching (FIND-TXT-004)
}

#[derive(Default, Clone, Copy, Debug)]
pub(crate) struct StagedStatsChange {
    pub(crate) docs_delta: i64,
    pub(crate) tokens_delta: i64,
}
