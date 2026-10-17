// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: CpyRestoreFilter Trait und Filter-Hilfsfunktionen für Anti-Resurrection beim Restore.
// INVARIANTEN: Zero-Panic, Fail-Closed, records with drop_record == true are filtered out.

/// Trait abstraction for filtering records during restore/import operations to prevent resurrection of erased data.
pub trait CpyRestoreFilter: Send + Sync {
    /// Returns `true` if the record for `key` at transaction `tx` was erased and MUST be dropped during restore.
    fn drop_record(&self, key: &[u8], tx: u64) -> bool;
}

impl<T: contextra_ports::RestoreGuard + ?Sized> CpyRestoreFilter for T {
    #[allow(deprecated)]
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        self.admit(key, contextra_types::TxId::new(tx)) == contextra_ports::RestoreVerdict::Reject
    }
}

/// No-op restore filter that allows all records to be restored.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpRestoreFilter;

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
