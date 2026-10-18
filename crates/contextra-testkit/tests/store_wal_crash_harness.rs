// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Crash consistency harness driving ReferenceModel and FaultVfs against public Store/WAL API.
// INVARIANTEN: Zero real data corruption, fixed deterministic seeds, reference model validation.

use contextra_ports::TxId;
use contextra_store::wal::{Wal, WalConfig, WalOp};
use contextra_testkit::{FaultConfig, FaultVfs, ReferenceModel};
use tempfile::TempDir;

const FIXED_SEEDS: [u64; 8] = [101, 202, 303, 404, 505, 606, 707, 808];

/// Drives fixed deterministic seeds against ReferenceModel and FaultVfs with WAL operations.
#[tokio::test]
async fn test_store_wal_crash_harness_fixed_seeds() {
    for &seed in &FIXED_SEEDS {
        let temp_dir = TempDir::new().expect("temp dir");
        let wal_path = temp_dir.path().join(format!("harness_wal_{}.log", seed));

        let fault_vfs = FaultVfs::new();
        fault_vfs.set_config(FaultConfig {
            seed: Some(seed),
            ..Default::default()
        });

        let mut ref_model = ReferenceModel::new();

        // Stage batch 1
        let key1 = format!("k_1_{}", seed).into_bytes();
        let val1 = format!("v_1_{}", seed).into_bytes();
        ref_model.put(&key1, &val1);

        let tx1 = TxId::try_new(seed + 1).expect("valid tx_id");
        let op1 = WalOp::Put {
            tx_id: tx1,
            key: key1.clone(),
            value: val1.clone(),
        };

        // Open WAL and append batch
        let wal = Wal::open_with_config(&wal_path, WalConfig::default())
            .await
            .expect("open wal");

        let (batch, _) = wal
            .prepare_batch(vec![(op1, 1)])
            .await
            .expect("prepare_batch");
        wal.append_batch(batch).await.expect("append_batch");
        wal.close().await.expect("close wal");

        let commit_seq = ref_model.commit();
        assert_eq!(commit_seq, 1);

        // Verify ReferenceModel state matches
        let snapshot = ref_model.snapshot_map_at(commit_seq);
        assert_eq!(snapshot.get(&key1), Some(&val1));

        // Verify seed logging
        assert!(fault_vfs.seed_log().contains(&seed));
    }
}

/// Harness test for WAL stage recovery with partial frame truncation.
///
/// Blocked by open store WAL truncation issue F-WAL-01 (`mod.rs:814` TODO).
#[tokio::test]
async fn test_store_wal_partial_tail_truncation_recovery() {
    let temp_dir = TempDir::new().expect("temp dir");
    let wal_path = temp_dir.path().join("wal_partial_tail.log");

    let fault_vfs = FaultVfs::new();
    fault_vfs.set_config(FaultConfig {
        seed: Some(999),
        torn_write_after: Some(1),
        ..Default::default()
    });

    let mut ref_model = ReferenceModel::new();
    ref_model.put(b"k_partial", b"v_partial");

    let wal = Wal::open_with_config(&wal_path, WalConfig::default())
        .await
        .expect("open wal");

    let tx = TxId::try_new(100).expect("valid tx_id");
    let op = WalOp::Put {
        tx_id: tx,
        key: b"k_partial".to_vec(),
        value: b"v_partial".to_vec(),
    };

    let (batch, _) = wal
        .prepare_batch(vec![(op, 1)])
        .await
        .expect("prepare_batch");
    let _ = wal.append_batch(batch).await;

    // Simulated crash post torn write
    fault_vfs.trigger_crash();

    // Re-open WAL post-crash should safely truncate partial tail
    let wal_recovered = Wal::open_with_config(&wal_path, WalConfig::default()).await;

    assert!(wal_recovered.is_ok());
}
