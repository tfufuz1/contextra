use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use redb::{ReadableTable, TableDefinition};
use std::ops::Bound;
use std::sync::Arc;
use tempfile::TempDir;

const TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("diff_table");

#[derive(Debug, Clone)]
enum Op {
    Put(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
    Get(Vec<u8>),
    Scan { start: Vec<u8>, end: Vec<u8> },
    Flush,
    Compact,
    CheckpointPinUnpin,
    Restart,
}

fn op_strategy() -> impl Strategy<Value = Op> {
    let key_strat = prop::collection::vec(any::<u8>(), 1..=32);
    let val_strat = prop::collection::vec(any::<u8>(), 1..=256);

    prop_oneof![
        (key_strat.clone(), val_strat).prop_map(|(k, v)| Op::Put(k, v)),
        key_strat.clone().prop_map(Op::Delete),
        key_strat.clone().prop_map(Op::Get),
        (key_strat.clone(), key_strat).prop_map(|(start, end)| Op::Scan { start, end }),
        Just(Op::Flush),
        Just(Op::Compact),
        Just(Op::CheckpointPinUnpin),
        Just(Op::Restart),
    ]
}

/// Strategy for large-payload differential testing under memory pressure.
/// Values range from 64 KiB up to 4 MiB; keys remain small (1..=64 bytes).
fn op_strategy_large() -> impl Strategy<Value = Op> {
    let key_strat = prop::collection::vec(any::<u8>(), 1..=64);
    let val_strat = prop::collection::vec(any::<u8>(), 64 * 1024..=4 * 1024 * 1024);

    prop_oneof![
        (key_strat.clone(), val_strat).prop_map(|(k, v)| Op::Put(k, v)),
        key_strat.clone().prop_map(Op::Delete),
        key_strat.clone().prop_map(Op::Get),
        (key_strat.clone(), key_strat).prop_map(|(start, end)| Op::Scan { start, end }),
        Just(Op::Flush),
        Just(Op::Compact),
        Just(Op::CheckpointPinUnpin),
        Just(Op::Restart),
    ]
}

/// Helper function to retrieve process Resident Set Size (RSS) in bytes.
/// Reads `/proc/self/statm` on Linux; returns `None` on other platforms.
fn get_process_rss_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
            let mut parts = content.split_whitespace();
            let _total_size = parts.next()?;
            if let Some(resident_pages_str) = parts.next() {
                if let Ok(pages) = resident_pages_str.parse::<u64>() {
                    let page_size = 4096u64; // Linux default page size
                    return Some(pages.saturating_mul(page_size));
                }
            }
        }
    }
    None
}

async fn assert_full_state_match(
    storage: &LsmStorage,
    redb_db: &redb::Database,
    context_msg: &str,
) -> Result<(), TestCaseError> {
    let lsm_all = storage
        .scan(Bound::Unbounded, Bound::Unbounded, None)
        .await
        .map_err(|e| TestCaseError::fail(e.to_string()))?;

    let read_txn = redb_db
        .begin_read()
        .map_err(|e| TestCaseError::fail(e.to_string()))?;
    let table = read_txn
        .open_table(TABLE)
        .map_err(|e| TestCaseError::fail(e.to_string()))?;
    let iter = table
        .iter()
        .map_err(|e| TestCaseError::fail(e.to_string()))?;

    let mut redb_all = Vec::new();
    for item in iter {
        let (k, v) = item.map_err(|e| TestCaseError::fail(e.to_string()))?;
        redb_all.push((k.value().to_vec(), v.value().to_vec()));
    }

    prop_assert_eq!(lsm_all, redb_all, "Full state mismatch at {}", context_msg);
    Ok(())
}

async fn run_differential(ops: Vec<Op>) -> Result<(), TestCaseError> {
    run_differential_with_config(ops, 1024, false).await
}

async fn run_differential_with_config(
    ops: Vec<Op>,
    memtable_size_limit: usize,
    check_rss: bool,
) -> Result<(), TestCaseError> {
    let lsm_dir = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
    let redb_dir = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;

    let lsm_config = LsmConfig {
        path: lsm_dir.path().to_path_buf(),
        memtable_size_limit,
        ..Default::default()
    };

    let mut storage = LsmStorage::new(lsm_config.clone())
        .await
        .map_err(|e| TestCaseError::fail(e.to_string()))?;

    let redb_path = redb_dir.path().join("redb.db");
    let mut redb_db =
        redb::Database::create(&redb_path).map_err(|e| TestCaseError::fail(e.to_string()))?;

    // Ensure table exists for reads
    {
        let write_txn = redb_db
            .begin_write()
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        {
            let _ = write_txn
                .open_table(TABLE)
                .map_err(|e| TestCaseError::fail(e.to_string()))?;
        }
        write_txn
            .commit()
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
    }

    let mut tx_counter = 1u64;
    let mut cumulative_payload_bytes: u64 = 0;
    let mut last_compact_rss: Option<u64> = get_process_rss_bytes();

    for (idx, op) in ops.into_iter().enumerate() {
        match op {
            Op::Put(k, v) => {
                cumulative_payload_bytes += (k.len() + v.len()) as u64;
                let tx = TxId::new(tx_counter);
                tx_counter += 1;

                storage
                    .put(tx, &k, &v)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                storage
                    .commit(tx)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                let write_txn = redb_db
                    .begin_write()
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                {
                    let mut table = write_txn
                        .open_table(TABLE)
                        .map_err(|e| TestCaseError::fail(e.to_string()))?;
                    table
                        .insert(k.as_slice(), v.as_slice())
                        .map_err(|e| TestCaseError::fail(e.to_string()))?;
                }
                write_txn
                    .commit()
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
            }
            Op::Delete(k) => {
                let tx = TxId::new(tx_counter);
                tx_counter += 1;

                storage
                    .delete(tx, &k)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                storage
                    .commit(tx)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                let write_txn = redb_db
                    .begin_write()
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                {
                    let mut table = write_txn
                        .open_table(TABLE)
                        .map_err(|e| TestCaseError::fail(e.to_string()))?;
                    table
                        .remove(k.as_slice())
                        .map_err(|e| TestCaseError::fail(e.to_string()))?;
                }
                write_txn
                    .commit()
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
            }
            Op::Get(k) => {
                let lsm_val = storage
                    .get(&k)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?
                    .map(|b| b.to_vec());

                let read_txn = redb_db
                    .begin_read()
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                let table = read_txn
                    .open_table(TABLE)
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                let redb_val = table
                    .get(k.as_slice())
                    .map_err(|e| TestCaseError::fail(e.to_string()))?
                    .map(|v| v.value().to_vec());

                prop_assert_eq!(
                    lsm_val,
                    redb_val,
                    "Get divergence at op index {}: key {:?}",
                    idx,
                    k
                );
            }
            Op::Scan { start, end } => {
                let (start_b, end_b) = if start <= end {
                    (&start, &end)
                } else {
                    (&end, &start)
                };

                let lsm_items = storage
                    .scan(
                        Bound::Included(start_b.as_slice()),
                        Bound::Excluded(end_b.as_slice()),
                        None,
                    )
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                let read_txn = redb_db
                    .begin_read()
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                let table = read_txn
                    .open_table(TABLE)
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                let range = table
                    .range::<&[u8]>((
                        Bound::Included(start_b.as_slice()),
                        Bound::Excluded(end_b.as_slice()),
                    ))
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                let mut redb_items = Vec::new();
                for item in range {
                    let (k, v) = item.map_err(|e| TestCaseError::fail(e.to_string()))?;
                    redb_items.push((k.value().to_vec(), v.value().to_vec()));
                }

                prop_assert_eq!(
                    lsm_items,
                    redb_items,
                    "Scan divergence at op index {}: start {:?}, end {:?}",
                    idx,
                    start_b,
                    end_b
                );
            }
            Op::Flush => {
                storage
                    .force_flush()
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                assert_full_state_match(&storage, &redb_db, &format!("Flush at op index {}", idx))
                    .await?;
            }
            Op::Compact => {
                let _ = storage
                    .maybe_compact()
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                if check_rss {
                    if let Some(curr_rss) = get_process_rss_bytes() {
                        if let Some(prev_rss) = last_compact_rss {
                            if curr_rss > prev_rss {
                                let growth = curr_rss - prev_rss;
                                let max_allowed_growth = cumulative_payload_bytes
                                    .saturating_mul(3)
                                    .saturating_add(64 * 1024 * 1024); // 64 MB baseline headroom
                                if growth > max_allowed_growth {
                                    return Err(TestCaseError::fail(format!(
                                        "Memory leak/spike detected at op index {}: RSS grew by {} bytes (prev: {}, curr: {}), exceeding threshold of {} bytes (written payload: {} bytes)",
                                        idx, growth, prev_rss, curr_rss, max_allowed_growth, cumulative_payload_bytes
                                    )));
                                }
                            }
                        }
                        last_compact_rss = Some(curr_rss);
                    }
                }

                assert_full_state_match(
                    &storage,
                    &redb_db,
                    &format!("Compact at op index {}", idx),
                )
                .await?;
            }
            Op::CheckpointPinUnpin => {
                let seq = storage
                    .last_seq_no()
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                storage
                    .pin_checkpoint(seq)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                assert_full_state_match(
                    &storage,
                    &redb_db,
                    &format!("Checkpoint pinned at seq {} at op index {}", seq, idx),
                )
                .await?;

                storage
                    .unpin_checkpoint(seq)
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;
                assert_full_state_match(
                    &storage,
                    &redb_db,
                    &format!("Checkpoint unpinned at seq {} at op index {}", seq, idx),
                )
                .await?;
            }
            Op::Restart => {
                drop(storage);
                storage = LsmStorage::new(lsm_config.clone())
                    .await
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                drop(redb_db);
                redb_db = redb::Database::open(&redb_path)
                    .map_err(|e| TestCaseError::fail(e.to_string()))?;

                assert_full_state_match(
                    &storage,
                    &redb_db,
                    &format!("Restart at op index {}", idx),
                )
                .await?;
            }
        }
    }

    assert_full_state_match(&storage, &redb_db, "End of sequence").await?;
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]
    #[test]
    fn differential_lsm_vs_redb(
        ops in prop::collection::vec(op_strategy(), 1..200)
    ) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async {
            run_differential(ops).await
        })?;
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2))]
    #[test]
    fn differential_redb_under_memory_pressure_large_values(
        ops in prop::collection::vec(op_strategy_large(), 200..250)
    ) {
        let rt = tokio::runtime::Runtime::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
        rt.block_on(async {
            // Force frequent memtable flushes & compaction with 8 KiB memtable limit and large values
            run_differential_with_config(ops, 8192, true).await
        })?;
    }
}

/// Tests serializability and state equivalence under 4 concurrent writer tasks
/// operating on disjoint key spaces against a single shared `LsmStorage` instance.
#[tokio::test]
async fn differential_redb_concurrent_compaction_under_load() {
    let lsm_dir = TempDir::new().unwrap();
    let redb_dir = TempDir::new().unwrap();

    let lsm_config = LsmConfig {
        path: lsm_dir.path().to_path_buf(),
        memtable_size_limit: 8192, // Small limit forces frequent flush/compaction during concurrent load
        ..Default::default()
    };

    let storage = Arc::new(LsmStorage::new(lsm_config).await.unwrap());
    let redb_path = redb_dir.path().join("redb.db");
    let redb_db = redb::Database::create(&redb_path).unwrap();

    // Ensure table exists for redb
    {
        let write_txn = redb_db.begin_write().unwrap();
        {
            let _ = write_txn.open_table(TABLE).unwrap();
        }
        write_txn.commit().unwrap();
    }

    // Deterministically generate operations for 4 concurrent worker tasks.
    // Task `t` uses key prefix `[t as u8, ...]` to guarantee disjoint key ranges.
    const NUM_TASKS: usize = 4;
    const OPS_PER_TASK: usize = 60;

    let mut all_task_ops: Vec<Vec<(u64, Op)>> = Vec::new();
    let mut global_seq_ops: Vec<Op> = Vec::new();
    let mut tx_counter = 1000u64;

    for task_id in 0..NUM_TASKS {
        let mut task_ops = Vec::new();
        for i in 0..OPS_PER_TASK {
            let key = vec![task_id as u8, (i % 20) as u8, ((i * 7) % 256) as u8];
            let val = vec![(task_id * 10) as u8; 1024 + i * 64]; // 1 KB+ values

            let op = if i % 7 == 0 {
                Op::Delete(key)
            } else if i % 13 == 0 {
                Op::Flush
            } else if i % 17 == 0 {
                Op::Compact
            } else {
                Op::Put(key, val)
            };

            task_ops.push((tx_counter, op.clone()));
            global_seq_ops.push(op);
            tx_counter += 1;
        }
        all_task_ops.push(task_ops);
    }

    // Apply all generated operations sequentially to redb (reference model)
    let mut redb_tx_counter = 1000u64;
    for op in &global_seq_ops {
        match op {
            Op::Put(k, v) => {
                let write_txn = redb_db.begin_write().unwrap();
                {
                    let mut table = write_txn.open_table(TABLE).unwrap();
                    table.insert(k.as_slice(), v.as_slice()).unwrap();
                }
                write_txn.commit().unwrap();
            }
            Op::Delete(k) => {
                let write_txn = redb_db.begin_write().unwrap();
                {
                    let mut table = write_txn.open_table(TABLE).unwrap();
                    table.remove(k.as_slice()).unwrap();
                }
                write_txn.commit().unwrap();
            }
            _ => {}
        }
        redb_tx_counter += 1;
    }
    let _ = redb_tx_counter;

    // Spawn 4 concurrent tasks executing against the shared LsmStorage
    let mut handles = Vec::new();
    for task_ops in all_task_ops {
        let st = Arc::clone(&storage);
        handles.push(tokio::spawn(async move {
            for (tx_id_raw, op) in task_ops {
                let tx = TxId::new(tx_id_raw);
                match op {
                    Op::Put(k, v) => {
                        st.put(tx, &k, &v).await.unwrap();
                        st.commit(tx).await.unwrap();
                    }
                    Op::Delete(k) => {
                        st.delete(tx, &k).await.unwrap();
                        st.commit(tx).await.unwrap();
                    }
                    Op::Get(k) => {
                        let _ = st.get(&k).await;
                    }
                    Op::Flush => {
                        let _ = st.force_flush().await;
                    }
                    Op::Compact => {
                        let _ = st.maybe_compact().await;
                    }
                    _ => {}
                }
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // Final flush & compact on LSM to ensure all data is persisted and compacted
    storage.force_flush().await.unwrap();
    let _ = storage.maybe_compact().await;

    // Verify final state matches redb exactly
    assert_full_state_match(&storage, &redb_db, "Concurrent Compaction Under Load")
        .await
        .expect("LSM storage state must match redb state after concurrent execution");
}
