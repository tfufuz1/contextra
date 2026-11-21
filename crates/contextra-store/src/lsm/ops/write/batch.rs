//! # Batch-Löschungen und Mengenoperationen (Ring 1)
//!
//! // STAND: 2026-11-11
//! // ZWECK: delete_many mit Schranke MAX_BATCH_SIZE
//! // INVARIANTEN: MAX_BATCH_SIZE Schranke, fail-closed

use super::super::super::engine::LsmStorage;
use super::super::super::validate::validate_key;
use super::super::super::MAX_BATCH_SIZE;
use crate::error::{StoreError, StoreResult as Result};
use contextra_ids::TxId;
use crate::staging::StagedOp;

/// Löscht mehrere Schlüssel in einem einzigen atomaren Staging-Schritt.
pub(crate) async fn delete_many(
    storage: &LsmStorage,
    tx_id: TxId,
    keys: Vec<Vec<u8>>,
) -> Result<u64> {
    if keys.len() > MAX_BATCH_SIZE {
        return Err(StoreError::InvalidInput(format!(
            "Batch size ({} items) exceeds limit of {} items",
            keys.len(),
            MAX_BATCH_SIZE
        )));
    }
    for key in &keys {
        validate_key(key)?;
    }
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(crate::error::StoreError::Storage(
            "Memory budget exceeded (95%) — delete staging rejected".into(),
        ));
    }
    let count = keys.len() as u64;
    if count == 0 {
        return Ok(0);
    }

    let ops: Vec<StagedOp> = keys.into_iter().map(|key| StagedOp::Delete { key }).collect();

    storage.tx_staging.stage_many(tx_id, ops)?;
    Ok(count)
}
