//! Ledger sink v2 port definitions and contracts (WP-02 §4.7).
//!
//! Provides append-only, tamper-evident audit tracking of completed erasures.

use crate::BoxFuture;
pub use contextra_types::{ErasureError, ErasureException, Guarantee, Timestamp};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A tamper-evident, append-only entry in the erasure ledger (WP-02 §4.7).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Monotonic zero-based entry index in the ledger.
    pub index: u64,
    /// Keyed HMAC-BLAKE3 of the erasure job ID.
    pub erasure_id_hmac: [u8; 32],
    /// Achieved erasure guarantee.
    pub guarantee: Guarantee,
    /// BLAKE3 root over per-layer witness digests in canonical order.
    pub layers_root: [u8; 32],
    /// Documented exceptions affecting the guarantee.
    pub exceptions: Vec<ErasureException>,
    /// BLAKE3 hash of the canonical erasure plan.
    pub plan_hash: [u8; 32],
    /// Policy schema version.
    pub policy_version: u32,
    /// UTC timestamp when the erasure was finalized.
    pub completed_at: Timestamp,
    /// Hash of the preceding ledger entry (`[0; 32]` for index 0).
    pub prev_hash: [u8; 32],
    /// HMAC over all fields including `prev_hash`.
    pub entry_mac: [u8; 32],
}

impl fmt::Debug for LedgerEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LedgerEntry")
            .field("index", &self.index)
            .field("guarantee", &self.guarantee)
            .field("exceptions_count", &self.exceptions.len())
            .field("policy_version", &self.policy_version)
            .field("completed_at", &self.completed_at)
            .finish()
    }
}

/// Draft payload submitted to [`LedgerSink::append`]; the sink assigns
/// `index`, `prev_hash`, and `entry_mac`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LedgerDraft {
    /// Keyed HMAC-BLAKE3 of the erasure job ID.
    pub erasure_id_hmac: [u8; 32],
    /// Achieved erasure guarantee.
    pub guarantee: Guarantee,
    /// BLAKE3 root over per-layer witness digests in canonical order.
    pub layers_root: [u8; 32],
    /// Documented exceptions affecting the guarantee.
    pub exceptions: Vec<ErasureException>,
    /// BLAKE3 hash of the canonical erasure plan.
    pub plan_hash: [u8; 32],
    /// Policy schema version.
    pub policy_version: u32,
    /// UTC timestamp when the erasure was finalized.
    pub completed_at: Timestamp,
}

/// Current head pointer of the append-only erasure ledger.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LedgerHead {
    /// Index of the latest entry (or `0` with zero hash if empty).
    pub index: u64,
    /// Hash / MAC of the latest entry.
    pub hash: [u8; 32],
}

/// Result of verifying the integrity of the entire ledger chain.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct LedgerVerification {
    /// `true` if every entry's chain hash and MAC verified cleanly.
    pub valid: bool,
    /// Total number of entries verified.
    pub entries_checked: u64,
    /// Verified head pointer.
    pub head: LedgerHead,
}

/// Append-only durable audit ledger sink (WP-02 §4.7).
pub trait LedgerSink: Send + Sync {
    /// Appends a verified erasure entry to the ledger.
    /// Idempotent over `(e.erasure_id_hmac, e.plan_hash)`.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::LedgerCorrupt`] oder [`ErasureError::Io`] bei
    /// Integritäts- bzw. Schreibfehlern.
    /// # Crash
    /// Schreibt append-only und stellt Dauerhaftigkeit vor Rückkehr via `fsync` sicher.
    fn append<'a>(&'a self, e: LedgerDraft) -> BoxFuture<'a, Result<LedgerEntry, ErasureError>>;

    /// Returns the current ledger head.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::LedgerCorrupt`] oder [`ErasureError::Io`] bei Lesefehlern.
    /// # Crash
    /// Reiner Lesezugriff ohne Mutation.
    fn head<'a>(&'a self) -> BoxFuture<'a, Result<LedgerHead, ErasureError>>;

    /// Verifies the full hash and MAC chain of the ledger.
    ///
    /// # Fehler
    /// Liefert [`ErasureError::LedgerCorrupt`] bei Kettenbruch oder [`ErasureError::Io`].
    /// # Crash
    /// Reiner Lese- und Prüfzugriff ohne Mutation.
    fn verify<'a>(&'a self) -> BoxFuture<'a, Result<LedgerVerification, ErasureError>>;
}

/// External anchor port for publishing ledger head commitments (e.g. remote log or HSM).
pub trait LedgerAnchor: Send + Sync {
    /// Publishes the current [`LedgerHead`] to an external anchor.
    ///
    /// # Fehler
    /// Liefert [`ErasureError`] falls die Verankerung fehlschlägt.
    /// # Crash
    /// Idempotente Veröffentlichung des aktuellen Kopfes.
    fn publish<'a>(&'a self, h: &'a LedgerHead) -> BoxFuture<'a, Result<(), ErasureError>>;
}
