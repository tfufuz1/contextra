//! ErasureLayer-Implementierung für doc_store (SPEC §4.2, §8.8, §12.1, ST-16, D-34).

use contextra_ports::erasure::{
    Armed, EraseTarget, ErasureLayer, KeyCoverage, LayerId, LayerScope, LayerWitness, PurgeCx,
    PurgeOutcome, Scanner,
};
use contextra_ports::BoxFuture;
use std::sync::Arc;

use crate::purge::backend::PurgePrimitives;

/// Konstante Schicht-ID für den Dokumentenspeicher (`doc_store`).
pub const DOC_STORE_LAYER_ID: LayerId = LayerId([
    b'd', b'o', b'c', b'_', b's', b't', b'o', b'r', b'e', 0, 0, 0, 0, 0, 0, 1,
]);

/// Repräsentiert die Schicht `doc_store` in der Erasure-Pipeline.
pub struct DocStoreLayer<P: PurgePrimitives> {
    purge: Arc<P>,
}

impl<P: PurgePrimitives> DocStoreLayer<P> {
    pub fn new(purge: Arc<P>) -> Self {
        Self { purge }
    }
}

impl<P: PurgePrimitives> ErasureLayer for DocStoreLayer<P> {
    fn id(&self) -> LayerId {
        DOC_STORE_LAYER_ID
    }

    fn resolve(&self, _t: &EraseTarget) -> BoxFuture<'_, std::result::Result<LayerScope, String>> {
        Box::pin(async move { Ok(LayerScope::empty()) })
    }

    fn on_gate(&self, _s: &LayerScope) -> BoxFuture<'_, std::result::Result<(), String>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge(
        &self,
        _s: &LayerScope,
        _cx: &PurgeCx,
    ) -> BoxFuture<'_, std::result::Result<PurgeOutcome, String>> {
        Box::pin(async move {
            let _seal = self.purge.seal_wal_at().await.map_err(|e| e.to_string())?;
            Ok(PurgeOutcome::new(0, 0))
        })
    }

    fn witness(
        &self,
        _s: &LayerScope,
        _sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, std::result::Result<LayerWitness, String>> {
        Box::pin(async move { Ok(LayerWitness::empty()) })
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }
}
