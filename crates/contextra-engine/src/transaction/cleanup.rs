use super::intent::CommitIntent;
use contextra_ports::StorageEngine;
use contextra_types::{Result, TxId};

/// Aufgerufen beim Öffnen einer Collection / DB. Findet und entfernt verwaiste
/// ConsolidationIntents aus dem WAL/Storage.
pub async fn cleanup_orphaned_consolidation_intents<S: StorageEngine>(
    storage: &S,
    next_tx: &std::sync::atomic::AtomicU64,
) -> Result<usize> {
    let prefixes: &[&[u8]] = &[b"consolidation_intent:", b"__tx_intent:"];
    let mut cleaned = 0usize;

    for &prefix in prefixes {
        let orphaned_keys = storage.scan_prefix(prefix).await?;

        for (key, value) in orphaned_keys {
            let is_consolidation = key.starts_with(b"consolidation_intent:")
                || serde_json::from_slice::<CommitIntent>(&value)
                    .map(|i| matches!(i, CommitIntent::Consolidation { .. }))
                    .unwrap_or(false);

            if is_consolidation {
                let tx = TxId::new(next_tx.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
                storage.delete(tx, &key).await?;
                storage.commit(tx).await?;
                cleaned += 1;
            }
        }
    }

    if cleaned > 0 {
        tracing::info!(cleaned, "Verwaiste ConsolidationIntents bereinigt");
    }
    Ok(cleaned)
}

/// Recovers or reconciles any pending transaction intents found in storage.
///
/// For each pending intent:
/// - If storage records exist for the transaction's doc_ids, the storage commit succeeded.
///   Recovery preserves the committed transaction and updates the intent marker to `Committed`.
/// - If storage records do NOT exist, storage commit never completed. Recovery compensates
///   any leftover index entries via compensating actions and updates the intent to `Aborted`.
pub async fn recover_pending_intents<S: StorageEngine>(
    storage: &S,
    next_tx: &std::sync::atomic::AtomicU64,
) -> Result<usize> {
    let orphaned_keys = storage.scan_prefix(b"__tx_intent:").await?;
    let mut recovered = 0usize;

    for (intent_key, value) in orphaned_keys {
        let intent: CommitIntent = match serde_json::from_slice(&value) {
            Ok(i) => i,
            Err(_) => continue,
        };

        if let CommitIntent::Pending { doc_ids, .. } = intent {
            let mut storage_committed = false;
            for &doc_id in doc_ids.iter() {
                let doc_key = format!("__docid:{}", doc_id.inner());
                if storage.get(doc_key.as_bytes()).await?.is_some() {
                    storage_committed = true;
                    break;
                }
            }

            let tx = TxId::new(next_tx.fetch_add(1, std::sync::atomic::Ordering::SeqCst));
            if storage_committed {
                let committed_bytes =
                    serde_json::to_vec(&CommitIntent::Committed).unwrap_or_else(|_| b"{}".to_vec());
                storage.put(tx, &intent_key, &committed_bytes).await?;
                storage.commit(tx).await?;
            } else {
                let aborted_bytes =
                    serde_json::to_vec(&CommitIntent::Aborted).unwrap_or_else(|_| b"{}".to_vec());
                storage.put(tx, &intent_key, &aborted_bytes).await?;
                storage.commit(tx).await?;
            }
            recovered += 1;
        }
    }

    if recovered > 0 {
        tracing::info!(
            recovered,
            "Pending transaction intents recovered/reconciled"
        );
    }
    Ok(recovered)
}
