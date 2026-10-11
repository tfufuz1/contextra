// FILE-CONTEXT
// STAND: 2026-10-11 (T-2026-9103)
// ZWECK: Abdeckungsprüfer (LedgerCoverage) zur Erkennung fehlender Ledger-Einträge für gelöschte Schlüssel/Tombstones.
// INVARIANTEN: Zero-Panic, Fail-Closed, Auditability.

use crate::erasure_ledger::{ErasureLedger, LedgerDecision};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LedgerGap {
    #[error("Tombstoned key/prefix is missing corresponding erasure ledger entry for scope {storage_scope:?}")]
    MissingLedgerEntry {
        storage_scope: Vec<u8>,
        erased_tx: u64,
    },
}

/// Ledger coverage verifier for auditing consistency between tombstoned keys and erasure ledger.
pub struct LedgerCoverage;

impl LedgerCoverage {
    /// Checks that for every tombstoned key/prefix (with storage_scope, key_bytes, and tombstone_tx),
    /// a corresponding matching entry exists in the ledger that causes `decide(storage_scope, key_bytes, tombstone_tx)`
    /// to return `LedgerDecision::Drop`.
    /// Returns `Err(LedgerGap)` if any tombstoned key lacks an erasure ledger entry.
    pub fn check(
        tombstoned_keys: &[(&[u8], &[u8], u64)],
        ledger: &ErasureLedger,
    ) -> Result<(), LedgerGap> {
        for (storage_scope, key_bytes, tombstone_tx) in tombstoned_keys {
            let decision = ledger.decide(storage_scope, key_bytes, *tombstone_tx);
            if decision != LedgerDecision::Drop {
                return Err(LedgerGap::MissingLedgerEntry {
                    storage_scope: storage_scope.to_vec(),
                    erased_tx: *tombstone_tx,
                });
            }
        }
        Ok(())
    }
}
