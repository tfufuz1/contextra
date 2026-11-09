//! Adapter-Schnittstellenskizzen für externe Retrieval-Engines (Tantivy, LanceDB)
//!
//! Gemäß Spezifikation WP-115 / E12 Vorbereitung:
//! Reine Schnittstellendefinitionen zur späteren Anbindung und vergleichenden
//! Evaluierung gegen Drittsysteme (Tantivy für BM25, LanceDB für Vektorsuche).
//!
//! Enthält KEINE neuen externen Abhängigkeiten (zero-dependency sketch).

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Abstraktes Dokument für den Engine-Vergleich
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkDocument {
    pub id: String,
    pub title: String,
    pub body: String,
    pub vector: Option<Vec<f32>>,
}

/// Einheitliche Treffer-Struktur externer Engines
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EngineHit {
    pub doc_id: String,
    pub score: f32,
}

/// Ausführungs-Statistik eines Query-Laufs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QueryLatencyStats {
    pub elapsed: Duration,
    pub hits_returned: usize,
}

/// Generischer Trait für externe oder interne Volltext-Such-Engines (z. B. Tantivy)
pub trait FullTextEngineAdapter: Send + Sync {
    /// Fehler-Typ der Engine
    type Error: std::fmt::Display;

    /// Name der Engine (z. B. "tantivy", "contextra-text")
    fn name(&self) -> &'static str;

    /// Indiziert einen Batch von Textdokumenten
    fn index_batch(&mut self, docs: &[BenchmarkDocument]) -> Result<Duration, Self::Error>;

    /// Führt eine Volltext-/BM25-Suche aus
    fn search(&self, query: &str, top_k: usize) -> Result<Vec<EngineHit>, Self::Error>;

    /// Erzwingt einen Flush / Commit auf die Platte
    fn commit(&mut self) -> Result<(), Self::Error>;
}

/// Generischer Trait für externe oder interne Vektor-Engines (z. B. LanceDB)
pub trait VectorEngineAdapter: Send + Sync {
    /// Fehler-Typ der Engine
    type Error: std::fmt::Display;

    /// Name der Engine (z. B. "lancedb", "contextra-vector")
    fn name(&self) -> &'static str;

    /// Dimension der Vektoren
    fn dimension(&self) -> usize;

    /// Fügt Vektoren mit Dokument-IDs ein
    fn insert_vectors(&mut self, docs: &[BenchmarkDocument]) -> Result<Duration, Self::Error>;

    /// Führt k-NN Ähnlichkeitssuche aus
    fn search_knn(&self, query_vector: &[f32], top_k: usize)
        -> Result<Vec<EngineHit>, Self::Error>;
}

/// Skizze eines Tantivy-Adapters (Vorbereitung für spätere Feature-Gated Implementierung)
pub struct TantivyAdapterSketch {
    pub index_path: String,
    pub is_initialized: bool,
}

impl TantivyAdapterSketch {
    pub fn new(index_path: impl Into<String>) -> Self {
        Self {
            index_path: index_path.into(),
            is_initialized: false,
        }
    }
}

impl FullTextEngineAdapter for TantivyAdapterSketch {
    type Error = String;

    fn name(&self) -> &'static str {
        "tantivy-external-adapter"
    }

    fn index_batch(&mut self, docs: &[BenchmarkDocument]) -> Result<Duration, Self::Error> {
        if !self.is_initialized {
            self.is_initialized = true;
        }
        let _ = docs;
        Ok(Duration::from_millis(10))
    }

    fn search(&self, _query: &str, _top_k: usize) -> Result<Vec<EngineHit>, Self::Error> {
        Ok(Vec::new())
    }

    fn commit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Skizze eines LanceDB-Adapters (Vorbereitung für spätere Feature-Gated Implementierung)
pub struct LanceDbAdapterSketch {
    pub uri: String,
    pub table_name: String,
    pub dimension: usize,
}

impl LanceDbAdapterSketch {
    pub fn new(uri: impl Into<String>, table_name: impl Into<String>, dimension: usize) -> Self {
        Self {
            uri: uri.into(),
            table_name: table_name.into(),
            dimension,
        }
    }
}

impl VectorEngineAdapter for LanceDbAdapterSketch {
    type Error = String;

    fn name(&self) -> &'static str {
        "lancedb-external-adapter"
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn insert_vectors(&mut self, docs: &[BenchmarkDocument]) -> Result<Duration, Self::Error> {
        let _ = docs;
        Ok(Duration::from_millis(25))
    }

    fn search_knn(
        &self,
        _query_vector: &[f32],
        _top_k: usize,
    ) -> Result<Vec<EngineHit>, Self::Error> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_sketches_basic_contract() {
        let mut tantivy = TantivyAdapterSketch::new("/tmp/tantivy_idx");
        assert_eq!(tantivy.name(), "tantivy-external-adapter");
        assert!(tantivy.commit().is_ok());

        let mut lancedb = LanceDbAdapterSketch::new("/tmp/lance_data", "vectors", 128);
        assert_eq!(lancedb.name(), "lancedb-external-adapter");
        assert_eq!(lancedb.dimension(), 128);
        assert!(lancedb.insert_vectors(&[]).is_ok());
    }
}
