// FILE-CONTEXT
// STAND: 2026-10-13 (T-2026-LA03)
// ZWECK: Integrationstests für erase_document, Optionenauswertung, Erasure Ledger Vor-Tombstone-Reihenfolge und Zeugen-Überprüfung.
// INVARIANTEN: Zero-Panic, Fail-Closed.

use contextra_crypto::deletion_proof::claim_text::WitAssurance;
use contextra_crypto::deletion_proof::types::DeletionLayer;
use contextra_engine::collection::crud::delete::DocEraseOptions;
use contextra_engine::deletion::forget_options::{ForgetOptions, VerifyDepth};
use contextra_engine::Collection;
use contextra_engine::no_graph_stubs::CsrGraph;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_vector::HnswIndex;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::tempdir;

async fn create_test_collection() -> Collection<LsmStorage, HnswIndex> {
    let dir = tempdir().expect("tempdir creation failed");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(
        LsmStorage::new(lsm_config)
            .await
            .expect("LsmStorage creation failed"),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("HnswIndex creation failed"),
    );
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));

    Collection::new(
        "test_col".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    )
}

#[tokio::test]
async fn test_erase_document_success_with_empty_witnesses() {
    let collection = create_test_collection().await;

    let doc_id = "doc123";
    let opts = DocEraseOptions {
        options: ForgetOptions::default().with_assurance(WitAssurance::S0),
    };

    let result = collection.erase_document(doc_id, &opts).await;
    assert!(
        result.is_ok(),
        "erase_document should succeed with S0 options: {:?}",
        result.err()
    );

    let erasure = result.expect("erasure result expected");
    assert!(
        erasure.witnesses.is_empty(),
        "Witnesses must be empty without real storage scans"
    );
}

#[tokio::test]
async fn test_erase_document_invalid_options_returns_error() {
    let collection = create_test_collection().await;

    let doc_id = "doc123";
    let opts = DocEraseOptions {
        options: ForgetOptions::default().with_assurance(WitAssurance::S4),
    };

    let result = collection.erase_document(doc_id, &opts).await;
    assert!(
        result.is_err(),
        "erase_document must fail for unsupported S4 assurance"
    );
}
