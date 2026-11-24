//! Deutsche Stoppwörter und Umlaut-Normalisierung.

use std::collections::HashSet;
use std::sync::OnceLock;

static GERMAN_STOPWORDS: OnceLock<HashSet<String>> = OnceLock::new();

// ANCHOR[contextra-text:MORPH-007]
/// Liefert die statische deutsche Stoppwort-Liste.
///
/// Wird per [`OnceLock`] beim ersten Zugriff lazy initialisiert.
/// Erweitert um hochfrequente deutsche Funktionswörter gemäß DACH-KMU-Domänenanalyse, 2026-09-01.
pub fn get_german_stopwords() -> &'static HashSet<String> {
    GERMAN_STOPWORDS.get_or_init(|| {
        let words = [
            "nicht", "auch", "noch", "sehr", "mehr", "dann", "beim", "nach", "wenn", "aber",
            "durch", "für", "mit", "von", "zum", "zur", "ist", "sind", "war", "waren", "wird",
            "werden", "kann", "können", "soll", "sollen", "muss", "müssen", "hat", "haben", "ein",
            "eine", "einen", "einem", "einer", "des", "dem", "den", "der", "die", "das", "eines",
            "am", "im", "in", "an", "zu", "und", "oder", "auf", "über",
        ];
        let mut set = HashSet::with_capacity(words.len());
        for &w in &words {
            set.insert(w.to_string());
        }
        set
    })
}

// ANCHOR[contextra-text:MORPH-008]
/// Prüft, ob ein kleingeschriebenes, normalisiertes Wort ein deutsches Stoppwort ist.
pub fn is_german_stopword(word: &str) -> bool {
    get_german_stopwords().contains(word)
}

// ANCHOR[contextra-text:MORPH-009]
/// Normalisiert deutsche Umlaute und Sonderzeichen für robusten Volltextabgleich.
///
/// # Ersetzungen
/// - `ä` -> `"ae"`
/// - `ö` -> `"oe"`
/// - `ü` -> `"ue"`
/// - `ß` -> `"ss"`
///
/// # Beispiel
/// ```
/// use contextra_text::morphology::{normalize_umlauts, GermanCompoundSplitter};
/// use contextra_text::morphology::MorphologicalTokenizer;
///
/// let splitter = GermanCompoundSplitter::new();
/// let normalized = normalize_umlauts("Bundesverfassungsgericht");
/// assert_eq!(normalized, "bundesverfassungsgericht");
/// let parts = splitter.decompose(&normalized);
/// assert!(parts.contains(&"gericht"));
/// ```
pub fn normalize_umlauts(input: &str) -> String {
    input
        .to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
}
