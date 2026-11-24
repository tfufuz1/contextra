// FILE-CONTEXT
// ZWECK: Isolierte Brücke für HNSW-Fallback und Konfigurationsableitung (WP-VEC-07 / S-80).
// INVARIANTEN: Kapselt alle Zugriffe auf crate::hnsw::*; keine direkten HNSW-Imports im DiskANN-Kern.
// HOTSPOTS: HnswFallbackBridge, create_hnsw_fallback, search_hnsw
// STAND: 2026-11-11

//! Brückenmodul für die Interaktion zwischen DiskANN und HNSW.
//!
//! Kapselt die Erstellung, Konfiguration und Suche des HNSW-Fallback-Indexes,
//! um die spätere Auslagerung von DiskANN in ein eigenständiges Crate vorzubereiten.

use std::sync::Arc;

use crate::error::Result;
use crate::types::{DistanceMetric, DocId, ScoredDocument};

// ANCHOR[contextra-vector:W07-HNSW-001]
/// Gekapselter HNSW-Fallback-Index.
#[derive(Clone)]
pub(crate) struct HnswFallbackBridge {
    inner: Arc<crate::hnsw::HnswIndex>,
}

impl HnswFallbackBridge {
    /// Erstellt eine neue Fallback-Brücke um den übergebenen Index.
    pub(crate) fn new(index: Arc<crate::hnsw::HnswIndex>) -> Self {
        Self { inner: index }
    }

    /// Führt eine Suche auf dem HNSW-Fallback durch.
    pub(crate) async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        self.inner
            .search_filtered_internal(query, k, None, None)
            .await
    }

    /// Liefert die innere Arc-Referenz (falls benötigt).
    pub(crate) fn as_inner(&self) -> &Arc<crate::hnsw::HnswIndex> {
        &self.inner
    }
}

// ANCHOR[contextra-vector:W07-HNSW-002]
/// Erzeugt einen neuen HNSW-Index als Fallback basierend auf den Parametern.
pub(crate) fn create_hnsw_fallback(
    dimension: usize,
    distance_metric: DistanceMetric,
    quantize: bool,
) -> Result<HnswFallbackBridge> {
    let hnsw_config = crate::hnsw::HnswConfig {
        dimension,
        distance_metric,
        quantize,
        ..crate::hnsw::HnswConfig::default()
    };
    let hnsw = Arc::new(crate::hnsw::HnswIndex::try_new(hnsw_config)?);
    Ok(HnswFallbackBridge::new(hnsw))
}
