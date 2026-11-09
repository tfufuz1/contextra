//! Standalone Beispiel für den Fake-/Passthrough-Reranker ohne ONNX Runtime.

use contextra_infer_onnx::traits::Reranker;
use contextra_infer_onnx::FakeReranker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reranker = FakeReranker::new();
    let query = "Contextra Baukasten";
    let candidates = vec![
        "Sichere lokale Speicherkerne für LLM Software",
        "Generische Volltextsuche und Vektorindizes",
        "Contextra Baukasten modulare Reranking Architektur",
    ];

    let results = reranker.score(query, &candidates)?;
    for (i, score) in results.iter().enumerate() {
        println!("Doc {}: score = {:.4}", i, score);
    }
    assert_eq!(results.len(), 3);
    Ok(())
}
