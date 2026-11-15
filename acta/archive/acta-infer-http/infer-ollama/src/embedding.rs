// FILE-CONTEXT
// STAND: 2026-08-30T18:54:39Z (SESSION: ed7b7b38)
// ZWECK: `TextEmbeddingEngine`-Implementierung für Ollama Embedding API
// INVARIANTEN: Validiert Embedding-Dimensionen beim ersten Aufruf und sperrt diese fest gegen spätere Abweichungen
// NICHT-OFFENSICHTLICH: Mismatch bei Embedding-Dimensionen liefert `OllamaError::Internal` um stille Index-Korruption zu verhindern
// HOTSPOTS: embed, embed_batch

use crate::client::{OllamaClient, OllamaConfig, DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL};
use crate::error::{OllamaError, Result};
use crate::model_info::known_dimension;
use crate::traits::{BoxFuture as LocalBoxFuture, Embedder};
use parking_lot::Mutex;
use std::sync::Arc;

/// Implementation of `TextEmbeddingEngine` using Ollama's HTTP API.
#[derive(Clone, Debug)]
pub struct OllamaEmbedder {
    client: OllamaClient,
    model: String,
    concurrency: usize,
    expected_dimension: Option<usize>,
    observed_dimension: Arc<Mutex<Option<usize>>>,
}

impl OllamaEmbedder {
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        let base_url_str = base_url.into();
        let model_str = model.into();
        let config = OllamaConfig {
            base_url: base_url_str,
            model: model_str.clone(),
            ..Default::default()
        };
        let expected_dimension = known_dimension(&model_str);
        Self {
            client: OllamaClient::with_config(config),
            model: model_str,
            concurrency: 8,
            expected_dimension,
            observed_dimension: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_config(config: OllamaConfig) -> Self {
        let model = config.model.clone();
        let expected_dimension = known_dimension(&model);
        Self {
            client: OllamaClient::with_config(config),
            model,
            concurrency: 8,
            expected_dimension,
            observed_dimension: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_BASE_URL, DEFAULT_EMBED_MODEL)
    }

    pub fn config(&self) -> &OllamaConfig {
        self.client.config()
    }

    pub fn client(&self) -> &OllamaClient {
        &self.client
    }

    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1);
        self
    }

    pub fn with_expected_dimension(mut self, dim: usize) -> Self {
        self.expected_dimension = Some(dim);
        self
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn embedding_dimension(&self) -> usize {
        if let Some(exp) = self.expected_dimension {
            exp
        } else {
            (*self.observed_dimension.lock()).unwrap_or_default()
        }
    }

    pub(crate) fn check_and_lock_dimension(&self, dim: usize) -> Result<()> {
        let mut lock = self.observed_dimension.lock();
        match *lock {
            None => {
                if let Some(expected_dim) = self.expected_dimension {
                    if dim != expected_dim {
                        return Err(OllamaError::Internal(format!(
                            "Ollama returned embedding of dimension {} but expected {}. Model '{}' may have changed. Rebuild the HNSW index.",
                            dim,
                            expected_dim,
                            self.model
                        )));
                    }
                }
                *lock = Some(dim);
                Ok(())
            }
            Some(observed_dim) => {
                if dim != observed_dim {
                    return Err(OllamaError::Internal(format!(
                        "Ollama embedding dimension mismatch: previously observed {}, but got {} on subsequent call for model '{}'. Silent index corruption prevented.",
                        observed_dim,
                        dim,
                        self.model
                    )));
                }
                Ok(())
            }
        }
    }
}

impl Embedder for OllamaEmbedder {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn embed<'a>(&'a self, text: &'a str) -> LocalBoxFuture<'a, Result<Vec<f32>>> {
        Box::pin(async move {
            let vec = self.client.embed(&self.model, text).await?;
            self.check_and_lock_dimension(vec.len())?;
            Ok(vec)
        })
    }

    fn embedding_dim(&self) -> usize {
        self.embedding_dimension()
    }

    fn embed_batch<'a>(
        &'a self,
        texts: &'a [&'a str],
    ) -> LocalBoxFuture<'a, Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            if texts.is_empty() {
                return Ok(Vec::new());
            }

            let output = self.client.embed_batch(&self.model, texts).await?;
            for vec in &output {
                self.check_and_lock_dimension(vec.len())?;
            }
            Ok(output)
        })
    }
}
