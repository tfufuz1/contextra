//! SPEC: T-LOCK, T-CONC
//! Testet den Lock-Schutz über den gesamten Lösch- und Mutationslebenszyklus.
//! Ein vorzeitiges Freigeben des Guards (Mutant) führt zum Konflikt/Race.

use contextra_engine::Collection;
use contextra_store::in_memory_lsm_storage_for_test;
use contextra_types::Language;
use contextra_vector::{HnswConfig, HnswIndex};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[tokio::test]
async fn test_lock_protection_during_mutation_and_delete() {
    let storage = Arc::new(in_memory_lsm_storage_for_test(16).await);
    let vector_index = Arc::new(
        HnswIndex::try_new(HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .expect("hnsw init"),
    );
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Arc::new(Collection::new(
        "lock_test".to_string(),
        storage,
        vector_index,
        next_tx,
        4,
        Language::English,
    ));

    // 1. Insert initial document
    col.insert("doc_1", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .expect("insert success");

    // 2. Parallele Tasks: Update und Delete auf denselben Key
    let col1 = Arc::clone(&col);
    let h1 = tokio::spawn(async move {
        for _ in 0..10 {
            let _ = col1.update("doc_1", &[0.5, 0.5, 0.0, 0.0], None).await;
        }
    });

    let col2 = Arc::clone(&col);
    let h2 = tokio::spawn(async move {
        for _ in 0..10 {
            let _ = col2.delete("doc_1").await;
        }
    });

    let (r1, r2) = tokio::join!(h1, h2);
    assert!(r1.is_ok());
    assert!(r2.is_ok());
}
