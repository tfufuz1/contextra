#![cfg(not(loom))]
use contextra_engine::{Contextra, ContextraConfig};
use contextra_ports::{BoxFuture, TextEmbeddingEngine};
use contextra_types::{EntityId, FusionWeights, GraphTraversalStrategy, Result};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

struct FakeEmbedder {
    dim: usize,
}

impl TextEmbeddingEngine for FakeEmbedder {
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>> {
        let _dim = self.dim;
        let is_quantum = text.contains("Quantum");
        Box::pin(async move {
            if is_quantum {
                Ok(vec![1.0, 0.0, 0.0, 0.0])
            } else {
                Ok(vec![0.0, 1.0, 0.0, 0.0])
            }
        })
    }
}

#[tokio::test]
async fn test_hybrid_query_fusion_signals_ranking_and_k_zero_boundary(
) -> contextra_types::Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("hybrid_fusion_col").await?;
    col.set_embedder(Arc::new(FakeEmbedder { dim: 4 })).await?;

    // 1. Insert documents with text
    col.insert_text_only(
        "doc_fusion_1",
        "Quantum computing advances in hardware and algorithms.",
        Some(json!({ "title": "Quantum Computing Advances", "topic": "quantum" })),
    )
    .await?;

    col.insert_text_only(
        "doc_fusion_2",
        "Classical distributed systems and cloud architecture.",
        Some(json!({ "title": "Classical Distributed Systems", "topic": "distributed" })),
    )
    .await?;

    col.relate("doc_fusion_1", "doc_fusion_2", "SIMILAR_TOPIC")
        .await?;

    // 2. Execute Hybrid Query combining vector, text, and anchor entities
    let anchor_id = EntityId::from_key("doc_fusion_1")?;

    let custom_weights = FusionWeights::new(0.4, 0.4, 0.2)?;

    let results = col
        .query()
        .text("Quantum Computing")
        .vector(&[1.0, 0.0, 0.0, 0.0])
        .anchors(vec![anchor_id])
        .fusion_weights(custom_weights)
        .strategy(GraphTraversalStrategy::Hops { max_hops: 2 })
        .include_provenance(true)
        .k(2)
        .execute()
        .await?;

    assert!(
        !results.is_empty(),
        "Hybrid query combining vector, text, and graph signals must return results"
    );

    assert_eq!(
        results[0].id, "doc_fusion_1",
        "doc_fusion_1 must be ranked highest due to strong vector, text, and graph match"
    );

    // Verify provenance attachments
    if let Some(ref prov) = results[0].provenance {
        assert!(
            prov.source_collection.is_some(),
            "Provenance must record source collection"
        );
    }

    // 3. Boundary test: Query with k = 0 MUST return empty results vector
    let k_zero_results = col.query().text("Quantum").k(0).execute().await?;

    assert!(
        k_zero_results.is_empty(),
        "Query with k = 0 must return an empty vector without panicking"
    );

    db.close().await?;
    Ok(())
}
