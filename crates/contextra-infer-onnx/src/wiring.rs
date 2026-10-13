// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Composition Root Wiring for CrossEncoderReranker (§20.2 / Rule 7).

use std::sync::Arc;
use contextra_ports::Reranker;
use contextra_types::{ContextraError, Result};

use crate::reranker::{CrossEncoderReranker, RerankConfig};

/// Dependencies required to build `RerankWiring`.
#[derive(Debug, Clone)]
pub struct RerankWiringDeps {
    /// Reranker configuration.
    pub config: RerankConfig,
    /// Explicit activation flag (default off).
    pub enabled: bool,
}

impl Default for RerankWiringDeps {
    fn default() -> Self {
        Self {
            config: RerankConfig::default(),
            enabled: false,
        }
    }
}

/// Composition Root Wiring wrapper for `CrossEncoderReranker`.
pub struct RerankWiring {
    reranker: Arc<dyn Reranker>,
    config: RerankConfig,
    enabled: bool,
}

impl std::fmt::Debug for RerankWiring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RerankWiring")
            .field("config", &self.config)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl RerankWiring {
    /// Constructs a new `RerankWiring` from `RerankWiringDeps` per Rule 7 contract.
    ///
    /// If `deps.enabled` is `false`, returns a passthrough reranker.
    /// If `deps.enabled` is `true`, constructs a `CrossEncoderReranker`. Missing or invalid
    /// model files yield an explicit `ContextraError` (never a silent no-op).
    pub fn build(deps: RerankWiringDeps) -> Result<Self> {
        let config = deps.config;
        let enabled = deps.enabled;

        let reranker: Arc<dyn Reranker> = if !enabled {
            Arc::new(CrossEncoderReranker::passthrough_with_config(config.clone()))
        } else {
            // Check file existence explicitly to ensure a clear error when enabled
            if !config.model_path.exists() {
                return Err(ContextraError::InvalidInput(format!(
                    "Reranker model file not found at {:?}",
                    config.model_path
                )));
            }
            if !config.tokenizer_path.exists() {
                return Err(ContextraError::InvalidInput(format!(
                    "Reranker tokenizer file not found at {:?}",
                    config.tokenizer_path
                )));
            }

            let reranker_instance = CrossEncoderReranker::new(config.clone())?;
            Arc::new(reranker_instance)
        };

        Ok(Self {
            reranker,
            config,
            enabled,
        })
    }

    /// Returns the active `Arc<dyn Reranker>` reference.
    pub fn reranker(&self) -> Arc<dyn Reranker> {
        Arc::clone(&self.reranker)
    }

    /// Returns whether the ONNX reranker is active (`true`) or passthrough (`false`).
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Returns a reference to the active `RerankConfig`.
    pub fn config(&self) -> &RerankConfig {
        &self.config
    }
}
