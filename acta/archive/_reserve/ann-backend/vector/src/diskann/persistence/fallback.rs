// FILE-CONTEXT
// ZWECK: Initialisierung des HNSW-Fallbacks für DiskANN (WP-VEC-07).
// INVARIANTEN: Gekapselte Erstellung über hnsw_bridge; saubere Fehlerweiterleitung.
// STAND: 2026-11-11

use super::super::hnsw_bridge::{create_hnsw_fallback, HnswFallbackBridge};
use super::super::types::DiskAnnIndex;
use crate::error::Result;

impl DiskAnnIndex {
    pub(crate) fn init_hnsw_fallback(
        &self,
    ) -> Result<crate::diskann::hnsw_bridge::HnswFallbackBridge> {
        let hnsw = crate::diskann::hnsw_bridge::create_hnsw_fallback(
            self.inner.config.dimension,
            self.inner.config.distance_metric,
            self.inner.config.quantize,
        )?;
        *self.inner.hnsw_fallback.write() = Some(hnsw.clone());
        Ok(hnsw)
    }
}
