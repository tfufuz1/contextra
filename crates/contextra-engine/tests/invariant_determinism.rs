use contextra_engine::{Contextra, ContextraConfig, SearchResult};
use serde_json::json;
use tempfile::TempDir;

async fn run_sequence(
    dir_path: &std::path::Path,
) -> contextra_types::Result<(Vec<SearchResult>, Option<serde_json::Value>)> {
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(dir_path, config).await?;

    // Perform sequence of operations
    db.insert(
        "doc_det_1",
        &[0.1, 0.2, 0.3, 0.4],
        Some(json!({ "title": "First", "category": "A", "val": 10 })),
    )
    .await?;

    db.insert(
        "doc_det_2",
        &[0.5, 0.6, 0.7, 0.8],
        Some(json!({ "title": "Second", "category": "B", "val": 20 })),
    )
    .await?;

    db.insert(
        "doc_det_3",
        &[0.11, 0.21, 0.31, 0.41],
        Some(json!({ "title": "Third", "category": "A", "val": 30 })),
    )
    .await?;

    db.put_kv("det_kv", &json!({ "alpha": 123, "beta": "xyz" }))
        .await?;

    db.relate("doc_det_1", "doc_det_3", "SIMILAR").await?;

    // Perform search query
    let search_results = db.search(&[0.1, 0.2, 0.3, 0.4], 3).await?;
    let kv_result = db.get_kv("det_kv").await?;

    db.close().await?;

    Ok((search_results, kv_result))
}

#[tokio::test]
async fn test_determinism_p28_identical_sequence_produces_bit_identical_output(
) -> contextra_types::Result<()> {
    let tmp1 = TempDir::new().expect("Failed to create temp dir 1");
    let tmp2 = TempDir::new().expect("Failed to create temp dir 2");

    let (results1, kv1) = run_sequence(tmp1.path()).await?;
    let (results2, kv2) = run_sequence(tmp2.path()).await?;

    // Assert KV outputs are bit-identical
    assert_eq!(kv1, kv2, "KV get output must be bit-identical across runs");

    // Assert search result counts
    assert_eq!(
        results1.len(),
        results2.len(),
        "Search result counts must match"
    );

    // Assert search results (IDs, scores, ranking, and metadata) are bit-identical
    for (res1, res2) in results1.iter().zip(results2.iter()) {
        assert_eq!(
            res1.id, res2.id,
            "Document IDs and ranking must match exactly"
        );
        assert_eq!(
            res1.score.to_bits(),
            res2.score.to_bits(),
            "Float search scores must be bit-identical"
        );
        assert_eq!(
            res1.metadata, res2.metadata,
            "Metadata maps must match exactly"
        );
    }

    Ok(())
}
