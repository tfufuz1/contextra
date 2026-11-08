// ZWECK: Trait-based EmbeddingProvider implementation for CandleEmbedClient.
// INVARIANTEN: No block-on in async context; spawn_blocking for CPU inference; embedding_dim strictly matches model dimension.

use contextra_ports::embedding::EmbeddingError;
use contextra_ports::{BoxFuture, EmbeddingProvider};
use contextra_types::{ContextraError, Result};
use std::sync::Arc;

use crate::embedding::CandleEmbedClient;

/// Maximum batch size for Candle GGUF embeddings.
///
/// Set to 256 (lower than ONNX's 512) because GGUF tensor activations and intermediate FP32/FP16 buffers
/// consume higher peak VRAM/RAM per batch item during Candle model forward passes.
pub const MAX_CANDLE_EMBED_BATCH_SIZE: usize = 256;

impl EmbeddingProvider for CandleEmbedClient {
    fn provider_name(&self) -> &str {
        "candle"
    }

    fn embedding_dim(&self) -> usize {
        // Derived dynamically from model/fingerprint metadata stored in self.dim
        self.dim
    }

    fn embed<'a>(
        &'a self,
        text: &'a str,
    ) -> BoxFuture<'a, std::result::Result<Vec<f32>, EmbeddingError>> {
        let model = Arc::clone(&self.model);
        let tokenizer = self.tokenizer.clone();
        let device = self.device.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let text_owned = text.to_string();

        Box::pin(async move {
            let _permit = semaphore.acquire().await.map_err(|_| {
                EmbeddingError::ComputationFailed("Candle embed semaphore closed".into())
            })?;

            tokio::task::spawn_blocking(move || {
                let mut guard = model.blocking_lock();
                guard
                    .embed(&text_owned, &tokenizer, &device)
                    .map_err(|e| EmbeddingError::ComputationFailed(e.to_string()))
            })
            .await
            .map_err(|e| {
                EmbeddingError::ComputationFailed(format!("Candle task join error: {e}"))
            })?
        })
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> BoxFuture<'a, std::result::Result<Vec<Vec<f32>>, EmbeddingError>> {
        let model = Arc::clone(&self.model);
        let tokenizer = self.tokenizer.clone();
        let device = self.device.clone();
        let semaphore = Arc::clone(&self.semaphore);

        Box::pin(async move {
            let limit = MAX_CANDLE_EMBED_BATCH_SIZE;
            if texts.len() > limit {
                return Err(EmbeddingError::Unavailable(format!(
                    "Batch size {} exceeds Candle max_batch_size {limit}. Split into smaller batches.",
                    texts.len()
                )));
            }

            let _permit = semaphore.acquire().await.map_err(|_| {
                EmbeddingError::ComputationFailed("Candle embed semaphore closed".into())
            })?;

            let texts_owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();

            tokio::task::spawn_blocking(move || {
                let mut guard = model.blocking_lock();
                let mut results = Vec::with_capacity(texts_owned.len());
                for text in &texts_owned {
                    let vec = guard
                        .embed(text, &tokenizer, &device)
                        .map_err(|e| EmbeddingError::ComputationFailed(e.to_string()))?;
                    results.push(vec);
                }
                Ok(results)
            })
            .await
            .map_err(|e| {
                EmbeddingError::ComputationFailed(format!("Candle task join error: {e}"))
            })?
        })
    }
}

/// Sichere Startup-Probe zur Überprüfung der Funktionalität und Dimensionsplausibilität
/// eines `EmbeddingProvider`s vor Inbetriebnahme.
pub fn verify_embedding_provider(
    provider: &dyn EmbeddingProvider,
    expected_dim: Option<usize>,
) -> Result<()> {
    let probe_text = "contextra startup probe";
    let probe_fut = provider.embed(probe_text);

    let vec = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        match handle.runtime_flavor() {
            tokio::runtime::RuntimeFlavor::MultiThread => {
                tokio::task::block_in_place(|| handle.block_on(probe_fut)).map_err(|e| {
                    ContextraError::Internal(format!(
                        "Embedding-Selbsttest Probe fehlgeschlagen / Embedding probe failed: {e}"
                    ))
                })?
            }
            _ => std::thread::scope(|s| {
                s.spawn(|| {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .map_err(|e| {
                                ContextraError::Internal(format!(
                                    "Konnte Tokio-Laufzeit für Probe nicht erstellen: {e}"
                                ))
                            })?;
                        rt.block_on(probe_fut).map_err(|e| {
                            ContextraError::Internal(format!(
                                "Embedding-Selbsttest Probe fehlgeschlagen / Embedding probe failed: {e}"
                            ))
                        })
                    })
                    .join()
                    .map_err(|_| {
                        ContextraError::Internal("Probe worker thread panicked".to_string())
                    })?
            })?,
        }
    } else {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| {
                ContextraError::Internal(format!(
                    "Konnte Tokio-Laufzeit für Probe nicht erstellen: {e}"
                ))
            })?;
        rt.block_on(probe_fut).map_err(|e| {
            ContextraError::Internal(format!(
                "Embedding-Selbsttest Probe fehlgeschlagen / Embedding probe failed: {e}"
            ))
        })?
    };

    // Probe check 1: Dimension > 0
    if vec.is_empty() {
        return Err(ContextraError::Internal(
            "Embedding-Selbsttest fehlgeschlagen: Vektordimension ist 0 / Embedding self-test failed: vector dimension is 0".to_string(),
        ));
    }

    // Probe check 2: Vector is not completely zero
    let is_all_zero = vec.iter().all(|&x| x == 0f32);
    if is_all_zero {
        return Err(ContextraError::Internal(
            "Embedding-Selbsttest fehlgeschlagen: Vektor ist komplett Null / Embedding self-test failed: vector is all zeros".to_string(),
        ));
    }

    // Probe check 3: Norm is plausible (finite, non-zero, within 1e-6..1e6)
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !norm.is_finite() || !(1e-6..=1e6).contains(&norm) {
        return Err(ContextraError::Internal(format!(
            "Embedding-Selbsttest fehlgeschlagen: Unplausibler Normwert {norm} / Embedding self-test failed: implausible norm {norm}"
        )));
    }

    // Probe check 4: Dimension matches expected_dim if provided
    if let Some(exp) = expected_dim {
        if vec.len() != exp {
            return Err(ContextraError::Internal(format!(
                "Embedding-Selbsttest fehlgeschlagen: Dimension {got} stimmt nicht mit erwarteter Indexdimension {exp} überein / Dimension mismatch: got {got}, expected {exp}",
                got = vec.len()
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::DefaultCandleEmbedModel;
    use crate::model_registry::ModelFingerprint;
    use candle_core::Device;

    #[tokio::test]
    async fn test_embed_batch_internal() {
        let model = Box::new(DefaultCandleEmbedModel { dim: 16 });
        let fingerprint = ModelFingerprint {
            hash: [1u8; 32],
            model_id: "test.gguf".to_string(),
            quantization: "Q4KM".to_string(),
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
        let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
            .map_err(|e| e.to_string())
            .expect("valid tokenizer bytes");
        let client = CandleEmbedClient::new(Device::Cpu, model, fingerprint, tokenizer);

        let texts = ["alpha", "beta"];
        let res = client
            .embed_batch(&texts)
            .await
            .expect("batch embedding should succeed");
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn test_verify_embedding_provider_valid() {
        let model = Box::new(DefaultCandleEmbedModel { dim: 8 });
        let fingerprint = ModelFingerprint {
            hash: [1u8; 32],
            model_id: "test.gguf".to_string(),
            quantization: "Q4KM".to_string(),
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
        let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
            .map_err(|e| e.to_string())
            .expect("valid tokenizer bytes");
        let client = CandleEmbedClient::new(Device::Cpu, model, fingerprint, tokenizer);

        let res = verify_embedding_provider(&client, Some(8));
        assert!(res.is_ok());

        let res_wrong_dim = verify_embedding_provider(&client, Some(16));
        assert!(res_wrong_dim.is_err());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn test_verify_embedding_provider_in_current_thread_runtime() {
        let model = Box::new(DefaultCandleEmbedModel { dim: 8 });
        let fingerprint = ModelFingerprint {
            hash: [1u8; 32],
            model_id: "test.gguf".to_string(),
            quantization: "Q4KM".to_string(),
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
        let tokenizer = tokenizers::Tokenizer::from_bytes(tokenizer_bytes.as_bytes())
            .map_err(|e| e.to_string())
            .expect("valid tokenizer bytes");
        let client = CandleEmbedClient::new(Device::Cpu, model, fingerprint, tokenizer);

        let res = verify_embedding_provider(&client, Some(8));
        assert!(res.is_ok());
    }
}
