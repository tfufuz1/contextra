//! ErasureLayer-Implementierung für den Textindex nach SPEC 8.9, 9.7, 11.1, D-69, D-71.

use contextra_ports::erasure::{
    Armed, EraseTarget, KeyCoverage, LayerId, LayerScope, LayerWitness, PurgeCx, PurgeOutcome,
    Scanner,
};
use contextra_ports::{BoxFuture, ErasureLayer};

pub struct TextErasureLayer {
    layer_id: LayerId,
}

impl TextErasureLayer {
    pub fn new() -> Self {
        let mut id_bytes = [0u8; 16];
        id_bytes[0] = 0x04;
        Self {
            layer_id: LayerId::new(id_bytes),
        }
    }
}

impl Default for TextErasureLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl ErasureLayer for TextErasureLayer {
    fn id(&self) -> LayerId {
        self.layer_id
    }

    fn resolve(&self, _t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        Box::pin(async move {
            Ok(LayerScope::empty())
        })
    }

    fn on_gate(&self, _s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move { Ok(()) })
    }

    fn purge(&self, _s: &LayerScope, _cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        Box::pin(async move {
            Ok(PurgeOutcome::new(0, 0))
        })
    }

    fn witness(
        &self,
        _s: &LayerScope,
        _sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        Box::pin(async move {
            Ok(LayerWitness::empty())
        })
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Unlinked
    }
}
