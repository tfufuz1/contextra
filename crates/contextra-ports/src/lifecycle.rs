//! Context preparer contracts.

// FILE-CONTEXT
// STAND: 2026-09-15T00:00:00Z
// ZWECK: ContextPreparer Trait-Definition.
// INVARIANTEN: Zero-panic doctrine.

use crate::types::{ContextChunk, ContextWindow, TokenBudget};
use crate::Result;

// AI-TAG[ARCH][MINOR][RESOLVED] (virtuell verschoben von contextra-router/ports_local.rs per TODO(welle-3), siehe docs/refactor/router-db-edge-audit.md)
/// Contract for trimming and preparing context windows tailored to token budgets.
pub trait ContextPreparer: Send + Sync {
    /// Prepares and trims context chunks according to the provided token budget and relevance threshold.
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow>;
}
