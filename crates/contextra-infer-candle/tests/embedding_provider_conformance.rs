// FILE-CONTEXT
// STAND: 2026-09-09T00:00:00Z (SESSION: CANDLE-EMBEDDING-PROVIDER)
// ZWECK: Conformance integration tests for CandleEmbedClient as EmbeddingProvider.
// INVARIANTEN: embedding_dim() strictly equals output vector length to prevent HNSW index corruption.

use candle_core::Device;
use contextra_infer_candle::embedding::CandleEmbedInner;
use contextra_infer_candle::model_registry::ModelFingerprint;
use contextra_infer_candle::{CandleEmbedClient, MAX_CANDLE_EMBED_BATCH_SIZE};
use contextra_ports::embedding::EmbeddingError;
use contextra_ports::EmbeddingProvider;
use contextra_types::Result;

struct MockConformantEmbedModel {
    dim: usize,
}

impl CandleEmbedInner for MockConformantEmbedModel {
    fn embed(
        &mut self,
        text: &str,
        _tokenizer: &tokenizers::Tokenizer,
        _device: &Device,
    ) -> Result<Vec<f32>> {
        // Generate deterministic non-zero values based on input length
        Ok(vec![text.len() as f32; self.dim])
    }

    fn dim(&self) -> usize {
        self.dim
    }
}

fn create_test_client(dim: usize) -> CandleEmbedClient {
    let mock_model = Box::new(MockConformantEmbedModel { dim });
    let fingerprint = ModelFingerprint {
        hash: [7u8; 32],
        model_id: "test_nomic_embed.gguf".to_string(),
        quantization: "Q4_K_M".to_string(),
    };
    let tokenizer_bytes = r#"{
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
    let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes()).unwrap();

    CandleEmbedClient::new(Device::Cpu, mock_model, fingerprint, tokenizer)
}

#[tokio::test]
async fn test_embedding_provider_trait_object_conformance() {
    let target_dim = 768;
    let client = create_test_client(target_dim);

    // Box as trait object Box<dyn EmbeddingProvider>
    let provider: Box<dyn EmbeddingProvider> = Box::new(client);

    assert_eq!(provider.provider_name(), "candle");

    let reported_dim = provider.embedding_dim();
    assert_eq!(
        reported_dim, target_dim,
        "embedding_dim() must match target model dim"
    );

    // Single embedding test
    let text = "Contextra pure Rust vector database search context";
    let vector = provider
        .embed(text)
        .await
        .expect("embed() should succeed for valid text");

    assert_eq!(
        vector.len(),
        reported_dim,
        "CRITICAL: embedding_dim() ({reported_dim}) != actual output vector length ({})! Mismatch would corrupt HNSW index.",
        vector.len()
    );
    assert!(!vector.is_empty());
}

#[tokio::test]
async fn test_embedding_provider_batch_conformance() {
    let target_dim = 384;
    let client = create_test_client(target_dim);
    let provider: Box<dyn EmbeddingProvider> = Box::new(client);

    let texts = vec!["First paragraph", "Second paragraph", "Third paragraph"];
    let batch_vectors = provider
        .embed_batch(&texts)
        .await
        .expect("embed_batch() should succeed");

    assert_eq!(batch_vectors.len(), 3);
    for (i, vec) in batch_vectors.iter().enumerate() {
        assert_eq!(
            vec.len(),
            provider.embedding_dim(),
            "Batch item {i} vector length ({}) != reported embedding_dim ({})",
            vec.len(),
            provider.embedding_dim()
        );
    }
}

#[tokio::test]
async fn test_embedding_provider_exact_batch_size_boundary_success_and_rejection() {
    let client = create_test_client(128);
    let provider: Box<dyn EmbeddingProvider> = Box::new(client);

    // Exactly MAX_CANDLE_EMBED_BATCH_SIZE (256 items) MUST succeed
    let exact_boundary_texts: Vec<&str> = vec!["item"; MAX_CANDLE_EMBED_BATCH_SIZE];
    let res_exact = provider.embed_batch(&exact_boundary_texts).await;
    assert!(
        res_exact.is_ok(),
        "Batch of size exactly MAX_CANDLE_EMBED_BATCH_SIZE (256) must succeed"
    );
    assert_eq!(res_exact.unwrap().len(), MAX_CANDLE_EMBED_BATCH_SIZE);

    // MAX_CANDLE_EMBED_BATCH_SIZE + 1 (257 items) MUST fail with EmbeddingError::Unavailable
    let oversized_texts: Vec<&str> = vec!["item"; MAX_CANDLE_EMBED_BATCH_SIZE + 1];
    let res_oversized = provider.embed_batch(&oversized_texts).await;
    assert!(res_oversized.is_err(), "Batch of size 257 must be rejected");
    if let Err(err) = res_oversized {
        assert!(
            matches!(err, EmbeddingError::Unavailable(_)),
            "Expected EmbeddingError::Unavailable, got {err:?}"
        );
        assert!(err.to_string().contains("exceeds Candle max_batch_size"));
    }
}
