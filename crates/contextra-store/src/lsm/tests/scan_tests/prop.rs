//! # Property Tests fuer Scan (Ring 1)
//!
//! ZWECK: Proptest-basierte Konsistenzpruefung von `scan_prefix_at` gegen Referenzmodell.

use super::*;
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Put(u8, Vec<u8>),
    Delete(u8),
}

#[test]
fn test_prop_lsm_scan_prefix_at_consistency() {
    let op_strategy = proptest::collection::vec(
        prop_oneof![
            (1u8..10, proptest::collection::vec(any::<u8>(), 1..10))
                .prop_map(|(k, v)| Op::Put(k, v)),
            (1u8..10).prop_map(Op::Delete),
        ],
        10..60,
    );

    proptest!(ProptestConfig::with_cases(20), |(ops in op_strategy)| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let tmp = tempfile::TempDir::new().unwrap();
            let config = LsmConfig {
                path: tmp.path().to_path_buf(),
                memtable_size_limit: 1024 * 1024,
                max_ram_mb: 64,
                tx_timeout: Duration::from_secs(60),
                compaction: CompactionConfig::default(),
                encryption_passphrase: None,
                ..Default::default()
            };
            let storage = LsmStorage::new(config).await.unwrap();

            let mut current_tx = 1u64;
            let mut tx_checkpoints = Vec::new();

            for op in ops {
                let tx = TxId::new(current_tx);
                match op {
                    Op::Put(key_id, val) => {
                        let key = format!("pfx:{}", key_id);
                        let _ = storage.put(tx, key.as_bytes(), &val).await;
                    }
                    Op::Delete(key_id) => {
                        let key = format!("pfx:{}", key_id);
                        let _ = storage.delete(tx, key.as_bytes()).await;
                    }
                }
                if storage.commit(tx).await.is_ok() {
                    let seq = storage.last_seq_no().await.unwrap();
                    tx_checkpoints.push((current_tx, seq));
                    current_tx += 1;
                }
            }

            for &(_tx_num, target_seq) in &tx_checkpoints {
                let scanned = storage.scan_prefix_at(b"pfx:", target_seq).await.unwrap();
                let actual_map: std::collections::BTreeMap<_, _> = scanned.into_iter().collect();

                let mut ref_map = std::collections::BTreeMap::new();
                let state = storage.state.read().await;

                let mut all_entries = Vec::new();
                for (k, v, seq, _tx) in state.memtable.iter() {
                    all_entries.push((k.to_vec(), v.to_vec(), seq));
                }
                for mt in &state.immutable_memtables {
                    for (k, v, seq, _tx) in mt.iter() {
                        all_entries.push((k.to_vec(), v.to_vec(), seq));
                    }
                }
                drop(state);

                let sstables = storage.sstables.read().await;
                for sst in sstables.iter() {
                    let sst_entries = sst.scan_prefix(b"pfx:").await.unwrap();
                    for (k, v, seq, _tx) in sst_entries {
                        all_entries.push((k.to_vec(), v.to_vec(), seq));
                    }
                }
                drop(sstables);

                all_entries.sort_by_key(|e| e.2 & !TOMBSTONE_BIT);

                for (k, v, seq) in all_entries {
                    let raw_seq = seq & !TOMBSTONE_BIT;
                    if raw_seq <= target_seq && k.starts_with(b"pfx:") {
                        if (seq & TOMBSTONE_BIT) != 0 {
                            ref_map.remove(&k);
                        } else {
                            ref_map.insert(k, v);
                        }
                    }
                }

                prop_assert_eq!(actual_map, ref_map, "scan_prefix_at at seq {} must match reference model", target_seq);
            }
            Ok(())
        }).unwrap();
    });
}
