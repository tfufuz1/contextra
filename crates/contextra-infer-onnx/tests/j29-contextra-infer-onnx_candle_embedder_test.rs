// FILE-CONTEXT
// STAND: 2026-09-17T00:00:00Z
// ZWECK: Integration test for create_candle_embedder wiring in contextra-infer-onnx.

#[cfg(feature = "candle-backend")]
mod candle_embedder_tests {
    use contextra_infer_candle::CandleQuantization;
    use contextra_infer_onnx::{create_candle_embedder, CandleEmbedClient};
    use contextra_ports::EmbeddingProvider;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_candle_embedder_returns_valid_embedding_provider() {
        let temp_dir = tempdir().unwrap();
        // CandleEmbedClient::from_dir succeeds on a valid temp dir (using DefaultCandleEmbedModel with dim=384)
        let client = CandleEmbedClient::from_dir(temp_dir.path(), CandleQuantization::Q4KM)
            .expect("Failed to create CandleEmbedClient from temp dir");

        let provider: Box<dyn EmbeddingProvider> = create_candle_embedder(client);

        assert_eq!(provider.provider_name(), "candle");
        assert_eq!(provider.embedding_dim(), 384);

        let vec = provider
            .embed("contextra test sentence")
            .await
            .expect("embed call on Candle provider trait object should succeed");

        assert_eq!(vec.len(), 384);
    }
}
