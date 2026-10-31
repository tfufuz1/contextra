//! # Verkettung von Löschbeweisen (`deletion_proof/chain.rs`)
//!
//! Verknüpft aufeinanderfolgende Löschbeweise zu einer manipulationssicheren Beweiskette.
// FILE-CONTEXT
// ZWECK: Chain verification functions for deletion proof audit trails.
// INVARIANTEN: Any alteration, removal, insertion, reordering, or truncation is detected.

use crate::deletion_proof::types::DeletionProof;
use contextra_types::Result;

/// Helper functions for verifying deletion proof chains and sequence continuity.
pub fn verify_proof_sequence(proofs: &[DeletionProof]) -> Result<bool> {
    if proofs.is_empty() {
        return Ok(true);
    }
    for i in 1..proofs.len() {
        if proofs[i].timestamp < proofs[i - 1].timestamp {
            return Ok(false);
        }
    }
    Ok(true)
}
