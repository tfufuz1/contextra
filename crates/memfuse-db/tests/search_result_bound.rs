use memfuse_core::DistanceMetric;
use memfuse_db::{Collection, Language};
use memfuse_graph::CsrGraph;
use memfuse_index::{HnswConfig, HnswIndex};
use memfuse_store::{LsmConfig, LsmStorage};
use serde_json::json;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tempfile::TempDir;

async fn create_test_collection(
    name: &str,
    dimension: usize,
) -> (Collection<LsmStorage, HnswIndex>, TempDir) {
    let dir = TempDir::new().expect("tempdir");
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await.expect("storage"));
    let hnsw_config = HnswConfig {
        dimension,
        max_elements: 1000,
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
        name.to_string(),
        storage,
        index,
        graph,
        next_tx,
        dimension,
        Language::English,
    );
    (col, dir)
}

#[tokio::test]
async fn proof_search_never_exceeds_k() {
    let (collection, _dir) = create_test_collection("test_never_exceeds_k", 4).await;

    // Index 500 documents with dummy embeddings
    let mut batch = Vec::new();
    for i in 0..500 {
        let vec = vec![0.1 * ((i % 10) as f32), 0.2, 0.3, 0.4];
        let meta = json!({
            "title": format!("doc{}", i),
            "idx": i,
            "text": format!("doc content number {}", i)
        });
        batch.push((format!("doc_{}", i), vec, Some(meta)));
    }
    collection.insert_many(&batch).await.unwrap();

    let query_vec = vec![0.1, 0.2, 0.3, 0.4];
    let k = 10;

    let results = collection
        .query()
        .text("doc")
        .vector(&query_vec)
        .k(k)
        .execute()
        .await
        .unwrap();

    assert!(
        results.len() <= k,
        "search returned {} items, expected <= {}",
        results.len(),
        k
    );

    // Explicit Regression Guard
    let source = include_str!("../src/collection/search.rs");
    let violations = source
        .lines()
        .filter(|l| l.contains("usize::MAX") && !l.contains("UNBOUNDED-OK"))
        .count();
    assert_eq!(
        violations, 0,
        "B-3 REGRESSION: usize::MAX ohne UNBOUNDED-OK in search.rs: {} Vorkommen",
        violations
    );
}

#[tokio::test]
async fn proof_search_returns_nonzero_results_for_matching_query() {
    let (collection, _dir) = create_test_collection("test_matching_query", 4).await;

    // Index 100 documents containing "rust programming language"
    let mut batch = Vec::new();
    for i in 0..100 {
        let vec = vec![0.1, 0.2, 0.3, 0.4];
        let meta = json!({
            "title": format!("doc{}", i),
            "text": format!("rust programming language document {}", i)
        });
        batch.push((format!("doc_{}", i), vec, Some(meta)));
    }
    collection.insert_many(&batch).await.unwrap();

    let k = 5;
    let results = collection
        .query()
        .text("programming")
        .k(k)
        .execute()
        .await
        .unwrap();

    assert!(
        !results.is_empty(),
        "search('programming') returned 0 results, expected >= 1"
    );
    assert!(
        results.len() <= k,
        "search('programming') returned {} results, expected <= {}",
        results.len(),
        k
    );
}

#[tokio::test]
async fn proof_search_with_k_zero_returns_empty() {
    let (collection, _dir) = create_test_collection("test_k_zero", 4).await;

    let mut batch = Vec::new();
    for i in 0..10 {
        let vec = vec![0.1, 0.2, 0.3, 0.4];
        let meta = json!({ "title": format!("doc{}", i), "text": format!("content {}", i) });
        batch.push((format!("doc_{}", i), vec, Some(meta)));
    }
    collection.insert_many(&batch).await.unwrap();

    let query_vec = vec![0.1, 0.2, 0.3, 0.4];

    let results = collection
        .query()
        .text("content")
        .vector(&query_vec)
        .k(0)
        .execute()
        .await
        .unwrap();

    assert_eq!(
        results.len(),
        0,
        "search with k=0 must return empty vector without panicking"
    );
}

#[test]
fn proof_usize_max_removed_from_search_path() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let search_rs_path = Path::new(&manifest_dir).join("src/collection/search.rs");
    let file_content = std::fs::read_to_string(&search_rs_path)
        .or_else(|_| std::fs::read_to_string("crates/memfuse-db/src/collection/search.rs"))
        .expect("Failed to read search.rs");

    let mut unannotated_found = Vec::new();

    for (line_no, line) in file_content.lines().enumerate() {
        if (line.contains("usize::MAX") || line.contains("u64::MAX"))
            && !line.contains("// UNBOUNDED-OK:")
        {
            unannotated_found.push((line_no + 1, line.trim().to_string()));
        }
    }

    if !unannotated_found.is_empty() {
        panic!(
            "Unbounded MAX found without '// UNBOUNDED-OK:' in search.rs:\n{:#?}",
            unannotated_found
        );
    }
}
