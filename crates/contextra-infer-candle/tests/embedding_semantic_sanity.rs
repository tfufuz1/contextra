// FILE-CONTEXT
// STAND: 2026-10-08
// ZWECK: Semantischer Kosinus-Sanity-Test für Candle-Embeddings mit echtem Modell.
// INVARIANTEN: Ignored by default; requires CONTEXTRA_EMBED_MODEL_DIR; no silent dummy skip when env is set.

use contextra_infer_candle::model_registry::CandleQuantization;
use contextra_infer_candle::CandleEmbedClient;
use contextra_ports::EmbeddingProvider;
use std::path::Path;

/// Computes cosine similarity between two vector slices.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector lengths must match");
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[tokio::test]
#[ignore = "requires CONTEXTRA_EMBED_MODEL_DIR"]
async fn test_embedding_semantic_sanity() -> Result<(), Box<dyn std::error::Error>> {
    let model_dir_env = match std::env::var("CONTEXTRA_EMBED_MODEL_DIR") {
        Ok(val) => val,
        Err(_) => {
            eprintln!("CONTEXTRA_EMBED_MODEL_DIR not set; skipping test_embedding_semantic_sanity.");
            return Ok(());
        }
    };

    let model_dir = Path::new(&model_dir_env);

    let client = CandleEmbedClient::from_dir(model_dir, CandleQuantization::Q4KM)?;

    let emb_hund = client.embed("Hund").await?;
    let emb_welpe = client.embed("Welpe").await?;
    let emb_katze = client.embed("Katze").await?;
    let emb_steuer = client.embed("Steuererklärung").await?;

    let cos_hund_welpe = cosine_similarity(&emb_hund, &emb_welpe);
    let cos_hund_katze = cosine_similarity(&emb_hund, &emb_katze);
    let cos_hund_steuer = cosine_similarity(&emb_hund, &emb_steuer);

    assert!(
        cos_hund_welpe > cos_hund_steuer,
        "Assertion 1 failed: cos(Hund, Welpe)={cos_hund_welpe} must be greater than cos(Hund, Steuererklärung)={cos_hund_steuer}"
    );

    assert!(
        cos_hund_katze > cos_hund_steuer,
        "Assertion 2 failed: cos(Hund, Katze)={cos_hund_katze} must be greater than cos(Hund, Steuererklärung)={cos_hund_steuer}"
    );

    Ok(())
}
