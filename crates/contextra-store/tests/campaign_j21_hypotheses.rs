//! Campaign J-21 Integration Test Suite verifying hypotheses H1 through H8
//! for Intent-Locks, Tenant Isolation, Deletion Proof Binding, DirLock,
//! Observer Fail-Open, Differential BTreeMap Model, and System Pressure.

use bytes::Bytes;
use contextra_core::{StorageEngine, TenantId, TxId};
use contextra_crypto::{crypto::KeyManager, kv_shredding::KeyRegistry};
use contextra_store::{
    kv::{KvDeleteMode, KvSegmentConfig, KvSegmentManager},
    lsm::{
        observer::{CommittedBatch, ObserverRegistry, WalObserver},
        DurabilityMode, LsmConfig, LsmStorage,
    },
    system_pressure::{PressureLevel, SystemPressureMonitor},
    tenant_codec::TenantScopedStorage,
};
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::Barrier;

async fn setup_test_lsm() -> (TempDir, LsmStorage) {
    let dir = TempDir::new().expect("Failed to create tempdir");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        tx_timeout: Duration::from_secs(5),
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let lsm = LsmStorage::new(config)
        .await
        .expect("Failed to open LsmStorage");
    (dir, lsm)
}

// ============================================================================
// H1: Intent Lock Cleanup & Abortion Resilience
// ============================================================================

#[tokio::test]
async fn test_h1_put_if_absent_intent_lock_cleanup_on_abort() {
    let (_dir, lsm) = setup_test_lsm().await;
    let key = b"key_h1";
    let val = b"val_h1";

    for i in 1..=1000u64 {
        let tx = TxId::new(i);
        // 1. Start put_if_absent in a future but abort/drop before commit
        {
            let fut = lsm.put_if_absent(tx, key, val);
            tokio::select! {
                _ = fut => {},
                _ = tokio::task::yield_now() => {},
            }
        }

        // Rollback transaction to clear both tx_buffer and intent_locks
        lsm.rollback(tx).await.unwrap();

        // 2. Next put_if_absent on same key must succeed
        let tx_next = TxId::new(i + 10000);
        let res = lsm.put_if_absent(tx_next, key, val).await;
        assert!(
            res.is_ok(),
            "Iteration {i}: put_if_absent failed with error: {:?}",
            res.err()
        );
        assert!(
            res.unwrap(),
            "Iteration {i}: put_if_absent failed to acquire lock/insert"
        );
        lsm.commit(tx_next).await.unwrap();

        // Clean up key for next iteration
        let tx_del = TxId::new(i + 20000);
        lsm.delete(tx_del, key).await.unwrap();
        lsm.commit(tx_del).await.unwrap();
    }

    // Direct API checks for lock cleanup functions
    let tx_dummy = TxId::new(99999);
    lsm.put_if_absent(tx_dummy, b"dummy_k", b"v").await.unwrap();
    lsm.rollback(tx_dummy).await.unwrap();
    lsm.clear_intent_locks_for_tx(tx_dummy);

    // After rollback and clear, another tx must acquire dummy_k
    let tx_check = TxId::new(999990);
    assert!(lsm.put_if_absent(tx_check, b"dummy_k", b"v").await.unwrap());

    lsm.clear_intent_locks_above_tx(TxId::new(100));
    lsm.cleanup_intent_locks_where(|_| true);
}

// ============================================================================
// H2: TOCTOU & Concurrent put_if_absent Safety
// ============================================================================

#[tokio::test]
async fn test_h2_concurrent_put_if_absent_toctou_safety() {
    let (_dir, lsm) = setup_test_lsm().await;
    let lsm = Arc::new(lsm);

    for run in 0..20 {
        let key = format!("toctou_key_{run}").into_bytes();
        let barrier = Arc::new(Barrier::new(64));
        let winners = Arc::new(AtomicU64::new(0));
        let winner_val = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let mut handles = Vec::new();
        for i in 0..64 {
            let lsm_c = Arc::clone(&lsm);
            let barrier_c = Arc::clone(&barrier);
            let winners_c = Arc::clone(&winners);
            let winner_val_c = Arc::clone(&winner_val);
            let key_c = key.clone();
            let val = format!("val_{i}").into_bytes();
            let tx = TxId::new((run * 100 + i + 1) as u64);

            handles.push(tokio::spawn(async move {
                barrier_c.wait().await;
                // Add micro-delay variation
                if i % 2 == 1 {
                    tokio::task::yield_now().await;
                }
                let inserted = lsm_c.put_if_absent(tx, &key_c, &val).await.unwrap_or(false);
                if inserted {
                    let commit_res = lsm_c.commit(tx).await;
                    if commit_res.is_ok() {
                        winners_c.fetch_add(1, Ordering::SeqCst);
                        let mut w = winner_val_c.lock().await;
                        *w = val;
                    }
                } else {
                    let _ = lsm_c.rollback(tx).await;
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(
            winners.load(Ordering::SeqCst),
            1,
            "Run {run}: Exactly one winner expected for put_if_absent"
        );

        let committed_val = lsm.get(&key).await.unwrap().unwrap();
        let expected_val = winner_val.lock().await.clone();
        assert_eq!(
            committed_val.as_ref(),
            expected_val.as_slice(),
            "Run {run}: Committed key must equal winner's value"
        );
    }
}

// ============================================================================
// H3: TenantScopedStorage & Multi-Tenant Isolation
// ============================================================================

#[tokio::test]
async fn test_h3_tenant_scoped_storage_isolation() {
    let (_dir, lsm) = setup_test_lsm().await;
    let lsm = Arc::new(lsm);

    let t1 = TenantId::try_new(1).unwrap();
    let t10 = TenantId::try_new(10).unwrap();
    let t100 = TenantId::try_new(100).unwrap();
    let t11 = TenantId::try_new(11).unwrap();
    let tmax = TenantId::try_new(u64::MAX).unwrap();

    // TenantId(0) must be rejected / cannot be created
    assert!(TenantId::try_new(0).is_err());

    let store1 = TenantScopedStorage::new(lsm.clone(), t1);
    let store10 = TenantScopedStorage::new(lsm.clone(), t10);
    let store100 = TenantScopedStorage::new(lsm.clone(), t100);
    let store11 = TenantScopedStorage::new(lsm.clone(), t11);
    let storemax = TenantScopedStorage::new(lsm.clone(), tmax);

    let tx = TxId::new(1001);

    // Overlapping keys
    let common_key = b"user/profile";
    let ff_prefix_key = vec![0xFF, 0xFF, 0x01];

    store1.put(tx, common_key, b"v1").await.unwrap();
    store10.put(tx, common_key, b"v10").await.unwrap();
    store100.put(tx, common_key, b"v100").await.unwrap();
    store11.put(tx, common_key, b"v11").await.unwrap();
    storemax.put(tx, common_key, b"vmax").await.unwrap();

    store1.put(tx, &ff_prefix_key, b"v1_ff").await.unwrap();
    store10.put(tx, &ff_prefix_key, b"v10_ff").await.unwrap();

    lsm.commit(tx).await.unwrap();

    // Verify isolation on get
    assert_eq!(
        store1.get(common_key).await.unwrap().unwrap(),
        Bytes::from_static(b"v1")
    );
    assert_eq!(
        store10.get(common_key).await.unwrap().unwrap(),
        Bytes::from_static(b"v10")
    );
    assert_eq!(
        store100.get(common_key).await.unwrap().unwrap(),
        Bytes::from_static(b"v100")
    );
    assert_eq!(
        store11.get(common_key).await.unwrap().unwrap(),
        Bytes::from_static(b"v11")
    );
    assert_eq!(
        storemax.get(common_key).await.unwrap().unwrap(),
        Bytes::from_static(b"vmax")
    );

    // Verify scan_prefix isolation
    let res1 = store1.scan_prefix(b"user").await.unwrap();
    assert_eq!(res1.len(), 1);
    assert_eq!(res1[0].1, b"v1");

    let res10 = store10.scan_prefix(b"user").await.unwrap();
    assert_eq!(res10.len(), 1);
    assert_eq!(res10[0].1, b"v10");

    // Test upper_bound_for_prefix edge case (0xFF prefix)
    let res_ff = store1.scan_prefix(&[0xFF, 0xFF]).await.unwrap();
    assert_eq!(res_ff.len(), 1);
    assert_eq!(res_ff[0].1, b"v1_ff");

    // delete_prefix isolation
    let tx_del = TxId::new(1002);
    store1.delete_prefix(tx_del, b"user").await.unwrap();
    store1.commit(tx_del).await.unwrap();

    assert!(store1.get(common_key).await.unwrap().is_none());
    assert_eq!(
        store10.get(common_key).await.unwrap().unwrap(),
        Bytes::from_static(b"v10")
    );
}

// ============================================================================
// H4: KV Segment Deletion Proof Binding State
// ============================================================================

#[test]
fn test_h4_kv_segment_deletion_proof_state_binding() {
    let registry = Arc::new(KeyRegistry::new());
    let master_key = Arc::new(
        KeyManager::try_new("master-passphrase-shredding", b"master-salt-123")
            .expect("KeyManager"),
    );

    // 1. TombstoneOnly manager
    let mgr_tombstone = KvSegmentManager::new(
        KvSegmentConfig {
            delete_mode: KvDeleteMode::TombstoneOnly,
        },
        registry.clone(),
        Some(master_key.clone()),
    );

    let proof_tombstone = mgr_tombstone.generate_deletion_proof(100);
    assert!(
        proof_tombstone.is_err(),
        "TombstoneOnly mode MUST reject deletion proof generation"
    );

    // 2. CryptoShred manager
    let mgr_shred = KvSegmentManager::new(
        KvSegmentConfig {
            delete_mode: KvDeleteMode::CryptoShred,
        },
        registry.clone(),
        Some(master_key.clone()),
    );

    let group_id = 42u64;
    let payload = mgr_shred.write_segment(group_id, b"secret_data").unwrap();

    // Proof before shredding -> key is active, so deletion proof is false (not deleted)
    let proof_before = mgr_shred.generate_deletion_proof(group_id).unwrap();
    assert!(
        !proof_before,
        "Deletion proof must be false when segment key is still active"
    );

    // Read segment works
    let read_back = mgr_shred.read_segment(&payload).unwrap();
    assert_eq!(read_back, b"secret_data");

    // Delete segment (crypto shred)
    let revoked = mgr_shred.delete_segment(group_id);
    assert!(revoked, "Subkey revocation must return true");

    // Proof after shredding -> key is inactive, so deletion proof is true (deleted)
    let proof_after = mgr_shred.generate_deletion_proof(group_id).unwrap();
    assert!(
        proof_after,
        "Deletion proof must be true after crypto shredding"
    );

    // Read segment fails after shredding
    assert!(mgr_shred.read_segment(&payload).is_err());
}

// ============================================================================
// H5: Single-Process LsmStorage Reopen / Lock Behavior
// ============================================================================

#[tokio::test]
async fn test_h5_lsm_storage_reopen_and_stale_lock() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    // 1. Open first LsmStorage
    let config1 = LsmConfig {
        path: db_path.clone(),
        durability_mode: DurabilityMode::Full,
        ..Default::default()
    };
    let lsm1 = LsmStorage::new(config1).await.unwrap();

    // 2. Second LsmStorage open on same path in same process MUST fail
    let config2 = LsmConfig {
        path: db_path.clone(),
        durability_mode: DurabilityMode::Full,
        ..Default::default()
    };
    let lsm2_res = LsmStorage::new(config2).await;
    assert!(
        lsm2_res.is_err(),
        "Second LsmStorage::open on active directory must fail with lock error"
    );

    // 3. Drop first LsmStorage, reopen succeeds
    drop(lsm1);
    let config3 = LsmConfig {
        path: db_path.clone(),
        durability_mode: DurabilityMode::Full,
        ..Default::default()
    };
    let lsm3 = LsmStorage::new(config3).await;
    assert!(
        lsm3.is_ok(),
        "Re-opening LsmStorage after drop must succeed"
    );
    drop(lsm3);

    // 4. Stale LOCK file left on disk (simulating SIGKILL)
    let stale_lock_path = db_path.join("LOCK");
    std::fs::write(&stale_lock_path, b"stale_pid_1234").unwrap();

    let config_stale = LsmConfig {
        path: db_path.clone(),
        durability_mode: DurabilityMode::Full,
        ..Default::default()
    };
    let lsm_stale = LsmStorage::new(config_stale).await;
    assert!(
        lsm_stale.is_ok(),
        "Opening LsmStorage over an un-held stale LOCK file must succeed"
    );
}

// ============================================================================
// H6: Observer Circuit Breaker & Fail-Open Resilience
// ============================================================================

struct SlowObserver {
    sleep_dur: Duration,
}

impl WalObserver for SlowObserver {
    fn on_commit(&self, _batch: &CommittedBatch<'_>, _seq_no: u64, _tx_id: TxId) {
        std::thread::sleep(self.sleep_dur);
    }
}

#[tokio::test]
async fn test_h6_observer_circuit_breaker_fail_open() {
    let registry = ObserverRegistry::new();
    registry.set_max_observer_latency(Duration::from_millis(5));

    let slow_obs: Arc<dyn WalObserver> = Arc::new(SlowObserver {
        sleep_dur: Duration::from_millis(50),
    });
    registry.register_observer(slow_obs.clone());

    let start = std::time::Instant::now();
    for i in 1..=100u64 {
        let tx = TxId::new(i);
        registry.notify(&[], i, tx, contextra_store::lsm::observer::WriteOrigin::UserWrite);
    }
    let elapsed = start.elapsed();

    // Verify circuit breaker tripped and commits did not block forever
    assert!(
        elapsed < Duration::from_secs(3),
        "100 commits took {:?}; observer circuit breaker failed to open and blocked commit path",
        elapsed
    );

    assert!(
        registry.is_any_circuit_breaker_open(),
        "Circuit breaker must be open for slow observer"
    );
    assert!(
        registry.dropped_count_for(&slow_obs) > 0,
        "Dropped count for slow observer must be > 0"
    );

    // Clear circuit breaker
    registry.clear_circuit_breaker(&slow_obs);
    assert!(
        !registry.is_any_circuit_breaker_open(),
        "Circuit breaker must be cleared"
    );
}

// ============================================================================
// H7: Differential Testing against BTreeMap Reference Oracle
// ============================================================================

#[tokio::test]
async fn test_h7_differential_btree_map_oracle() {
    let seed = 0x43_4f_4e_54_58_54_52u64; // Deterministic seed
    let mut rng_state = seed;
    let mut next_u32 = || {
        rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_state >> 32) as u32
    };

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let mut model: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
    let mut tx_counter = 1u64;

    let open_db = |path: &std::path::Path| {
        let p = path.to_path_buf();
        Box::pin(async move {
            let config = LsmConfig {
                path: p,
                memtable_size_limit: 128 * 1024,
                ..Default::default()
            };
            LsmStorage::new(config).await.unwrap()
        })
    };

    let mut lsm = open_db(&db_path).await;

    for seq in 0..500 {
        let tx = TxId::new(tx_counter);
        tx_counter += 1;

        let op_type = next_u32() % 6;
        let key = format!("key_{}", next_u32() % 50).into_bytes();
        let val = format!("val_{}_{seq}", next_u32() % 100).into_bytes();

        match op_type {
            0 => {
                // Put
                lsm.put(tx, &key, &val).await.unwrap();
                lsm.commit(tx).await.unwrap();
                model.insert(key, val);
            }
            1 => {
                // Delete
                lsm.delete(tx, &key).await.unwrap();
                lsm.commit(tx).await.unwrap();
                model.remove(&key);
            }
            2 => {
                // Put if absent
                let inserted = lsm.put_if_absent(tx, &key, &val).await.unwrap();
                if inserted {
                    lsm.commit(tx).await.unwrap();
                    if !model.contains_key(&key) {
                        model.insert(key, val);
                    }
                } else {
                    lsm.rollback(tx).await.unwrap();
                }
            }
            3 => {
                // Rollback simulated
                lsm.put(tx, &key, &val).await.unwrap();
                lsm.rollback(tx).await.unwrap();
                // Model unchanged
            }
            4 => {
                // Flush
                lsm.flush().await.unwrap();
            }
            5 => {
                // Reopen / Crash simulation after confirmed commit
                drop(lsm);
                lsm = open_db(&db_path).await;
            }
            _ => unreachable!(),
        }

        // Verify model invariant after each batch of operations
        if seq % 25 == 0 {
            for (k, v) in &model {
                let got = lsm.get(k).await.unwrap();
                assert_eq!(
                    got.as_deref(),
                    Some(v.as_slice()),
                    "Mismatch at seq {seq} for key {:?}",
                    String::from_utf8_lossy(k)
                );
            }
        }
    }
}

// ============================================================================
// H8: SystemPressure Monitor Monotonicity & Escalation
// ============================================================================

fn level_rank(p: PressureLevel) -> u8 {
    match p {
        PressureLevel::Normal => 0,
        PressureLevel::Elevated => 1,
        PressureLevel::Critical => 2,
    }
}

#[test]
fn test_h8_system_pressure_monitor_escalation() {
    let monitor = SystemPressureMonitor::new(Duration::from_millis(100));

    // Normal
    let p_normal = monitor.compute_pressure(10, 0.1, 0.1, 5, 10);
    assert_eq!(p_normal.pressure_level, PressureLevel::Normal);

    // Elevated WAL depth
    let p_elevated_wal = monitor.compute_pressure(150, 0.1, 0.1, 5, 10);
    assert_eq!(p_elevated_wal.pressure_level, PressureLevel::Elevated);

    // Critical WAL depth
    let p_critical_wal = monitor.compute_pressure(600, 0.1, 0.1, 5, 10);
    assert_eq!(p_critical_wal.pressure_level, PressureLevel::Critical);

    // Monotonicity check
    assert!(
        level_rank(p_normal.pressure_level) < level_rank(p_elevated_wal.pressure_level)
    );
    assert!(
        level_rank(p_elevated_wal.pressure_level) < level_rank(p_critical_wal.pressure_level)
    );
}
