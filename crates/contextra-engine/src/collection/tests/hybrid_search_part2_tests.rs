//! # Modul: hybrid_search_tests.rs
//!
//! **Zweck:** Tests für kombinierte Text- und Vektorsuchpfade.
//!
//! **Architektur:** Gehört zur Crate `contextra-engine` (Ring 2).
//! Eingehaltene Invarianten: Fail-Closed bei Fehlern, strikte Mandanten- und Indexisolation.

use super::fixtures::*;

#[tokio::test]
async fn test_hybrid_search_single_fusion_strategy_regression_parity() -> contextra_types::Result<()>
{
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::{FusionStrategy, SignalFusionStrategies};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "regression_parity".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "regression parity test document one"})),
    )
    .await?;

    col.insert(
        "doc_2",
        &[0.5, 0.5, 0.0, 0.0],
        Some(serde_json::json!({"text": "regression parity test document two"})),
    )
    .await?;

    // Calling with single FusionStrategy converted via From<FusionStrategy> (.into())
    let single_rrf: SignalFusionStrategies = FusionStrategy::Rrf.into();
    let uniform_rrf = SignalFusionStrategies::uniform(FusionStrategy::Rrf);

    let res_single = col
        .hybrid_search_with_strategy(
            "regression",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(single_rrf),
        )
        .await?;

    let res_uniform = col
        .hybrid_search_with_strategy(
            "regression",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(uniform_rrf),
        )
        .await?;

    assert_eq!(res_single.len(), res_uniform.len());
    for (a, b) in res_single.iter().zip(res_uniform.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.score.to_bits(), b.score.to_bits());
    }

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_fusion_strategy_rrf_golden_parity() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "golden_parity".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust vector search engine"})),
    )
    .await?;

    col.insert(
        "doc2",
        &[0.8, 0.2, 0.0, 0.0],
        Some(serde_json::json!({"text": "rust database storage"})),
    )
    .await?;

    // Default strategy (None) vs explicit FusionStrategy::Rrf
    let default_res = col
        .hybrid_search_with_strategy(
            "rust",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;

    let explicit_rrf_res = col
        .hybrid_search_with_strategy(
            "rust",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            None,
            Some(FusionStrategy::Rrf.into()),
        )
        .await?;

    assert_eq!(default_res.len(), explicit_rrf_res.len());
    for (a, b) in default_res.iter().zip(explicit_rrf_res.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.score, b.score);
    }

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_score_normalized_reorders_results() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "score_norm_reorder".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    // Insert doc_a, doc_b, doc_c configured to demonstrate RRF vs ScoreNormalized reordering.
    // doc_a: high vector similarity [1.0, 0, 0, 0], moderate text match
    // doc_b: low vector similarity [0.1, 0.9, 0, 0], highest text match
    // doc_c: low vector similarity [0.0, 1.0, 0, 0], medium text match
    col.insert(
        "doc_a",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "search query target alpha"})),
    )
    .await?;

    col.insert(
        "doc_b",
        &[0.1, 0.9, 0.0, 0.0],
        Some(serde_json::json!({"text": "search query target alpha query query"})),
    )
    .await?;

    col.insert(
        "doc_c",
        &[0.0, 1.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "search query target alpha query"})),
    )
    .await?;

    let weights = contextra_types::FusionWeights::new(0.40, 0.30, 0.30)
        .map_err(|e| contextra_types::ContextraError::InvalidInput(e.to_string()))?;

    let rrf_results = col
        .hybrid_search_with_strategy(
            "query",
            &[1.0, 0.0, 0.0, 0.0],
            3,
            None,
            Some(&weights),
            None,
            None,
            Some(FusionStrategy::Rrf.into()),
        )
        .await?;

    let norm_results = col
        .hybrid_search_with_strategy(
            "query",
            &[1.0, 0.0, 0.0, 0.0],
            3,
            None,
            Some(&weights),
            None,
            None,
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    assert_eq!(rrf_results.len(), 3);
    assert_eq!(norm_results.len(), 3);

    // Verify that ScoreNormalized and RRF result orderings differ due to CombSUM score normalization
    let rrf_ids: Vec<&str> = rrf_results.iter().map(|r| r.id.as_str()).collect();
    let norm_ids: Vec<&str> = norm_results.iter().map(|r| r.id.as_str()).collect();

    assert_ne!(
        rrf_ids, norm_ids,
        "ScoreNormalized must produce a different result ordering than RRF in this constructed scenario. RRF: {rrf_ids:?}, ScoreNormalized: {norm_ids:?}"
    );

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_score_normalized_constant_scores_fallback_to_rrf(
) -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "constant_scores_fallback".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    // Insert 2 documents with identical embeddings and identical text content -> constant score distribution
    col.insert(
        "doc_same_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "identical text content"})),
    )
    .await?;

    col.insert(
        "doc_same_2",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "identical text content"})),
    )
    .await?;

    let norm_res = col
        .hybrid_search_with_strategy(
            "identical",
            &[1.0, 0.0, 0.0, 0.0],
            2,
            None,
            None,
            None,
            None,
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    assert_eq!(norm_res.len(), 2);
    // Degenerate score distribution triggers fallback to RRF without panic
    assert!(norm_res[0].score.is_finite());
    assert!(norm_res[1].score.is_finite());

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_score_normalized_determinism_100_runs() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::FusionStrategy;
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "determinism_100_runs".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "deterministic search alpha"})),
    )
    .await?;

    col.insert(
        "doc_2",
        &[0.7, 0.3, 0.0, 0.0],
        Some(serde_json::json!({"text": "deterministic search beta"})),
    )
    .await?;

    col.insert(
        "doc_3",
        &[0.5, 0.5, 0.0, 0.0],
        Some(serde_json::json!({"text": "deterministic search gamma"})),
    )
    .await?;

    let baseline = col
        .hybrid_search_with_strategy(
            "deterministic",
            &[1.0, 0.0, 0.0, 0.0],
            3,
            None,
            None,
            None,
            None,
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    for run_idx in 1..=100 {
        let run_res = col
            .hybrid_search_with_strategy(
                "deterministic",
                &[1.0, 0.0, 0.0, 0.0],
                3,
                None,
                None,
                None,
                None,
                Some(FusionStrategy::ScoreNormalized.into()),
            )
            .await?;

        assert_eq!(
            baseline.len(),
            run_res.len(),
            "Run #{run_idx} result count mismatched baseline"
        );

        for (a, b) in baseline.iter().zip(run_res.iter()) {
            assert_eq!(
                a.id, b.id,
                "Run #{run_idx} document ID order mismatch: expected {}, got {}",
                a.id, b.id
            );
            assert_eq!(
                a.score.to_bits(),
                b.score.to_bits(),
                "Run #{run_idx} score bit representation mismatch for doc {}: expected {}, got {}",
                a.id,
                a.score,
                b.score
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_community_boost_consistency_across_fusion_strategies(
) -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_types::{EntityId, FusionStrategy};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let storage = Arc::new(
        LsmStorage::new(LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        })
        .await?,
    );
    let index = Arc::new(HnswIndex::try_new(HnswConfig {
        dimension: 4,
        ..Default::default()
    })?);
    let col = Collection::new(
        "community_boost_consistency".to_string(),
        storage,
        index,
        Arc::new(CsrGraph::new()),
        Arc::new(AtomicU64::new(1)),
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_comm_target",
        &[0.8, 0.2, 0.0, 0.0],
        Some(serde_json::json!({"text": "community boosted topic"})),
    )
    .await?;

    col.insert(
        "doc_comm_other",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "community boosted topic"})),
    )
    .await?;

    let eid_target = EntityId::from_key("doc_comm_target")?;

    col.relate("doc_comm_target", "doc_neighbor", "knows")
        .await?;
    col.run_community_detection().await?;

    // Test with FusionStrategy::Rrf
    let rrf_boosted = col
        .hybrid_search_with_strategy(
            "community",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            Some(eid_target),
            Some(FusionStrategy::Rrf.into()),
        )
        .await?;

    // Test with FusionStrategy::ScoreNormalized
    let norm_boosted = col
        .hybrid_search_with_strategy(
            "community",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            None,
            Some(eid_target),
            Some(FusionStrategy::ScoreNormalized.into()),
        )
        .await?;

    assert!(!rrf_boosted.is_empty());
    assert!(!norm_boosted.is_empty());

    // In both strategies, community boosting must boost doc_comm_target to rank #1
    assert_eq!(
        rrf_boosted[0].id, "doc_comm_target",
        "Community member must rank #1 under RRF post-fusion boost"
    );
    assert_eq!(
        norm_boosted[0].id, "doc_comm_target",
        "Community member must rank #1 under ScoreNormalized post-fusion boost"
    );

    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_snapshot_unsupported_strategies() -> contextra_types::Result<()> {
    use contextra_graph::csr::CsrGraph;
    use contextra_store::lsm::{LsmConfig, LsmStorage};
    use contextra_vector::{HnswConfig, HnswIndex};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    let dir = tempfile::TempDir::new().map_err(contextra_types::ContextraError::from)?;
    let lsm_config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::new(lsm_config).await?);
    let hnsw_config = HnswConfig {
        dimension: 4,
        ..Default::default()
    };
    let index = Arc::new(HnswIndex::try_new(hnsw_config)?);
    let graph = Arc::new(CsrGraph::new());
    let next_tx = Arc::new(AtomicU64::new(1));
    let col = Collection::new(
        "test_snapshot_unsupported".to_string(),
        storage,
        index,
        graph,
        next_tx,
        4,
        contextra_text::Language::English,
    );

    col.insert(
        "doc_1",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"text": "graph node 1"})),
    )
    .await?;

    let ppr_strat = contextra_types::GraphTraversalStrategy::PersonalizedPageRank(
        contextra_types::PprConfig::default(),
    );
    let ppr_res = col
        .hybrid_search_with_strategy(
            "graph",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            None,
            None,
            Some(&ppr_strat),
            None,
            None,
        )
        .await;

    assert!(
        ppr_res.is_ok(),
        "PPR under snapshot isolation must succeed, got {:?}",
        ppr_res
    );

    let eid_1 = contextra_types::EntityId::from_key("doc_1")?;
    let anchors = vec![eid_1];
    let path_rag_strat = contextra_types::GraphTraversalStrategy::PathRag {
        max_hops: 2,
        sufficiency_threshold: 0.5,
    };
    let path_res = col
        .hybrid_search_with_strategy(
            "graph",
            &[1.0, 0.0, 0.0, 0.0],
            5,
            Some(&anchors),
            None,
            Some(&path_rag_strat),
            None,
            None,
        )
        .await;

    assert!(
        path_res.is_ok(),
        "PathRag under snapshot isolation is supported via path_rag_at, got {:?}",
        path_res
    );

    Ok(())
}
