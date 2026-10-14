// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: ONNX & Cross-Encoder Rerank Wiring for contextra facade (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; Builds RerankWiring and integrates Reranker port into search path.

#![forbid(unsafe_code)]

#[cfg(feature = "onnx")]
use std::sync::Arc;

#[cfg(feature = "onnx")]
use contextra_core::error::ContextraError;
#[cfg(feature = "onnx")]
use contextra_infer_onnx::wiring::{RerankWiring, RerankWiringDeps};
#[cfg(feature = "onnx")]
use contextra_ports::Reranker;

/// Bundles ONNX reranking capabilities.
#[cfg(feature = "onnx")]
pub struct OnnxWiringBundle {
    pub rerank_wiring: RerankWiring,
}

#[cfg(feature = "onnx")]
impl OnnxWiringBundle {
    /// Builds `OnnxWiringBundle` with CrossEncoderReranker.
    pub fn build(enabled: bool) -> Result<Self, ContextraError> {
        let deps = RerankWiringDeps {
            enabled,
            ..Default::default()
        };

        let rerank_wiring = RerankWiring::build(deps)?;
        Ok(Self { rerank_wiring })
    }

    /// Returns the configured reranker trait object reference.
    pub fn reranker(&self) -> Arc<dyn Reranker> {
        self.rerank_wiring.reranker()
    }
}
