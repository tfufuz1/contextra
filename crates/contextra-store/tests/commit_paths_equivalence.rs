use contextra_core::{StorageEngine, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::collections::BTreeMap;
use std::sync::Arc;
use tempfile::TempDir;

async fn run_workload(storage: Arc<LsmStorage>, num_tasks: usize, ops_per_task: usize) {
    let mut set = tokio::task::JoinSet::new();

    for task_idx in 0..num_tasks {
        let st = Arc::clone(&storage);
        set.spawn(async move {
            for op_idx in 0..ops_per_task {
                let tx_num = (task_idx * ops_per_task + op_idx + 1) as u64;
                let tx = TxId::new(tx_num);
                let key_num = (op_idx * 7 + task_idx) % 50;
                let key = format!("k_{:04}", key_num).into_bytes();
                let val = format!("v_{}_{}", task_idx, op_idx).into_bytes();

                st.put(tx, &key, &val).await.expect("put");

                if op_idx % 5 == 0 && key_num % 3 == 0 {
                    // Staged delete
                    let del_key = format!("k_{:04}", (key_num + 1) % 50).into_bytes();
                    st.delete(tx, &del_key).await.expect("delete");
                }

                st.commit(tx).await.expect("commit");
            }
        });
    }

    while let Some(res) = set.join_next().await {
        res.expect("task panicked");
    }
}

async fn collect_all_keys(storage: &LsmStorage) -> BTreeMap<Vec<u8>, Vec<u8>> {
    let mut map = BTreeMap::new();
    for i in 0..50 {
        let key = format!("k_{:04}", i).into_bytes();
        if let Some(val) = storage.get(&key).await.expect("get") {
            map.insert(key, val.to_vec());
        }
    }
    map
}

#[tokio::test]
async fn test_commit_paths_equivalence_single_vs_group() {
    let num_tasks = 20;
    let ops_per_task = 25;

    // Instance 1: Single commit (group_commit_window_micros = 0)
    let tmp1 = TempDir::new().expect("temp dir 1");
    let config1 = LsmConfig {
        path: tmp1.path().to_path_buf(),
        memtable_size_limit: 64 * 1024 * 1024,
        max_ram_mb: 512,
        group_commit_window_micros: 0,
        ..Default::default()
    };
    let storage1 = Arc::new(LsmStorage::new(config1).await.expect("create storage 1"));

    // Instance 2: Group commit (group_commit_window_micros = 200)
    let tmp2 = TempDir::new().expect("temp dir 2");
    let config2 = LsmConfig {
        path: tmp2.path().to_path_buf(),
        memtable_size_limit: 64 * 1024 * 1024,
        max_ram_mb: 512,
        group_commit_window_micros: 200,
        ..Default::default()
    };
    let storage2 = Arc::new(LsmStorage::new(config2).await.expect("create storage 2"));

    // Run identical workloads concurrently
    let handle1 = tokio::spawn(run_workload(Arc::clone(&storage1), num_tasks, ops_per_task));
    let handle2 = tokio::spawn(run_workload(Arc::clone(&storage2), num_tasks, ops_per_task));

    handle1.await.expect("workload 1 failed");
    handle2.await.expect("workload 2 failed");

    // Collect end states
    let state1 = collect_all_keys(&storage1).await;
    let state2 = collect_all_keys(&storage2).await;

    assert_eq!(
        state1, state2,
        "Single-commit and Group-commit paths produced different end states"
    );

    // Verify persistence across restart
    drop(storage1);
    drop(storage2);

    let config1_reopen = LsmConfig {
        path: tmp1.path().to_path_buf(),
        ..Default::default()
    };
    let storage1_reopen = LsmStorage::new(config1_reopen).await.expect("reopen 1");
    let state1_reopen = collect_all_keys(&storage1_reopen).await;

    let config2_reopen = LsmConfig {
        path: tmp2.path().to_path_buf(),
        ..Default::default()
    };
    let storage2_reopen = LsmStorage::new(config2_reopen).await.expect("reopen 2");
    let state2_reopen = collect_all_keys(&storage2_reopen).await;

    assert_eq!(state1_reopen, state2_reopen);
    assert_eq!(state1, state1_reopen);
}
