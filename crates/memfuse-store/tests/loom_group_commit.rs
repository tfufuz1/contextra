//! Loom-basierter Determinismus-Beweis für Lock-Handoff & Group-Commit-Reihenfolge.
//! Nacharbeit zu Commit 694fa8c2: Richtige Lock-Reihenfolge (Lock-Handoff) und
//! Verifikation der physischen WAL-Schreibreihenfolge.
//!
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p memfuse-store --test loom_group_commit --release
//!
//! MANUELLER MUTATIONSTEST (Dokumentation / Verifikation):
//! Falls im Leader-Pfad von `GroupCommitEngine::commit()` die Zeilen für Lock-Handoff
//!   `let truncate_guard = self.wal.truncate_lock.lock().await;`
//!   `drop(_commit_lock);`
//! vertauscht werden zu:
//!   `drop(_commit_lock);`
//!   `let truncate_guard = self.wal.truncate_lock.lock().await;`
//! dann entsteht eine Race-Condition: Parallele Tasks können `commit_mutex` vor der physischen
//! WAL-Write-Phase übernehmen und ihre Batch-HMACs auf Basis des alten `last_hmac` berechnen.
//! Bei der anschließenden HMAC-Ketten-Rekonstruktion in `verify_wal_hmac_chain` schlägt die
//! Assertion `entry.prev_hmac == expected_prev_hmac` fehl.
//!
//! Die Lock-Handoff-Sequenz (`truncate_lock.lock()` ERWERBEN bevor `_commit_lock` FREIGEGEBEN wird)
//! garantiert deterministische WAL-Schreibreihenfolge analog zu `lsm/mod.rs:788–792`.

#![allow(unexpected_cfgs)]

use memfuse_core::{MemFuseError, TxId};
use memfuse_store::wal::{PreparedBatch, Wal, WalOp};
use std::sync::Arc;

struct GroupCommitRequest {
    _tx_id: u64,
    batch: PreparedBatch,
    sender: tokio::sync::oneshot::Sender<Result<(), MemFuseError>>,
}

struct PendingCommitQueue {
    requests: Vec<GroupCommitRequest>,
    first_prev_hmac: [u8; 32],
}

struct GroupCommitEngine {
    commit_mutex: tokio::sync::Mutex<()>,
    pending_commit_queue: tokio::sync::Mutex<Option<PendingCommitQueue>>,
    wal: Wal,
}

impl GroupCommitEngine {
    fn new(wal: Wal) -> Self {
        Self {
            commit_mutex: tokio::sync::Mutex::new(()),
            pending_commit_queue: tokio::sync::Mutex::new(None),
            wal,
        }
    }

    /// Nachbildung der Gruppen-Commit-Logik aus `lsm/mod.rs` & `lsm/group_commit.rs`
    /// unter Verwendung der ECHTEN `Wal::prepare_batch` und `Wal::append_batch_locked` Implementation.
    async fn commit(&self, tx_id: u64) -> Result<(), MemFuseError> {
        // PHASE 1: commit_mutex erwerben und WAL Entry vorbereiten
        let _commit_lock = self.commit_mutex.lock().await;

        let op = WalOp::Put {
            tx_id: TxId::new(tx_id),
            key: format!("key-{tx_id}").into_bytes(),
            value: b"val".to_vec(),
        };
        let (batch, prev_hmac_snapshot) = self.wal.prepare_batch(vec![(op, tx_id)]).await?;

        // PHASE 2: Prüfen ob bereits eine Pending Queue existiert
        let mut queue_guard = self.pending_commit_queue.lock().await;

        if let Some(ref mut queue) = *queue_guard {
            // Follower-Pfad: In bestehende Leader-Queue einreihen und auf Leader warten
            let (tx, rx) = tokio::sync::oneshot::channel();
            let req = GroupCommitRequest {
                _tx_id: tx_id,
                batch,
                sender: tx,
            };
            queue.requests.push(req);
            drop(queue_guard);
            drop(_commit_lock);

            rx.await
                .map_err(|_| MemFuseError::Internal("Leader dropped commit channel".into()))?
        } else {
            // Leader-Pfad: Queue initialisieren
            let leader_batch = batch;
            *queue_guard = Some(PendingCommitQueue {
                requests: Vec::new(),
                first_prev_hmac: prev_hmac_snapshot,
            });
            drop(queue_guard);
            drop(_commit_lock);

            // Zero-Wait / Latency window: Yield um Follower-Tasks Zeit zum Einreihen zu geben
            tokio::task::yield_now().await;

            // Leader übernimmt Queue zur physischen WAL-I/O Phase
            let _commit_lock = self.commit_mutex.lock().await;
            let mut queue_guard = self.pending_commit_queue.lock().await;
            let pending_queue = queue_guard
                .take()
                .expect("pending commit queue must exist for leader");
            drop(queue_guard);

            let mut combined_batch = leader_batch;
            for r in &pending_queue.requests {
                combined_batch.extend(r.batch.clone());
            }

            // LOCK-HANDOFF (P0-A Fix analog zu lsm/mod.rs:788–792):
            // Acquire truncate_lock BEFORE dropping commit_mutex.
            let truncate_guard = self.wal.truncate_lock.lock().await;
            drop(_commit_lock);

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
                let err_msg = e.to_string();
                for r in pending_queue.requests {
                    let _ = r.sender.send(Err(MemFuseError::Storage(err_msg.clone())));
                }
                return Err(e);
            }

            // Re-acquire commit_mutex for MemTable / visibility updates & follower notification (lsm/mod.rs:810)
            let _commit_lock = self.commit_mutex.lock().await;

            for r in pending_queue.requests {
                let _ = r.sender.send(Ok(()));
            }

            Ok(())
        }
    }
}

/// Rekonstruiert und verifiziert die physische HMAC-Kette im WAL.
async fn verify_wal_hmac_chain(wal: &Wal) -> Result<usize, MemFuseError> {
    let size = wal.size();
    let mut expected_prev_hmac = [0u8; 32];
    let mut entry_count = 0usize;

    wal.scan_entries_with_callback(size, |_seq, entry, _pos| {
        assert_eq!(
            entry.prev_hmac, expected_prev_hmac,
            "WAL entry prev_hmac mismatch at entry {entry_count}! HMAC chain broken."
        );
        expected_prev_hmac = entry.checksum;
        entry_count += 1;
        true
    })
    .await?;

    let final_hmac = wal.last_hmac_snapshot().await;
    assert_ne!(final_hmac, [0u8; 32], "last_hmac must not be zero");
    assert_eq!(
        expected_prev_hmac, final_hmac,
        "Final entry checksum must match wal.last_hmac_snapshot()"
    );

    Ok(entry_count)
}

#[cfg(loom)]
#[test]
fn test_loom_group_commit_last_hmac_race() {
    loom::model(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("failed to build tokio runtime");

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

            let engine_3 = Arc::clone(&engine);
            let task3 = tokio::spawn(async move {
                let _ = engine_3.commit(3).await;
            });

            let _ = tokio::join!(task1, task2, task3);

            let entry_count = verify_wal_hmac_chain(&engine.wal)
                .await
                .expect("HMAC chain verification failed");
            assert_eq!(entry_count, 3, "Expected 3 committed entries in WAL");

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

            let written = physical_seqs.lock().map(|g| g.clone()).unwrap_or_default();
            assert_eq!(
                written.len(),
                3,
                "Exactly 3 physical entries must be written to WAL"
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

    let entry_count = verify_wal_hmac_chain(&engine.wal)
        .await
        .expect("HMAC chain verification failed");
    assert_eq!(entry_count, 3, "Expected 3 committed entries in WAL");
}
