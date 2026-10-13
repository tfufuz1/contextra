//! Restore filter adapter for anti-resurrection (§C4.8, WP-1.1).

#![forbid(unsafe_code)]

use contextra_checkpoint::restore_filter::CpyRestoreFilter;
use contextra_store::erasure_ledger::{ErasureLedger, LedgerDecision};
use std::sync::Arc;

/// Restore filter adapter implementing [`CpyRestoreFilter`] backed by [`ErasureLedger`].
pub struct DpiRestoreFilter {
    ledger: Arc<ErasureLedger>,
    storage_scope: Vec<u8>,
}

impl DpiRestoreFilter {
    /// Creates a new `DpiRestoreFilter` wrapping `ledger` for `storage_scope`.
    #[must_use]
    pub fn new(ledger: Arc<ErasureLedger>, storage_scope: impl Into<Vec<u8>>) -> Self {
        Self {
            ledger,
            storage_scope: storage_scope.into(),
        }
    }

    /// Creates a default `DpiRestoreFilter` for LSM storage scope `b"lsm"`.
    #[must_use]
    pub fn for_lsm(ledger: Arc<ErasureLedger>) -> Self {
        Self::new(ledger, b"lsm".to_vec())
    }
}

impl CpyRestoreFilter for DpiRestoreFilter {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        self.ledger.decide(&self.storage_scope, key, tx) == LedgerDecision::Drop
    }
}
