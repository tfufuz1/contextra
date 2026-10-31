//! Metriken zur Quantifizierung morphologischer Zerlegungseffekte.

// ANCHOR[contextra-text:MORPH-001]
/// Metriken zur Erfassung der Token-Expansions- und Zerlegungswirkung.
#[derive(Debug, Clone, Copy)]
pub struct TokenReductionMetrics {
    /// Anzahl ursprünglicher Token vor der morphologischen Zerlegung.
    pub original_tokens: usize,
    /// Anzahl resultierender Token nach Zerlegung von Komposita.
    pub decomposed_tokens: usize,
}

impl TokenReductionMetrics {
    // ANCHOR[contextra-text:MORPH-002]
    /// Berechnet das Expansionsverhältnis der Token-Menge.
    ///
    /// Ein Wert > 1,0 bedeutet eine Erhöhung der Tokenanzahl durch Komposita-Splitting.
    ///
    /// # Zielwert
    /// Für deutsche Fachtexte wird ein Zuwachs von > 20% (> 1,2) angestrebt,
    /// um die Trefferquote (Recall) im BM25-Index signifikant zu steigern.
    ///
    /// # Ablauf
    /// 1. Liefert 0.0, falls keine Ausgangstoken vorhanden sind (`original_tokens == 0`).
    /// 2. Teilt `decomposed_tokens` als Gleitkommazahl durch `original_tokens`.
    pub fn expansion_ratio(&self) -> f32 {
        if self.original_tokens == 0 {
            return 0.0;
        }
        self.decomposed_tokens as f32 / self.original_tokens as f32
    }
}
