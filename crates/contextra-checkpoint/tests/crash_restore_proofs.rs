//! # Crash-, Fault-Injection- und Restore-Beweise für contextra-checkpoint
//!
//! Verifiziert die geforderten Invarianten:
//! - Fehler an jedem Schritt der Checkpointerstellung (Pin, Put, Commit) hinterlassen einen sauberen Zustand.
//! - Fehler während Restore führen zu sicheren Rollbacks und atomarer Konsistenz.
//! - Roundtrip-Property: Daten schreiben -> Snapshot erstellen -> Daten mutieren -> Restore -> Daten exakt identisch.
//! - Verwaiste Checkpoints und Pins (Orphan-State) werden sauber erkannt und bereinigt.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::indexing_slicing,
    clippy::type_complexity
)]

use bytes::Bytes;
use contextra_checkpoint::contract::{BoxFuture, InMemorySnapshotSource, SnapshotSource};
use contextra_checkpoint::{
    CheckpointError, CheckpointResult, InstanceOrphanRegistry, PersistentCheckpointStore,
};
use contextra_ids::TxId;
use parking_lot::RwLock;
use proptest::prelude::*;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// Speicher mit programmierbarer Fehlerinjektion auf den SnapshotSource-Primitiven.
#[derive(Debug, Default)]
struct FaultInjectingSource {
    data: RwLock<BTreeMap<Vec<u8>, Vec<u8>>>,
    uncommitted: RwLock<BTreeMap<TxId, BTreeMap<Vec<u8>, Option<Vec<u8>>>>>,
    tx_history: RwLock<BTreeMap<TxId, BTreeMap<Vec<u8>, Option<Vec<u8>>>>>,
    last_seq: AtomicU64,
    last_tx: AtomicU64,
    pinned_seqs: RwLock<BTreeSet<u64>>,

    // Injected faults
    fail_pin: AtomicBool,
    fail_put: AtomicBool,
    fail_commit: AtomicBool,
    fail_rollback_to_tx: AtomicBool,
}

impl FaultInjectingSource {
    fn new() -> Self {
        Self::default()
    }

    fn is_pinned(&self, seq_no: u64) -> bool {
        self.pinned_seqs.read().contains(&seq_no)
    }
}

impl SnapshotSource for FaultInjectingSource {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, CheckpointResult<Option<Bytes>>> {
        Box::pin(async move {
            let val = self.data.read().get(key).cloned().map(Bytes::from);
            Ok(val)
        })
    }

    fn put<'a>(
        &'a self,
        tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            if self.fail_put.load(Ordering::SeqCst) {
                return Err(CheckpointError::Internal("Injected Put Failure".into()));
            }
            let mut uncommitted = self.uncommitted.write();
            uncommitted
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), Some(value.to_vec()));
            if tx_id.inner() < TxId::INTERNAL_BASE {
                self.last_tx.fetch_max(tx_id.inner(), Ordering::SeqCst);
            }
            Ok(())
        })
    }

    fn delete<'a>(&'a self, tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            let mut uncommitted = self.uncommitted.write();
            uncommitted
                .entry(tx_id)
                .or_default()
                .insert(key.to_vec(), None);
            if tx_id.inner() < TxId::INTERNAL_BASE {
                self.last_tx.fetch_max(tx_id.inner(), Ordering::SeqCst);
            }
            Ok(())
        })
    }

    fn commit<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            if self.fail_commit.load(Ordering::SeqCst) {
                return Err(CheckpointError::Internal("Injected Commit Failure".into()));
            }
            let mut uncommitted = self.uncommitted.write();
            if let Some(changes) = uncommitted.remove(&tx_id) {
                let mut data = self.data.write();
                for (k, v) in &changes {
                    if let Some(val) = v {
                        data.insert(k.clone(), val.clone());
                    } else {
                        data.remove(k);
                    }
                }
                self.tx_history.write().insert(tx_id, changes);
                self.last_seq.fetch_add(1, Ordering::SeqCst);
            }
            Ok(())
        })
    }

    fn rollback<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            self.uncommitted.write().remove(&tx_id);
            Ok(())
        })
    }

    fn rollback_to_tx<'a>(&'a self, tx_id: TxId) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            if self.fail_rollback_to_tx.load(Ordering::SeqCst) {
                return Err(CheckpointError::Internal(
                    "Injected RollbackToTx Failure".into(),
                ));
            }
            self.uncommitted.write().clear();

            let mut history = self.tx_history.write();
            let mut data = self.data.write();

            let to_revert: Vec<TxId> = history
                .range(TxId::new(tx_id.inner() + 1)..)
                .map(|(&tx, _)| tx)
                .collect();

            for revert_tx in to_revert.into_iter().rev() {
                if let Some(changes) = history.remove(&revert_tx) {
                    for (k, _) in changes {
                        data.remove(&k);
                    }
                }
            }
            self.last_tx.store(tx_id.inner(), Ordering::SeqCst);
            Ok(())
        })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<u64>> {
        Box::pin(async move { Ok(self.last_seq.load(Ordering::SeqCst)) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, CheckpointResult<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
    }

    fn pin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            if self.fail_pin.load(Ordering::SeqCst) {
                return Err(CheckpointError::Internal("Injected Pin Failure".into()));
            }
            self.pinned_seqs.write().insert(seq_no);
            Ok(())
        })
    }

    fn unpin_checkpoint<'a>(&'a self, seq_no: u64) -> BoxFuture<'a, CheckpointResult<()>> {
        Box::pin(async move {
            self.pinned_seqs.write().remove(&seq_no);
            Ok(())
        })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, CheckpointResult<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.read();
            let pairs = data
                .range(prefix.to_vec()..)
                .take_while(|(k, _)| k.starts_with(prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            Ok(pairs)
        })
    }
}

/// Beweis 1: Fehlerinjektion an jedem Schritt der Checkpoint-Erstellung
/// Garantiert, dass nie ein halber oder unvollständiger Snapshot sichtbar bleibt
#[tokio::test]
async fn test_fault_injection_during_snapshot_creation() {
    let source = Arc::new(FaultInjectingSource::new());
    let store = PersistentCheckpointStore::open(source.clone(), "test_crash_ns")
        .await
        .expect("open store");

    // Fall 1: Fehlerinjektion bei Pin (Schritt 1)
    source.fail_pin.store(true, Ordering::SeqCst);
    let res = store
        .create_checkpoint(
            "cp_fail_pin",
            "coll_fail",
            10,
            TxId::new(100),
            json!({"test": "pin"}),
        )
        .await;
    assert!(res.is_err(), "Pin failure must propagate error");
    assert!(
        store
            .get_checkpoint("cp_fail_pin")
            .await
            .expect("query")
            .is_none(),
        "Failed checkpoint must not exist in store"
    );
    assert!(!source.is_pinned(10), "Seq 10 must not remain pinned");
    source.fail_pin.store(false, Ordering::SeqCst);

    // Fall 2: Fehlerinjektion bei Put (Schritt 2: Metadaten schreiben)
    source.fail_put.store(true, Ordering::SeqCst);
    let res = store
        .create_checkpoint(
            "cp_fail_put",
            "coll_fail",
            20,
            TxId::new(200),
            json!({"test": "put"}),
        )
        .await;
    assert!(res.is_err(), "Put failure must propagate error");
    assert!(
        store
            .get_checkpoint("cp_fail_put")
            .await
            .expect("query")
            .is_none(),
        "Failed checkpoint must not exist in store"
    );
    assert!(!source.is_pinned(20), "Seq 20 must be cleanly unpinned");
    source.fail_put.store(false, Ordering::SeqCst);

    // Fall 3: Fehlerinjektion bei Commit (Schritt 3: Transaktion bestätigen)
    source.fail_commit.store(true, Ordering::SeqCst);
    let res = store
        .create_checkpoint(
            "cp_fail_commit",
            "coll_fail",
            30,
            TxId::new(300),
            json!({"test": "commit"}),
        )
        .await;
    assert!(res.is_err(), "Commit failure must propagate error");
    assert!(
        store
            .get_checkpoint("cp_fail_commit")
            .await
            .expect("query")
            .is_none(),
        "Failed checkpoint must not exist in store"
    );
    assert!(!source.is_pinned(30), "Seq 30 must be cleanly unpinned");
    source.fail_commit.store(false, Ordering::SeqCst);

    // Fall 4: Erfolgreicher Durchlauf nach Fehlerbeseitigung
    let ok_res = store
        .create_checkpoint(
            "cp_success",
            "coll_fail",
            40,
            TxId::new(400),
            json!({"test": "ok"}),
        )
        .await;
    assert!(ok_res.is_ok(), "Clean creation must succeed");
    assert!(
        store
            .get_checkpoint("cp_success")
            .await
            .expect("query")
            .is_some(),
        "Checkpoint must exist after clean creation"
    );
    assert!(source.is_pinned(40), "Seq 40 must be pinned");
}

/// Beweis 2: Fehlerinjektion bei Wiederherstellung (Restore)
#[tokio::test]
async fn test_fault_injection_during_restore() {
    let source = Arc::new(FaultInjectingSource::new());
    let store = PersistentCheckpointStore::open(source.clone(), "test_restore_ns")
        .await
        .expect("open store");

    store
        .create_checkpoint(
            "cp_target",
            "coll_r",
            50,
            TxId::new(500),
            json!({"version": 1}),
        )
        .await
        .expect("create target checkpoint");

    // Fehlerinjektion bei Rollback-Aufruf
    source.fail_rollback_to_tx.store(true, Ordering::SeqCst);
    let restore_err = store.restore_checkpoint("cp_target").await;
    assert!(
        restore_err.is_err(),
        "Restore must fail if rollback_to_tx fails"
    );

    // Nach Fehlerbeseitigung
    source.fail_rollback_to_tx.store(false, Ordering::SeqCst);
    let restore_ok = store.restore_checkpoint("cp_target").await;
    assert!(restore_ok.is_ok(), "Restore must succeed after clear");
}

/// Beweis 3: Orphan-Erkennung und Bereinigung
#[tokio::test]
async fn test_orphan_detection_and_sweep() {
    let source = Arc::new(FaultInjectingSource::new());
    let orphan_reg = Arc::new(InstanceOrphanRegistry::new(""));

    // Pin registrieren und unvollendet droppen
    let pin_guard =
        contextra_checkpoint::PinGuard::pin(source.clone(), 999, orphan_reg.clone()).await;
    assert!(pin_guard.is_ok());
    drop(pin_guard);

    assert_eq!(
        orphan_reg.get_orphan_pins().len(),
        1,
        "Orphan pin must be registered"
    );
    assert!(
        source.is_pinned(999),
        "Source should still have 999 pinned before recovery"
    );

    // Bereinigung über PersistentCheckpointStore::open_and_recover_with_orphan_registry
    let (_store, report) = PersistentCheckpointStore::open_and_recover_with_orphan_registry(
        source.clone(),
        "test_orphan_sweep",
        orphan_reg.clone(),
    )
    .await
    .expect("open_and_recover");

    assert_eq!(report.recovered_pins.len(), 1);
    assert_eq!(report.recovered_pins[0], 999);
    assert!(
        !source.is_pinned(999),
        "Source must no longer have 999 pinned after sweep"
    );
    assert_eq!(orphan_reg.get_orphan_pins().len(), 0);
}

// Beweis 4: Property-basierte Roundtrip-Prüfung (Snapshot -> Mutation -> Restore -> identischer Zustand)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_roundtrip_property_snapshot_restore(
        initial_key in "[a-z0-9_]{1,16}",
        initial_val in "[a-z0-9_]{1,32}",
        mutated_val in "[a-z0-9_]{1,32}",
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = Arc::new(InMemorySnapshotSource::new());
            let store = PersistentCheckpointStore::open(storage.clone(), "prop_roundtrip")
                .await
                .unwrap();

            // 1. Initialzustand schreiben
            let tx1 = TxId::new(100);
            storage.put(tx1, initial_key.as_bytes(), initial_val.as_bytes()).await.unwrap();
            storage.commit(tx1).await.unwrap();

            assert_eq!(storage.get_sync(initial_key.as_bytes()), Some(initial_val.as_bytes().to_vec()));

            // 2. Checkpoint bei Zustand 1 anlegen
            store.create_checkpoint("cp_base", "coll_base", 1, tx1, json!({})).await.unwrap();

            // 3. Zustand mutieren
            let tx2 = TxId::new(200);
            storage.put(tx2, initial_key.as_bytes(), mutated_val.as_bytes()).await.unwrap();
            storage.commit(tx2).await.unwrap();

            assert_eq!(storage.get_sync(initial_key.as_bytes()), Some(mutated_val.as_bytes().to_vec()));

            // 4. Restore auf Checkpoint 1
            store.restore_checkpoint("cp_base").await.unwrap();

            // 5. Verifikation: Zustand ist exakt identisch zu Zustand 1
            assert_eq!(storage.get_sync(initial_key.as_bytes()), Some(initial_val.as_bytes().to_vec()));
        });
    }
}
