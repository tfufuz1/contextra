// FILE-CONTEXT
// ZWECK: RAII-Guards fuer Rebuild-Synchronisation und Snapshot-Pinning im SequenceLog.
// INVARIANTEN: RebuildGuard setzt atomar rebuilding=false und signalisiert Condvar beim Drop.
// HOTSPOTS: RebuildGuard::drop, SnapshotPinGuard::drop
// STAND: TS:2026-11-12T00:00:00Z

use super::core::HnswIndexCore;
use std::sync::atomic::{AtomicBool, Ordering};

// ANCHOR[contextra-vector:HNSW-REBUILD-GUARD]
/// Guard, der das `rebuilding`-Flag auf `false` zuruecksetzt und wartende Threads weckt.
pub struct RebuildGuard<'a>(pub &'a AtomicBool, pub &'a parking_lot::Condvar);

impl<'a> Drop for RebuildGuard<'a> {
    fn drop(&mut self) {
        // NOTE: SeqCst guarantees all threads observing rebuilding flag observe completed rebuild state.
        self.0.store(false, Ordering::SeqCst);
        self.1.notify_all();
    }
}

// ANCHOR[contextra-vector:HNSW-SNAPSHOT-PIN-GUARD]
/// Guard, der einen Snapshot im `SequenceLog` pinnt und beim Drop wieder freigibt.
pub struct SnapshotPinGuard<'a>(pub &'a HnswIndexCore, pub(crate) u64);

impl<'a> SnapshotPinGuard<'a> {
    pub(crate) fn new(core: &'a HnswIndexCore, seq_no: u64) -> Self {
        core.cold.seq_log.write().pin_snapshot(seq_no);
        Self(core, seq_no)
    }
}

impl<'a> Drop for SnapshotPinGuard<'a> {
    fn drop(&mut self) {
        self.0.cold.seq_log.write().unpin_snapshot(self.1);
    }
}
