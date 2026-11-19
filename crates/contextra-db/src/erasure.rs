//! Thin integration boundary around the canonical erasure state machine.

use std::sync::Arc;

use contextra_erasure::chain::{Chain, EraseTarget, Receipt, RetentionAttestation};
use contextra_erasure::error::ErasureError;
use contextra_ids::ErasureId;

/// Starts and recovers the canonical erasure chain without duplicating its state.
///
/// Call [`Self::recover_before_reads`] while opening the database, before exposing a
/// [`crate::TenantScopedStorage`] handle.  This preserves the gate across crashes.
pub struct ErasureWiring {
    chain: Arc<Chain>,
}

impl ErasureWiring {
    /// Binds DB orchestration to the single canonical saga implementation.
    pub fn new(chain: Arc<Chain>) -> Self {
        Self { chain }
    }

    /// Completes recovery before any read path is made available.
    pub fn recover_before_reads(&self) -> Result<(), ErasureError> {
        self.chain.recover().map(|_| ())
    }

    /// Durably starts an erasure. The returned receipt is available after takt 1;
    /// it intentionally does not claim that later purge or measurement completed.
    pub async fn erase(
        &self,
        id: ErasureId,
        target: EraseTarget,
        attestation: RetentionAttestation,
    ) -> Result<Receipt, ErasureError> {
        self.chain.erase(id, target, attestation).await
    }

    /// Gives writers access to the same commit barrier used by the saga gate step.
    pub fn chain(&self) -> &Arc<Chain> {
        &self.chain
    }
}
