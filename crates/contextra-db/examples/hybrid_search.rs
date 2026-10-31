//! # Hybrid Search — BM25 + Vector Fusion
//!
//! # Zweck des Beispiels
//! Demonstriert die hybride Multi-Signal-Suche in Contextra:
//! - Kombination aus dichten Vektoren (semantische Ähnlichkeit via HNSW)
//! - und Volltext-Keyword-Scoring (BM25 auf dem Text-Feld der JSON-Metadaten).
//! - Beide Ranglisten werden mathematisch via **Reciprocal Rank Fusion (RRF)** fusioniert.
//!
//! # Warum RRF?
//! RRF eliminiert das Problem unterschiedlicher Score-Skalen (Kosinus-Distanz vs. BM25-Log-Odds),
//! indem nur die Rangpositionen der Treffer in den jeweiligen Teillisten gewichtet werden:
//! `RRF(d) = sum( 1 / (60 + rank_i(d)) )`.
//!
//! Ausführen mit: `cargo run --example hybrid_search`

use contextra_db::{Contextra, ContextraConfig};

// ANCHOR[contextra-db:EXAMPLE-HYBRID-001]
/// Einstiegspunkt für die Demonstration der Hybrid-Suche.
///
/// Vergleicht eine reine Vektorsuche mit einer kombinierten Hybrid-Suche (BM25 + Vektor).
#[tokio::main]
async fn main() -> contextra_types::Result<()> {
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config("./hybrid_data", config).await?;

    // Dokumente mit Vektor UND Text-Metadaten einfügen.
    // Contextra indiziert das Feld "text" automatisch im BM25-Invertierten-Index.
    let docs = vec![
        (
            "doc-rust".to_string(),
            vec![1.0, 0.0, 0.0, 0.0],
            Some(serde_json::json!({
                "text": "Rust provides memory safety without garbage collection through ownership",
                "category": "systems"
            })),
        ),
        (
            "doc-python".to_string(),
            vec![0.8, 0.2, 0.0, 0.0],
            Some(serde_json::json!({
                "text": "Python excels at rapid prototyping with its dynamic type system",
                "category": "scripting"
            })),
        ),
        (
            "doc-gc".to_string(),
            vec![0.5, 0.5, 0.0, 0.0],
            Some(serde_json::json!({
                "text": "Garbage collection strategies in modern programming languages",
                "category": "systems"
            })),
        ),
        (
            "doc-ml".to_string(),
            vec![0.0, 1.0, 0.0, 0.0],
            Some(serde_json::json!({
                "text": "Machine learning frameworks for training neural networks",
                "category": "ai"
            })),
        ),
    ];

    // Batch-Insert zur effizienten Ablage mehrerer Dokumente in einem Durchlauf
    db.insert_many(&docs).await?;

    // --- Hybrid Search ---
    // Kombiniert:
    //   • BM25-Stichwortabgleich auf "memory safety garbage collection"
    //   • Vektorähnlichkeit zur Anfrage-Einbettung [0.9, 0.1, 0.0, 0.0]
    // Ergebnisse werden via Reciprocal Rank Fusion (RRF) zusammengeführt.
    let text_query = "memory safety garbage collection";
    let vector_query = [0.9, 0.1, 0.0, 0.0];

    let results = db.hybrid_search(text_query, &vector_query, 3, None).await?;

    println!("=== Hybrid Search: '{}' ===", text_query);
    for (i, result) in results.iter().enumerate() {
        println!(
            "  {}. {} (fused score: {:.4})",
            i + 1,
            result.id,
            result.score
        );
        if let Some(meta) = &result.metadata {
            if let Some(text) = meta.get("text").and_then(|v| v.as_str()) {
                println!("     text: \"{}\"", text);
            }
        }
    }

    // --- Vergleich: Reine Vektorsuche (ohne Textfilterung) ---
    let vector_only = db.search(&vector_query, 3).await?;

    println!("\n=== Vector-Only Search (same embedding) ===");
    for (i, result) in vector_only.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i + 1, result.id, result.score);
    }

    db.close().await?;
    let _ = tokio::fs::remove_dir_all("./hybrid_data").await;

    println!("\n✅ Hybrid search demo complete!");
    Ok(())
}
