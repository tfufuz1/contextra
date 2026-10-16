//! FILE-CONTEXT
//! STAND: 2026-10-16
//! ZWECK: Thread-sicherer Handle für Anti-Resurrection ErasureLedger (CTRALEDG v1).
//! INVARIANTEN: Zero-Panic, Fail-Closed, lock-free decision query für Hotpath/Replay.

use super::{CTRALRecord, ErasureLedger, LedgerDecision, LedgerEntry};
use crate::compaction::purge_filter::CompactionRecordFilter;
use crate::wal::replay_filter::WalReplayFilter;
use contextra_ports::Clock;
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

    /// Appends legacy entries to the erasure ledger.
    pub async fn append(&self, entries: &[LedgerEntry]) -> Result<()> {
        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard.append(entries).await
    }

    /// Appends an `Intent` record (0x01).
    pub async fn append_intent(
        &self,
        tx: u64,
        clock: &dyn Clock,
        purge_id: [u8; 16],
        scope: &[u8],
        families: &[&[u8]],
        payload: &[u8],
    ) -> Result<[u8; 32]> {
        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard
            .append_intent(tx, clock, purge_id, scope, families, payload)
            .await
    }

    /// Appends a `Purged` record (0x02).
    pub async fn append_purged(
        &self,
        tx: u64,
        clock: &dyn Clock,
        purge_id: [u8; 16],
        scope: &[u8],
        families: &[&[u8]],
    ) -> Result<[u8; 32]> {
        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard
            .append_purged(tx, clock, purge_id, scope, families)
            .await
    }

    /// Appends CTRALRecords in batch.
    pub async fn append_records(&self, records: &[CTRALRecord]) -> Result<[u8; 32]> {
        let _guard = self.append_mutex.lock().await;
        let mut guard = self.ledger.write();
        guard.append_records(records).await
    }

    /// Decides whether a key version at transaction `tx` should be kept or dropped.
    pub fn decide(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> LedgerDecision {
        self.ledger.read().decide(storage_scope, key, tx)
    }

    /// Checks if key is in deleted scope for `RestoreGuard`.
    pub fn is_in_deleted_scope(&self, storage_scope: &[u8], key: &[u8], tx: u64) -> bool {
        self.ledger.read().is_in_deleted_scope(storage_scope, key, tx)
    }

    /// Returns the database instance ID associated with this ledger.
    pub fn db_instance_id(&self) -> [u8; 16] {
        *self.ledger.read().db_instance_id()
    }

    /// Returns the last chain MAC of the ledger log.
    pub fn last_chain_mac(&self) -> [u8; 32] {
        *self.ledger.read().last_chain_mac()
    }

    /// Returns all current ledger entries.
    pub fn entries(&self) -> Vec<LedgerEntry> {
        self.ledger.read().entries()
    }

    /// Returns raw CTRALRecords.
    pub fn records(&self) -> Vec<CTRALRecord> {
        self.ledger.read().records().to_vec()
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
