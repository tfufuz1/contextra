//! # Eigenständiges Baukasten-Beispiel für `contextra-infer-onnx`
//!
//! Demonstriert die Verwendung von [`CrossEncoderReranker`] im Passthrough-Modus
//! ganz ohne externe ONNX-Runtime-Bibliotheken oder Netzwerk-Downloads.
//!
//! # Ausführung
//! ```bash
//! cargo run --example fake_reranker -p contextra-infer-onnx
//! ```

use contextra_infer_onnx::reranker::CrossEncoderReranker;
use contextra_ports::Reranker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Contextra Infer ONNX: Baukasten-Beispiel ===");

    // 1. Instanziierung des Rerankers als dyn ports::Reranker
    let reranker: Box<dyn Reranker> = Box::new(CrossEncoderReranker::passthrough());

    // 2. Abfrage und Kandidatendokumente definieren
    let query = "Was sind die Kerninvarianten von Contextra?";
    let candidates = vec![
        "Contextra erzwingt #![forbid(unsafe_code)] in allen sicheren Schichten.".to_string(),
        "Ein lokales Dokumenten-Management-System ohne Serverzwang.".to_string(),
        "Passwort-Manager und Identitätsverwaltung.".to_string(),
    ];

    println!("Suchanfrage: {query}");
    println!("Anzahl Kandidaten: {}", candidates.len());

    // 3. Reranking über das ports::Reranker Trait ausführen
    let results = reranker.rerank(query, &candidates).await?;

    println!("\nErgebnisse nach Reranking:");
    for r in &results {
        let original_text = candidates
            .get(r.original_index)
            .map(|s| s.as_str())
            .unwrap_or("<unbekannt>");
        println!(
            "  Rang {}: Index {}, Score {:.4} | Text: \"{}\"",
            r.index, r.original_index, r.score, original_text
        );
    }

    Ok(())
}
