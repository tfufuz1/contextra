//! # Schreibpfad RAII-Guards (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: RAII-Guards zur Gewährleistung von Fail-Closed, Cancel-Safety und Rollback
//! // INVARIANTEN: ADR-0XX-P0A, INV-STORE-1
//!
//! Stellt RAII-Guards für den LSM-Schreibpfad bereit:
//! - [`IntentLockGuard`]: Bereinigt Intent-Locks im Fehler- oder Cancel-Fall.
//! - [`SingleCommitGuard`]: Führt im Fehler-/Abbruchfall ein atomares WAL-Truncate und State-Rollback durch.
//! - [`LeaderCancelGuard`]: Benachrichtigt wartende Follower im Group-Commit bei Abbruch des Leaders.

use super::super::super::engine::LsmStorage;
use super::super::super::group_commit::PendingCommitQueue;
use super::super::super::Wal;
use crate::wal::PoisonCause;
use contextra_ids::{TxId, TOMBSTONE_BIT};

pub(super) struct IntentLockGuard<'a>(pub(super) &'a LsmStorage, pub(super) TxId);

impl<'a> Drop for IntentLockGuard<'a> {
    fn drop(&mut self) {
        self.0.clear_intent_locks_for_tx(self.1);
    }
}

pub(super) struct SingleCommitGuard<'a> {
    pub(super) storage: &'a LsmStorage,
    pub(super) wal: &'a Wal,
    pub(super) tx_id: TxId,
    pub(super) first_seq: u64,
    pub(super) prev_hmac: [u8; 32],
    pub(super) committed: bool,
}

impl<'a> Drop for SingleCommitGuard<'a> {
    fn drop(&mut self) {
        if !self.committed {
            if !self.wal.rollback_hmac_sync(self.prev_hmac) {
                self.wal
                    .poison_with_cause(PoisonCause::HmacChainDiverged);
            }
            if self.first_seq > 0 {
                if let Some(ref seq_val) = self.storage.sequence_log_ssi_validator {
                    seq_val.forget_from(self.first_seq);
                }
            }
            self.storage.cleanup_intent_locks_for_tx(self.tx_id);
        }
    }
}

pub(super) struct LeaderCancelGuard<'a> {
    pub(super) storage: &'a LsmStorage,
    pub(super) wal: &'a Wal,
    pub(super) leader_tx_id: TxId,
    pub(super) leader_first_seq: u64,
    pub(super) pending_queue: Option<PendingCommitQueue>,
    pub(super) prev_hmac: Option<[u8; 32]>,
    pub(super) committed: bool,
}

impl<'a> Drop for LeaderCancelGuard<'a> {
    fn drop(&mut self) {
        if !self.committed {
            let mut hmac_diverged = false;
            if let Some(prev) = self.prev_hmac {
                if !self.wal.rollback_hmac_sync(prev) {
                    self.wal
                        .poison_with_cause(PoisonCause::HmacChainDiverged);
                    hmac_diverged = true;
                }
            }
            if let Some(pending_queue) = self.pending_queue.take() {
                for r in pending_queue.requests {
                    let r_first_seq = r
                        .mem_updates
                        .first()
                        .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
                        .unwrap_or(0);
                    if r_first_seq > 0 {
                        if let Some(ref seq_val) = self.storage.sequence_log_ssi_validator {
                            seq_val.forget_from(r_first_seq);
                        }
                    }
                    self.storage.cleanup_intent_locks_for_tx(r.tx_id);
                    let err_msg = if hmac_diverged {
                        "Leader cancelled group commit: WAL HMAC chain diverged (handle poisoned)"
                    } else {
                        "Leader cancelled group commit"
                    };
                    let _ = r.sender.send(Err(crate::error::StoreError::Storage(err_msg.into())));
                }
            } else if let Ok(mut queue_guard) = self.storage.pending_commit_queue.try_lock() {
                if let Some(pending_queue) = queue_guard.take() {
                    for r in pending_queue.requests {
                        let r_first_seq = r
                            .mem_updates
                            .first()
                            .map(|(_, _, seq)| seq & !TOMBSTONE_BIT)
                            .unwrap_or(0);
                        if r_first_seq > 0 {
                            if let Some(ref seq_val) = self.storage.sequence_log_ssi_validator {
                                seq_val.forget_from(r_first_seq);
                            }
                        }
                        self.storage.cleanup_intent_locks_for_tx(r.tx_id);
                        let err_msg = if hmac_diverged {
                            "Leader cancelled group commit: WAL HMAC chain diverged (handle poisoned)"
                        } else {
                            "Leader cancelled group commit"
                        };
                        let _ = r.sender.send(Err(crate::error::StoreError::Storage(err_msg.into())));
                    }
                }
            }
            if self.leader_first_seq > 0 {
                if let Some(ref seq_val) = self.storage.sequence_log_ssi_validator {
                    seq_val.forget_from(self.leader_first_seq);
                }
            }
            self.storage.cleanup_intent_locks_for_tx(self.leader_tx_id);
        }
    }
}
