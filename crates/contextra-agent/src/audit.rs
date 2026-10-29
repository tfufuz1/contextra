// FILE-CONTEXT Header (Format v3)
// ZWECK: Append-only immutable audit trail logging for agent state transitions.
// INVARIANTEN: Keyed `audit:{task_id}:step:{n}`; zero deletion/update paths by design.
// NICHT-OFFENSICHTLICH: replay_task scans prefix and sorts by step_count for deterministic replay.
// HOTSPOTS: append (ll. 35-50), replay_task (ll. 52-70).
// STAND: TS:2026-09-11T14:41:11Z (SESSION: b979ee5a)

//! Immutable audit trail for agent workflow executions.
//!
//! Provides append-only logging of every step an agent takes.
//! Entries are stored via [`Collection`] and keyed `audit:{task_id}:step:{n}`.

use crate::context::{validate_node_id, validate_task_id};
use contextra_db::Collection;
use contextra_ports::StorageEngine;
use contextra_store::LsmStorage;
use contextra_types::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Single immutable record of an agent step execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Task identifier associated with this audit entry.
    pub task_id: String,
    /// Step sequence index within the task workflow.
    pub step_count: u64,
    /// Node identifier executed at this step.
    pub node_id: String,
    /// Tokens consumed by executing this step.
    pub tokens_consumed: usize,
    /// Step execution payload output.
    pub payload: serde_json::Value,
    /// Optional error description if the step failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Optional storage transaction ID associated with this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_id: Option<contextra_types::TxId>,
}

/// Summary statistics for legacy audit entry migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MigrationStats {
    /// Number of legacy entries successfully migrated.
    pub migrated: usize,
    /// Number of legacy entries that failed migration.
    pub failed: usize,
}

/// Append-only audit log backed by a Contextra collection.
pub struct AuditLog<S: StorageEngine = LsmStorage> {
    collection: Arc<Collection<S>>,
}

/// Validates payload and optional error string of an audit entry for non-emptiness and absence of null bytes.
pub fn validate_audit_payload_and_error(
    payload: &serde_json::Value,
    error: Option<&str>,
) -> Result<()> {
    if let Some(err) = error {
        if err.trim().is_empty() {
            return Err(contextra_types::ContextraError::InvalidInput(
                "Audit error message cannot be empty when provided".to_string(),
            ));
        }
        if err.contains('\0') {
            return Err(contextra_types::ContextraError::InvalidInput(
                "Audit error message cannot contain null bytes".to_string(),
            ));
        }
    }

    if let Some(s) = payload.as_str() {
        if s.trim().is_empty() {
            return Err(contextra_types::ContextraError::InvalidInput(
                "Audit string payload cannot be empty".to_string(),
            ));
        }
    }

    let json_str = payload.to_string();
    if json_str.contains('\0') || json_str.contains("\\u0000") {
        return Err(contextra_types::ContextraError::InvalidInput(
            "Audit payload cannot contain null bytes".to_string(),
        ));
    }

    Ok(())
}

impl<S: StorageEngine> AuditLog<S> {
    /// Constructs a new [`AuditLog`] wrapping a collection reference.
    pub fn new(collection: Arc<Collection<S>>) -> Self {
        Self { collection }
    }

    /// Appends an immutable audit entry directly to a collection without needing an [`AuditLog`] wrapper.
    pub async fn append_to(collection: &Collection<S>, entry: &AuditEntry) -> Result<()> {
        validate_task_id(&entry.task_id)?;
        validate_node_id(&entry.node_id)?;
        validate_audit_payload_and_error(&entry.payload, entry.error.as_deref())?;

        let audit_id = format!("audit:{}:step:{}", entry.task_id, entry.step_count);
        let payload = serde_json::to_value(entry)
            .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?;

        collection.put_kv_if_absent(&audit_id, &payload).await
    }

    /// Appends an immutable audit entry directly via LSM storage without HNSW vector index participation (AC-3).
    /// Enforces append-only immutability: returns `ContextraError::Conflict` if an entry for the step already exists.
    /// A `Conflict` error indicates that a step with this ID already exists - the caller must choose a new `step_count`
    /// (e.g. using `next_retry_step_count()`), and NOT automatically overwrite.
    pub async fn append(&self, entry: &AuditEntry) -> Result<()> {
        Self::append_to(&self.collection, entry).await
    }

    /// Replays all audit entries for a given task via scan_prefix.
    pub async fn replay_task(&self, task_id: &str) -> Result<Vec<AuditEntry>> {
        validate_task_id(task_id)?;

        let prefix = format!("audit:{}:step:", task_id);
        let raw = self.collection.scan_prefix(&prefix, None).await?;

        let mut entries: Vec<AuditEntry> = raw
            .into_iter()
            .filter_map(|(_key, meta)| {
                let entry_val = meta.get("metadata").cloned().unwrap_or(meta);
                serde_json::from_value::<AuditEntry>(entry_val)
                    .map_err(|e| {
                        tracing::warn!("AuditLog: Deserialization error: {e}");
                    })
                    .ok()
            })
            .collect();

        // Sort by step_count for deterministic replay
        entries.sort_by_key(|e| e.step_count);
        Ok(entries)
    }
}

async fn migrate_single_legacy_audit_entry<S: StorageEngine, V: contextra_ports::VectorIndex>(
    collection: &Collection<S, V>,
    key: &str,
    val: &serde_json::Value,
) -> Result<bool> {
    let obj = match val.as_object() {
        Some(o) if o.contains_key("embedding") => o,
        _ => return Ok(false),
    };

    let entry_val = obj.get("metadata").cloned().unwrap_or_else(|| val.clone());
    let doc_id = contextra_types::DocId::from_key(key).map_err(|err| {
        tracing::error!(key = %key, error = %err, "Failed to generate DocId from key during migration");
        err
    })?;

    let tx = collection.allocate_tx().map_err(|err| {
        tracing::error!(key = %key, error = %err, "Failed to allocate transaction during migration");
        err
    })?;

    if let Err(err) = collection.vector_index().delete(tx, doc_id).await {
        tracing::error!(doc_id = ?doc_id, key = %key, error = %err, "Failed to delete zero-vector");
        let _ = collection.storage().rollback(tx).await;
        return Err(err);
    }

    if let Err(err) = collection.vector_index().commit(tx).await {
        tracing::error!(doc_id = ?doc_id, key = %key, error = %err, "Failed to commit vector index deletion");
        let _ = collection.storage().rollback(tx).await;
        return Err(err);
    }

    let doc_key = collection.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
    if let Err(err) = collection.storage().delete(tx, &doc_key).await {
        tracing::error!(doc_id = ?doc_id, key = %key, error = %err, "Failed to delete doc_key mapping");
        let _ = collection.storage().rollback(tx).await;
        return Err(err);
    }

    if let Err(err) = collection.put_kv(key, &entry_val).await {
        tracing::error!(key = %key, error = %err, "Failed to save pure KV entry");
        let _ = collection.storage().rollback(tx).await;
        return Err(err);
    }

    if let Err(err) = collection.storage().commit(tx).await {
        tracing::error!(key = %key, error = %err, "Failed to commit storage transaction");
        let _ = collection.storage().rollback(tx).await;
        return Err(err);
    }

    Ok(true)
}

/// Migrates legacy zero-vector audit entries from the HNSW vector index and doc_key mappings
/// into the direct LSM KV store format (`put_kv`).
///
/// Returns [`MigrationStats`] detailing successfully migrated and failed entries.
pub async fn migrate_legacy_audit_entries<S: StorageEngine, V: contextra_ports::VectorIndex>(
    collection: &Collection<S, V>,
) -> Result<MigrationStats> {
    let raw = collection.scan_prefix("audit:", None).await?;
    let mut stats = MigrationStats::default();

    for (key, val) in raw {
        if !key.starts_with("audit:") {
            continue;
        }

        match migrate_single_legacy_audit_entry(collection, &key, &val).await {
            Ok(true) => stats.migrated += 1,
            Ok(false) => {}
            Err(_) => stats.failed += 1,
        }
    }

    if stats.migrated > 0 || stats.failed > 0 {
        tracing::info!(
            migrated = stats.migrated,
            failed = stats.failed,
            "Legacy zero-vector audit entry migration completed"
        );
    }
    Ok(stats)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use contextra_graph::CsrGraph;
    use contextra_testkit::InMemoryStorageEngine;
    use contextra_vector::{HnswConfig, HnswIndex};

    #[tokio::test]
    async fn test_audit_log_in_memory_storage() {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let index = Arc::new(HnswIndex::try_new(HnswConfig::default()).unwrap()); // unwrap allowed
        let graph_index = Arc::new(CsrGraph::new());
        let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));

        let collection = Arc::new(Collection::new(
            "test_audit".to_string(),
            storage,
            index,
            graph_index,
            next_tx,
            1536,
            contextra_text::Language::English,
        ));

        let audit_log = AuditLog::new(collection);

        let entry1 = AuditEntry {
            task_id: "task-123".to_string(),
            step_count: 1,
            node_id: "node-start".to_string(),
            tokens_consumed: 50,
            payload: serde_json::json!({"action": "init"}),
            error: None,
            tx_id: None,
        };

        let entry2 = AuditEntry {
            task_id: "task-123".to_string(),
            step_count: 2,
            node_id: "node-process".to_string(),
            tokens_consumed: 120,
            payload: serde_json::json!({"action": "compute"}),
            error: None,
            tx_id: None,
        };

        audit_log.append(&entry1).await.unwrap(); // unwrap allowed
        audit_log.append(&entry2).await.unwrap(); // unwrap allowed

        let replayed = audit_log.replay_task("task-123").await.unwrap(); // unwrap allowed
        assert_eq!(replayed.len(), 2);
        assert_eq!(replayed[0].step_count, 1);
        assert_eq!(replayed[0].node_id, "node-start");
        assert_eq!(replayed[1].step_count, 2);
        assert_eq!(replayed[1].node_id, "node-process");
    }

    #[tokio::test]
    async fn test_audit_log_duplicate_step_rejected() {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let index = Arc::new(HnswIndex::try_new(HnswConfig::default()).unwrap());
        let graph_index = Arc::new(CsrGraph::new());
        let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));

        let collection = Arc::new(Collection::new(
            "test_audit_dup".to_string(),
            storage,
            index,
            graph_index,
            next_tx,
            1536,
            contextra_text::Language::English,
        ));

        let audit_log = AuditLog::new(collection);

        let entry = AuditEntry {
            task_id: "task-dup".to_string(),
            step_count: 1,
            node_id: "node-1".to_string(),
            tokens_consumed: 50,
            payload: serde_json::json!({"step": 1}),
            error: None,
            tx_id: None,
        };

        audit_log.append(&entry).await.unwrap();

        let dup_res = audit_log.append(&entry).await;
        assert!(
            matches!(dup_res, Err(contextra_types::ContextraError::Conflict(_))),
            "Expected Conflict error on duplicate audit step append"
        );
    }

    #[tokio::test]
    async fn test_audit_log_null_byte_and_empty_input_boundary_guards() {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let index = Arc::new(HnswIndex::try_new(HnswConfig::default()).unwrap());
        let graph_index = Arc::new(CsrGraph::new());
        let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));

        let collection = Arc::new(Collection::new(
            "test_audit_guards".to_string(),
            storage,
            index,
            graph_index,
            next_tx,
            1536,
            contextra_text::Language::English,
        ));

        let audit_log = AuditLog::new(collection);

        // 1. Empty and null-byte task_ids
        let task_ids_to_test = vec!["", "\0", "\0\0", "\0task", "task\0id", "task\0"];

        for task_id in &task_ids_to_test {
            let entry = AuditEntry {
                task_id: task_id.to_string(),
                step_count: 1,
                node_id: "valid_node".to_string(),
                tokens_consumed: 10,
                payload: serde_json::json!({"ok": true}),
                error: None,
                tx_id: None,
            };
            assert!(
                matches!(
                    audit_log.append(&entry).await,
                    Err(contextra_types::ContextraError::InvalidInput(_))
                ),
                "Expected InvalidInput for task_id: {:?}",
                task_id
            );
            assert!(
                matches!(
                    audit_log.replay_task(task_id).await,
                    Err(contextra_types::ContextraError::InvalidInput(_))
                ),
                "Expected InvalidInput for replay_task with task_id: {:?}",
                task_id
            );
        }

        // 2. Empty and null-byte node_ids
        let node_ids_to_test = vec!["", "\0", "\0node", "node\0id", "node\0"];

        for node_id in &node_ids_to_test {
            let entry = AuditEntry {
                task_id: "valid_task".to_string(),
                step_count: 1,
                node_id: node_id.to_string(),
                tokens_consumed: 10,
                payload: serde_json::json!({"ok": true}),
                error: None,
                tx_id: None,
            };
            assert!(
                matches!(
                    audit_log.append(&entry).await,
                    Err(contextra_types::ContextraError::InvalidInput(_))
                ),
                "Expected InvalidInput for node_id: {:?}",
                node_id
            );
        }

        // 3. Empty string and null-byte payloads
        let invalid_payloads = vec![
            serde_json::Value::String("".to_string()),
            serde_json::Value::String("\0".to_string()),
            serde_json::Value::String("\0payload".to_string()),
            serde_json::Value::String("pay\0load".to_string()),
            serde_json::Value::String("payload\0".to_string()),
            serde_json::json!({"nested": "val\0null"}),
        ];

        for payload in invalid_payloads {
            let entry = AuditEntry {
                task_id: "valid_task".to_string(),
                step_count: 1,
                node_id: "valid_node".to_string(),
                tokens_consumed: 10,
                payload,
                error: None,
                tx_id: None,
            };
            assert!(
                matches!(
                    audit_log.append(&entry).await,
                    Err(contextra_types::ContextraError::InvalidInput(_))
                ),
                "Expected InvalidInput for payload: {:?}",
                entry.payload
            );
        }

        // 4. Empty and null-byte error messages
        let invalid_errors = vec!["", "\0", "\0err", "err\0or", "error\0"];

        for err_msg in invalid_errors {
            let entry = AuditEntry {
                task_id: "valid_task".to_string(),
                step_count: 1,
                node_id: "valid_node".to_string(),
                tokens_consumed: 0,
                payload: serde_json::Value::Null,
                error: Some(err_msg.to_string()),
                tx_id: None,
            };
            assert!(
                matches!(
                    audit_log.append(&entry).await,
                    Err(contextra_types::ContextraError::InvalidInput(_))
                ),
                "Expected InvalidInput for error: {:?}",
                err_msg
            );
        }
    }

    #[tokio::test]
    async fn test_migrate_legacy_audit_entries() -> contextra_types::Result<()> {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let index = Arc::new(
            HnswIndex::try_new(HnswConfig::default())
                .map_err(|e| contextra_types::ContextraError::Internal(e.to_string()))?,
        );
        let graph_index = Arc::new(CsrGraph::new());
        let next_tx = Arc::new(std::sync::atomic::AtomicU64::new(1));

        let collection = Arc::new(Collection::new(
            "test_audit_migrate".to_string(),
            storage.clone(),
            index,
            graph_index,
            next_tx,
            1536,
            contextra_text::Language::English,
        ));

        let key = "audit:task-mig:step:1";
        let doc_id = contextra_types::DocId::from_key(key)?;
        let tx = contextra_types::TxId::new(1);

        let legacy_doc = serde_json::json!({
            "id": key,
            "embedding": vec![0.0f32; 1536],
            "metadata": {
                "task_id": "task-mig",
                "step_count": 1,
                "node_id": "node-1",
                "tokens_consumed": 15,
                "payload": {"status": "ok"}
            }
        });

        let user_key = collection.namespaced_key(key.as_bytes(), 0);
        let doc_key = collection.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
        let data = serde_json::to_vec(&legacy_doc)
            .map_err(|e| contextra_types::ContextraError::Serialization(e.to_string()))?;

        storage.put(tx, &user_key, &data).await?;
        storage.put(tx, &doc_key, &data).await?;
        storage.commit(tx).await?;

        let stats = migrate_legacy_audit_entries(&collection).await?;
        assert_eq!(stats.migrated, 1);
        assert_eq!(stats.failed, 0);

        let migrated_val = collection.get_kv(key).await?;
        assert!(migrated_val.is_some());
        if let Some(entry_obj) = migrated_val {
            assert_eq!(
                entry_obj.get("node_id").and_then(|v| v.as_str()),
                Some("node-1")
            );
        }
        Ok(())
    }
}
