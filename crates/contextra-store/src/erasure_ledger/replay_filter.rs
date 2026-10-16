//! FILE-CONTEXT
//! STAND: 2026-10-16
//! ZWECK: Thread-sicherer Handle für ErasureLedger und Implementierung von WalReplayFilter und CompactionRecordFilter.
//! INVARIANTEN: Zero-Panic, Fail-Closed, non-blocking sync decision evaluate für Hotpath/Replay.

use super::{
    ErasureLedger, LedgerDecision, LedgerEntry, RecordType,
};
use crate::compaction::purge_filter::CompactionRecordFilter;
use crate::wal::replay_filter::WalReplayFilter;
use contextra_types::Result;
use std::sync::Arc;

/// Thread-safe handle wrapping [`ErasureLedger`] with sync read and async append locks.
#[derive(Debug)]
pub struct ErasureLedgerHandle {
    ledger: parking_lot::RwLock<ErasureLedger>,
    append_mutex: tokio::sync::Mutex<()>,
}

impl ErasureLedgerHandle {
    /// Constructs a new [`ErasureLedgerHandle`] wrapping the given ledger.
    pub fn new(ledger: ErasureLedger) -> Self {
        Self {
            ledger: parking_lot::RwLock::new(ledger),
            append_mutex: tokio::sync::Mutex::new(()),
        }
    }

    /// Appends entries to the erasure ledger with atomic fsync durability.
    pub async fn append(&self, entries: &[LedgerEntry]) -> Result<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard.append(entries).await
    }

    /// Appends Intent record (with plaintext prefixes for resume).
    pub async fn append_intent(
        &self,
        tx: u64,
        purge_id: [u8; 16],
        scope_digest: [u8; 32],
        family_digests: Vec<[u8; 32]>,
        payload: Vec<u8>,
    ) -> Result<()> {
        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard.append_intent(tx, purge_id, scope_digest, family_digests, payload).await
    }

    /// Appends Purged record (only digests, no plaintext payload).
    pub async fn append_purged(
        &self,
        tx: u64,
        purge_id: [u8; 16],
        scope_digest: [u8; 32],
        family_digests: Vec<[u8; 32]>,
    ) -> Result<()> {
        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard.append_purged(tx, purge_id, scope_digest, family_digests).await
    }

    /// Group append multiple records sharing a single fsync.
    pub async fn append_batch(
        &self,
        records_data: &[(RecordType, u64, [u8; 16], [u8; 32], Vec<[u8; 32]>, Vec<u8>)],
    ) -> Result<()> {
        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard.append_batch(records_data).await
    }

    /// Decides whether a key version at transaction `tx` should be kept or dropped.
    pub fn decide(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> LedgerDecision {
        self.ledger.read().decide(storage_scope, key, tx)
    }

    /// Returns the database instance ID associated with this ledger.
    pub fn db_instance_id(&self) -> [u8; 16] {
        *self.ledger.read().db_instance_id()
    }

    /// Returns the last chain MAC of the ledger log.
    pub fn last_chain_mac(&self) -> [u8; 32] {
        *self.ledger.read().last_chain_mac()
    }

    /// Returns a copy of all current ledger entries.
    pub fn entries(&self) -> Vec<LedgerEntry> {
        self.ledger.read().entries().to_vec()
    }

    /// Returns the head sequence and MAC of the ledger.
    pub fn read_head_info(&self) -> (u64, [u8; 32]) {
        self.ledger.read().read_head_info()
    }
}

/// Filter implementing both [`WalReplayFilter`] and [`CompactionRecordFilter`] via [`ErasureLedgerHandle`].
#[derive(Clone, Debug)]
pub struct LedgerReplayFilter {
    handle: Arc<ErasureLedgerHandle>,
}

impl LedgerReplayFilter {
    /// Constructs a new [`LedgerReplayFilter`] backed by the provided ledger handle.
    pub fn new(handle: Arc<ErasureLedgerHandle>) -> Self {
        Self { handle }
    }
}

impl WalReplayFilter for LedgerReplayFilter {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        self.handle.decide(b"lsm", key, tx) == LedgerDecision::Drop
    }
}

impl CompactionRecordFilter for LedgerReplayFilter {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        self.handle.decide(b"lsm", key, tx) == LedgerDecision::Drop
    }
}
