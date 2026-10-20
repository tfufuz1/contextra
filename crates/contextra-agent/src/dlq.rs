// FILE-CONTEXT Header (Format v3)
// ZWECK: Persistent dead-letter queue storage abstraction for failed agent steps.
// INVARIANTEN: Key prefix "dlq:" for LSM prefix isolation; atomic put/delete operations.
// NICHT-OFFENSICHTLICH: drain reads all DLQ entries and deletes them atomically in a single batch transaction.
// HOTSPOTS: push (ll. 25-45), drain (ll. 50-80).
// STAND: TS:2026-09-06T11:18:35Z (SESSION: 820afd9c)

// AI-TAG[INVENTORY-DRIFT][MINOR] RESOLVED: AGT-AGENT-0399362d — Documented dlq.rs inventory drift in audit report. (TS: 2026-09-06T11:18:35Z) (SESSION: 820afd9c)

//! Persistent Dead-Letter-Queue for failed agent step executions.

use crate::step::StepDeadLetter;
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, Result, TxId};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::OnceCell;

fn extract_tx_id(val: &serde_json::Value) -> Option<u64> {
    val.as_u64()
        .or_else(|| val.as_str().and_then(|s| s.parse::<u64>().ok()))
}

fn check_json_for_tx_id(val_bytes: &[u8], target_tx_id: u64) -> bool {
    if let Ok(meta) = serde_json::from_slice::<serde_json::Value>(val_bytes) {
        let tx_val = meta.get("tx_id").and_then(extract_tx_id).or_else(|| {
            meta.get("metadata")
                .and_then(|m| m.get("tx_id"))
                .and_then(extract_tx_id)
        });
        if let Some(tx_u64) = tx_val {
            if tx_u64 == target_tx_id {
                return true;
            }
        }
    }
    false
}

/// Persistente Dead-Letter-Queue für fehlgeschlagene Agent-Schritte.
/// Verwendet denselben Storage wie der Agent (LSM) mit einem fixen Key-Prefix.
pub struct DeadLetterQueue {
    storage: Arc<dyn StorageEngine>,
    next_tx: OnceCell<AtomicU64>,
    metrics_sink: Option<Arc<dyn contextra_ports::MetricsSink>>,
}

impl DeadLetterQueue {
    pub const PREFIX: &'static [u8] = b"dlq:";

    pub fn new(storage: Arc<dyn StorageEngine>) -> Self {
        Self {
            storage,
            next_tx: OnceCell::new(),
            metrics_sink: None,
        }
    }

    pub fn with_metrics_sink(mut self, sink: Arc<dyn contextra_ports::MetricsSink>) -> Self {
        self.metrics_sink = Some(sink);
        self
    }

    pub fn new_with_metrics(
        storage: Arc<dyn StorageEngine>,
        sink: Arc<dyn contextra_ports::MetricsSink>,
    ) -> Self {
        Self {
            storage,
            next_tx: OnceCell::new(),
            metrics_sink: Some(sink),
        }
    }

    async fn report_depth(&self) -> Result<usize> {
        let letters = self.list().await?;
        Ok(letters.len())
    }

    pub async fn push(&self, letter: &StepDeadLetter) -> Result<()> {
        let key = format!(
            "dlq:{}:{}:{}",
            letter.session_id, letter.node_id, letter.step_index
        );
        let value =
            serde_json::to_vec(letter).map_err(|e| ContextraError::Serialization(e.to_string()))?;

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
        self.report_depth().await?;
        Ok(())
    }

    pub async fn drain(&self) -> Result<Vec<StepDeadLetter>> {
        let entries = self.storage.scan_prefix(Self::PREFIX).await?;
        let mut letters = Vec::with_capacity(entries.len());
        let mut keys_to_delete = Vec::with_capacity(entries.len());

        for (key, val) in entries {
            let letter: StepDeadLetter = serde_json::from_slice(&val)
                .map_err(|e| ContextraError::Serialization(e.to_string()))?;
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

        if let Some(ref sink) = self.metrics_sink {
            sink.record_gauge("dlq_depth", 0.0, &[]);
        }

        Ok(letters)
    }

    pub async fn list(&self) -> Result<Vec<StepDeadLetter>> {
        let entries = self.storage.scan_prefix(Self::PREFIX).await?;
        let mut letters = Vec::with_capacity(entries.len());

        for (_key, val) in entries {
            let letter: StepDeadLetter = serde_json::from_slice(&val)
                .map_err(|e| ContextraError::Serialization(e.to_string()))?;
            letters.push(letter);
        }

        if let Some(ref sink) = self.metrics_sink {
            sink.record_gauge("dlq_depth", letters.len() as f64, &[]);
        }

        Ok(letters)
    }

    /// Entfernt einen spezifischen DLQ-Eintrag anhand seines `(session_id, node_id, step_index)` Schlüssels.
    pub async fn remove(&self, letter: &StepDeadLetter) -> Result<bool> {
        let key = format!(
            "dlq:{}:{}:{}",
            letter.session_id, letter.node_id, letter.step_index
        );
        let tx = self.allocate_tx().await?;
        if let Err(e) = self.storage.delete(tx, key.as_bytes()).await {
            if let Err(rollback_err) = self.storage.rollback(tx).await {
                tracing::error!(error = %rollback_err, "Failed to rollback transaction after delete failure in DLQ");
            }
            return Err(e);
        }
        if let Err(e) = self.storage.commit(tx).await {
            if let Err(rollback_err) = self.storage.rollback(tx).await {
                tracing::error!(error = %rollback_err, "Failed to rollback transaction after commit failure in DLQ");
            }
            return Err(e);
        }
        self.report_depth().await?;
        Ok(true)
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
                if check_json_for_tx_id(&val_bytes, letter_tx_id.0) {
                    return Ok(true);
                }
            }
            let namespaced_doc_key = format!("__col:agent:\x000{}", doc_id);
            if let Ok(Some(val_bytes)) = self.storage.get(namespaced_doc_key.as_bytes()).await {
                if check_json_for_tx_id(&val_bytes, letter_tx_id.0) {
                    return Ok(true);
                }
            }
        }

        let entries = self.storage.scan_prefix(b"").await?;
        for (k, val_bytes) in entries {
            let key_str = String::from_utf8_lossy(&k);
            if (key_str.contains(&state_doc_id) || key_str.contains(&audit_id))
                && check_json_for_tx_id(&val_bytes, letter_tx_id.0)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn allocate_tx(&self) -> Result<TxId> {
        let counter = self
            .next_tx
            .get_or_try_init(|| async {
                let last_tx = self.storage.last_tx_id().await?.inner();
                let initial = last_tx.checked_add(1).ok_or_else(|| {
                    ContextraError::Transaction("Initial TxId overflow beyond u64::MAX".to_string())
                })?;
                Ok::<AtomicU64, ContextraError>(AtomicU64::new(initial))
            })
            .await?;

        // ORDERING-BEGRÜNDUNG:
        // Ordering::SeqCst in `fetch_update` gewährleistet eine strikt globale Sequenzordnung aller
        // allozierten Transaktions-IDs über sämtliche nebenläufigen Threads hinweg.
        // Der CAS-Loop stellt sicher, dass Werte an der Bereichsgrenze (MAX_COLLECTION_SEQUENCE
        // bzw. u64::MAX) mit Err abgelehnt werden, statt im Unmanaged-Gap zu landen oder umzubrechen (wrap-around).
        let tx_val = counter
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |val| {
                if val == u64::MAX {
                    None // Overflow - reject wrap-around at upper boundary
                } else if val > TxId::MAX_COLLECTION_SEQUENCE && val < TxId::INTERNAL_BASE {
                    None // In the invalid origin gap (AGT-GRAPH-001) - reject
                } else {
                    Some(val + 1)
                }
            })
            .map_err(|_| {
                ContextraError::Transaction(
                    "TxId sequence overflow or invalid range boundary reached".to_string(),
                )
            })?;

        TxId::try_new(tx_val)
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
    async fn test_dlq_new_with_metrics_constructor() -> Result<()> {
        use contextra_ports::{MetricEvent, TestMetricsSink};

        let storage = Arc::new(InMemoryStorageEngine::new());
        let metrics = Arc::new(TestMetricsSink::new());
        let dlq = DeadLetterQueue::new_with_metrics(
            storage,
            metrics.clone() as Arc<dyn contextra_ports::MetricsSink>,
        );

        let letter = StepDeadLetter {
            session_id: "sess-metrics".to_string(),
            node_id: "node-m".to_string(),
            step_index: 0,
            tx_id: None,
            failure_reason: DeadLetterReason::Timeout { timeout_ms: 1000 },
            input: serde_json::json!({"test": "val"}),
            attempt: 0,
            failed_at_secs: 100,
        };

        dlq.push(&letter).await?;
        let listed = dlq.list().await?;
        assert_eq!(listed.len(), 1);

        let events = metrics.events();
        assert!(!events.is_empty());
        let last_event = events.last().cloned().ok_or_else(|| {
            ContextraError::Internal("Expected at least one metric event".to_string())
        })?;
        assert_eq!(
            last_event,
            MetricEvent::Gauge {
                name: "dlq_depth".to_string(),
                value: 1.0,
                labels: vec![],
            }
        );

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_allocate_tx_concurrent_n_threads_m_calls() -> Result<()> {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let dlq = Arc::new(DeadLetterQueue::new(storage));

        const THREADS: usize = 8;
        const CALLS_PER_THREAD: usize = 50;

        let mut handles = Vec::new();

        for _ in 0..THREADS {
            let dlq_clone = Arc::clone(&dlq);
            handles.push(tokio::spawn(async move {
                let mut local_txs = Vec::with_capacity(CALLS_PER_THREAD);
                for _ in 0..CALLS_PER_THREAD {
                    let tx = dlq_clone.allocate_tx().await?;
                    assert!(
                        tx.is_valid_origin(),
                        "Allocated TxId must have valid origin range"
                    );
                    local_txs.push(tx);
                }
                Ok::<Vec<TxId>, ContextraError>(local_txs)
            }));
        }

        let mut all_txs = Vec::with_capacity(THREADS * CALLS_PER_THREAD);
        for handle in handles {
            let thread_txs = handle
                .await
                .map_err(|e| ContextraError::Internal(e.to_string()))??;

            // Strictly monotonic per thread
            for window in thread_txs.windows(2) {
                if let (Some(w0), Some(w1)) = (window.first(), window.get(1)) {
                    assert!(
                        w0 < w1,
                        "Per-thread TxId sequence must be strictly monotonic"
                    );
                }
            }

            all_txs.extend(thread_txs);
        }

        // Global uniqueness across all threads
        let total_count = all_txs.len();
        let unique_count = all_txs
            .into_iter()
            .collect::<std::collections::HashSet<_>>()
            .len();
        assert_eq!(
            unique_count, total_count,
            "All allocated TxIds across threads must be distinct"
        );

        Ok(())
    }

    #[derive(Debug)]
    struct BoundaryStorageEngine {
        last_tx: TxId,
    }

    impl BoundaryStorageEngine {
        fn new(last_tx: TxId) -> Self {
            Self { last_tx }
        }
    }

    impl Default for BoundaryStorageEngine {
        fn default() -> Self {
            Self {
                last_tx: TxId::INVALID,
            }
        }
    }

    impl StorageEngine for BoundaryStorageEngine {
        fn get<'a>(
            &'a self,
            _key: &'a [u8],
        ) -> contextra_ports::BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn get_at_seq<'a>(
            &'a self,
            _key: &'a [u8],
            _seq: u64,
        ) -> contextra_ports::BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { Ok(None) })
        }
        fn put<'a>(
            &'a self,
            _tx: TxId,
            _k: &'a [u8],
            _v: &'a [u8],
        ) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn put_if_absent<'a>(
            &'a self,
            _tx: TxId,
            _k: &'a [u8],
            _v: &'a [u8],
        ) -> contextra_ports::BoxFuture<'a, Result<bool>> {
            Box::pin(async move { Ok(true) })
        }
        fn delete<'a>(
            &'a self,
            _tx: TxId,
            _k: &'a [u8],
        ) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn commit<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn rollback_to_tx<'a>(&'a self, _tx: TxId) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn flush<'a>(&'a self) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn stats<'a>(
            &'a self,
        ) -> contextra_ports::BoxFuture<'a, Result<contextra_ports::StorageStats>> {
            Box::pin(async move {
                Ok(contextra_ports::StorageStats {
                    num_segments: 0,
                    total_size_bytes: 0,
                    memtable_size_bytes: 0,
                })
            })
        }
        fn last_seq_no<'a>(&'a self) -> contextra_ports::BoxFuture<'a, Result<u64>> {
            Box::pin(async move { Ok(0) })
        }
        fn last_tx_id<'a>(&'a self) -> contextra_ports::BoxFuture<'a, Result<TxId>> {
            let tx = self.last_tx;
            Box::pin(async move { Ok(tx) })
        }
        fn pin_checkpoint<'a>(&'a self, _seq: u64) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn unpin_checkpoint<'a>(&'a self, _seq: u64) -> contextra_ports::BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn scan_prefix<'a>(
            &'a self,
            _p: &'a [u8],
        ) -> contextra_ports::BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
        fn scan<'a>(
            &'a self,
            _s: std::ops::Bound<&'a [u8]>,
            _e: std::ops::Bound<&'a [u8]>,
            _l: Option<usize>,
        ) -> contextra_ports::BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
            Box::pin(async move { Ok(Vec::new()) })
        }
    }

    #[tokio::test]
    async fn test_allocate_tx_collection_sequence_boundary_returns_err() -> Result<()> {
        let storage = Arc::new(BoundaryStorageEngine::new(TxId::try_new(
            TxId::MAX_COLLECTION_SEQUENCE,
        )?));
        let dlq = DeadLetterQueue::new(storage);

        // First allocation sees counter initialized at MAX_COLLECTION_SEQUENCE + 1 (in the gap)
        let res = dlq.allocate_tx().await;
        assert!(
            res.is_err(),
            "Allocation exceeding MAX_COLLECTION_SEQUENCE must return Err instead of emitting gap TxId"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_allocate_tx_u64_max_overflow_returns_err() -> Result<()> {
        let storage = Arc::new(BoundaryStorageEngine::new(TxId::try_new(u64::MAX)?));
        let dlq = DeadLetterQueue::new(storage);

        let res = dlq.allocate_tx().await;
        assert!(
            res.is_err(),
            "Allocation wrapping u64::MAX must return Err rather than wrapping around to 0"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_dlq_poison_messages_returns_serialization_error() -> Result<()> {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let dlq = DeadLetterQueue::new(storage.clone());

        // Insert corrupt non-JSON payload into DLQ key
        let tx = TxId::try_new(1)?;
        storage
            .put(tx, b"dlq:sess-poison:node:0", b"not-valid-json-content")
            .await?;

        let list_res = dlq.list().await;
        assert!(
            matches!(list_res, Err(ContextraError::Serialization(_))),
            "Corrupt JSON payload in DLQ must return ContextraError::Serialization"
        );

        let drain_res = dlq.drain().await;
        assert!(
            matches!(drain_res, Err(ContextraError::Serialization(_))),
            "Drain on poison message in DLQ must return ContextraError::Serialization"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_dlq_empty_queue_ops() -> Result<()> {
        let storage = Arc::new(InMemoryStorageEngine::new());
        let dlq = DeadLetterQueue::new(storage);

        let listed = dlq.list().await?;
        assert!(listed.is_empty());

        let drained = dlq.drain().await?;
        assert!(drained.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_dlq_metrics_gauge_depth() -> Result<()> {
        use contextra_ports::{MetricEvent, TestMetricsSink};

        let storage = Arc::new(InMemoryStorageEngine::new());
        let metrics = Arc::new(TestMetricsSink::new());
        let dlq = DeadLetterQueue::new(storage)
            .with_metrics_sink(metrics.clone() as Arc<dyn contextra_ports::MetricsSink>);

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

        // 1. Push letter1 -> Gauge depth = 1.0
        dlq.push(&letter1).await?;
        let events = metrics.events();
        assert!(!events.is_empty());
        let last_event1 = events
            .last()
            .cloned()
            .ok_or_else(|| ContextraError::Internal("Expected metric event".to_string()))?;
        assert_eq!(
            last_event1,
            MetricEvent::Gauge {
                name: "dlq_depth".to_string(),
                value: 1.0,
                labels: vec![],
            }
        );

        // 2. Push letter2 -> Gauge depth = 2.0
        metrics.clear();
        dlq.push(&letter2).await?;
        let events = metrics.events();
        let last_event2 = events
            .last()
            .cloned()
            .ok_or_else(|| ContextraError::Internal("Expected metric event".to_string()))?;
        assert_eq!(
            last_event2,
            MetricEvent::Gauge {
                name: "dlq_depth".to_string(),
                value: 2.0,
                labels: vec![],
            }
        );

        // 3. Remove letter1 -> Gauge depth = 1.0
        metrics.clear();
        let removed = dlq.remove(&letter1).await?;
        assert!(removed);
        let events = metrics.events();
        let last_event3 = events
            .last()
            .cloned()
            .ok_or_else(|| ContextraError::Internal("Expected metric event".to_string()))?;
        assert_eq!(
            last_event3,
            MetricEvent::Gauge {
                name: "dlq_depth".to_string(),
                value: 1.0,
                labels: vec![],
            }
        );

        // 4. Drain remaining -> Gauge depth = 0.0
        metrics.clear();
        let drained = dlq.drain().await?;
        assert_eq!(drained.len(), 1);
        let events = metrics.events();
        let last_event4 = events
            .last()
            .cloned()
            .ok_or_else(|| ContextraError::Internal("Expected metric event".to_string()))?;
        assert_eq!(
            last_event4,
            MetricEvent::Gauge {
                name: "dlq_depth".to_string(),
                value: 0.0,
                labels: vec![],
            }
        );

        Ok(())
    }
}
