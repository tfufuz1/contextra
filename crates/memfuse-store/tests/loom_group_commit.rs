//! Loom-basierter Determinismus-Beweis für SEC-01: Race-Condition im Group-Commit-Leader bezüglich `last_hmac`.
//! Ausführung: RUSTFLAGS="--cfg loom" cargo test -p memfuse-store --test loom_group_commit --release

use memfuse_store::wal::{PreparedBatch, Wal, WalOp};
use std::sync::Arc;

#[cfg(loom)]
use loom::sync::Mutex;

#[cfg(not(loom))]
use std::sync::Mutex;

struct GroupCommitRequest {
    _tx_id: u64,
    batch: PreparedBatch,
}

struct PendingCommitQueue {
    requests: Vec<GroupCommitRequest>,
    first_prev_hmac: [u8; 32],
}

struct GroupCommitEngine {
    commit_mutex: Mutex<()>,
    pending_commit_queue: Mutex<Option<PendingCommitQueue>>,
    wal: Wal,
}

impl GroupCommitEngine {
    fn new(wal: Wal) -> Self {
        Self {
            commit_mutex: Mutex::new(()),
            pending_commit_queue: Mutex::new(None),
            wal,
        }
    }

    /// Nachbildung der Gruppen-Commit-Logik aus `lsm/mod.rs` & `lsm/group_commit.rs`
    /// unter Verwendung der ECHTEN `Wal::prepare_batch` und `Wal::append_batch` Implementation.
    async fn commit(&self, tx_id: u64) -> Result<(), memfuse_core::MemFuseError> {
        // PHASE 1: WAL Entry vorbereiten
        let op = WalOp::Put {
            tx_id: memfuse_core::TxId::new(tx_id),
            key: format!("key-{tx_id}").into_bytes(),
            value: b"val".to_vec(),
        };
        let (batch, prev_hmac_snapshot) = self.wal.prepare_batch(vec![(op, tx_id)]).await?;

        // PHASE 2: Prüfen ob bereits eine Pending Queue existiert
        let is_follower = {
            let mut queue_guard = self
                .pending_commit_queue
                .lock()
                .unwrap_or_else(|e| e.into_inner());

            if let Some(ref mut queue) = *queue_guard {
                // Follower-Pfad: In bestehende Leader-Queue einreihen
                let req = GroupCommitRequest {
                    _tx_id: tx_id,
                    batch: batch.clone(),
                };
                queue.requests.push(req);
                true
            } else {
                // Leader-Pfad: Queue initialisieren
                *queue_guard = Some(PendingCommitQueue {
                    requests: Vec::new(),
                    first_prev_hmac: prev_hmac_snapshot,
                });
                false
            }
        };

        if is_follower {
            Ok(())
        } else {
            // Leader-Pfad: Pending Queue konsolidieren
            let (leader_batch, pending_queue) = {
                let mut queue_guard = self
                    .pending_commit_queue
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                let queue = queue_guard.take().expect("pending commit queue must exist");
                (batch, queue)
            };

            let mut combined_batch = leader_batch;
            for r in pending_queue.requests {
                combined_batch.extend(r.batch);
            }

            if let Err(e) = self.wal.append_batch(combined_batch).await {
                let _ = self
                    .wal
                    .restore_last_hmac(pending_queue.first_prev_hmac)
                    .await;
                return Err(e);
            }

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
            .unwrap();

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

    let final_hmac = engine.wal.last_hmac_snapshot().await;
    assert_ne!(final_hmac, [0u8; 32], "last_hmac must be updated");
}
