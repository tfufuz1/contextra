// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Testsuite J05 für Purge-Resume Standalone Error (N1), Idempotenz und SSTable-Swap-Invarianten (WP-5a).
// INVARIANTEN:
// - Standalone resume fails closed with Err when intent phase <= PurgeCompaction, preserving `purge.intent` on disk.
// - Resumption with active `LsmStorage` completes P4 compaction and physically purges target keys from SSTables.
// - Double/nested resume is a safe no-op returning `Ok(None)`.
// - Differential proptest against BTreeMap reference model verifies SSTable-swap and post-purge query correctness.

use contextra_ports::erasure::PurgeError;
use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::engine::LsmStorage;
use contextra_store::lsm::purge::failpoint::OrxPurgeError;
use contextra_store::lsm::purge::intent::OrxPurgeIntent;
use contextra_store::lsm::purge::orchestrator::OrxHooks;
use contextra_store::lsm::purge::plan::PurgePhase;
use contextra_store::lsm::purge::resume::{resume_pending, resume_pending_for_storage};
use contextra_testkit::erasure_harness::raw_sweep;
use contextra_types::TxId;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::collections::BTreeMap;
use tempfile::tempdir;

/// Reference model for LSM storage state and purge behavior verification.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct J05ReferenceModel {
    /// Maps key -> list of (seq, Option<value>), sorted by seq ascending.
    /// `None` represents a deleted key (tombstone).
    pub store: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>,
}

impl J05ReferenceModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn put(&mut self, seq: u64, key: Vec<u8>, value: Vec<u8>) {
        let entries = self.store.entry(key).or_default();
        entries.push((seq, Some(value)));
        entries.sort_by_key(|(s, _)| *s);
    }

    pub fn delete(&mut self, seq: u64, key: Vec<u8>) {
        let entries = self.store.entry(key).or_default();
        entries.push((seq, None));
        entries.sort_by_key(|(s, _)| *s);
    }

    pub fn get_at_seq(&self, key: &[u8], seq: u64) -> Option<Vec<u8>> {
        if let Some(entries) = self.store.get(key) {
            for (v_seq, val) in entries.iter().rev() {
                if *v_seq <= seq {
                    return val.clone();
                }
            }
        }
        None
    }

    pub fn purge(&mut self, scope: &PurgeScope, as_of_tx: u64) {
        let mut empty_keys = Vec::new();
        for (key, entries) in self.store.iter_mut() {
            if scope.matches_key(key) {
                entries.retain(|(seq, _)| *seq > as_of_tx);
                if entries.is_empty() {
                    empty_keys.push(key.clone());
                }
            }
        }
        for key in empty_keys {
            self.store.remove(&key);
        }
    }

    pub fn state_at_seq(&self, seq: u64) -> BTreeMap<Vec<u8>, Vec<u8>> {
        let mut result = BTreeMap::new();
        for (key, _) in self.store.iter() {
            if let Some(val) = self.get_at_seq(key, seq) {
                result.insert(key.clone(), val);
            }
        }
        result
    }
}

// ============================================================================
// Crash-Resume Matrix: Absturz nach Purge-Phasen P1..P4 -> Standalone Fehler & Storage Completion
// ============================================================================
#[tokio::test]
async fn test_crash_resume_matrix_p1_to_p4_standalone_error_and_storage_completion(
) -> contextra_types::Result<()> {
    let phases_to_test = [
        PurgePhase::Admitted,
        PurgePhase::FenceAcquired,
        PurgePhase::Flush,
        PurgePhase::PurgeCompaction,
    ];

    for phase in phases_to_test {
        let dir = tempdir()?;
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };

        let canary_key = b"t:1:purge_canary_key_j05".to_vec();
        let canary_val = b"CANARY_DATA_PURGE_TARGET_J05_998877".to_vec();

        // 1. Storage initialisieren und Daten schreiben
        {
            let storage = LsmStorage::open(config.clone()).await?;
            storage.put(TxId(1), &canary_key, &canary_val).await?;
            storage.put(TxId(2), b"t:1:keep_key", b"KEEP_VALUE").await?;
            storage.commit(TxId(1)).await?;
            storage.commit(TxId(2)).await?;
            StorageEngine::flush(&storage).await?;
        }

        // Positivkontrolle: Canary vor Purge auf Disk vorhanden
        let pre_sweep =
            raw_sweep(dir.path(), &[canary_val.clone()]).expect("Pre-purge raw sweep executed");
        assert!(
            !pre_sweep.is_clean(),
            "Positivkontrolle: Canary muss vor Purge auf Disk existieren (Phase {:?})",
            phase
        );

        // 2. Synthetische `purge.intent` Datei im Testordner mit Phase phase erzeugen
        let scope = PurgeScope::Keys(vec![canary_key.clone()]);
        let purge_id = [0x55u8; 16];
        let intent = OrxPurgeIntent::new(purge_id, &scope, 100, None, None, phase);
        let intent_file = dir.path().join("purge.intent");
        intent
            .write_to_file(&intent_file)
            .expect("Write synthetic purge.intent succeeds");
        assert!(
            intent_file.exists(),
            "purge.intent file exists before resume"
        );

        let hooks = OrxHooks::default();

        // 3. Standalone resume_pending ohne LsmStorage handle -> MUSS mit Err fehlschlagen!
        let standalone_res = resume_pending(dir.path(), &hooks, None).await;
        assert!(
            standalone_res.is_err(),
            "Standalone resume without LsmStorage for phase {:?} must fail with Err",
            phase
        );
        match standalone_res {
            Err(OrxPurgeError::Io(io_err)) => {
                assert!(
                    io_err.to_string().contains("Storage handle required")
                        || io_err.to_string().contains("Purge failed at phase"),
                    "Error description must reference storage requirement or phase failure: {}",
                    io_err
                );
            }
            Err(other) => panic!("Expected OrxPurgeError::Io, got: {:?}", other),
            Ok(_) => panic!(
                "Standalone resume should not succeed without storage for phase {:?}",
                phase
            ),
        }

        // ZWINGENDE INVARIANTE: purge.intent DARF NICHT gelöscht worden sein!
        assert!(
            intent_file.exists(),
            "purge.intent file must remain on disk when standalone resume fails at phase {:?}",
            phase
        );

        // 4. Nun Storage öffnen und resume_pending_for_storage ausführen
        let storage = LsmStorage::open(config).await?;
        let storage_res = resume_pending_for_storage(&storage, &hooks, None).await?;
        assert!(
            storage_res.is_some(),
            "resume_pending_for_storage must complete pending purge for phase {:?}",
            phase
        );

        // ZWINGENDE INVARIANTE: purge.intent MUSS nach erfolgreicher Resumption entfernt sein
        assert!(
            !intent_file.exists(),
            "purge.intent file must be deleted after successful completion at phase {:?}",
            phase
        );

        // 5. Verifikation: Key per get_at_seq nicht mehr auffindbar und physisch per raw_sweep weg
        let get_res = storage.get_at_seq(&canary_key, 100).await?;
        assert_eq!(
            get_res, None,
            "Purged key must return None post-resume for phase {:?}",
            phase
        );

        let post_sweep =
            raw_sweep(dir.path(), &[canary_val.clone()]).expect("Post-purge raw sweep executed");
        assert!(
            post_sweep.is_clean(),
            "Invariante N1/I3: Canary must be physically purged post-resume for phase {:?}",
            phase
        );
    }

    Ok(())
}

// ============================================================================
// Idempotenz: Doppeltes Resume, Leeres Intent, 0 Keys
// ============================================================================
#[tokio::test]
async fn test_idempotency_double_resume_and_zero_keys() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let k1 = b"t:1:idem_k1".to_vec();
    let v1 = b"IDEM_VAL_1".to_vec();
    storage.put(TxId(1), &k1, &v1).await?;
    storage.commit(TxId(1)).await?;
    StorageEngine::flush(&storage).await?;

    let hooks = OrxHooks::default();

    // 1. Erster Purge über Intent
    let scope = PurgeScope::Keys(vec![k1.clone()]);
    let intent = OrxPurgeIntent::new([0x11; 16], &scope, 10, None, None, PurgePhase::Admitted);
    let intent_file = dir.path().join("purge.intent");
    intent.write_to_file(&intent_file).unwrap();

    let res1 = resume_pending_for_storage(&storage, &hooks, None).await?;
    assert!(res1.is_some(), "First resume succeeds and returns report");

    // 2. Zweites Resume (doppeltes Resume) -> No-op, gibt Ok(None) zurück
    let res2 = resume_pending_for_storage(&storage, &hooks, None).await?;
    assert!(
        res2.is_none(),
        "Second resume must be a clean no-op returning Ok(None)"
    );

    // 3. Intent mit 0 Keys / leeres Scope
    let empty_scope = PurgeScope::Keys(Vec::new());
    let empty_intent = OrxPurgeIntent::new(
        [0x22; 16],
        &empty_scope,
        10,
        None,
        None,
        PurgePhase::Admitted,
    );
    empty_intent.write_to_file(&intent_file).unwrap();

    let res_empty = resume_pending_for_storage(&storage, &hooks, None).await?;
    assert!(
        res_empty.is_some(),
        "Resume with 0 keys intent completes successfully"
    );
    let report_empty = res_empty.unwrap();
    assert_eq!(report_empty.records_purged, 0);

    Ok(())
}

// ============================================================================
// Edge-Case: Keys, die zwischenzeitlich neu geschrieben wurden (Tombstone vs Neuwrite)
// ============================================================================
#[tokio::test]
async fn test_tombstone_vs_rewritten_key_sequence_filtering() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let key = b"t:1:rewrite_key".to_vec();

    // Tx 1: Erstes Schreiben (soll ge-purged werden)
    storage.put(TxId(1), &key, b"OLD_VAL_TX_1").await?;
    storage.commit(TxId(1)).await?;
    let seq1 = storage.last_applied_seq();
    StorageEngine::flush(&storage).await?;

    // Tx 2: Neu-Schreiben nach dem Cutoff (darf NICHT ge-purged werden)
    storage.put(TxId(2), &key, b"NEW_VAL_TX_2").await?;
    storage.commit(TxId(2)).await?;
    let seq2 = storage.last_applied_seq();
    StorageEngine::flush(&storage).await?;

    // Purge mit as_of_tx = seq1 (reicht nur bis Tx 1)
    let scope = PurgeScope::Keys(vec![key.clone()]);
    let report = storage.purge_keys(scope, seq1, None).await?;

    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    // Neuerer Wert bei seq2 muss uneingeschränkt lesbar sein
    let val_seq2 = storage
        .get_at_seq(&key, seq2 + 10)
        .await?
        .map(|b| b.to_vec());
    assert_eq!(
        val_seq2,
        Some(b"NEW_VAL_TX_2".to_vec()),
        "Key rewritten post-as_of_tx must remain intact post-purge"
    );

    Ok(())
}

// ============================================================================
// Proptest: SSTable-Swap unter parallelen Operationen gegen BTreeMap Referenzmodell
// ============================================================================
#[derive(Debug, Clone)]
pub enum J05PropOp {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    PurgePrefix(Vec<u8>),
    Flush,
}

fn j05_prop_op_strategy() -> impl Strategy<Value = J05PropOp> {
    let key_strat = prop::collection::vec(any::<u8>(), 1..=12);
    let val_strat = prop::collection::vec(any::<u8>(), 1..=64);

    prop_oneof![
        (key_strat.clone(), val_strat).prop_map(|(k, v)| J05PropOp::Put(k, v)),
        key_strat.clone().prop_map(J05PropOp::Delete),
        key_strat
            .clone()
            .prop_map(|prefix| J05PropOp::PurgePrefix(prefix[..prefix.len().min(3)].to_vec())),
        Just(J05PropOp::Flush),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 15,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]
    #[test]
    fn prop_sstable_swap_and_reference_model(
        ops in prop::collection::vec(j05_prop_op_strategy(), 1..=25)
    ) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async move {
            let dir = tempdir().map_err(|e| TestCaseError::fail(e.to_string()))?;
            let config = LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            };
            let storage = LsmStorage::open(config).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
            let mut model = J05ReferenceModel::new();
            let mut tx_counter = 1u64;

            for op in ops {
                match op {
                    J05PropOp::Put(k, v) => {
                        let tx = tx_counter;
                        tx_counter += 1;
                        storage.put(TxId(tx), &k, &v).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        storage.commit(TxId(tx)).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        let seq = storage.last_applied_seq();
                        model.put(seq, k, v);
                    }
                    J05PropOp::Delete(k) => {
                        let tx = tx_counter;
                        tx_counter += 1;
                        storage.delete(TxId(tx), &k).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        storage.commit(TxId(tx)).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        let seq = storage.last_applied_seq();
                        model.delete(seq, k);
                    }
                    J05PropOp::PurgePrefix(prefix) => {
                        let as_of_seq = storage.last_applied_seq();
                        let scope = PurgeScope::Prefixes(vec![prefix]);
                        match storage.purge_keys(scope.clone(), as_of_seq, None).await {
                            Ok(report) => {
                                model.purge(&scope, as_of_seq);
                                for file in &report.obsolete_files {
                                    let _ = contextra_durable_fs::durable_remove(file);
                                }
                            }
                            Err(PurgeError::CompactionBusy) => {
                                // CompactionBusy is a valid fail-closed outcome under concurrency
                            }
                            Err(e) => return Err(TestCaseError::fail(format!("Purge failed: {:?}", e))),
                        }
                    }
                    J05PropOp::Flush => {
                        StorageEngine::flush(&storage).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                    }
                }
            }

            // State-Vergleich am Ende
            let snap_seq = storage.last_applied_seq();
            let model_state = model.state_at_seq(snap_seq);
            for (k, expected_v) in model_state {
                let actual_v = storage.get_at_seq(&k, snap_seq).await.map_err(|e| TestCaseError::fail(e.to_string()))?.map(|b| b.to_vec());
                prop_assert_eq!(actual_v, Some(expected_v), "Divergence post-swap for key {:?}", k);
            }

            Ok(())
        })?;
    }
}
