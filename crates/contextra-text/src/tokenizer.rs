// FILE-CONTEXT: Tokenisierung & Stopword-Filterung.
// ZWECK: Zerschneidet Eingabetexte in normalisierte Wort-Tokens mit optionaler deutscher Morphologie.
// INVARIANTEN: Tokenisierung muss deterministisch zwischen Indexierung und Query-Pfad identisch sein.
// NICHT-OFFENSICHTLICH: Stoppwörter werden per OnceLock geladen; DefaultTokenizer filtert Alphatoken + Stoppwörter.
// HOTSPOTS: tokenize, DefaultTokenizer::tokenize, GermanMorphTokenizer::tokenize
// STAND: TS:2026-09-10T19:25:46Z (SESSION: c844907e)

//! Tokenisierung, Stoppwort-Filterung und Schutz atomarer Token (URLs, E-Mails).
//!
//! # Ring-Zuordnung
//! Ring 0 (Kern).
//!
//! # Funktionsweise
//! Zerlegt Text in einzelne Wort-Token unter Verwendung von `unicode-segmentation`.
//! URLs und E-Mail-Adressen werden über reguläre Ausdrücke geschützt, damit sie
//! nicht fälschlich an Interpunktionszeichen aufgespalten werden.

use contextra_types::{ContextraError, Result};
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::OnceLock;
use unicode_segmentation::UnicodeSegmentation;

static STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();
static PROTECTED_REGEX: OnceLock<std::result::Result<Regex, String>> = OnceLock::new();
static SHARED_SPLITTER: OnceLock<Arc<crate::morphology::GermanCompoundSplitter>> = OnceLock::new();

// ANCHOR[contextra-text:TOK-001]
/// Kompiliert ein reguläres Muster für geschützte Token mit Fail-Closed-Fehlerbehandlung.
///
/// # Fehler
/// Gibt [`ContextraError::Text`] zurück, falls der reguläre Ausdruck syntaktisch ungültig ist.
pub fn compile_protected_regex(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|e| {
        ContextraError::Text(format!("Invalid protected regex pattern '{pattern}': {e}"))
    })
}

// ANCHOR[contextra-text:TOK-002]
/// Liefert den gecachten regulären Standardausdruck für geschützte URLs und E-Mail-Adressen.
pub fn get_protected_regex() -> Result<&'static Regex> {
    let res = PROTECTED_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(?:https?://[^\s]+|[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})")
            .map_err(|e| e.to_string())
    });
    match res {
        Ok(re) => Ok(re),
        Err(msg) => Err(ContextraError::Text(format!(
            "Protected regex initialization failed: {msg}"
        ))),
    }
}

fn get_shared_splitter() -> Arc<crate::morphology::GermanCompoundSplitter> {
    SHARED_SPLITTER
        .get_or_init(|| Arc::new(crate::morphology::GermanCompoundSplitter::new()))
        .clone()
}

fn clean_protected_match(m: &str) -> &str {
    if m.to_lowercase().starts_with("http://") || m.to_lowercase().starts_with("https://") {
        m.trim_end_matches(|c: char| {
            matches!(
                c,
                '.' | ',' | ';' | '!' | '?' | ')' | ']' | '>' | '"' | '\''
            )
        })
    } else {
        m
    }
}

enum TextSegment<'a> {
    Regular(&'a str),
    Protected(String),
}

fn segment_text<'a>(text: &'a str) -> Vec<TextSegment<'a>> {
    let re = match get_protected_regex() {
        Ok(re) => re,
        Err(_) => return vec![TextSegment::Regular(text)],
    };
    let mut segments = Vec::new();
    let mut last_idx = 0;

    while last_idx < text.len() {
        if let Some(mat) = re.find(&text[last_idx..]) {
            let start = last_idx + mat.start();
            let raw_match = mat.as_str();
            let cleaned = clean_protected_match(raw_match);

            if cleaned.is_empty() {
                last_idx += mat.end();
                continue;
            }

            if start > last_idx {
                segments.push(TextSegment::Regular(&text[last_idx..start]));
            }

            segments.push(TextSegment::Protected(cleaned.to_lowercase()));

            let match_end = start + cleaned.len();
            last_idx = match_end;
        } else {
            if last_idx < text.len() {
                segments.push(TextSegment::Regular(&text[last_idx..]));
            }
            break;
        }
    }

    segments
}

fn get_stopwords() -> &'static HashSet<String> {
    STOPWORDS.get_or_init(|| {
        let words = vec![
            "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into",
            "is", "it", "no", "not", "of", "on", "or", "such", "that", "the", "their", "then",
            "there", "these", "they", "this", "to", "was", "will", "with", "i", "me", "my", "we",
            "our", "you", "your", "he", "she", "his", "her", "it", "its", "they", "them", "their",
            "der", "die", "das", "ein", "eine", "einer", "eines", "dem", "den", "des", "am", "im",
            "in", "an", "zu", "für", "und", "oder", "ist", "sind", "war", "von", "mit", "auf",
            "über",
        ];
        words.into_iter().map(|w| w.to_string()).collect()
    })
}

// ANCHOR[contextra-text:TOK-003]
/// Trait für Text-Tokenizer mit unterschiedlichen Filter- oder Zerlegungsstrategien.
pub trait Tokenizer: Send + Sync {
    /// Zerlegt den übergebenen Text in kleingeschriebene Wort-Token und filtert Stoppwörter heraus.
    fn tokenize(&self, text: &str) -> Vec<String>;
}

// ANCHOR[contextra-text:TOK-004]
/// Standard-Tokenizer basierend auf Unicode-Wortgrenzen und generischen Stoppwörtern.
pub struct DefaultTokenizer;

impl DefaultTokenizer {
    // ANCHOR[contextra-text:TOK-005]
    /// Fallible Konstruktion eines `DefaultTokenizer` mit Validierung des regulären Schutzmusters.
    pub fn try_new() -> Result<Self> {
        let _ = get_stopwords();
        let _ = get_protected_regex()?;
        Ok(Self)
    }

    // ANCHOR[contextra-text:TOK-006]
    /// Konstruiert einen `DefaultTokenizer`.
    #[deprecated(
        since = "0.1.0",
        note = "Use DefaultTokenizer::try_new() instead for fallible initialization"
    )]
    pub fn new() -> Self {
        Self::try_new().unwrap_or(Self)
    }
}

impl Default for DefaultTokenizer {
    fn default() -> Self {
        #[allow(deprecated)]
        Self::new()
    }
}

impl Tokenizer for DefaultTokenizer {
    // ANCHOR[contextra-text:TOK-007]
    // Ablauf von tokenize (DefaultTokenizer):
    // 1. Text segmentieren (geschützte URLs/E-Mails separat erfassen).
    // 2. Reguläre Abschnitte nach Unicode-Wörtern trennen, klein schreiben und Stoppwörter verwerfen.
    // 3. Geschützte Abschnitte unverändert als atomare Tokens übernehmen.
    fn tokenize(&self, text: &str) -> Vec<String> {
        let stopwords = get_stopwords();
        let segments = segment_text(text);
        let mut tokens = Vec::new();

        for seg in segments {
            match seg {
                TextSegment::Regular(s) => {
                    tokens.extend(
                        s.unicode_words()
                            .map(|w| w.to_lowercase())
                            .filter(|w| !stopwords.contains(w)),
                    );
                }
                TextSegment::Protected(p) => {
                    tokens.push(p);
                }
            }
        }

        tokens
    }
}

// ANCHOR[contextra-text:TOK-008]
/// Deutscher Tokenizer mit morphologischer Zerlegung von Wortzusammensetzungen.
pub struct GermanMorphTokenizer {
    splitter: Arc<crate::morphology::GermanCompoundSplitter>,
}

impl GermanMorphTokenizer {
    // ANCHOR[contextra-text:TOK-009]
    /// Erzeugt eine neue Instanz des deutschen morphologischen Tokenizers.
    pub fn try_new() -> Result<Self> {
        let _ = get_protected_regex()?;
        Ok(Self {
            splitter: get_shared_splitter(),
        })
    }

    // ANCHOR[contextra-text:TOK-010]
    /// Erzeugt einen deutschen Tokenizer mit geteiltem Standard-Splitter.
    #[deprecated(
        since = "0.1.0",
        note = "Use GermanMorphTokenizer::try_new() instead for fallible initialization"
    )]
    pub fn new() -> Self {
        Self::try_new().unwrap_or_else(|_| Self {
            splitter: get_shared_splitter(),
        })
    }
}

impl Default for GermanMorphTokenizer {
    fn default() -> Self {
        #[allow(deprecated)]
        Self::new()
    }
}

impl Tokenizer for GermanMorphTokenizer {
    // ANCHOR[contextra-text:TOK-011]
    // Ablauf von tokenize (GermanMorphTokenizer):
    // 1. Text segmentieren (geschützte URLs/Mails isolieren).
    // 2. Wörter extrahieren, normalisieren und gegen Stoppwortliste prüfen.
    // 3. Komposita zerlegen und sowohl Originalwort als auch Teilbestandteile hinzufügen.
    fn tokenize(&self, text: &str) -> Vec<String> {
        use crate::morphology::{normalize_umlauts, MorphologicalTokenizer};
        let stopwords = get_stopwords();
        let segments = segment_text(text);
        let mut tokens = Vec::new();

        for seg in segments {
            match seg {
                TextSegment::Regular(s) => {
                    for word in s.unicode_words() {
                        let lower = word.to_lowercase();
                        let normalized = normalize_umlauts(&lower);
                        if stopwords.contains(&lower) || stopwords.contains(&normalized) {
                            continue;
                        }

                        let components = if normalized.chars().all(|c| !c.is_uppercase()) {
                            self.splitter.decompose(&normalized)
                        } else {
                            vec![normalized.as_str()]
                        };
                        if components.len() > 1 {
                            // Collect component strings first to avoid borrow issues with lower
                            let mut comp_strs: Vec<String> = Vec::new();
                            for c in &components {
                                comp_strs.push(c.to_string());
                                let norm_c = normalize_umlauts(c);
                                if norm_c != *c {
                                    comp_strs.push(norm_c);
                                }
                            }
                            // Keep original compound for exact matches
                            tokens.push(lower.clone());
                            if normalized != lower {
                                tokens.push(normalized);
                            }
                            // Add decomposed components for recall
                            tokens.extend(comp_strs);
                        } else {
                            tokens.push(lower.clone());
                            if normalized != lower {
                                tokens.push(normalized);
                            }
                        }
                    }
                }
                TextSegment::Protected(p) => {
                    tokens.push(p);
                }
            }
        }

        tokens
    }
}

// ANCHOR[contextra-text:TOK-012]
/// Hilfsfunktion zur Tokenisierung mit dem [`DefaultTokenizer`].
pub fn tokenize(text: &str) -> Vec<String> {
    DefaultTokenizer.tokenize(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10000))]

        #[test]
        fn tokenizer_never_panics(s in ".*") {
            let _ = DefaultTokenizer.tokenize(&s);
        }

        #[test]
        fn german_tokenizer_never_panics(s in ".*") {
            let _ = GermanMorphTokenizer::new().tokenize(&s);
        }

        #[test]
        fn prop_high_density_multibyte_never_panics(
            s in proptest::collection::vec(
                prop_oneof![
                    proptest::char::range('\u{00C0}', '\u{017F}'),
                    proptest::char::range('\u{1F300}', '\u{1FAFF}'),
                    proptest::char::range('\u{0300}', '\u{036F}'),
                    proptest::char::range('a', 'z'),
                ],
                0..100
            ).prop_map(|chars| chars.into_iter().collect::<String>())
        ) {
            use crate::morphology::MorphologicalTokenizer;

            // 1. DefaultTokenizer
            let _ = DefaultTokenizer.tokenize(&s);

            // 2. GermanMorphTokenizer
            let german_tok = GermanMorphTokenizer::new();
            let _ = german_tok.tokenize(&s);

            // 3. normalize_umlauts
            let norm = crate::morphology::normalize_umlauts(&s);

            // 4. GermanCompoundSplitter
            let splitter = crate::morphology::GermanCompoundSplitter::new();
            let _ = splitter.decompose(&norm);

            // 5. BM25 scoring model
            let bm25 = crate::bm25::BM25::default();
            let _ = bm25.score_term(1, 10, 10.0, 1, 100);
        }
    }

    #[test]
    fn test_tokenizer_edge_cases_bounded() {
        let long_str = "a".repeat(100_000);
        let edge_cases = ["", " \t\n", "🦀🦀", long_str.as_str(), "\0", "ÄÖÜß"];
        let german_tok = GermanMorphTokenizer::new();
        for input in &edge_cases {
            let tokens = DefaultTokenizer.tokenize(input);
            assert!(tokens.len() <= input.chars().count() + 1);
            let g_tokens = german_tok.tokenize(input);
            let _ = g_tokens;
        }
    }

    #[test]
    fn test_try_new_constructors_succeed() {
        assert!(DefaultTokenizer::try_new().is_ok());
        assert!(GermanMorphTokenizer::try_new().is_ok());
    }

    #[test]
    fn test_get_protected_regex_success() {
        let res = get_protected_regex();
        assert!(
            res.is_ok(),
            "Default protected regex must compile successfully"
        );
    }

    #[test]
    fn test_compile_protected_regex_invalid_pattern_returns_contextra_error() {
        let invalid_pattern = "(?i)[a-z";
        let res = compile_protected_regex(invalid_pattern);
        assert!(
            res.is_err(),
            "Invalid regex pattern must return an Err, got: {:?}",
            res
        );
        if let Err(ContextraError::Text(msg)) = res {
            assert!(
                msg.contains("Invalid protected regex pattern"),
                "Expected invalid regex message, got: {}",
                msg
            );
        } else {
            unreachable!("Expected ContextraError::Text");
        }
    }

    #[test]
    fn test_tokenizer_handles_unicode() {
        let text = "Ärger über Ölpreise";
        let tokens = tokenize(text);
        // "über" is in our stopword list now
        assert_eq!(tokens, vec!["ärger", "ölpreise"]);
    }

    #[test]
    fn test_tokenizer_filters_stopwords() {
        let text = "The quick brown fox jumps over the lazy dog";
        let tokens = tokenize(text);
        assert!(!tokens.contains(&"the".to_string()));
        assert!(tokens.contains(&"quick".to_string()));
    }

    #[test]
    fn test_german_morph_tokenizer() {
        let tokenizer = GermanMorphTokenizer::new();
        let tokens = tokenizer.tokenize("Das Bundesverfassungsgericht");
        // "Das" is stopword
        assert!(tokens.contains(&"bundesverfassungsgericht".to_string()));
        assert!(tokens.contains(&"gericht".to_string()));
    }

    #[test]
    fn german_morph_tokenizer_case_default_constructor() {
        let tokenizer = GermanMorphTokenizer::default();
        let tokens = tokenizer.tokenize("Datenschutzrichtlinie");
        assert!(!tokens.is_empty());
        assert!(tokens.contains(&"datenschutzrichtlinie".to_string()));
    }

    #[test]
    fn test_url_email_protection_bug_txt_001() {
        let text = "Kontakt: user@example.com oder besuche https://api.example.com/v2/search";

        let default_tokens = DefaultTokenizer.tokenize(text);
        assert!(
            default_tokens.contains(&"user@example.com".to_string()),
            "DefaultTokenizer should protect email as atomic token. Got: {:?}",
            default_tokens
        );
        assert!(
            default_tokens.contains(&"https://api.example.com/v2/search".to_string()),
            "DefaultTokenizer should protect URL as atomic token. Got: {:?}",
            default_tokens
        );
        assert!(
            !default_tokens.contains(&"example".to_string())
                && !default_tokens.contains(&"com".to_string())
                && !default_tokens.contains(&"api".to_string()),
            "Email/URL parts should not appear as separate tokens"
        );

        let german_tokens = GermanMorphTokenizer::new().tokenize(text);
        assert!(
            german_tokens.contains(&"user@example.com".to_string()),
            "GermanMorphTokenizer should protect email as atomic token. Got: {:?}",
            german_tokens
        );
        assert!(
            german_tokens.contains(&"https://api.example.com/v2/search".to_string()),
            "GermanMorphTokenizer should protect URL as atomic token. Got: {:?}",
            german_tokens
        );
    }

    #[test]
    fn test_german_compound_no_regression() {
        let text = "Im Softwareentwicklungskontext entstehen oft neue Herausforderungen.";
        let german_tok = GermanMorphTokenizer::new();
        let tokens = german_tok.tokenize(text);

        // Verify that Softwareentwicklungskontext or its components exist in tokens
        assert!(
            tokens.iter().any(|t| t.contains("software")
                || t.contains("entwicklung")
                || t.contains("kontext")),
            "German compound words must continue to be decomposed/tokenized. Got: {:?}",
            tokens
        );
    }

    #[test]
    fn test_trie_caching_performance_bug_txt_002() {
        use std::time::Instant;

        let start = Instant::now();
        for _ in 0..100 {
            let tok = GermanMorphTokenizer::new();
            let _ = tok.tokenize("Testtext für Performance");
        }
        let elapsed = start.elapsed();

        println!("100 GermanMorphTokenizer instantiations took {:?}", elapsed);
        // With shared Trie, 100 instantiations take < 10ms (usually < 1ms), whereas without shared Trie it took > 100ms.
        assert!(
            elapsed.as_millis() < 500,
            "100 instantiations took too long: {:?}. Shared Trie caching should make instantiation lightweight.",
            elapsed
        );
    }

    #[test]
    fn test_tokenizers_panic_free_edge_cases() {
        let long_string = "a".repeat(100_000);
        let edge_cases = ["", " ", ".", "🦀", long_string.as_str()];
        let german_tokenizer = GermanMorphTokenizer::new();

        for text in &edge_cases {
            let default_tokens = tokenize(text);
            assert!(
                default_tokens.len() <= text.len(),
                "DefaultTokenizer token count must not exceed text length"
            );

            let german_tokens = german_tokenizer.tokenize(text);
            // German morph tokenizer may expand compounds, but for these edge cases token count is within reasonable limits
            assert!(
                !german_tokens.is_empty()
                    || text.trim().is_empty()
                    || text == &"."
                    || text == &"🦀",
                "GermanMorphTokenizer must complete without panic for edge case input"
            );
        }
    }

    #[test]
    fn test_tokenizer_multibyte_slicing_boundaries() {
        // Multi-byte Unicode characters, emojis, and combined URLs/emails at slicing boundaries
        let text =
            "Prüfung über https://münchen.de/v1?q=äöü&x=🚀#anchor und user.äöü@sub.münchen.de!";
        let default_tokens = DefaultTokenizer.tokenize(text);
        assert!(!default_tokens.is_empty());

        let german_tokens = GermanMorphTokenizer::new().tokenize(text);
        assert!(!german_tokens.is_empty());
        assert!(
            german_tokens
                .iter()
                .any(|t| t.contains("pruefung") || t.contains("prüfung")),
            "GermanMorphTokenizer must tokenize umlaut words across boundaries without panic"
        );
    }
}
