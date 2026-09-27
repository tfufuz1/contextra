// FILE-CONTEXT
// STAND: 2026-09-09T00:00:00Z
// ZWECK: Crate-internes MVCC Snapshot-Pinning und TxId-skopierte Rollbacks.
// INVARIANTEN: Crate-intern (pub(crate)) — öffentliche Checkpoints nur via contextra-checkpoint (ADR-011).
// HOTSPOTS: Checkpointer::create_checkpoint
// SIEHE AUCH: DECISIONS.md ADR-011, ADR-015

//! Native State Checkpointing (Crate-internal MVCC Snapshot-Pinning).
//!
//! # Architektur & Sichtbarkeit
//! Dieser Modul ist strikt crate-intern (`pub(crate)`). Er bietet LSM-spezifisches Snapshot-Pinning
//! und TxId-skopierte Transactional Rollbacks für MVCC.
//!
//! WARNUNG: Dieser Typ darf NIEMALS außerhalb von `contextra-store` exportiert oder direkt verwendet werden.
//! Für die öffentliche Checkpoint-API (benannte Checkpoints, Trait-basierter `CheckpointCoordinator`, RAII `CheckpointGuard`)
//! ist gemäß ADR-011 ausschließlich `contextra-checkpoint` zu verwenden.

// DECISION-REF: ADR-011 — Consolidated Checkpoint Subsystem Architecture
// DECISION-REF: ADR-015 — Integration von RAII CheckpointGuard in contextra-checkpoint (AGT-CKPT-001 / AGT-STORE-002)
// ARCHITEKTUR: `contextra-checkpoint` stellt den generischen `CheckpointGuard<S: StorageEngine>` und `PersistentCheckpointStore`
//             bereit. `contextra-store::checkpoint` bietet LSM-spezifische transactional rollbacks (TxId-skopiert).
use crate::lsm::LsmStorage;
use contextra_core::{Result, TxId};
use contextra_ports::Clock;
use std::sync::Arc;

/// Represents a Point-in-Time snapshot of the agent's memory state.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct StateCheckpoint {
    pub tx_id: TxId,
    pub timestamp_ms: u64,
}

/// The Checkpointer manages WAL replay bounds for deterministic time-travel.
#[allow(dead_code)]
pub struct Checkpointer {
    storage: Arc<LsmStorage>,
    clock: Arc<dyn Clock>,
}

#[allow(dead_code)]
impl Checkpointer {
    /// Creates a new Checkpointer.
    pub fn new(storage: Arc<LsmStorage>, clock: Arc<dyn Clock>) -> Self {
        Self { storage, clock }
    }

    /// Records a new checkpoint at the current transaction ID marking an agent step.
    ///
    /// Uses the injected [`Clock`] port for deterministic time derivation (INV-CHECKPOINT-DETERMINISM-1).
    pub fn create_checkpoint(&self, tx_id: TxId) -> Result<StateCheckpoint> {
        let timestamp_ms = self.clock.now_unix_nanos() / 1_000_000;
        Ok(StateCheckpoint {
            tx_id,
            timestamp_ms,
        })
    }

    /// Rolls the database state back to a specific checkpoint.
    /// This is the foundation for Time-Travel Debugging in SAOS.
    pub(crate) async fn rollback_to(&self, checkpoint: &StateCheckpoint) -> Result<()> {
        tracing::info!(
            "Initiating Time-Travel Rollback to TX: {}",
            checkpoint.tx_id
        );
        self.storage.rollback_to_tx(checkpoint.tx_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsm::{LsmConfig, LsmStorage};
    use contextra_core::StorageEngine;
    use contextra_testkit::ManualClock;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_checkpoint_timestamp_deterministic_with_manual_clock() {
        let tmp = TempDir::new().expect("temp dir");
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(LsmStorage::new(config).await.expect("create storage"));

        // Test 1: ManualClock initialized to 1,000,000,000 nanos = 1,000 ms
        let manual_clock = Arc::new(ManualClock::new(1_000_000_000));
        let checkpointer = Checkpointer::new(storage.clone(), manual_clock.clone());

        let tx1 = TxId::new(1);
        let cp1 = checkpointer.create_checkpoint(tx1).expect("cp1");

        // Calling again without advancing clock yields identical timestamp_ms
        let tx2 = TxId::new(2);
        let cp2 = checkpointer.create_checkpoint(tx2).expect("cp2");

        assert_eq!(cp1.timestamp_ms, 1_000);
        assert_eq!(cp2.timestamp_ms, 1_000);
        assert_eq!(cp1.timestamp_ms, cp2.timestamp_ms);

        // Test 2: Advancing ManualClock by 5s (5,000 ms)
        manual_clock.advance(Duration::from_secs(5));

        let tx3 = TxId::new(3);
        let cp3 = checkpointer.create_checkpoint(tx3).expect("cp3");

        assert_eq!(cp3.timestamp_ms, 6_000);
        assert_eq!(cp3.timestamp_ms - cp1.timestamp_ms, 5_000);
    }

    #[tokio::test]
    async fn test_rollback_to_checkpoint() {
        let tmp = TempDir::new().expect("temp dir"); // expect
        let config = LsmConfig {
            path: tmp.path().to_path_buf(),
            ..Default::default()
        };
        let storage = Arc::new(LsmStorage::new(config).await.expect("create storage")); // expect
        let clock = Arc::new(contextra_ports::SystemClock::new());
        let checkpointer = Checkpointer::new(storage.clone(), clock);

        let tx1 = TxId::new(1);
        storage.put(tx1, b"key1", b"val1").await.unwrap(); // unwrap
        storage.commit(tx1).await.unwrap(); // unwrap

        let cp1 = checkpointer.create_checkpoint(tx1).unwrap(); // unwrap

        let tx2 = TxId::new(2);
        storage.put(tx2, b"key2", b"val2").await.unwrap(); // unwrap
        storage.commit(tx2).await.unwrap(); // unwrap

        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val1"))
        ); // unwrap
        assert_eq!(
            storage.get(b"key2").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val2"))
        ); // unwrap

        checkpointer.rollback_to(&cp1).await.expect("rollback"); // expect

        assert_eq!(
            storage.get(b"key1").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val1"))
        ); // unwrap
        assert_eq!(storage.get(b"key2").await.unwrap(), None); // unwrap

        let tx3 = TxId::new(3);
        storage.put(tx3, b"key3", b"val3").await.unwrap(); // unwrap
        storage.commit(tx3).await.unwrap(); // unwrap
        assert_eq!(
            storage.get(b"key3").await.unwrap(),
            Some(bytes::Bytes::from_static(b"val3"))
        );
    }
}
