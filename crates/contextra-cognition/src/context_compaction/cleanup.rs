use contextra_ports::StorageEngine;
use contextra_types::{Result, TxId};
use contextra_engine::transaction::CommitIntent;

/// Aufgerufen beim Öffnen einer Collection / DB. Findet und entfernt verwaiste
/// ConsolidationIntents aus dem WAL/Storage.
///
/// Verwaiste Intents entstehen wenn consolidate_with_retry() nach commit(intent_key)
/// aber vor dem eigentlichen Dokument-Commit crashed.
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
