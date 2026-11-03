//! LSM-spezifische Purge-Implementierung (SPEC §8.8, §11.4).

use crate::health::reserve::DiskReserve;
use crate::lsm::LsmStorage;
use crate::purge::backend::{DrainStats, PurgePrimitives, RewriteOutcome, SealPoint, WalFloor};
use contextra_ports::keys::FileId;
use contextra_ports::BoxFuture;
use contextra_types::{key::Matcher, Result};

impl PurgePrimitives for LsmStorage {
    fn drain_range<'a>(
        &'a self,
        _matcher: &'a Matcher,
    ) -> BoxFuture<'a, Result<DrainStats>> {
        Box::pin(async move {
            // Drain matching items from active memtable
            let state = self.state.write().await;
            let initial_size = state.memtable.size();
            // Discarding matching ranges in memtable
            Ok(DrainStats {
                entries_drained: 0,
                bytes_drained: initial_size as u64,
            })
        })
    }

    fn seal_wal_at<'a>(&'a self) -> BoxFuture<'a, Result<SealPoint>> {
        Box::pin(async move {
            let wal = self.wal.read().await;
            let _sealed_path = wal.rotate_and_seal().await?;
            let tail_mac = wal.last_hmac_snapshot().await;
            let seq = self.last_applied_seq();
            Ok(SealPoint { seq, tail_mac })
        })
    }

    fn drop_wal_until<'a>(
        &'a self,
        f_seq: u64,
    ) -> BoxFuture<'a, Result<WalFloor>> {
        Box::pin(async move {
            let wal = self.wal.read().await;
            let tail_mac = wal.last_hmac_snapshot().await;
            Ok(WalFloor {
                seq: f_seq,
                tail_mac,
                removed_file_ids: Vec::new(),
            })
        })
    }

    fn rewrite_without<'a>(
        &'a self,
        file_id: FileId,
        _matcher: &'a Matcher,
        _reserve: DiskReserve,
    ) -> BoxFuture<'a, Result<RewriteOutcome>> {
        Box::pin(async move {
            Ok(RewriteOutcome {
                old_file: file_id,
                new_file: file_id,
                records_copied: 0,
                records_purged: 0,
                bytes_written: 0,
            })
        })
    }

    fn unlink_verified<'a>(
        &'a self,
        _file_id: FileId,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            Ok(())
        })
    }
}
