// FILE-CONTEXT
// ZWECK: SequenceLog-Kompaktierung und Snapshot-Verwaltung fuer GenericHnswIndex.
// INVARIANTEN: Aufrufe delegieren thread-sicher ueber RwLock an contextra_mvcc::SequenceLog.
// HOTSPOTS: GenericHnswIndex::pin_snapshot, GenericHnswIndex::compact_seq_log
// STAND: TS:2026-11-12T00:00:00Z

use super::index::GenericHnswIndex;
use crate::distance::DistanceKernel;

impl<K: DistanceKernel> GenericHnswIndex<K> {
    pub fn compact_seq_log(&self, min_active_seqno: u64) {
        self.inner.cold.seq_log.write().compact(min_active_seqno);
    }

    pub fn pin_snapshot(&self, seq_no: u64) {
        self.inner.cold.seq_log.write().pin_snapshot(seq_no);
    }

    pub fn unpin_snapshot(&self, seq_no: u64) {
        self.inner.cold.seq_log.write().unpin_snapshot(seq_no);
    }
}
