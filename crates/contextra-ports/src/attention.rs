#![forbid(unsafe_code)]

//! Attention exporter port definitions for KV-cache eviction integration.

/// Unique identifier for an inference request within a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub u64);

/// Port implemented by inference backends (Ring 2) to export attention weights
/// from the latest prefill step to the KV-cache eviction worker (Ring 1).
pub trait AttentionExporter: Send + Sync {
    /// Returns accumulated attention weights per token position for the given request ID.
    ///
    /// Returns `None` if no attention weights are tracked or available for the request.
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>>;
}
