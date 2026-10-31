//! Durchleitungs-Tokenizer für Sprachen ohne Komposita-Zerlegung.

use super::compound_splitter::MorphologicalTokenizer;

// ANCHOR[contextra-text:MORPH-003]
/// Passthrough-Tokenizer für Sprachen, die keine Zerlegung von Zusammensetzungen benötigen.
///
/// Gibt eingegebene Token unverändert als Einzelkomponente zurück.
pub struct PassthroughTokenizer {
    lang: String,
}

impl PassthroughTokenizer {
    // ANCHOR[contextra-text:MORPH-004]
    /// Erzeugt einen neuen Durchleitungs-Tokenizer für den angegebenen ISO-Sprachcode (z. B. "en").
    pub fn new(lang: impl Into<String>) -> Self {
        Self { lang: lang.into() }
    }
}

impl MorphologicalTokenizer for PassthroughTokenizer {
    // ANCHOR[contextra-text:MORPH-005]
    /// Gibt das Token unverändert als Einzelelement in einem Vektor zurück.
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str> {
        vec![token]
    }

    // ANCHOR[contextra-text:MORPH-006]
    /// Liefert den konfigurierten Sprachcode.
    fn language(&self) -> &str {
        &self.lang
    }
}
