#![cfg(not(loom))]

use contextra_engine::collection::{
    Collection, LoadPolicy, StoredDocument, STORED_DOCUMENT_MAGIC, STORED_DOCUMENT_VERSION_V2,
};
use contextra_graph::CsrGraph;
use contextra_ports::StorageEngine;
use contextra_store::{LsmConfig, LsmStorage};
use contextra_text::Language;
use contextra_types::{DocId, Result};
use contextra_vector::{HnswConfig, HnswIndex};
use serde::Serialize;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

#[derive(Serialize)]
struct V1DocHelper<'a> {
    id: &'a str,
    embedding: &'a [f32],
    metadata: Option<serde_json::Value>,
}

#[tokio::test]
async fn legacy_stored_document_v1() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| contextra_types::ContextraError::Storage(e.to_string()))?;
    let db_path = temp_dir.path().to_path_buf();
    let dimension = 16;

    let storage = Arc::new(LsmStorage::new(LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    }).await?);

    let hnsw = Arc::new(HnswIndex::try_new(HnswConfig {
        m: 4,
        ef_construction: 8,
        ef_search: 8,
        dimension,
        ..Default::default()
    })?);

    let col = Collection::new(
        "v1_test".into(),
        storage.clone(),
        hnsw.clone(),
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        dimension,
        Language::English,
    );

    // 1. Populate storage with 50 v1 JSON fixture documents
    let fixture_dir = Path::new("crates/contextra-engine/tests/fixtures/stored_document_v1");
    let tx = col.allocate_tx()?;

    for i in 0..50 {
        let file_path = fixture_dir.join(format!("doc_{i}.json"));
        let json_data = std::fs::read_to_string(&file_path)
            .map_err(|e| contextra_types::ContextraError::Storage(e.to_string()))?;

        let doc: StoredDocument = serde_json::from_str(&json_data)
            .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;

        // Write raw v1 JSON text to storage as user key
        let user_key = col.namespaced_key(doc.id.as_bytes(), 0);
        col.storage()
            .put(tx, &user_key, json_data.as_bytes())
            .await?;

        // Write doc_key metadata
        let doc_id = DocId::from_key(&doc.id)?;
        let doc_key = col.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        let meta_data = serde_json::to_vec(&contextra_engine::collection::StoredDocumentMeta::from(&doc))
            .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;
        col.storage().put(tx, &doc_key, &meta_data).await?;
    }

    col.storage().commit(tx).await?;

    // Verify raw on-disk bytes before migration start with '{' (JSON text format v1)
    let sample_user_key = col.namespaced_key(b"doc_0", 0);
    let raw_before = col
        .storage()
        .get(&sample_user_key)
        .await?
        .expect("doc_0 must exist in storage");
    assert_eq!(
        raw_before[0], b'{',
        "Unmigrated doc_0 must start with '{' (JSON v1)"
    );

    // 2. Perform initial search / load index with migration
    let query_vector = vec![0.5f32; dimension];

    let report_1 = col.load_index_with_report(LoadPolicy::Strict).await?;

    assert_eq!(report_1.scanned, 50, "Must scan 50 documents");
    assert_eq!(report_1.decoded, 50, "Must decode 50 documents");
    assert_eq!(
        report_1.migrated_v1, 50,
        "First load_index_with_report must migrate 50 v1 documents to v2"
    );
    assert_eq!(report_1.skipped_undecodable, 0);
    assert_eq!(report_1.failed_inserts, 0);

    #[allow(deprecated)]
    let search_results_after_migration = col.search_filtered(&query_vector, 5, None).await?;
    assert_eq!(
        search_results_after_migration.len(),
        5,
        "Search must return 5 results"
    );

    // 3. Verify on-disk records are now stored as v2 binary payloads
    let raw_after = col
        .storage()
        .get(&sample_user_key)
        .await?
        .expect("doc_0 must exist in storage");
    assert_eq!(
        &raw_after[0..4],
        STORED_DOCUMENT_MAGIC,
        "Migrated doc_0 must start with magic header 'CXSD'"
    );
    assert_eq!(
        raw_after[4], STORED_DOCUMENT_VERSION_V2,
        "Migrated doc_0 must have version byte 0x02"
    );

    // Verify decoding of raw_after produces original doc_0 contents
    let decoded_after = StoredDocument::from_slice(&raw_after)?;
    assert_eq!(decoded_after.id, "doc_0");
    assert_eq!(decoded_after.embedding.len(), dimension);

    // 4. Second open / load_index migrates nothing (idempotency check)
    let report_2 = col.load_index_with_report(LoadPolicy::Strict).await?;
    assert_eq!(report_2.scanned, 50);
    assert_eq!(report_2.decoded, 50);
    assert_eq!(
        report_2.migrated_v1, 0,
        "Second open/load_index must migrate 0 documents"
    );

    Ok(())
}

#[tokio::test]
async fn measure_v1_vs_v2_10k_synthetic() -> Result<()> {
    let dir_v1 = tempfile::tempdir().map_err(|e| contextra_types::ContextraError::Storage(e.to_string()))?;
    let dir_v2 = tempfile::tempdir().map_err(|e| contextra_types::ContextraError::Storage(e.to_string()))?;
    let dim = 16;
    let count = 10_000;
    let batch_size = 1_000;

    let storage_v1 = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir_v1.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let storage_v2 = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir_v2.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );

    let hnsw_v1 = Arc::new(HnswIndex::try_new(HnswConfig {
        m: 4,
        ef_construction: 8,
        ef_search: 8,
        dimension: dim,
        ..Default::default()
    })?);
    let hnsw_v2 = Arc::new(HnswIndex::try_new(HnswConfig {
        m: 4,
        ef_construction: 8,
        ef_search: 8,
        dimension: dim,
        ..Default::default()
    })?);

    let col_v1 = Collection::new(
        "v1_bench".into(),
        storage_v1,
        hnsw_v1,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        dim,
        Language::English,
    );
    let col_v2 = Collection::new(
        "v2_bench".into(),
        storage_v2,
        hnsw_v2,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        dim,
        Language::English,
    );

    let vec_data = vec![0.123456f32; dim];
    let mut total_v1_bytes = 0usize;

    // Write 10k synthetic documents
    for chunk in 0..(count / batch_size) {
        let tx_v1 = col_v1.allocate_tx()?;
        let tx_v2 = col_v2.allocate_tx()?;

        for idx in 0..batch_size {
            let i = chunk * batch_size + idx;
            let id = format!("bench_doc_{i}");
            let doc = StoredDocument {
                id: id.clone(),
                embedding: vec_data.clone(),
                metadata: Some(
                    serde_json::json!({ "index": i, "content": "Synthetic document content for benchmark" }),
                ),
            };

            // Genuine v1 JSON text
            let v1_helper = V1DocHelper {
                id: &doc.id,
                embedding: &doc.embedding,
                metadata: doc.metadata.clone(),
            };
            let v1_json = serde_json::to_vec(&v1_helper)
                .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;
            total_v1_bytes += v1_json.len();
            let k1 = col_v1.namespaced_key(id.as_bytes(), 0);
            col_v1.storage().put(tx_v1, &k1, &v1_json).await?;

            // Native v2 binary payload
            let k2 = col_v2.namespaced_key(id.as_bytes(), 0);
            col_v2.insert_op(&crate::transaction::DbTransaction::new(&col_v2, tx_v2), &id, &vec_data, doc.metadata.clone()).await?;
        }

        col_v1.storage().commit(tx_v1).await?;
        col_v2.storage().commit(tx_v2).await?;
    }

    // Benchmark load_index time
    let start_v1 = std::time::Instant::now();
    let rep_v1 = col_v1.load_index_with_report(LoadPolicy::Strict).await?;
    let duration_v1 = start_v1.elapsed();

    let start_v2 = std::time::Instant::now();
    let rep_v2 = col_v2.load_index_with_report(LoadPolicy::Strict).await?;
    let duration_v2 = start_v2.elapsed();

    let avg_v1_bytes = total_v1_bytes as f64 / count as f64;

    // Calculate total on-disk bytes for v2 collection
    let sample_k2 = col_v2.namespaced_key(b"bench_doc_0", 0);
    let sample_v2_payload = col_v2.storage().get(&sample_k2).await?.unwrap();
    let avg_v2_bytes = sample_v2_payload.len() as f64;

    eprintln!("\n=== Benchmark StoredDocument v1 vs v2 (10,000 Documents, dim={dim}) ===");
    eprintln!(
        "Format v1 (JSON text):    avg {:7.2} bytes/doc | load_index + migration time: {:?}",
        avg_v1_bytes, duration_v1
    );
    eprintln!(
        "Format v2 (CXSD binary): avg {:7.2} bytes/doc | load_index time: {:?}",
        avg_v2_bytes, duration_v2
    );
    eprintln!(
        "Migration count: v1 -> v2 = {} docs migrated",
        rep_v1.migrated_v1
    );
    eprintln!(
        "Loaded doc counts: v1 col = {}, v2 col = {}",
        rep_v1.decoded, rep_v2.decoded
    );
    eprintln!("========================================================================\n");

    assert_eq!(rep_v1.migrated_v1, count as u64);
    assert_eq!(rep_v2.migrated_v1, 0);

    Ok(())
}
