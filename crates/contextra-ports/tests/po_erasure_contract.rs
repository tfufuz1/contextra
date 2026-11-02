//! Contract tests for PO-04 and PO-05: ErasureLayer trait and types.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "../src/erasure.rs"]
pub mod erasure;

use contextra_ports::BoxFuture;
use erasure::{
    Armed, EraseTarget, ErasureLayer, KeyCoverage, LayerId, LayerScope, LayerWitness, PurgeCx,
    PurgeOutcome, Scanner,
};
use std::sync::Arc;

struct MockErasureLayer;

impl ErasureLayer for MockErasureLayer {
    fn id(&self) -> LayerId {
        LayerId::new(*b"mock-layer-00001")
    }

    fn resolve(&self, _t: &EraseTarget) -> BoxFuture<'_, Result<LayerScope, String>> {
        Box::pin(async { Ok(LayerScope::empty()) })
    }

    fn on_gate(&self, _s: &LayerScope) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async { Ok(()) })
    }

    fn purge(&self, _s: &LayerScope, _cx: &PurgeCx) -> BoxFuture<'_, Result<PurgeOutcome, String>> {
        Box::pin(async { Ok(PurgeOutcome::new(0, 0)) })
    }

    fn witness(
        &self,
        _s: &LayerScope,
        _sc: &Scanner<Armed>,
    ) -> BoxFuture<'_, Result<LayerWitness, String>> {
        Box::pin(async { Ok(LayerWitness::empty()) })
    }

    fn key_coverage(&self) -> KeyCoverage {
        KeyCoverage::Full
    }
}

#[test]
fn po_04_erasure_layer_dyn_compatible() {
    let mock = MockErasureLayer;
    let dyn_layer: Box<dyn ErasureLayer> = Box::new(mock);
    assert_eq!(dyn_layer.key_coverage(), KeyCoverage::Full);
    assert_eq!(dyn_layer.id().as_bytes(), b"mock-layer-00001");
}

#[test]
fn po_04_erasure_layer_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockErasureLayer>();
    assert_send_sync::<Box<dyn ErasureLayer>>();
    assert_send_sync::<Arc<dyn ErasureLayer>>();
}

#[tokio::test]
async fn po_05_erasure_layer_witness_read_only() {
    let mock = MockErasureLayer;
    let armed_scanner = Scanner::<Armed>::new();
    let scope = LayerScope::empty();
    let witness_res = mock.witness(&scope, &armed_scanner).await;
    assert!(witness_res.is_ok());
}

#[tokio::test]
async fn po_05_erasure_layer_purge_idempotence() {
    let mock = MockErasureLayer;
    let scope = LayerScope::empty();
    let cx = PurgeCx::new(100);

    let first = mock.purge(&scope, &cx).await;
    let second = mock.purge(&scope, &cx).await;

    assert!(first.is_ok());
    assert!(second.is_ok());
    if let (Ok(f), Ok(s)) = (first, second) {
        assert_eq!(f, s);
    }
}
