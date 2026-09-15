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
async fn proof_search_bounded_by_k() {
    let (collection, _dir) = create_test_collection("test_bound", 4).await;

    // Insert 20 documents
    let mut batch = Vec::new();
    for i in 0..20 {
        let vec = vec![0.1 * (i as f32), 0.2, 0.3, 0.4];
        let meta = json!({ "title": format!("doc{}", i), "idx": i, "text": format!("doc content {}", i) });
        batch.push((format!("doc_{}", i), vec, Some(meta)));
    }
    collection.insert_many(&batch).await.unwrap();

    let query_vec = vec![0.1, 0.2, 0.3, 0.4];
    let k = 5;

    // Direct vector search
    #[allow(deprecated)]
    let results = collection.search(&query_vec, k).await.unwrap();
    assert!(
        results.len() <= k,
        "search(k=5) returned {} items, expected <= 5",
        results.len()
    );

    // Hybrid search
    #[allow(deprecated)]
    let hybrid_results = collection
        .hybrid_search("doc", &query_vec, k, None)
        .await
        .unwrap();
    assert!(
        hybrid_results.len() <= k,
        "hybrid_search(k=5) returned {} items, expected <= 5",
        hybrid_results.len()
    );

    // Query builder
    let builder_results = collection
        .query()
        .text("doc")
        .vector(&query_vec)
        .k(k)
        .execute()
        .await
        .unwrap();
    assert!(
        builder_results.len() <= k,
        "query().k(5) returned {} items, expected <= 5",
        builder_results.len()
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
