#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_durability_after_wal_commit_and_simulated_crash() -> contextra_types::Result<()> {
    let tmp_dir = TempDir::new().expect("Failed to create temporary directory");
    let path = tmp_dir.path().to_path_buf();

    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    // Phase 1: Open Contextra, perform committed CRUD operations
    {
        let db = Contextra::open_with_config(&path, config.clone()).await?;

        // Insert documents
        db.insert(
            "doc_durable_1",
            &[0.1, 0.2, 0.3, 0.4],
            Some(json!({ "title": "Durable Doc 1", "status": "active" })),
        )
        .await?;

        db.insert(
            "doc_durable_2",
            &[0.5, 0.6, 0.7, 0.8],
            Some(json!({ "title": "Durable Doc 2", "status": "active" })),
        )
        .await?;

        // KV insert
        db.put_kv("kv_key_1", &json!({ "version": 1, "owner": "test" }))
            .await?;

        // Graph relation
        db.relate("doc_durable_1", "doc_durable_2", "REFERENCES")
            .await?;

        // Force a flush to underlying storage so WAL entries exist on disk
        db.flush().await?;

        // Hard process crash simulation: drop db without calling close() or wait_shutdown()
        drop(db);
    }

    // Phase 2: Verify WAL files or disk files exist after crash
    let mut entries = tokio::fs::read_dir(&path).await.unwrap();
    let mut found_wal = false;
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("wal") && name.ends_with(".log") {
            found_wal = true;
            break;
        }
    }
    assert!(
        found_wal,
        "WAL file (wal-*.log or wal.log) must exist on disk after flush"
    );

    // Phase 3: Reopen engine from disk (triggers automatic repair_on_open)
    let db_recovered = Contextra::open_with_config(&path, config).await?;

    // Phase 4: Assert all committed data is completely intact and unchanged
    let doc1 = db_recovered.get("doc_durable_1").await?;
    assert!(
        doc1.is_some(),
        "doc_durable_1 must exist after WAL crash recovery"
    );
    let doc1_unwrapped = doc1.unwrap();
    assert_eq!(doc1_unwrapped.id, "doc_durable_1");
    assert_eq!(
        doc1_unwrapped
            .metadata
            .as_ref()
            .and_then(|m| m.get("title"))
            .and_then(|v| v.as_str()),
        Some("Durable Doc 1")
    );

    let doc2 = db_recovered.get("doc_durable_2").await?;
    assert!(
        doc2.is_some(),
        "doc_durable_2 must exist after WAL crash recovery"
    );
    let doc2_unwrapped = doc2.unwrap();
    assert_eq!(doc2_unwrapped.id, "doc_durable_2");

    let kv1 = db_recovered.get_kv("kv_key_1").await?;
    assert!(
        kv1.is_some(),
        "kv_key_1 must exist after WAL crash recovery"
    );
    assert_eq!(
        kv1.as_ref()
            .and_then(|v| v.get("version"))
            .and_then(|v| v.as_u64()),
        Some(1)
    );

    // Vector search test post-recovery
    let search_results = db_recovered.search(&[0.1, 0.2, 0.3, 0.4], 2).await?;
    assert!(
        !search_results.is_empty(),
        "Search results must not be empty after recovery"
    );
    assert_eq!(search_results[0].id, "doc_durable_1");

    db_recovered.close().await?;
    Ok(())
}
