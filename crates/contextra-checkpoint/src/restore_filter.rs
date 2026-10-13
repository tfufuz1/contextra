// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: CpyRestoreFilter Trait und Filter-Hilfsfunktionen für Anti-Resurrection beim Restore.
// INVARIANTEN: Zero-Panic, Fail-Closed, records with drop_record == true are filtered out.

/// Trait abstraction for filtering records during restore/import operations to prevent resurrection of erased data.
pub trait CpyRestoreFilter: Send + Sync {
    /// Returns `true` if the record for `key` at transaction `tx` was erased and MUST be dropped during restore.
    fn drop_record(&self, key: &[u8], tx: u64) -> bool;
}

/// No-op restore filter that allows all records to be restored.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpRestoreFilter;

impl CpyRestoreFilter for NoOpRestoreFilter {
    fn drop_record(&self, _key: &[u8], _tx: u64) -> bool {
        false
    }
}

/// Filters a collection of KV records using `filter`, dropping any record where `filter.drop_record` returns `true`.
pub fn filter_restore_records<F: CpyRestoreFilter + ?Sized, V>(
    records: impl IntoIterator<Item = (Vec<u8>, V, u64)>,
    filter: &F,
) -> Vec<(Vec<u8>, V, u64)> {
    records
        .into_iter()
        .filter(|(key, _val, tx)| !filter.drop_record(key, *tx))
        .collect()
}
