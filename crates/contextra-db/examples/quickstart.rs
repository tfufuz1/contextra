//! # Contextra Quickstart
//!
//! # Zweck des Beispiels
//! Dieses Beispiel demonstriert den grundlegenden Arbeitsablauf für Junior-Entwickler:
//! 1. Öffnen (oder Erzeugen) einer Datenbank mit konfigurierter Vektordimension.
//! 2. Einfügen von Dokumenten mit dichten Vektoren (Embeddings) und strukturierten JSON-Metadaten.
//! 3. Semantische Ähnlichkeitssuche mittels Kosinus-Distanz auf dem HNSW-Index.
//! 4. Punktueller Dokumentenzugriff (Direktabfrage nach Primärschlüssel).
//! 5. Geordnetes Schließen und Aufräumen der Datenbank.
//!
//! Ausführen mit: `cargo run --example quickstart`

use contextra_db::{Contextra, ContextraConfig};

// ANCHOR[contextra-db:EXAMPLE-QUICKSTART-001]
/// Hauptfunktion des Schnellstart-Beispiels.
///
/// Zeigt Initialisierung, Dokumentenablage, Vektorsuche und sauberes Beenden.
#[tokio::main]
async fn main() -> contextra_types::Result<()> {
    // 1. Datenbank öffnen (oder neu erzeugen) mit 4-dimensionalen Vektoren
    let config = ContextraConfig {
        dimension: 4,
        ..Default::default()
    };
    let db = Contextra::open_with_config("./example_data", config).await?;

    // 2. Dokumente mit dichten Vektoren und Metadaten einfügen
    db.insert(
        "rust-lang",
        &[1.0, 0.0, 0.0, 0.0],
        Some(serde_json::json!({"topic": "programming", "text": "Rust is a systems programming language"})),
    )
    .await?;

    db.insert(
        "python-lang",
        &[0.9, 0.1, 0.0, 0.0],
        Some(serde_json::json!({"topic": "programming", "text": "Python is great for AI and data science"})),
    )
    .await?;

    db.insert(
        "cooking-101",
        &[0.0, 0.0, 1.0, 0.0],
        Some(serde_json::json!({"topic": "cooking", "text": "How to make perfect pasta"})),
    )
    .await?;

    // 3. Semantische Vektorsuche — die 2 ähnlichsten Dokumente finden
    let query = [0.95, 0.05, 0.0, 0.0];
    let results = db.search(&query, 2).await?;

    println!("=== Semantic Search Results ===");
    for result in &results {
        println!("  {} (score: {:.4})", result.id, result.score);
        if let Some(meta) = &result.metadata {
            println!("    metadata: {}", meta);
        }
    }

    // 4. Einzelnes Dokument direkt über Primärschlüssel abrufen
    if let Some(doc) = db.get("rust-lang").await? {
        println!("\n=== Direct Lookup ===");
        println!("  Found: {} {:?}", doc.id, doc.metadata);
    }

    // 5. Datenbank schließen und Beispieldaten auf Datenträger entfernen
    db.close().await?;

    // Remove example data directory
    let _ = tokio::fs::remove_dir_all("./example_data").await;

    println!("\n✅ Quickstart complete!");
    Ok(())
}
