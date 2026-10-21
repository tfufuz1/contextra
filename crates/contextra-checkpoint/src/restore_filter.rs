// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: CpyRestoreFilter Trait und Filter-Hilfsfunktionen für Anti-Resurrection beim Restore.
// INVARIANTEN: Zero-Panic, Fail-Closed, records with drop_record == true are filtered out.

use std::sync::Arc;

/// Trait abstraction for filtering records during restore/import operations to prevent resurrection of erased data.
pub trait CpyRestoreFilter: Send + Sync {
    /// Returns `true` if the record for `key` at transaction `tx` was erased and MUST be dropped during restore.
    fn drop_record(&self, key: &[u8], tx: u64) -> bool;
}

/// Explicit restore policy eliminating fail-open `Option` states.
#[derive(Clone)]
pub enum RestorePolicy {
    /// Filter records using the provided anti-resurrection filter.
    Filter(Arc<dyn CpyRestoreFilter>),
    /// Strictly forbid restore operations after erasure ledger initialization.
    ForbidAfterErasure,
}

impl RestorePolicy {
    /// Returns `true` if the record for `key` at transaction `tx` MUST be dropped during restore.
    #[must_use]
    pub fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        match self {
            Self::Filter(f) => f.drop_record(key, tx),
            Self::ForbidAfterErasure => true, // Fail closed
        }
    }
}

impl<T: contextra_ports::RestoreGuard + ?Sized> CpyRestoreFilter for T {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        let tx_id = contextra_types::TxId::try_new(tx)
            .unwrap_or(contextra_types::TxId::INVALID);
        self.admit(key, tx_id) == contextra_ports::RestoreVerdict::Reject
    }
}

/// No-op restore filter that allows all records to be restored (testing only).
#[cfg(any(test, feature = "testing"))]
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpRestoreFilter;

#[cfg(any(test, feature = "testing"))]
impl CpyRestoreFilter for NoOpRestoreFilter {
    fn drop_record(&self, _key: &[u8], _tx: u64) -> bool {
        false
    }
}

/// Report summarizing the outcome of a restore record filtering operation.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RestoreFilterReport {
    /// Number of records retained after filtering.
    pub kept: usize,
    /// Number of records dropped due to anti-resurrection rules.
    pub dropped: usize,
}

/// Filters a collection of KV records using `filter`, dropping any record where `filter.drop_record` returns `true`.
pub fn filter_restore_records<F: CpyRestoreFilter + ?Sized, V>(
    records: impl IntoIterator<Item = (Vec<u8>, V, u64)>,
    filter: &F,
) -> Vec<(Vec<u8>, V, u64)> {
    let (filtered, _report) = filter_restore_records_with_report(records, filter);
    filtered
}

/// Filters a collection of KV records using `filter`, returning the retained records along with a `RestoreFilterReport`.
pub fn filter_restore_records_with_report<F: CpyRestoreFilter + ?Sized, V>(
    records: impl IntoIterator<Item = (Vec<u8>, V, u64)>,
    filter: &F,
) -> (Vec<(Vec<u8>, V, u64)>, RestoreFilterReport) {
    let mut kept_records = Vec::new();
    let mut report = RestoreFilterReport::default();

    for (key, val, tx) in records {
        if filter.drop_record(&key, tx) {
            report.dropped += 1;
        } else {
            report.kept += 1;
            kept_records.push((key, val, tx));
        }
    }

    (kept_records, report)
}
