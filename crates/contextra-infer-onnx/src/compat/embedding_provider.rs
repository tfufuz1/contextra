//! Brücke für `contextra_ports::EmbeddingProvider`.

use crate::embedder::OnnxEmbedder;
use crate::traits::Embedder;
use contextra_ports::{BoxFuture, EmbeddingError, EmbeddingProvider};

impl EmbeddingProvider for OnnxEmbedder {
    fn provider_name(&self) -> &str {
        "onnx"
    }

    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, std::result::Result<Vec<f32>, EmbeddingError>> {
        let res = Embedder::embed(self, text)
            .map_err(|e| EmbeddingError::ComputationFailed(e.to_string()));
        Box::pin(async move { res })
    }

    fn embedding_dim(&self) -> usize {
        Embedder::dimension(self)
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, EmbeddingError>> {
        let res = Embedder::embed_batch(self, texts)
            .map_err(|e| EmbeddingError::ComputationFailed(e.to_string()));
        Box::pin(async move { res })
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl EmbeddingProvider for crate::fakes::FakeEmbedder {
    fn provider_name(&self) -> &str {
        "fake-onnx"
    }

    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, std::result::Result<Vec<f32>, EmbeddingError>> {
        let res = Embedder::embed(self, text)
            .map_err(|e| EmbeddingError::ComputationFailed(e.to_string()));
        Box::pin(async move { res })
    }

    fn embedding_dim(&self) -> usize {
        Embedder::dimension(self)
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, EmbeddingError>> {
        let res = Embedder::embed_batch(self, texts)
            .map_err(|e| EmbeddingError::ComputationFailed(e.to_string()));
        Box::pin(async move { res })
    }
}
