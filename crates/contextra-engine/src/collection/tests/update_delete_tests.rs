//! # Modul: update_delete_tests.rs
//!
//! **Zweck:** Tests für Dokument-Updates und Löschkonsistenz.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

use super::fixtures::*;

#[tokio::test]
async fn test_update_document_importance_persists_model_id_provenance() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    col.insert("doc_test_prov", &vec, None).await.unwrap(); // unwrap

    col.update_document_importance("doc_test_prov", 0.92, "llama3:latest")
        .await
        .unwrap(); // unwrap

    let doc = col.get("doc_test_prov").await.unwrap().unwrap(); // unwrap
    let meta = doc.metadata.unwrap(); // unwrap

    let imp = meta.get("importance").unwrap();
    let imp_score: contextra_types::MemoryImportance = serde_json::from_value(imp.clone()).unwrap();
    assert_eq!(imp_score.base_score.value(), 0.92);
    assert_eq!(
        meta.get("model_id").and_then(|v| v.as_str()),
        Some("llama3:latest")
    );
}

#[tokio::test]
async fn test_expiry_cleanup_deletes_decayed_working_memory() {
    use contextra_store::LsmStorage;
    use contextra_types::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    );

    let created_tx = TxId::new(10);
    let imp = MemoryImportance::new(
        ImportanceScore::new(0.5),
        DecayFunction::Exponential { half_life_tx: 5 },
        created_tx,
    );

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    col.insert(
        "doc_decayed",
        &vec,
        Some(json!({
            "importance": imp
        })),
    )
    .await
    .unwrap(); // unwrap

    // Advance TxId far enough so effective_score < 0.05
    // At created_tx=10, half_life=5:
    // Tx 10: 0.5 * 1.0 = 0.5
    // Tx 15: 0.5 * 0.5 = 0.25
    // Tx 20: 0.5 * 0.25 = 0.125
    // Tx 25: 0.5 * 0.125 = 0.0625
    // Tx 30: 0.5 * 0.0625 = 0.03125 (< 0.05)
    next_tx.store(35, Ordering::SeqCst);

    let count = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(count, 1, "Decayed working memory document should be reaped");
    assert!(col.get("doc_decayed").await.unwrap().is_none()); // unwrap
}

#[tokio::test]
async fn test_expiry_cleanup_never_deletes_semantic_no_decay() {
    use contextra_store::LsmStorage;
    use contextra_types::{DecayFunction, ImportanceScore, MemoryImportance, TxId};
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        next_tx.clone(),
        4,
        contextra_text::Language::English,
    );

    let created_tx = TxId::new(10);
    let imp = MemoryImportance::new(
        ImportanceScore::new(0.01), // even with base score < 0.05!
        DecayFunction::None,
        created_tx,
    );

    let vec = vec![1.0, 0.0, 0.0, 0.0];
    col.insert(
        "doc_semantic",
        &vec,
        Some(json!({
            "importance": imp
        })),
    )
    .await
    .unwrap(); // unwrap

    // Advance TxId very far
    next_tx.store(100_000, Ordering::SeqCst);

    let count = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(
        count, 0,
        "Semantic document with DecayFunction::None must never be deleted"
    );
    assert!(col.get("doc_semantic").await.unwrap().is_some()); // unwrap
}

#[tokio::test]
async fn test_ttl_missing_created_at_does_not_expire() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_no_created_at",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"ttl_ms": 10})),
    )
    .await
    .unwrap(); // unwrap
    let reaped = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(reaped, 0);
    assert!(col.get("doc_no_created_at").await.unwrap().is_some()); // unwrap
}

#[tokio::test]
async fn test_ttl_zero_does_not_expire() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_zero_ttl",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"created_at_ms": 100, "ttl_ms": 0})),
    )
    .await
    .unwrap(); // unwrap
    let reaped = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(reaped, 0);
    assert!(col.get("doc_zero_ttl").await.unwrap().is_some()); // unwrap
}

#[tokio::test]
async fn test_ttl_overflow_does_not_expire() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use serde_json::json;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let col = Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_overflow",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({"created_at_ms": u64::MAX - 10, "ttl_ms": 100})),
    )
    .await
    .unwrap(); // unwrap
    let reaped = col.trigger_expiry_cleanup().await.unwrap(); // unwrap
    assert_eq!(reaped, 0);
    assert!(col.get("doc_overflow").await.unwrap().is_some()); // unwrap
}

#[tokio::test]
async fn test_t_del_lock_held_during_delete_op_and_commit() {
    // T-DEL-LOCK-1: delete() MUSS kv_locks für die ID während des gesamten
    // delete_op + commit halten.
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use std::time::Duration;
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // unwrap
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(), // unwrap
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(), // unwrap
    );
    let col = Arc::new(Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    col.insert("race_doc", &[1.0, 0.0, 0.0, 0.0], None)
        .await
        .unwrap(); // unwrap

    // Externes Lock auf denselben Shard erwerben, den delete("race_doc") benötigt.
    let guard = col.kv_locks.lock_for("race_doc").await;

    // delete im Hintergrund – muss blockieren, solange wir das Lock halten.
    let col2 = col.clone();
    let handle = tokio::spawn(async move { col2.delete("race_doc").await });

    // Dem Hintergrund-Task Zeit geben, das Lock anzufordern.
    tokio::time::sleep(Duration::from_millis(30)).await;

    // try_lock_for: Mit Fix → None (delete wartet auf das Lock, das wir halten).
    let second_try = col.kv_locks.try_lock_for("race_doc");

    // Externes Lock freigeben → delete kann jetzt abschließen.
    drop(guard);

    handle.await.unwrap().unwrap(); // unwrap

    assert!(
        second_try.is_none(),
        "T-DEL-LOCK-1: delete() muss kv_lock für den gesamten Zeitraum delete_op + commit halten. Wenn dieser Test fehlschlägt, wurde das Lock zu früh freigegeben (Bug B: drop(_guard) vor delete_op)."
    );
}

/// SPEC §5.4, §14.2, B-13 / T-DEL-RACE-1:
/// Deterministischer Race-Test zwischen nebenläufigem Schreiben (Insert/Update)
/// und Löschen (Delete) desselben Dokuments/Clusters.
/// Invariante: Entweder gewinnt das Löschen oder das Schreiben deterministisch;
/// es darf kein inkonsistenter Zustand (z. B. verwaister Vektoreintrag oder Teillöschung) entstehen.
#[tokio::test]
async fn test_delete_write_race_same_cluster() {
    use contextra_store::LsmStorage;
    use contextra_vector::HnswIndex;
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let storage = Arc::new(
        LsmStorage::new(contextra_store::LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await
        .unwrap(),
    );
    let index = Arc::new(
        HnswIndex::try_new(contextra_vector::HnswConfig {
            dimension: 4,
            ..Default::default()
        })
        .unwrap(),
    );
    let col = Arc::new(Collection::new(
        "default".to_string(),
        storage,
        index,
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    ));

    let key = "race_target_doc";
    col.insert(key, &[0.1, 0.2, 0.3, 0.4], None).await.unwrap();

    let (barrier_start, barrier_done) = (
        Arc::new(tokio::sync::Barrier::new(3)),
        Arc::new(tokio::sync::Barrier::new(3)),
    );

    let col_delete = Arc::clone(&col);
    let b_start_del = Arc::clone(&barrier_start);
    let b_done_del = Arc::clone(&barrier_done);
    let del_handle = tokio::spawn(async move {
        b_start_del.wait().await;
        let res = col_delete.delete(key).await;
        b_done_del.wait().await;
        res
    });

    let col_write = Arc::clone(&col);
    let b_start_write = Arc::clone(&barrier_start);
    let b_done_write = Arc::clone(&barrier_done);
    let write_handle = tokio::spawn(async move {
        b_start_write.wait().await;
        let res = col_write.update(key, &[0.5, 0.6, 0.7, 0.8], None).await;
        b_done_write.wait().await;
        res
    });

    barrier_start.wait().await;
    barrier_done.wait().await;

    let del_res = del_handle.await.unwrap();
    let write_res = write_handle.await.unwrap();

    // Entweder delete oder update war zuerst am Lock.
    // Falls delete vor update war, schlägt update typischerweise fehl oder überschreibt sauber.
    // Aber vor allem darf der Speicherzustand konsistent sein:
    let final_doc = col.get(key).await.unwrap();
    if final_doc.is_some() {
        assert!(
            write_res.is_ok(),
            "Wenn das Dokument existiert, muss update erfolgreich gewesen sein"
        );
    } else {
        // Dokument ist gelöscht.
        assert!(
            del_res.is_ok(),
            "Wenn das Dokument gelöscht ist, muss delete erfolgreich gewesen sein"
        );
    }
}
