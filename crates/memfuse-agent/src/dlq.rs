// FILE-CONTEXT Header (Format v3)
// ZWECK: Persistent dead-letter queue storage abstraction for failed agent steps.
// INVARIANTEN: Key prefix "dlq:" for LSM prefix isolation; atomic put/delete operations.
// NICHT-OFFENSICHTLICH: drain reads all DLQ entries and deletes them atomically in a single batch transaction.
// HOTSPOTS: push (ll. 25-45), drain (ll. 50-80).
// STAND: TS:2026-09-06T11:18:35Z (SESSION: 820afd9c)

// AI-TAG[INVENTORY-DRIFT][MINOR] RESOLVED: AGT-AGENT-0399362d — Documented dlq.rs inventory drift in audit report. (TS: 2026-09-06T11:18:35Z) (SESSION: 820afd9c)

//! Persistent Dead-Letter-Queue for failed agent step executions.

use crate::step::StepDeadLetter;
use memfuse_core::traits::StorageEngine;
use memfuse_core::{MemFuseError, Result, TxId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Persistente Dead-Letter-Queue für fehlgeschlagene Agent-Schritte.
/// Verwendet denselben Storage wie der Agent (LSM) mit einem fixen Key-Prefix.
pub struct DeadLetterQueue {
    storage: Arc<dyn StorageEngine>,
    next_tx: OnceCell<AtomicU64>,
}

impl DeadLetterQueue {
    pub const PREFIX: &'static [u8] = b"dlq:";

    pub fn new(storage: Arc<dyn StorageEngine>) -> Self {
        Self {
            storage,
            next_tx: OnceCell::new(),
        }
    }

    pub async fn push(&self, letter: &StepDeadLetter) -> Result<()> {
        let key = format!(
            "dlq:{}:{}:{}",
            letter.session_id, letter.node_id, letter.step_index
        );
        let value =
            serde_json::to_vec(letter).map_err(|e| MemFuseError::Serialization(e.to_string()))?;

        let tx = self.allocate_tx().await?;
        if let Err(e) = self.storage.put(tx, key.as_bytes(), &value).await {
            if let Err(rollback_err) = self.storage.rollback(tx).await {
                tracing::error!(error = %rollback_err, "Failed to rollback transaction after put failure in DLQ");
            }
            return Err(e);
        }
        if let Err(e) = self.storage.commit(tx).await {
            if let Err(rollback_err) = self.storage.rollback(tx).await {
                tracing::error!(error = %rollback_err, "Failed to rollback transaction after commit failure in DLQ");
            }
            return Err(e);
        }
        Ok(())
    }

    pub async fn drain(&self) -> Result<Vec<StepDeadLetter>> {
        let entries = self.storage.scan_prefix(Self::PREFIX).await?;
        let mut letters = Vec::with_capacity(entries.len());
        let mut keys_to_delete = Vec::with_capacity(entries.len());

        for (key, val) in entries {
            let letter: StepDeadLetter = serde_json::from_slice(&val)
                .map_err(|e| MemFuseError::Serialization(e.to_string()))?;
            letters.push(letter);
            keys_to_delete.push(key);
        }

        if !keys_to_delete.is_empty() {
            let tx = self.allocate_tx().await?;
            if let Err(e) = self.storage.delete_many(tx, keys_to_delete).await {
                if let Err(rollback_err) = self.storage.rollback(tx).await {
                    tracing::error!(error = %rollback_err, "Failed to rollback transaction after delete_many failure in DLQ");
                }
                return Err(e);
            }
            if let Err(e) = self.storage.commit(tx).await {
                if let Err(rollback_err) = self.storage.rollback(tx).await {
                    tracing::error!(error = %rollback_err, "Failed to rollback transaction after commit failure in DLQ");
                }
                return Err(e);
            }
        }

        Ok(letters)
    }

    pub async fn list(&self) -> Result<Vec<StepDeadLetter>> {
        let entries = self.storage.scan_prefix(Self::PREFIX).await?;
        let mut letters = Vec::with_capacity(entries.len());

        for (_key, val) in entries {
            let letter: StepDeadLetter = serde_json::from_slice(&val)
                .map_err(|e| MemFuseError::Serialization(e.to_string()))?;
            letters.push(letter);
        }

        Ok(letters)
    }

    /// Prüft vor dem Replay eines DLQ-Eintrags, ob die im Eintrag assoziierte `TxId`
    /// bereits im WAL/Storage für denselben `(Session, Node, Step)`-Schlüssel committet ist.
    ///
    /// Gibt `Ok(true)` zurück, falls das Event bereits erfolgreich committet wurde (Replay = No-Op).
    /// Gibt `Ok(false)` zurück, falls das Event noch nicht committet wurde und neu ausgeführt werden muss.
    pub async fn is_already_committed(&self, letter: &StepDeadLetter) -> Result<bool> {
        let letter_tx_id = match letter.tx_id {
            Some(tx) => tx,
            None => return Ok(false),
        };

        let state_doc_id = format!("task:{}:step:{}", letter.session_id, letter.step_index);
        let audit_id = format!("audit:{}:step:{}", letter.session_id, letter.step_index);

        for doc_id in [&state_doc_id, &audit_id] {
            if let Ok(Some(val_bytes)) = self.storage.get(doc_id.as_bytes()).await {
                if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&val_bytes) {
                    let tx_val = meta.get("tx_id").and_then(|v| v.as_u64()).or_else(|| {
                        meta.get("metadata")
                            .and_then(|m| m.get("tx_id"))
                            .and_then(|v| v.as_u64())
                    });
                    if let Some(tx_u64) = tx_val {
                        if tx_u64 == letter_tx_id.0 {
                            return Ok(true);
                        }
                    }
                }
            }
            let namespaced_doc_key = format!("__col:agent:\x000{}", doc_id);
            if let Ok(Some(val_bytes)) = self.storage.get(namespaced_doc_key.as_bytes()).await {
                if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&val_bytes) {
                    let tx_val = meta.get("tx_id").and_then(|v| v.as_u64()).or_else(|| {
                        meta.get("metadata")
                            .and_then(|m| m.get("tx_id"))
                            .and_then(|v| v.as_u64())
                    });
                    if let Some(tx_u64) = tx_val {
                        if tx_u64 == letter_tx_id.0 {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        let entries = self.storage.scan_prefix(b"").await?;
        for (k, val_bytes) in entries {
            let key_str = String::from_utf8_lossy(&k);
            if key_str.contains(&state_doc_id) || key_str.contains(&audit_id) {
                if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(&val_bytes) {
                    let tx_val = meta.get("tx_id").and_then(|v| v.as_u64()).or_else(|| {
                        meta.get("metadata")
                            .and_then(|m| m.get("tx_id"))
                            .and_then(|v| v.as_u64())
                    });
                    if let Some(tx_u64) = tx_val {
                        if tx_u64 == letter_tx_id.0 {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    pub async fn allocate_tx(&self) -> Result<TxId> {
        let counter = self
            .next_tx
            .get_or_try_init(|| async {
                let last_tx = self.storage.last_tx_id().await?.0;
                Ok::<AtomicU64, MemFuseError>(AtomicU64::new(last_tx + 1))
            })
            .await?;

        // AI-TAG[SMELL][MINOR] RESOLVED: AGT-AGENT-49bfd02e — Replaced non-atomic last_tx_id + 1 read with OnceCell initialized AtomicU64 fetch_add. (TS: 2026-09-11T14:30:00Z)
        let tx_val = counter.fetch_add(1, Ordering::SeqCst);
        Ok(TxId::new(tx_val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::InMemoryStorageEngine;
    use crate::step::DeadLetterReason;

    #[tokio::test]
    async fn test_dlq_push_list_drain() -> Result<()> {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let dlq = DeadLetterQueue::new(storage);

        let letter1 = StepDeadLetter {
            session_id: "sess-1".to_string(),
            node_id: "node-a".to_string(),
            step_index: 0,
            tx_id: None,
            failure_reason: DeadLetterReason::Timeout { timeout_ms: 5000 },
            input: serde_json::json!({"query": "test"}),
            attempt: 0,
            failed_at_secs: 1000,
        };

        let letter2 = StepDeadLetter {
            session_id: "sess-1".to_string(),
            node_id: "node-b".to_string(),
            step_index: 1,
            tx_id: None,
            failure_reason: DeadLetterReason::ToolError {
                message: "Tool failed".to_string(),
            },
            input: serde_json::json!({"action": "exec"}),
            attempt: 1,
            failed_at_secs: 1005,
        };

        dlq.push(&letter1).await?;
        dlq.push(&letter2).await?;

        let listed = dlq.list().await?;
        assert_eq!(listed.len(), 2);

        let drained = dlq.drain().await?;
        assert_eq!(drained.len(), 2);

        let listed_after_drain = dlq.list().await?;
        assert!(listed_after_drain.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_dlq_allocate_tx_uniqueness() -> Result<()> {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let dlq = DeadLetterQueue::new(storage);

        let tx1 = dlq.allocate_tx().await?;
        let tx2 = dlq.allocate_tx().await?;
        assert_ne!(tx1, tx2);
        Ok(())
    }
}
