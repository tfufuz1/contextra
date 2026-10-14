// FILE-CONTEXT
// ZWECK: CRASH-10 Integrationstest: Verwerfen des Aufrufer-Futures nach dem Commit-Punkt führt zum Roll-forward.

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_engine::collection::crud::delete::DocEraseOptions;
use contextra_engine::deletion::forget_options::ForgetOptions;
use contextra_engine::erasure_supervisor::ErasureSupervisor;
use contextra_engine::Collection;
use contextra_graph::CsrGraph;
use contextra_ports::{EraseCtx, ErasureHooks, HookToken};
use contextra_store::{LsmConfig, LsmStorage};
use contextra_types::{ContextraError, TxId};
use contextra_vector::HnswIndex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tempfile::tempdir;

struct TrackingHooks {
    begin_called: AtomicBool,
    tombstoned_called: AtomicBool,
    finish_called: AtomicBool,
}

impl ErasureHooks for TrackingHooks {
    fn begin(&self, _ctx: &EraseCtx) -> Result<HookToken, ContextraError> {
        self.begin_called.store(true, Ordering::SeqCst);
        Ok(HookToken(100))
    }

    fn tombstoned(&self, _token: &HookToken, _tx: TxId) -> Result<(), ContextraError> {
        self.tombstoned_called.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn finish(&self, _token: HookToken) -> Result<Option<contextra_ports::ErasureAttestation>, ContextraError> {
        self.finish_called.store(true, Ordering::SeqCst);
        Ok(None)
    }
}

#[tokio::test]
async fn test_crash10_cancel_after_commit_completes_erasure() {
    let dir = tempdir().expect("tempdir");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("LsmStorage"));
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("HnswIndex"),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    let hooks = Arc::new(TrackingHooks {
        begin_called: AtomicBool::new(false),
        tombstoned_called: AtomicBool::new(false),
        finish_called: AtomicBool::new(false),
    });

    let collection = Arc::new(
        Collection::new(
            "crash10_col".to_string(),
            storage,
            index,
            graph,
            next_tx,
            4,
            contextra_text::Language::English,
        )
        .with_erasure_hooks(hooks.clone()),
    );

    let doc_id = "doc_crash10";
    collection
        .insert(doc_id, &[0.1f32; 4], None)
        .await
        .expect("insert");

    let supervisor = ErasureSupervisor::new();
    let col_clone = collection.clone();
    let doc_id_str = doc_id.to_string();

    // Spawn roll-forward task via supervisor
    let rx = supervisor.spawn_roll_forward(async move {
        let opts = DocEraseOptions {
            options: ForgetOptions {
                assurance: WitAssurance::S0,
                ..ForgetOptions::default()
            },
        };
        col_clone.erase_document(&doc_id_str, &opts).await
    });

    // Drop the receiver (simulating caller future cancellation after dispatch)
    drop(rx);

    // Yield control to allow spawned supervisor task to complete
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Verify CRASH-10 invariants: Document is deleted and hooks completed
    let doc = collection.get(doc_id).await.expect("get");
    assert!(
        doc.is_none(),
        "CRASH-10: Document MUST be tombstoned even if caller dropped future after commit"
    );

    assert!(
        hooks.begin_called.load(Ordering::SeqCst),
        "begin hook must have been called"
    );
    assert!(
        hooks.tombstoned_called.load(Ordering::SeqCst),
        "tombstoned hook must have been called"
    );
    assert!(
        hooks.finish_called.load(Ordering::SeqCst),
        "finish hook must have been called"
    );
}
