use memfuse_core::{DistanceMetric, HybridQuery};
use memfuse_db::{Collection, Language};
use memfuse_graph::CsrGraph;
use memfuse_index::{HnswConfig, HnswIndex};
use memfuse_store::{LsmConfig, LsmStorage};
use serde_json::json;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::TempDir;

#[test]
fn proof_usize_max_removed_from_search_path() {
    let search_rs_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("collection")
        .join("search.rs");

    let content = std::fs::read_to_string(&search_rs_path)
        .expect("Failed to read src/collection/search.rs");

    let mut unapproved_usize_max = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        if line.contains("usize::MAX") && !line.contains("// UNBOUNDED-OK") {
            unapproved_usize_max.push((line_num + 1, line.trim().to_string()));
        }
    }

    if !unapproved_usize_max.is_empty() {
        panic!(
            "Found unapproved usize::MAX calls in search.rs without '// UNBOUNDED-OK':\n{:#?}",
            unapproved_usize_max
        );
    }
}

#[tokio::test]
async fn proof_search_bounded_by_k() {
    let dir = TempDir::new().expect("tempdir");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("storage"));
    let hnsw_config = HnswConfig {
        dimension: 4,
        max_elements: 100,
        m: 16,
        ef_construction: 64,
        ef_search: 64,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config).expect("hnsw index"));
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "test_bound".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        Language::English,
    );

    // Insert 20 documents
    for i in 1..=20 {
        let doc_id = format!("doc_{i}");
        let embedding = vec![0.1 * (i as f32), 0.2, 0.3, 0.4];
        let meta = json!({ "content": format!("sample document content number {i}") });
        col.insert(&doc_id, &embedding, Some(meta))
            .await
            .expect("insert doc");
    }

    let k = 3;
    let query = HybridQuery::builder()
        .with_text_query("sample content")
        .with_vector_query(vec![0.1, 0.2, 0.3, 0.4])
        .with_k(k)
        .build()
        .expect("build hybrid query");

    #[allow(deprecated)]
    let results = col
        .hybrid_search_with_query(&query)
        .await
        .expect("hybrid search");

    assert!(
        results.len() <= k,
        "Expected at most {k} results, got {}",
        results.len()
    );
}
