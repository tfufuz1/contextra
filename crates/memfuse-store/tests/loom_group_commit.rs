//! Loom-basierter Determinismus-Beweis für Lock-Handoff & Group-Commit-Reihenfolge.
//! Nacharbeit zu Commit 694fa8c2: Richtige Lock-Reihenfolge (Lock-Handoff) und
//! Verifikation der physischen WAL-Schreibreihenfolge.
//!
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p memfuse-store --test loom_group_commit --release

#![allow(unexpected_cfgs)]

use memfuse_core::{MemFuseError, TxId};
use memfuse_store::wal::{PreparedBatch, Wal, WalOp};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct GroupCommitRequest {
    _tx_id: u64,
    batch: PreparedBatch,
}

struct PendingCommitQueue {
    requests: Vec<GroupCommitRequest>,
    first_prev_hmac: [u8; 32],
}

struct GroupCommitEngine {
    commit_mutex: tokio::sync::Mutex<()>,
    pending_commit_queue: tokio::sync::Mutex<Option<PendingCommitQueue>>,
    next_seq_no: AtomicU64,
    prep_order: std::sync::Mutex<Vec<u64>>,
    wal: Wal,
}

impl GroupCommitEngine {
    fn new(wal: Wal) -> Self {
        Self {
            commit_mutex: tokio::sync::Mutex::new(()),
            pending_commit_queue: tokio::sync::Mutex::new(None),
            next_seq_no: AtomicU64::new(1),
            prep_order: std::sync::Mutex::new(Vec::new()),
            wal,
        }
    }

    /// Nachbildung der echten Gruppen-Commit-Logik aus `lsm/mod.rs`
    /// unter Verwendung der ECHTEN `commit_mutex` + `truncate_lock` Lock-Handoff-Reihenfolge.
    async fn commit(&self, tx_id: u64) -> Result<(), MemFuseError> {
        // PHASE 1: commit_mutex erwerben für Sequenz-Allokierung & Batch-Vorbereitung
        let commit_lock = self.commit_mutex.lock().await;

        let seq_no = self.next_seq_no.fetch_add(1, Ordering::SeqCst);
        {
            let mut order = self
                .prep_order
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            order.push(tx_id);
        }

        let op = WalOp::Put {
            tx_id: TxId::new(tx_id),
            key: format!("key-{tx_id}").into_bytes(),
            value: b"val".to_vec(),
        };
        let (batch, prev_hmac_snapshot) = self.wal.prepare_batch(vec![(op, seq_no)]).await?;

        // PHASE 2: Group Commit Einreihung / Leader-Auswahl
        let mut queue_guard = self.pending_commit_queue.lock().await;

        if let Some(ref mut queue) = *queue_guard {
            // Follower-Pfad: In bestehende Leader-Queue einreihen
            let req = GroupCommitRequest {
                _tx_id: tx_id,
                batch: batch.clone(),
            };
            queue.requests.push(req);
            drop(queue_guard);
            drop(commit_lock);
            Ok(())
        } else {
            // Leader-Pfad: Queue initialisieren und Leader-Rolle übernehmen
            *queue_guard = Some(PendingCommitQueue {
                requests: Vec::new(),
                first_prev_hmac: prev_hmac_snapshot,
            });
            drop(queue_guard);

            // Leader-Pfad: Pending Queue übernehmen
            let mut queue_guard = self.pending_commit_queue.lock().await;
            let pending_queue = queue_guard
                .take()
                .expect("pending commit queue must exist for leader");
            drop(queue_guard);

            let mut combined_batch = batch;
            for r in pending_queue.requests {
                combined_batch.extend(r.batch);
            }

            // LOCK-HANDOFF (P0-A / Commit 694fa8c2):
            // Acquire truncate_lock BEFORE dropping commit_mutex!
            let truncate_guard = self.wal.truncate_lock.lock().await;
            drop(commit_lock);

            let append_res = self
                .wal
                .append_batch_locked(combined_batch, &truncate_guard)
                .await;
            drop(truncate_guard);

            if let Err(e) = append_res {
                let _commit_lock = self.commit_mutex.lock().await;
                let _ = self
                    .wal
                    .restore_last_hmac(pending_queue.first_prev_hmac)
                    .await;
                return Err(e);
            }

            // Re-acquire commit_mutex für Post-Commit Status-Updates / Visibility Advancement
            let _commit_lock = self.commit_mutex.lock().await;
            Ok(())
        }
    }
}

#[cfg(loom)]
#[test]
fn test_loom_group_commit_last_hmac_race() {
    loom::model(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("tokio runtime build failed");

        rt.block_on(async {
            let wal = Wal::open_loom();
            let engine = Arc::new(GroupCommitEngine::new(wal));

            let engine_1 = Arc::clone(&engine);
            let task1 = tokio::spawn(async move {
                let _ = engine_1.commit(1).await;
            });

            let engine_2 = Arc::clone(&engine);
            let task2 = tokio::spawn(async move {
                let _ = engine_2.commit(2).await;
            });

            let _ = tokio::join!(task1, task2);

            let final_hmac = engine.wal.last_hmac_snapshot().await;
            assert_ne!(final_hmac, [0u8; 32], "last_hmac must be updated");

            let file_size = engine.wal.size();
            let physical_seqs = Arc::new(std::sync::Mutex::new(Vec::new()));
            let physical_seqs_clone = Arc::clone(&physical_seqs);

            engine
                .wal
                .scan_entries_with_callback(file_size, move |seq, entry, _pos| {
                    if let Ok(mut guard) = physical_seqs_clone.lock() {
                        guard.push((seq, entry.tx_id().inner()));
                    }
                    true
                })
                .await
                .expect("scan entries succeeds");

            let written = physical_seqs
                .lock()
                .map(|g| g.clone())
                .unwrap_or_default();
            assert_eq!(
                written.len(),
                2,
                "Exactly 2 physical entries must be written to WAL"
            );

            for window in written.windows(2) {
                assert!(
                    window[0].0 < window[1].0,
                    "Physical WAL seq_no must strictly increase: {} vs {}",
                    window[0].0,
                    window[1].0
                );
            }

            let _ = engine.wal.rotate_and_seal().await;
        });
    });
}

#[cfg(not(loom))]
#[tokio::test]
async fn test_loom_group_commit_last_hmac_race_non_loom() {
    let dir = tempfile::tempdir().expect("tempdir creation failed");
    let wal_path = dir.path().join("wal.log");
    let wal = Wal::open(&wal_path).await.expect("open wal failed");
    let engine = Arc::new(GroupCommitEngine::new(wal));

    let mut set = tokio::task::JoinSet::new();
    for i in 1..=3 {
        let engine_clone = Arc::clone(&engine);
        set.spawn(async move { engine_clone.commit(i as u64).await });
    }

    while let Some(res) = set.join_next().await {
        res.expect("task panicked").expect("commit failed");
    }

    // 1. Endzustand HMAC Prüfen
    let final_hmac = engine.wal.last_hmac_snapshot().await;
    assert_ne!(
        final_hmac, [0u8; 32],
        "last_hmac must be updated after group commit"
    );

    // 2. Physische Schreibreihenfolge per Wal::scan_entries_with_callback verifizieren
    let file_size = engine.wal.size();
    let physical_seqs = Arc::new(std::sync::Mutex::new(Vec::new()));
    let physical_seqs_clone = Arc::clone(&physical_seqs);

    engine
        .wal
        .scan_entries_with_callback(file_size, move |seq, entry, _pos| {
            if let Ok(mut guard) = physical_seqs_clone.lock() {
                guard.push((seq, entry.tx_id().inner()));
            }
            true
        })
        .await
        .expect("scan entries succeeds");

    let written = physical_seqs
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    assert_eq!(
        written.len(),
        3,
        "Exactly 3 physical entries must be written to WAL"
    );

    // Verifiziere monokausal aufsteigende seq_no-Folge
    for window in written.windows(2) {
        assert!(
            window[0].0 < window[1].0,
            "Physical WAL seq_no must strictly increase: {} vs {}",
            window[0].0,
            window[1].0
        );
    }

    let prep_order = engine
        .prep_order
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();
    assert_eq!(
        prep_order.len(),
        3,
        "All 3 transactions must record preparation order"
    );
}
