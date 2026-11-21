//! # Einzelne Schreib- und Löschoperationen (Ring 1)
//!
//! // STAND: 2026-11-11
//! // ZWECK: Einzelne Schreiboperationen put, put_if_absent, delete, delete_prefix
//! // INVARIANTEN: INV-STORE-1, INV-STORE-5
//!
//! Verwaltet atomare Staged-Writes im TxBuffer sowie TOCTOU-sicheres `put_if_absent`.

use super::super::super::engine::LsmStorage;
use super::super::super::validate::{validate_key, validate_value};
use crate::error::StoreResult as Result;
use contextra_ids::TxId;
use crate::staging::StagedOp;
use std::sync::atomic::Ordering;

/// Schreibt einen Schlüssel-Wert-Eintrag in den Staging-Puffer der Transaktion.
pub(crate) async fn put(storage: &LsmStorage, tx_id: TxId, key: &[u8], value: &[u8]) -> Result<()> {
    validate_key(key)?;
    validate_value(value)?;
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(crate::error::StoreError::Storage(
            "Memory budget exceeded (95%)".into(),
        ));
    }
    storage.tx_staging.stage(
        tx_id,
        StagedOp::Put { key: key.to_vec(), value: value.to_vec() },
    )?;
    Ok(())
}

/// Fügt einen Schlüssel atomar nur dann ein, wenn er weder committet noch aktiv gestaged ist.
pub(crate) async fn put_if_absent(
    storage: &LsmStorage,
    tx_id: TxId,
    key: &[u8],
    value: &[u8],
) -> Result<bool> {
    validate_key(key)?;
    validate_value(value)?;
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(crate::error::StoreError::Storage(
            "Memory budget exceeded (95%)".into(),
        ));
    }

    // 1. Staged write check across all active transactions
    if storage.tx_staging.is_put_staged(key) {
        return Ok(false);
    }

    // 2. Intent Lock check and registration
    {
        let mut locks = storage
            .intent_locks
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(&existing_tx) = locks.get(key) {
            if existing_tx != tx_id {
                return Ok(false);
            }
        } else {
            locks.insert(key.to_vec(), tx_id);
        }
    }

    // 3. Query committed state
    let current_max_seq = storage.next_seq_no.load(Ordering::Acquire);
    let is_present = match storage.get_at_seq(key, current_max_seq).await {
        Ok(opt) => opt.is_some(),
        Err(e) => {
            let mut locks = storage
                .intent_locks
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if locks.get(key) == Some(&tx_id) {
                locks.remove(key);
            }
            return Err(e);
        }
    };

    if is_present {
        let mut locks = storage
            .intent_locks
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if locks.get(key) == Some(&tx_id) {
            locks.remove(key);
        }
        return Ok(false);
    }

    // 4. Stage operation in tx_buffer
    if let Err(e) = storage.tx_staging.stage(
        tx_id,
        StagedOp::Put { key: key.to_vec(), value: value.to_vec() },
    ) {
        let mut locks = storage
            .intent_locks
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if locks.get(key) == Some(&tx_id) {
            locks.remove(key);
        }
        return Err(e);
    }

    Ok(true)
}

/// Markiert einen Schlüssel als gelöscht (Tombstone).
pub(crate) async fn delete(storage: &LsmStorage, tx_id: TxId, key: &[u8]) -> Result<()> {
    validate_key(key)?;
    storage.apply_backpressure().await;
    if !storage.budget.has_memory_capacity() {
        return Err(crate::error::StoreError::Storage(
            "Memory budget exceeded (95%) — delete staging rejected".into(),
        ));
    }
    storage.tx_staging.stage(
        tx_id,
        StagedOp::Delete { key: key.to_vec() },
    )?;
    Ok(())
}

/// Löscht alle Schlüssel mit dem angegebenen Präfix.
pub(crate) async fn delete_prefix(storage: &LsmStorage, tx_id: TxId, prefix: &[u8]) -> Result<u64> {
    let matching_keys: Vec<Vec<u8>> = storage
        .scan_prefix(prefix)
        .await?
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    storage.delete_many(tx_id, matching_keys).await
}
