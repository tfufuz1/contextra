use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use redb::{ReadableTable, TableDefinition};
use std::ops::Bound;
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
    let lsm_dir = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;
    let redb_dir = TempDir::new().map_err(|e| TestCaseError::fail(e.to_string()))?;

    let lsm_config = LsmConfig {
        path: lsm_dir.path().to_path_buf(),
        memtable_size_limit: 1024,
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

    for (idx, op) in ops.into_iter().enumerate() {
        match op {
            Op::Put(k, v) => {
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
