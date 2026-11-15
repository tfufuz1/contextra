//! Durchleitungs-Tokenizer für Sprachen ohne Komposita-Zerlegung.

use super::compound_splitter::MorphologicalTokenizer;

/// Passthrough-Tokenizer für Sprachen, die keine Zerlegung von Zusammensetzungen benötigen.
///
/// Gibt eingegebene Token unverändert als Einzelkomponente zurück.
pub struct PassthroughTokenizer {
    lang: String,
}

impl PassthroughTokenizer {
    /// Erzeugt einen neuen Durchleitungs-Tokenizer für den angegebenen ISO-Sprachcode (z. B. "en").
    pub fn new(lang: impl Into<String>) -> Self {
        Self { lang: lang.into() }
    }
}

impl MorphologicalTokenizer for PassthroughTokenizer {
    /// Gibt das Token unverändert als Einzelelement in einem Vektor zurück.
    fn decompose<'a>(&self, token: &'a str) -> Vec<&'a str> {
        vec![token]
    }

    /// Liefert den konfigurierten Sprachcode.
    fn language(&self) -> &str {
        &self.lang
    }
}
