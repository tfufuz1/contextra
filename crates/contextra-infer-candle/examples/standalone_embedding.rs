//! Eigenständiges Beispiel zur Demonstration von `contextra-infer-candle` als autarkem Baukasten (M10).

use candle_core::Device;
use contextra_infer_candle::embedding::CandleEmbedInner;
use contextra_infer_candle::{CandleEmbedClient, ModelFingerprint};
use contextra_ports::EmbeddingProvider;
use contextra_types::Result;
use tokenizers::Tokenizer;

struct StandaloneFakeEmbedModel {
    dimension: usize,
}

impl CandleEmbedInner for StandaloneFakeEmbedModel {
    fn embed(&mut self, text: &str, _tokenizer: &Tokenizer, _device: &Device) -> Result<Vec<f32>> {
        let mut vec = vec![0.0f32; self.dimension];
        let bytes = text.as_bytes();
        for (i, b) in bytes.iter().enumerate() {
            if let Some(elem) = vec.get_mut(i % self.dimension) {
                *elem += *b as f32 / 255.0;
            }
        }
        let norm: f32 = vec.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vec {
                *v /= norm;
            }
        }
        Ok(vec)
    }

    fn dim(&self) -> usize {
        self.dimension
    }
}

fn dummy_tokenizer() -> Tokenizer {
    let json = r#"{
        "version": "1.0",
        "truncation": null,
        "padding": null,
        "added_tokens": [],
        "normalizer": null,
        "pre_tokenizer": null,
        "post_processor": null,
        "decoder": null,
        "model": { "type": "BPE", "dropout": null, "unk_token": null, "continuing_subword_prefix": null, "end_of_word_suffix": null, "fuse_unk": false, "vocab": {}, "merges": [] }
    }"#;
    match Tokenizer::from_bytes(json.as_bytes()) {
        Ok(t) => t,
        Err(_) => Tokenizer::new(tokenizers::models::bpe::BPE::default()),
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-infer-candle Standalone Example ===");

    let dim = 384;
    let model = Box::new(StandaloneFakeEmbedModel { dimension: dim });
    let fingerprint = ModelFingerprint {
        hash: [0x42u8; 32],
        model_id: "standalone-bge-micro.gguf".to_string(),
        quantization: "Q4KM".to_string(),
    };
    let tokenizer = dummy_tokenizer();

    let client = CandleEmbedClient::new(Device::Cpu, model, fingerprint, tokenizer);

    let query = "Contextra modular building block architecture";
    let embedding = client.embed(query).await?;

    println!("Query: '{}'", query);
    println!("Embedding dimension: {}", embedding.len());
    if let Some(first_five) = embedding.get(..5) {
        println!("Embedding first 5 dimensions: {:?}", first_five);
    }

    let batch = vec!["Contextra memory", "Deterministic retrieval", "Zero-panic ML"];
    let batch_embeddings = client.embed_batch(&batch).await?;
    println!("Generated batch embeddings count: {}", batch_embeddings.len());

    assert_eq!(embedding.len(), dim);
    assert_eq!(batch_embeddings.len(), 3);
    println!("=== Standalone verification successful ===");

    Ok(())
}
