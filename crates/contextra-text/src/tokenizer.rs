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

#[cfg(not(feature = "types-compat"))]
use crate::error::{Result, TextError as ContextraError};
#[cfg(feature = "types-compat")]
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

// ANCHOR[contextra-text:TOK-013]
/// Repräsentiert ein strukturiertes Token aus der Textanalyse-Pipeline.
///
/// Ein Token umfasst den extrahierten Term, Byte-Offsets in den Originaltext
/// (stets auf validen UTF-8 Zeichengrenzen) sowie das Positionsinkrement
/// für Phrasen- und Proximity-Suchen.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Token {
    /// Normalisierter Termwert.
    pub term: String,
    /// Start-Byte-Offset im Originaltext (auf gültiger Zeichengrenze).
    pub start_offset: usize,
    /// End-Byte-Offset im Originaltext (auf gültiger Zeichengrenze).
    pub end_offset: usize,
    /// Positionsinkrement (Standard: 1 für aufeinanderfolgende Wörter, 0 für Synonyme/Komponenten an gleicher Position).
    pub position_increment: usize,
}

impl Token {
    /// Erzeugt ein neues Token mit Term, Offsets und Positionsinkrement.
    pub fn new(
        term: impl Into<String>,
        start_offset: usize,
        end_offset: usize,
        position_increment: usize,
    ) -> Self {
        Self {
            term: term.into(),
            start_offset,
            end_offset,
            position_increment,
        }
    }
}

// ANCHOR[contextra-text:TOK-014]
/// Strategie bei Überschreitung konfigurierter Kapazitätsgrenzen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum OverflowPolicy {
    /// Überschüssige Eingaben oder Token werden deterministisch abgeschnitten.
    #[default]
    Truncate,
    /// Eine Überschreitung führt zu einem Fail-Closed Fehler.
    Error,
}

// ANCHOR[contextra-text:TOK-015]
/// Konfigurierbare Ressourcen- und Sicherheitsgrenzen der Analyse-Pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PipelineLimits {
    /// Maximale Eingabelänge in Bytes (Default: 1 MiB).
    pub max_input_length: usize,
    /// Maximale Tokenlänge in Bytes (Default: 256 Bytes).
    pub max_token_length: usize,
    /// Maximale Anzahl von Tokens je Dokument (Default: 100_000).
    pub max_tokens_per_doc: usize,
    /// Strategie bei Überschreitung der Grenzen.
    pub overflow_policy: OverflowPolicy,
}

impl Default for PipelineLimits {
    fn default() -> Self {
        Self {
            max_input_length: 1_048_576, // 1 MiB
            max_token_length: 256,       // 256 Bytes
            max_tokens_per_doc: 100_000, // 100,000 Tokens
            overflow_policy: OverflowPolicy::Truncate,
        }
    }
}

// ANCHOR[contextra-text:TOK-016]
/// Konfigurierbare Normalisierungsrichtlinie für Textanalyse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum NormalizationPolicy {
    /// Nur Unicode Lowercasing.
    #[default]
    Lowercase,
    /// NFKC-Normalisierung (Ligaturen auflösen, ZWJ/Formatzeichen entfernen, Fullwidth normalisieren) + Lowercasing.
    Nfkc,
    /// Spezifische deutsche Umlaut-Normalisierung (`ä->ae`, `ö->oe`, `ü->ue`, `ß->ss`) + Lowercasing.
    GermanUmlauts,
    /// Vollständige Kombination aus NFKC, Umlaut-Normalisierung und Lowercasing.
    Complete,
}

impl NormalizationPolicy {
    /// Führt die Normalisierung deterministisch und idempotent auf dem String aus.
    pub fn normalize(&self, text: &str) -> String {
        match self {
            Self::Lowercase => text.to_lowercase(),
            Self::Nfkc => {
                let mut out = String::with_capacity(text.len());
                for c in text.chars() {
                    match c {
                        'ﬁ' => out.push_str("fi"),
                        'ﬂ' => out.push_str("fl"),
                        'ﬀ' => out.push_str("ff"),
                        'ﬃ' => out.push_str("ffi"),
                        'ﬄ' => out.push_str("ffl"),
                        'ﬅ' => out.push_str("ft"),
                        'ﬆ' => out.push_str("st"),
                        'ß' | 'ẞ' => out.push_str("ss"),
                        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
                        '\u{FF01}'..='\u{FF5E}' => {
                            if let Some(ascii) = char::from_u32(c as u32 - 0xFEE0) {
                                out.push(ascii);
                            } else {
                                out.push(c);
                            }
                        }
                        other => out.push(other),
                    }
                }
                out.to_lowercase()
            }
            Self::GermanUmlauts => crate::morphology::normalize_umlauts(text),
            Self::Complete => {
                let nfkc = Self::Nfkc.normalize(text);
                crate::morphology::normalize_umlauts(&nfkc)
            }
        }
    }
}

// ANCHOR[contextra-text:TOK-017]
/// Trait für Token-Filter in einer Analyse-Pipeline.
pub trait TokenFilter: Send + Sync {
    /// Wendet den Filter auf eine Sequenz von Tokens an.
    fn filter(&self, tokens: Vec<Token>) -> Vec<Token>;
    /// Eindeutiger Bezeichner des Filters für die Fingerprint-Generierung.
    fn filter_id(&self) -> &str;
}

// ANCHOR[contextra-text:TOK-018]
/// Trait für vollständige Textanalyse-Pipelines (`Tokenizer -> Filter* -> Stemmer`).
///
/// Garantiert den identischen Ergebnisvertrag für Dokumenten-Indexierung und Abfrage.
pub trait Analyzer: Send + Sync {
    /// Analysiert den gegebenen Text und liefert eine strukturierte Token-Sequenz.
    fn analyze(&self, text: &str) -> Result<Vec<Token>>;

    /// Liefert die stabile, kryptographisch eindeutige Fingerprint-Zeichenkette
    /// bestehend aus Name, Version und Konfigurations-Hash.
    fn fingerprint(&self) -> String;
}

// ANCHOR[contextra-text:TOK-019]
/// Modulare, konfigurierbare Textanalyse-Pipeline.
pub struct AnalyzerPipeline {
    name: String,
    version: u32,
    limits: PipelineLimits,
    normalization: NormalizationPolicy,
    stopwords: Option<HashSet<String>>,
    filters: Vec<Arc<dyn TokenFilter>>,
    german_splitter: Option<Arc<crate::morphology::GermanCompoundSplitter>>,
}

impl AnalyzerPipeline {
    /// Erzeugt eine neue leere Analyse-Pipeline mit Standard-Grenzen.
    pub fn new(name: impl Into<String>, version: u32) -> Self {
        Self {
            name: name.into(),
            version,
            limits: PipelineLimits::default(),
            normalization: NormalizationPolicy::default(),
            stopwords: None,
            filters: Vec::new(),
            german_splitter: None,
        }
    }

    /// Konfiguriert die Ressourcen- und Sicherheitsgrenzen.
    pub fn with_limits(mut self, limits: PipelineLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Konfiguriert die Normalisierungsrichtlinie.
    pub fn with_normalization(mut self, norm: NormalizationPolicy) -> Self {
        self.normalization = norm;
        self
    }

    /// Konfiguriert die Stoppwortliste.
    pub fn with_stopwords(mut self, stopwords: HashSet<String>) -> Self {
        self.stopwords = Some(stopwords);
        self
    }

    /// Fügt einen Token-Filter am Ende der Filterkette hinzu.
    pub fn with_filter(mut self, filter: Arc<dyn TokenFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Aktiviert die deutsche Komposita-Zerlegung.
    pub fn with_german_morphology(
        mut self,
        splitter: Arc<crate::morphology::GermanCompoundSplitter>,
    ) -> Self {
        self.german_splitter = Some(splitter);
        self
    }

    /// Erzeugt die Standard-Pipeline für englischen und generischen Text.
    pub fn default_english() -> Self {
        let stopwords = get_stopwords().clone();
        Self::new("standard_english", 1)
            .with_normalization(NormalizationPolicy::Lowercase)
            .with_stopwords(stopwords)
    }

    /// Erzeugt die Standard-Pipeline für deutsche Texte inklusive Komposita-Zerlegung und Umlauten.
    pub fn default_german() -> Self {
        let stopwords = crate::morphology::get_german_stopwords().clone();
        let splitter = get_shared_splitter();
        Self::new("standard_german", 1)
            .with_normalization(NormalizationPolicy::GermanUmlauts)
            .with_stopwords(stopwords)
            .with_german_morphology(splitter)
    }
}

impl Analyzer for AnalyzerPipeline {
    #[allow(clippy::useless_conversion)]
    fn analyze(&self, text: &str) -> Result<Vec<Token>> {
        // 1. Eingabelängen-Prüfung
        let input_to_process = if text.len() > self.limits.max_input_length {
            match self.limits.overflow_policy {
                OverflowPolicy::Error => {
                    return Err(crate::error::TextError::CapacityExceeded {
                        requested: text.len(),
                        maximum: self.limits.max_input_length,
                    }
                    .into());
                }
                OverflowPolicy::Truncate => {
                    // Strikt an UTF-8 Zeichengrenze abschneiden
                    let mut cut = self.limits.max_input_length;
                    while cut > 0 && !text.is_char_boundary(cut) {
                        cut = cut.saturating_sub(1);
                    }
                    &text[..cut]
                }
            }
        } else {
            text
        };

        let segments = segment_text(input_to_process);
        let mut tokens = Vec::new();
        let mut position = 0usize;

        for seg in segments {
            if tokens.len() >= self.limits.max_tokens_per_doc {
                match self.limits.overflow_policy {
                    OverflowPolicy::Error => {
                        return Err(crate::error::TextError::CapacityExceeded {
                            requested: tokens.len() + 1,
                            maximum: self.limits.max_tokens_per_doc,
                        }
                        .into());
                    }
                    OverflowPolicy::Truncate => break,
                }
            }

            match seg {
                TextSegment::Regular(s) => {
                    // Offset von s innerhalb input_to_process bestimmen
                    let s_offset = s.as_ptr() as usize - input_to_process.as_ptr() as usize;

                    for (word_offset, word) in s.unicode_word_indices() {
                        if tokens.len() >= self.limits.max_tokens_per_doc {
                            if self.limits.overflow_policy == OverflowPolicy::Error {
                                return Err(crate::error::TextError::CapacityExceeded {
                                    requested: tokens.len() + 1,
                                    maximum: self.limits.max_tokens_per_doc,
                                }
                                .into());
                            }
                            break;
                        }

                        let start = s_offset.saturating_add(word_offset);
                        let end = start.saturating_add(word.len());

                        // Zeichengrenzen-Invariante absichern
                        if !input_to_process.is_char_boundary(start)
                            || !input_to_process.is_char_boundary(end)
                        {
                            continue;
                        }

                        // Normalisierung
                        let normalized = self.normalization.normalize(word);

                        // Tokenlängen-Filter
                        if normalized.len() > self.limits.max_token_length {
                            if self.limits.overflow_policy == OverflowPolicy::Error {
                                return Err(crate::error::TextError::CapacityExceeded {
                                    requested: normalized.len(),
                                    maximum: self.limits.max_token_length,
                                }
                                .into());
                            }
                            continue;
                        }

                        // Stoppwort-Filter
                        if let Some(ref sw) = self.stopwords {
                            let lower = word.to_lowercase();
                            if sw.contains(&lower) || sw.contains(&normalized) {
                                continue;
                            }
                        }

                        position = position.saturating_add(1);

                        // Komposita-Zerlegung falls konfiguriert
                        if let Some(ref splitter) = self.german_splitter {
                            use crate::morphology::MorphologicalTokenizer;
                            let components = if normalized.chars().all(|c| !c.is_uppercase()) {
                                splitter.decompose(&normalized)
                            } else {
                                vec![normalized.as_str()]
                            };

                            if components.len() > 1 {
                                // Original-Token auf Position position
                                tokens.push(Token::new(&normalized, start, end, 1));

                                // Zerlegte Komponenten mit pos_inc = 0 (Synonym an gleicher Position)
                                for comp in components {
                                    let comp_norm = self.normalization.normalize(comp);
                                    if !comp_norm.is_empty()
                                        && comp_norm.len() <= self.limits.max_token_length
                                    {
                                        if let Some(ref sw) = self.stopwords {
                                            if sw.contains(&comp_norm) {
                                                continue;
                                            }
                                        }
                                        tokens.push(Token::new(comp_norm, start, end, 0));
                                    }
                                }
                            } else {
                                tokens.push(Token::new(normalized, start, end, 1));
                            }
                        } else {
                            tokens.push(Token::new(normalized, start, end, 1));
                        }
                    }
                }
                TextSegment::Protected(p) => {
                    let p_len = p.len();
                    if p_len <= self.limits.max_token_length {
                        let start = input_to_process.find(&p).unwrap_or(0);
                        let end = (start + p_len).min(input_to_process.len());
                        position = position.saturating_add(1);
                        tokens.push(Token::new(p, start, end, 1));
                    }
                }
            }
        }

        // Benutzerdefinierte Filter anwenden
        for filter in &self.filters {
            tokens = filter.filter(tokens);
        }

        Ok(tokens)
    }

    fn fingerprint(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(self.name.as_bytes());
        hasher.update(&(self.limits.max_input_length as u64).to_le_bytes());
        hasher.update(&(self.limits.max_token_length as u64).to_le_bytes());
        hasher.update(&(self.limits.max_tokens_per_doc as u64).to_le_bytes());
        let norm_id = match self.normalization {
            NormalizationPolicy::Lowercase => "lowercase",
            NormalizationPolicy::Nfkc => "nfkc",
            NormalizationPolicy::GermanUmlauts => "german_umlauts",
            NormalizationPolicy::Complete => "complete",
        };
        hasher.update(norm_id.as_bytes());

        if let Some(ref sw) = self.stopwords {
            hasher.update(b"stopwords:");
            let mut sorted_sw: Vec<&String> = sw.iter().collect();
            sorted_sw.sort_unstable();
            for w in sorted_sw {
                hasher.update(w.as_bytes());
                hasher.update(b",");
            }
        }

        for f in &self.filters {
            hasher.update(b"filter:");
            hasher.update(f.filter_id().as_bytes());
        }

        if self.german_splitter.is_some() {
            hasher.update(b"german_morphology:active");
        }

        let hash = hasher.finalize();
        format!("{}-v{}-{}", self.name, self.version, &hash.to_hex()[..16])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(proptest::test_runner::Config::with_cases(10000))]

        #[test]
        fn test_tokenizer_never_panics(s in ".*") {
            let _ = DefaultTokenizer.tokenize(&s);
        }

        #[test]
        fn test_german_tokenizer_never_panics(s in ".*") {
            let _ = GermanMorphTokenizer::try_new().expect("try_new succeeds").tokenize(&s);
        }

        #[test]
        fn test_prop_high_density_multibyte_never_panics(
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
            let german_tok = GermanMorphTokenizer::try_new().expect("try_new succeeds");
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
        let german_tok = GermanMorphTokenizer::try_new().expect("try_new succeeds");
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
        let tokenizer = GermanMorphTokenizer::try_new().expect("try_new succeeds");
        let tokens = tokenizer.tokenize("Das Bundesverfassungsgericht");
        // "Das" is stopword
        assert!(tokens.contains(&"bundesverfassungsgericht".to_string()));
        assert!(tokens.contains(&"gericht".to_string()));
    }

    #[test]
    fn test_german_morph_tokenizer_case_default_constructor() {
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

        let german_tokens = GermanMorphTokenizer::try_new()
            .expect("try_new succeeds")
            .tokenize(text);
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
        let german_tok = GermanMorphTokenizer::try_new().expect("try_new succeeds");
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
            let tok = GermanMorphTokenizer::try_new().expect("try_new succeeds");
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
        let german_tokenizer = GermanMorphTokenizer::try_new().expect("try_new succeeds");

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

        let german_tokens = GermanMorphTokenizer::try_new()
            .expect("try_new succeeds")
            .tokenize(text);
        assert!(!german_tokens.is_empty());
        assert!(
            german_tokens
                .iter()
                .any(|t| t.contains("pruefung") || t.contains("prüfung")),
            "GermanMorphTokenizer must tokenize umlaut words across boundaries without panic"
        );
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    fn test_analyzer_pipeline_offsets_and_identity() {
        let pipeline = AnalyzerPipeline::default_english();
        let text = "Hello world! Test of the emergency broadcast system.";
        let tokens1 = pipeline.analyze(text).expect("analyze succeeds");
        let tokens2 = pipeline.analyze(text).expect("analyze succeeds");

        assert_eq!(
            tokens1, tokens2,
            "Analyzer pipeline must be 100% deterministic"
        );
        for t in &tokens1 {
            assert!(text.is_char_boundary(t.start_offset));
            assert!(text.is_char_boundary(t.end_offset));
            assert!(t.start_offset < t.end_offset);
            assert_eq!(&text[t.start_offset..t.end_offset].to_lowercase(), &t.term);
        }
    }

    #[test]
    fn test_analyzer_pipeline_fingerprint_stability() {
        let p1 = AnalyzerPipeline::default_english();
        let p2 = AnalyzerPipeline::default_english();
        assert_eq!(p1.fingerprint(), p2.fingerprint());

        let p_de = AnalyzerPipeline::default_german();
        assert_ne!(p1.fingerprint(), p_de.fingerprint());
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
    fn test_analyzer_pipeline_limits_overflow_policy() {
        let limits = PipelineLimits {
            max_input_length: 10,
            max_token_length: 256,
            max_tokens_per_doc: 100_000,
            overflow_policy: OverflowPolicy::Error,
        };
        let p = AnalyzerPipeline::new("test_err", 1).with_limits(limits);
        let res = p.analyze("This string is way longer than 10 bytes");
        assert!(res.is_err(), "Must error on max_input_length overflow");

        let limits_trunc = PipelineLimits {
            max_input_length: 10,
            max_token_length: 256,
            max_tokens_per_doc: 100_000,
            overflow_policy: OverflowPolicy::Truncate,
        };
        let p_trunc = AnalyzerPipeline::new("test_trunc", 1).with_limits(limits_trunc);
        let tokens = p_trunc
            .analyze("This string is way longer than 10 bytes")
            .unwrap();
        assert!(!tokens.is_empty());
        for t in tokens {
            assert!(t.end_offset <= 10);
        }
    }
}
