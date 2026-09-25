#![no_main]

use arbitrary::Arbitrary;
use contextra_core::{StorageEngine, TxId};
use contextra_store::{LsmConfig, LsmStorage};
use libfuzzer_sys::fuzz_target;
use std::collections::HashMap;
use tempfile::tempdir;

#[derive(Arbitrary, Debug, Clone)]
enum CompactionOp {
    Put {
        key_suffix: u8,
        val_byte: u8,
    },
    Delete {
        key_suffix: u8,
    },
    Flush,
    Compact,
}

#[derive(Arbitrary, Debug)]
struct CompactionInterleaveInput {
    ops: Vec<CompactionOp>,
}

fuzz_target!(|input: CompactionInterleaveInput| {
    if input.ops.is_empty() {
        return;
    }

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => return,
    };

    rt.block_on(async {
        let dir = match tempdir() {
            Ok(d) => d,
            Err(_) => return,
        };

        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            memtable_size_limit: 1024,
            ..Default::default()
        };

        let storage = match LsmStorage::new(config).await {
            Ok(s) => s,
            Err(_) => return,
        };

        let mut oracle: HashMap<Vec<u8>, Option<Vec<u8>>> = HashMap::new();
        let mut tx_counter = 1u64;

        for op in input.ops {
            match op {
                CompactionOp::Put {
                    key_suffix,
                    val_byte,
                } => {
                    let key = format!("k_{key_suffix}").into_bytes();
                    let val = vec![val_byte; (val_byte as usize % 64) + 1];
                    let tx_id = TxId::new(tx_counter);
                    tx_counter += 1;

                    if storage.put(tx_id, &key, &val).await.is_ok()
                        && storage.commit(tx_id).await.is_ok()
                    {
                        oracle.insert(key, Some(val));
                    }
                }
                CompactionOp::Delete { key_suffix } => {
                    let key = format!("k_{key_suffix}").into_bytes();
                    let tx_id = TxId::new(tx_counter);
                    tx_counter += 1;

                    if storage.delete(tx_id, &key).await.is_ok()
                        && storage.commit(tx_id).await.is_ok()
                    {
                        oracle.insert(key, None);
                    }
                }
                CompactionOp::Flush => {
                    if let Err(_e) = storage.flush().await {}
                }
                CompactionOp::Compact => {
                    let _ = storage.maybe_compact().await;
                }
            }

            for (key, expected_val) in &oracle {
                let actual = storage.get(key).await;
                match (actual, expected_val) {
                    (Ok(Some(val)), Some(exp)) => {
                        assert_eq!(
                            &val[..],
                            &exp[..],
                            "Value mismatch after compaction interleave for key {:?}",
                            key
                        );
                    }
                    (Ok(None), None) => {}
                    (Ok(actual_res), expected_res) => {
                        panic!(
                            "Inconsistency for key {:?}: got {:?}, expected {:?}",
                            key, actual_res, expected_res
                        );
                    }
                    (Err(e), _) => {
                        panic!("Read error during oracle validation: {:?}", e);
                    }
                }
            }
        }
    });
});
