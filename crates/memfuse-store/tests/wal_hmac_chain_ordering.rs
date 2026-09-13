// FILE-CONTEXT
// ZWECK: Regressionstest — WAL HMAC-Kette enthält keine Gabelungen nach Group-Commit.
// INVARIANT: Jeder Entry hat eindeutigen prev_hmac; Sequenznummern sind strikt monoton.
// ERSTELLT: 2026-09-13 (Audit-Befund P-1, Architektur-Review 2026-09-13)
// ABHÄNGIG VON: prepare_batch → append_batch Aufruf-Reihenfolge in LsmStorage::commit()

use memfuse_core::{StorageEngine, TxId};
use memfuse_store::lsm::{LsmConfig, LsmStorage};
use memfuse_store::wal::Wal;
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_group_commit_hmac_chain_no_bifurcation() {
    let dir = tempdir().expect("tempdir creation failed");
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        group_commit_window_micros: 500,
        ..Default::default()
    };

    let storage = Arc::new(
        LsmStorage::new(config.clone())
            .await
            .expect("LsmStorage creation failed"),
    );

    let num_tasks = 8;
    let mut set = tokio::task::JoinSet::new();

    for i in 0..num_tasks {
        let storage = Arc::clone(&storage);
        set.spawn(async move {
            let tx_id = TxId::new((i + 1) as u64);
            let key = format!("key-{i}").into_bytes();
            let val = b"val";

            storage.put(tx_id, &key, val).await?;
            storage.commit(tx_id).await
        });
    }

    let mut task_count = 0;
    while let Some(res) = set.join_next().await {
        task_count += 1;
        let commit_res = res.expect("Task panicked during execution");
        assert!(
            commit_res.is_ok(),
            "Group commit failed: {:?}",
            commit_res.err()
        );
    }
    assert_eq!(task_count, num_tasks, "Not all tasks completed");

    let wal_path = config.path.join("wal.log");

    let wal = Wal::open(&wal_path)
        .await
        .expect("Failed to open WAL for replay verification");
    let replayed = wal.replay().await.expect("WAL replay failed");

    assert_eq!(
        replayed.len(),
        num_tasks,
        "Replayed WAL entry count mismatch"
    );

    // Invariant 1: No HMAC chain bifurcation (each entry must have a unique prev_hmac)
    let mut prev_hmacs = HashSet::new();
    for (_, entry, _) in &replayed {
        assert!(
            prev_hmacs.insert(entry.prev_hmac),
            "HMAC chain bifurcation detected: two entries share prev_hmac {:?}",
            entry.prev_hmac
        );
    }

    // Invariant 2: Strictly monotonically increasing sequence numbers
    let seq_nos: Vec<u64> = replayed.iter().map(|(seq, _, _)| *seq).collect();
    let mut sorted_seqs = seq_nos.clone();
    sorted_seqs.sort_unstable();
    sorted_seqs.dedup();
    assert_eq!(
        seq_nos.len(),
        sorted_seqs.len(),
        "Duplicate or non-monotone seq_nos detected"
    );

    // Invariant 3: Unbroken HMAC chain linkage (entry[i].prev_hmac == entry[i-1].checksum)
    let mut replayed_sorted = replayed.clone();
    replayed_sorted.sort_by_key(|(seq, _, _)| *seq);

    for i in 1..replayed_sorted.len() {
        let (_, prev_entry, _) = &replayed_sorted[i - 1];
        let (_, curr_entry, _) = &replayed_sorted[i];
        assert_eq!(
            curr_entry.prev_hmac, prev_entry.checksum,
            "HMAC chain broken between seq {} and seq {}: prev_hmac {:?} != expected checksum {:?}",
            prev_entry.seq_no, curr_entry.seq_no, curr_entry.prev_hmac, prev_entry.checksum
        );
    }
}
