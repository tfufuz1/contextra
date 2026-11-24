//! # Observer-Typen & Schnittstellen (Ring 1)
//!
//! // FILE-CONTEXT
//! // STAND: 2026-11-11
//! // ZWECK: Definition der Datentypen und des WalObserver-Traits
//! // INVARIANTEN: Fail-open Semantik, non-blocking Commit

use crate::wal::{WalEntry, WalOp};
use contextra_ids::TxId;
use std::time::Duration;

/// Default maximum latency allowed for WAL observer callback execution (1 millisecond).
pub const DEFAULT_MAX_OBSERVER_LATENCY: Duration = Duration::from_millis(1);

/// Origin tag indicating where a committed batch originated from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WriteOrigin {
    /// Regular user/application write transaction.
    UserWrite,
}

/// Lightweight view reference into a `WalEntry`.
#[derive(Debug, Clone, Copy)]
pub struct WalEntryRef<'a> {
    pub entry: &'a WalEntry,
}

impl<'a> WalEntryRef<'a> {
    /// Creates a new `WalEntryRef` from a `WalEntry`.
    pub fn new(entry: &'a WalEntry) -> Self {
        Self { entry }
    }

    /// Returns a reference to the operation inside the entry.
    pub fn op(&self) -> &'a WalOp {
        &self.entry.op
    }

    /// Returns the sequence number of the entry.
    pub fn seq_no(&self) -> u64 {
        self.entry.seq_no
    }

    /// Returns a reference to the entry's checksum.
    pub fn checksum(&self) -> &'a [u8; 32] {
        &self.entry.checksum
    }

    /// Returns a reference to the entry's previous HMAC checksum.
    pub fn prev_hmac(&self) -> &'a [u8; 32] {
        &self.entry.prev_hmac
    }

    /// Returns the transaction ID associated with this entry.
    pub fn tx_id(&self) -> TxId {
        self.entry.tx_id()
    }
}

/// Context describing a committed transaction passed to observers.
#[derive(Debug, Clone, Copy)]
pub struct CommitContext {
    pub tx_id: TxId,
    pub origin: WriteOrigin,
    pub durable: bool,
}

/// Batch of committed WAL entries passed to observers.
pub struct CommittedBatch<'a> {
    pub entries: Vec<WalEntryRef<'a>>,
    pub origin: WriteOrigin,
}

/// Synchronous, deterministic observer interface for committed WAL batches.
pub trait WalObserver: Send + Sync {
    /// Indicates whether this observer requires durable WAL commits.
    ///
    /// Returns `false` by default. Observers setting this to `false` will also receive notifications
    /// for `DurabilityMode::MemoryOnly` commits.
    fn requires_durability(&self) -> bool {
        false
    }

    /// Notification callback executed upon batch commit.
    ///
    /// Invoked via a bounded, non-blocking dispatcher mechanism. The commit path enforces a
    /// configurable maximum latency deadline (`max_observer_latency`). Observers taking longer
    /// than this deadline will time out on the commit path, increment drop counters, and trip
    /// the circuit breaker to prevent blocking subsequent transaction commits.
    fn on_commit(&self, batch: &CommittedBatch<'_>, seq_no: u64, tx_id: TxId);
}
