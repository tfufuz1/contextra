//! Lokale Traits für Embedding- und Cross-Encoder-Inferenz.
//!
//! Entspricht den Anforderungen an Klasse-A-Bausteine: Ausschließlich Standardtypen
//! in der öffentlichen Schnittstelle, keine Abhängigkeiten zu `contextra-ports` oder `contextra-types`.

use crate::error::Result;

/// Synchroner Trait für Vektor-Embeddings.
pub trait Embedder: Send + Sync {
    /// Erzeugt ein Embedding für einen einzelnen Text.
    ///
    /// # Errors
    /// Gibt einen Fehler zurück, falls Tokenisierung oder Inferenz fehlschlagen.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Erzeugt Embeddings für einen Batch von Texten.
    ///
    /// Das Ergebnis muss bit- oder innerhalb enger numerischer Toleranz äquivalent
    /// zu sequentiellen Einzelaufrufen von `embed` sein.
    ///
    /// # Errors
    /// Gibt einen Fehler zurück, wenn die Batch-Verarbeitung fehlschlägt.
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text)?);
        }
        Ok(results)
    }

    /// Gibt die feste Ausgabedimension des Modells zurück.
    fn dimension(&self) -> usize;
}

/// Synchroner Trait für Cross-Encoder Reranking.
pub trait Reranker: Send + Sync {
    /// Berechnet Relevanz-Scores für eine Query und eine Liste von Dokumenten.
    ///
    /// Gibt einen Vektor von Scores gleicher Länge wie `docs` zurück.
    ///
    /// # Errors
    /// Gibt einen Fehler zurück, falls die Berechnung fehlschlägt.
    fn score(&self, query: &str, docs: &[&str]) -> Result<Vec<f32>>;
}
