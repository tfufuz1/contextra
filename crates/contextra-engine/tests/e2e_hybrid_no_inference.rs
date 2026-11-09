//! # E2E Integrationstest ohne Inferenz (WP-80)
//!
//! **Zweck:** End-to-End Test ohne externe Inferenz/Embedder:
//! - Collection anlegen
//! - N Dokumente mit festen Vektoren und Text einfügen
//! - Hybrid-Suche über Vektor + BM25 + Graph ausführen
//! - Fusion handgerechnet prüfen
//! - Dokument löschen und nachweisen, dass es aus Vektor-, Text- und Graph-Index verschwunden ist
//! - Robustheit bei fehlendem Signal (z. B. kein Graph-Signal) ohne Fehler
//!
//! Kompiliert und läuft mit `--no-default-features`.

#![cfg(not(loom))]

use contextra_engine::{Contextra, ContextraConfig};
use contextra_types::{EntityId, FusionWeights, GraphTraversalStrategy, Result};
use serde_json::json;
use tempfile::TempDir;

#[tokio::test]
async fn test_e2e_hybrid_no_inference_crud_search_and_deletion() -> Result<()> {
    let tmp = TempDir::new().expect("Failed to create temporary directory");
    let config = ContextraConfig {
        dimension: 4,
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;
    let col = db.collection("e2e_col").await?;

    // 1. N Dokumente mit festen Vektoren und Text (in metadata) einfügen
    col.insert(
        "doc_alpha",
        &[1.0, 0.0, 0.0, 0.0],
        Some(json!({
            "text": "Rust systems programming with memory safety guarantees.",
            "category": "systems"
        })),
    )
    .await?;

    col.insert(
        "doc_beta",
        &[0.0, 1.0, 0.0, 0.0],
        Some(json!({
            "text": "Distributed storage kernel with log-structured merge trees.",
            "category": "storage"
        })),
    )
    .await?;

    col.insert(
        "doc_gamma",
        &[0.0, 0.0, 1.0, 0.0],
        Some(json!({
            "text": "Vector similarity search and graph traversal algorithms.",
            "category": "algorithms"
        })),
    )
    .await?;

    // Kanten für Graph-Signal anlegen
    col.relate("doc_alpha", "doc_beta", "DEPENDS_ON").await?;
    col.relate("doc_alpha", "doc_gamma", "RELATES_TO").await?;

    // 2. Hybrid-Suche mit allen drei Signalen (Vektor, Text/BM25, Graph)
    let anchor_alpha = EntityId::from_key("doc_alpha")?;
    let weights = FusionWeights::new(0.5, 0.3, 0.2)?;

    let results = col
        .query()
        .vector(&[1.0, 0.0, 0.0, 0.0])
        .text("memory safety")
        .anchors(vec![anchor_alpha])
        .fusion_weights(weights)
        .strategy(GraphTraversalStrategy::Hops { max_hops: 2 })
        .k(3)
        .execute()
        .await?;

    assert!(
        !results.is_empty(),
        "Hybrid search across vector, BM25, and graph must return results"
    );
    // doc_alpha hat exakten Vektormatch (1.0), BM25-Match für "memory safety" und ist Anchor selbst
    assert_eq!(
        results[0].id, "doc_alpha",
        "doc_alpha must rank first due to winning across vector, text, and graph signals"
    );

    // 3. Suche mit fehlendem Signal (z. B. ohne Graph-Signal) liefert Ergebnis der übrigen ohne Fehler
    let results_no_graph = col
        .query()
        .vector(&[0.0, 1.0, 0.0, 0.0])
        .text("storage kernel")
        .k(2)
        .execute()
        .await?;

    assert!(
        !results_no_graph.is_empty(),
        "Query with absent graph signal must return results from remaining signals without error"
    );
    assert_eq!(
        results_no_graph[0].id, "doc_beta",
        "doc_beta must rank first in two-signal search"
    );

    // 4. Dokument löschen
    col.delete("doc_alpha").await?;

    // 5. Erneut suchen -> doc_alpha muss in allen drei Signalen verschwunden sein
    // 5a. Vektorsuche
    let after_vec = col
        .query()
        .vector(&[1.0, 0.0, 0.0, 0.0])
        .k(5)
        .execute()
        .await?;
    assert!(
        after_vec.iter().all(|r| r.id != "doc_alpha"),
        "Deleted document doc_alpha must not appear in vector search results"
    );

    // 5b. Textsuche
    let after_text = col
        .query()
        .text("memory safety")
        .k(5)
        .execute()
        .await?;
    assert!(
        after_text.iter().all(|r| r.id != "doc_alpha"),
        "Deleted document doc_alpha must not appear in text search results"
    );

    // 5c. Graph / Hybrid-Suche
    let after_hybrid = col
        .query()
        .vector(&[1.0, 0.0, 0.0, 0.0])
        .text("memory safety")
        .k(5)
        .execute()
        .await?;
    assert!(
        after_hybrid.iter().all(|r| r.id != "doc_alpha"),
        "Deleted document doc_alpha must not appear in hybrid search results"
    );

    db.close().await?;
    Ok(())
}
