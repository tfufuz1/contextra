//! Deutsche Stoppwörter und Umlaut-Normalisierung.

use std::collections::BTreeSet;
use std::sync::OnceLock;

static GERMAN_STOPWORDS: OnceLock<BTreeSet<String>> = OnceLock::new();

// ANCHOR[contextra-text:MORPH-007]
/// Liefert die statische deutsche Stoppwort-Liste.
pub fn get_german_stopwords() -> &'static BTreeSet<String> {
    GERMAN_STOPWORDS.get_or_init(|| {
        let words = [
            "nicht", "auch", "noch", "sehr", "mehr", "dann", "beim", "nach", "wenn", "aber",
            "durch", "für", "mit", "von", "zum", "zur", "ist", "sind", "war", "waren", "wird",
            "werden", "kann", "können", "soll", "sollen", "muss", "müssen", "hat", "haben", "ein",
            "eine", "einen", "einem", "einer", "des", "dem", "den", "der", "die", "das", "eines",
            "am", "im", "in", "an", "zu", "und", "oder", "auf", "über",
        ];
        let mut set = BTreeSet::new();
        for &w in &words {
            set.insert(w.to_string());
        }
        set
    })
}

// ANCHOR[contextra-text:MORPH-008]
pub fn is_german_stopword(word: &str) -> bool {
    get_german_stopwords().contains(word)
}

// ANCHOR[contextra-text:MORPH-009]
pub fn normalize_umlauts(input: &str) -> String {
    input
        .to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
}
