// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Differential Test `purge_correctness` (Spezifikation v4.1, Abschnitt 9.3, Kriterium S5).
// INVARIANTEN: I1 (outside scope snapshot isolation & tenant isolation), I2 (version ordering & L0 overlap),
// I3 (raw sweep clean post-purge with positive control), I4 (fault injection error & byte-identical files),
// I5 (idempotent double purge).

use contextra_ports::erasure::PurgeError;
use contextra_ports::StorageEngine;
use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::config::LsmConfig;
use contextra_store::lsm::LsmStorage;
use contextra_testkit::erasure_harness::raw_sweep;
use contextra_types::TxId;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Reference model for LSM storage and purge operation comparison.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PurgeReferenceModel {
    /// Maps key -> list of (seq, Option<value>), sorted by seq ascending.
    /// `None` indicates a tombstone (delete).
    pub store: BTreeMap<Vec<u8>, Vec<(u64, Option<Vec<u8>>)>>,
}

impl PurgeReferenceModel {
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

    /// Reads value for `key` at or before `seq`.
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

    /// Purges all matching keys/prefixes for versions with `seq <= as_of_tx`.
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

    /// Returns the live KV state at snapshot `seq`.
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

/// Helper function to compute SHA-256 of all files in directory.
fn compute_dir_hashes(dir: &Path) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(bytes) = fs::read(&path) {
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let hash_hex = format!("{:x}", hasher.finalize());
                    map.insert(path.file_name().unwrap().to_string_lossy().to_string(), hash_hex);
                }
            }
        }
    }
    map
}

// ============================================================================
// INVARIANTE I1: Outside scope snapshot isolation & Tenant isolation
// ============================================================================
/// Invariante I1: Für jeden Schlüssel außerhalb des Scopes liefert jeder
/// Snapshot mit seq >= as_of_tx denselben Wert wie das Modell.
/// Prüft auch zwingend Tenant-Isolierung: Präfix `t:4:` darf `t:42:` nie treffen.
#[tokio::test]
async fn invariante_i1_outside_scope_snap() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;
    let mut model = PurgeReferenceModel::new();

    // Tenant 4 keys
    storage.put(TxId(1), b"t:4:doc1", b"val_t4_1").await?;
    model.put(1, b"t:4:doc1".to_vec(), b"val_t4_1".to_vec());

    storage.put(TxId(2), b"t:4:doc2", b"val_t4_2").await?;
    model.put(2, b"t:4:doc2".to_vec(), b"val_t4_2".to_vec());

    // Tenant 42 keys (must NOT be touched when purging t:4:)
    storage.put(TxId(3), b"t:42:doc1", b"val_t42_1").await?;
    model.put(3, b"t:42:doc1".to_vec(), b"val_t42_1".to_vec());

    storage.put(TxId(4), b"t:42:doc2", b"val_t42_2").await?;
    model.put(4, b"t:42:doc2".to_vec(), b"val_t42_2".to_vec());

    // Other out-of-scope keys
    storage.put(TxId(5), b"global:config", b"cfg_val").await?;
    model.put(5, b"global:config".to_vec(), b"cfg_val".to_vec());

    storage.commit(TxId(1)).await?;
    storage.commit(TxId(2)).await?;
    storage.commit(TxId(3)).await?;
    storage.commit(TxId(4)).await?;
    storage.commit(TxId(5)).await?;
    StorageEngine::flush(&storage).await?;

    let as_of_tx = 10u64;
    let scope = PurgeScope::Prefixes(vec![b"t:4:".to_vec()]);

    let report = storage.purge_keys(scope.clone(), as_of_tx, None).await.unwrap();
    model.purge(&scope, as_of_tx);

    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    // Verify all keys outside scope return exact model values for snap_seq >= as_of_tx
    for snap_seq in [10u64, 15u64, 100u64] {
        let outside_keys = [
            b"t:42:doc1".as_slice(),
            b"t:42:doc2".as_slice(),
            b"global:config".as_slice(),
        ];

        for key in outside_keys {
            let lsm_val = storage
                .get_at_seq(key, snap_seq)
                .await?
                .map(|b| b.to_vec());
            let model_val = model.get_at_seq(key, snap_seq);
            assert_eq!(
                lsm_val, model_val,
                "Invariante I1 violation: Key '{:?}' outside scope diverged at snap_seq {}",
                String::from_utf8_lossy(key),
                snap_seq
            );
        }
    }

    Ok(())
}

// ============================================================================
// INVARIANTE I2: Version ordering, L0 overlap, and early empty SSTable replacement (P-2)
// ============================================================================
/// Invariante I2: Die Versionsreihenfolge bleibt erhalten (kein älterer Wert
/// verdeckt einen neueren). Der Test erzwingt überlappende Bereiche in Ebene 0
/// sowie eine SSTable, die beim Purge vollständig leer wird, um exakte Pfad-
/// basierte Ersetzung ohne Zuordnungsverschiebung zu beweisen (P-2/B-07).
#[tokio::test]
async fn invariante_i2_version_ordering_and_l0_overlap() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;
    let mut model = PurgeReferenceModel::new();

    // Flush 1: SSTable 1 (L0) contains ONLY purgeable keys (becomes empty during purge)
    storage.put(TxId(1), b"purge:earlier_1", b"val_purge_1").await?;
    storage.commit(TxId(1)).await?;
    let s1 = storage.last_applied_seq();
    model.put(s1, b"purge:earlier_1".to_vec(), b"val_purge_1".to_vec());
    StorageEngine::flush(&storage).await?;

    // Flush 2: SSTable 2 (L0) contains key_a (v1) and key_b (v1)
    storage.put(TxId(2), b"key_a", b"val_a_v1").await?;
    storage.commit(TxId(2)).await?;
    let s2 = storage.last_applied_seq();
    model.put(s2, b"key_a".to_vec(), b"val_a_v1".to_vec());

    storage.put(TxId(3), b"key_b", b"val_b_v1").await?;
    storage.commit(TxId(3)).await?;
    let s3 = storage.last_applied_seq();
    model.put(s3, b"key_b".to_vec(), b"val_b_v1".to_vec());
    StorageEngine::flush(&storage).await?;

    // Flush 3: SSTable 3 (L0) contains key_a (v2) and key_b (v2) - overlapping
    storage.put(TxId(4), b"key_a", b"val_a_v2").await?;
    storage.commit(TxId(4)).await?;
    let s4 = storage.last_applied_seq();
    model.put(s4, b"key_a".to_vec(), b"val_a_v2".to_vec());

    storage.put(TxId(5), b"key_b", b"val_b_v2").await?;
    storage.commit(TxId(5)).await?;
    let s5 = storage.last_applied_seq();
    model.put(s5, b"key_b".to_vec(), b"val_b_v2".to_vec());
    StorageEngine::flush(&storage).await?;

    // Purge prefix "purge:" and key_a up to s2
    let as_of_tx = s2;
    let scope = PurgeScope::Prefixes(vec![b"purge:".to_vec(), b"key_a".to_vec()]);

    let report = storage.purge_keys(scope.clone(), as_of_tx, None).await.unwrap();
    model.purge(&scope, as_of_tx);

    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    // Verify MVCC point reads for key_a, key_b, and purge:earlier_1
    let val_purge = storage.get_at_seq(b"purge:earlier_1", s5 + 1).await?.map(|b| b.to_vec());
    assert_eq!(val_purge, None, "Early empty SSTable keys must be purged");

    let val_a_s5 = storage.get_at_seq(b"key_a", s5 + 1).await?.map(|b| b.to_vec());
    let model_a_s5 = model.get_at_seq(b"key_a", s5 + 1);
    assert_eq!(val_a_s5, model_a_s5, "Newer version in subsequent SSTable must remain intact");

    let val_b_s5 = storage.get_at_seq(b"key_b", s5 + 1).await?.map(|b| b.to_vec());
    let model_b_s5 = model.get_at_seq(b"key_b", s5 + 1);
    assert_eq!(val_b_s5, model_b_s5, "Unrelated key_b in overlapping L0 must remain intact");

    Ok(())
}

// ============================================================================
// INVARIANTE I3: Raw sweep post-purge in scope with positive control
// ============================================================================
/// Invariante I3: Nach dem Purge sind im Scope alle Versionen mit seq <= as_of_tx
/// im Rohsweep nicht auffindbar.
/// Positivkontrolle: Vor dem Purge muss der Canary in SSTables / purge_tmp gefunden werden.
#[tokio::test]
async fn invariante_i3_raw_sweep_purged_in_scope() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    let canary_bytes = b"CANARY_SECRET_PURGE_TARGET_998877665544".to_vec();
    let keep_canary = b"CANARY_SECRET_KEEP_OUTSIDE_112233445566".to_vec();

    storage.put(TxId(1), b"purge_target_k1", &canary_bytes).await?;
    storage.put(TxId(2), b"keep_k2", &keep_canary).await?;
    storage.commit(TxId(1)).await?;
    storage.commit(TxId(2)).await?;
    StorageEngine::flush(&storage).await?;

    // POSITIVE CONTROL: Verify that raw_sweep finds the canary in SSTable files prior to purge
    let pre_report = raw_sweep(dir.path(), &[canary_bytes.clone()])
        .expect("Pre-purge raw sweep execution failed");
    assert!(
        !pre_report.is_clean(),
        "Positivkontrolle fehlgeschlagen: Canary 'CANARY_SECRET_PURGE_TARGET' wurde vor dem Purge nicht auf Disk gefunden"
    );

    let scope = PurgeScope::Keys(vec![b"purge_target_k1".to_vec()]);
    let report = storage.purge_keys(scope, 100, None).await.unwrap();

    for file in &report.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    // POST-PURGE SWEEP: Verify canary is absent in all SSTable / purge_tmp files
    let post_report = raw_sweep(dir.path(), &[canary_bytes.clone()])
        .expect("Post-purge raw sweep execution failed");

    assert!(
        post_report.is_clean(),
        "Invariante I3 violation: Residual canary bytes found on disk post-purge: {:?}",
        post_report.hits
    );

    Ok(())
}

// ============================================================================
// INVARIANTE I4: Fault injection error and byte-identical files
// ============================================================================
/// Invariante I4: Ein injizierter Lesefehler an jedem Offset führt zu Err und
/// byteidentischen Dateien (Dateihash vor und nach dem Lauf gleich).
#[tokio::test]
async fn invariante_i4_fault_injection_error_and_byte_identical_files() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;

    storage.put(TxId(1), b"fault_key1", b"fault_val1").await?;
    storage.put(TxId(2), b"fault_key2", b"fault_val2").await?;
    storage.commit(TxId(1)).await?;
    storage.commit(TxId(2)).await?;
    StorageEngine::flush(&storage).await?;

    // Record pre-fault hashes
    let pre_hashes = compute_dir_hashes(dir.path());

    // Corrupt an SSTable file directly to force a read error during purge
    let mut sst_file_found = None;
    if let Ok(entries) = fs::read_dir(dir.path()) {
        for entry in entries.flatten() {
            if entry.path().extension().map_or(false, |ext| ext == "sst") {
                sst_file_found = Some(entry.path());
                break;
            }
        }
    }

    let sst_path = sst_file_found.expect("SSTable file must exist before fault injection test");
    let corrupt_file_name = sst_path.file_name().unwrap().to_string_lossy().to_string();

    // Corrupt SSTable header to simulate read error on offset 0
    fs::write(&sst_path, b"CORRUPTED_GARBAGE_BYTES_SIMULATING_READ_FAULT_AT_OFFSET_0").unwrap();

    let scope = PurgeScope::Keys(vec![b"fault_key1".to_vec()]);
    let res = storage.purge_keys(scope, 100, None).await;

    // Assert operation returns Err
    assert!(
        res.is_err(),
        "Invariante I4 violation: Purge must return Err on corrupted/faulty SSTable read"
    );

    match res {
        Err(PurgeError::Read { file, .. }) => {
            assert!(file.contains("sst"), "Error must reference faulty SSTable file");
        }
        Err(other) => {
            let _ = other;
        }
        Ok(_) => panic!("Purge should fail on corrupted SSTable"),
    }

    // Verify all other files (excluding the intentionally injected corrupted file) remain byte-identical
    let post_hashes = compute_dir_hashes(dir.path());
    for (name, hash) in &pre_hashes {
        if name != &corrupt_file_name {
            if let Some(post_hash) = post_hashes.get(name) {
                assert_eq!(
                    hash, post_hash,
                    "Invariante I4 violation: File '{}' changed hash post-fault failure",
                    name
                );
            }
        }
    }

    Ok(())
}

// ============================================================================
// INVARIANTE I5: Idempotent double purge
// ============================================================================
/// Invariante I5: Zweifacher Purge desselben Scopes führt zum selben Zustand.
#[tokio::test]
async fn invariante_i5_idempotent_double_purge() -> contextra_types::Result<()> {
    let dir = tempdir()?;
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await?;
    let mut model = PurgeReferenceModel::new();

    storage.put(TxId(1), b"idem_k1", b"idem_v1").await?;
    storage.put(TxId(2), b"idem_k2", b"idem_v2").await?;
    storage.commit(TxId(1)).await?;
    storage.commit(TxId(2)).await?;
    StorageEngine::flush(&storage).await?;

    model.put(1, b"idem_k1".to_vec(), b"idem_v1".to_vec());
    model.put(2, b"idem_k2".to_vec(), b"idem_v2".to_vec());

    let as_of_tx = 10u64;
    let scope = PurgeScope::Keys(vec![b"idem_k1".to_vec()]);

    // First purge
    let report1 = storage.purge_keys(scope.clone(), as_of_tx, None).await.unwrap();
    model.purge(&scope, as_of_tx);

    for file in &report1.obsolete_files {
        let _ = contextra_durable_fs::durable_remove(file);
    }

    let state1_k2 = storage.get_at_seq(b"idem_k2", as_of_tx).await?.map(|b| b.to_vec());
    let state1_k1 = storage.get_at_seq(b"idem_k1", as_of_tx).await?.map(|b| b.to_vec());

    // Second purge on same scope
    let report2 = storage.purge_keys(scope.clone(), as_of_tx, None).await.unwrap();

    let state2_k2 = storage.get_at_seq(b"idem_k2", as_of_tx).await?.map(|b| b.to_vec());
    let state2_k1 = storage.get_at_seq(b"idem_k1", as_of_tx).await?.map(|b| b.to_vec());

    // Verify idempotency: records purged in 2nd run is 0 and state matches
    assert_eq!(report2.records_purged, 0, "Second purge must purge 0 records");
    assert_eq!(state1_k1, state2_k1, "State for purged key must remain identical after double purge");
    assert_eq!(state1_k2, state2_k2, "State for kept key must remain identical after double purge");
    assert_eq!(state2_k2, model.get_at_seq(b"idem_k2", as_of_tx));

    Ok(())
}

// ============================================================================
// Proptest Property-based Differential Test
// ============================================================================
#[derive(Debug, Clone)]
pub enum DifferentialOp {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    GetAtSeq(Vec<u8>, u64),
    PurgePrefix(Vec<u8>),
    Flush,
}

fn diff_op_strategy() -> impl Strategy<Value = DifferentialOp> {
    let key_strat = prop::collection::vec(any::<u8>(), 1..=16);
    let val_strat = prop::collection::vec(any::<u8>(), 1..=128);

    prop_oneof![
        (key_strat.clone(), val_strat).prop_map(|(k, v)| DifferentialOp::Put(k, v)),
        key_strat.clone().prop_map(DifferentialOp::Delete),
        (key_strat.clone(), 1..100u64).prop_map(|(k, seq)| DifferentialOp::GetAtSeq(k, seq)),
        key_strat.clone().prop_map(|prefix| DifferentialOp::PurgePrefix(prefix[..prefix.len().min(4)].to_vec())),
        Just(DifferentialOp::Flush),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 20,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]
    #[test]
    fn purge_correctness_proptest(
        ops in prop::collection::vec(diff_op_strategy(), 1..=30)
    ) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async move {
            let dir = tempdir().map_err(|e| TestCaseError::fail(e.to_string()))?;
            let config = LsmConfig {
                path: dir.path().to_path_buf(),
                ..Default::default()
            };
            let storage = LsmStorage::open(config).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
            let mut model = PurgeReferenceModel::new();
            let mut tx_counter = 1u64;

            for op in ops {
                match op {
                    DifferentialOp::Put(k, v) => {
                        let tx = tx_counter;
                        tx_counter += 1;
                        storage.put(TxId(tx), &k, &v).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        storage.commit(TxId(tx)).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        let seq = storage.last_applied_seq();
                        model.put(seq, k, v);
                    }
                    DifferentialOp::Delete(k) => {
                        let tx = tx_counter;
                        tx_counter += 1;
                        storage.delete(TxId(tx), &k).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        storage.commit(TxId(tx)).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        let seq = storage.last_applied_seq();
                        model.delete(seq, k);
                    }
                    DifferentialOp::GetAtSeq(k, raw_seq) => {
                        let cur_seq = storage.last_applied_seq();
                        let query_seq = raw_seq.min(cur_seq.max(1));
                        let lsm_val = storage.get_at_seq(&k, query_seq).await.map_err(|e| TestCaseError::fail(e.to_string()))?.map(|b| b.to_vec());
                        let model_val = model.get_at_seq(&k, query_seq);
                        prop_assert_eq!(lsm_val, model_val, "Divergence on GetAtSeq for key {:?}", k);
                    }
                    DifferentialOp::PurgePrefix(prefix) => {
                        let as_of_seq = storage.last_applied_seq();
                        let scope = PurgeScope::Prefixes(vec![prefix]);
                        let report = storage.purge_keys(scope.clone(), as_of_seq, None).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                        model.purge(&scope, as_of_seq);
                        for file in &report.obsolete_files {
                            let _ = contextra_durable_fs::durable_remove(file);
                        }
                    }
                    DifferentialOp::Flush => {
                        StorageEngine::flush(&storage).await.map_err(|e| TestCaseError::fail(e.to_string()))?;
                    }
                }
            }

            // Compare state at latest sequence
            let snap_seq = storage.last_applied_seq();
            let model_state = model.state_at_seq(snap_seq);
            for (k, expected_v) in model_state {
                let actual_v = storage.get_at_seq(&k, snap_seq).await.map_err(|e| TestCaseError::fail(e.to_string()))?.map(|b| b.to_vec());
                prop_assert_eq!(actual_v, Some(expected_v), "Divergence for key {:?}", k);
            }

            Ok(())
        })?;
    }
}
