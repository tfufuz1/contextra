//! Active WAL segment sealing (SPEC §8.8, §11.4, D-73).

use super::backend::SealPoint;
use crate::wal::Wal;
use contextra_types::Result;

/// Seals the active WAL segment at the current sequence boundary.
pub async fn seal_active_wal(wal: &Wal, current_seq: u64) -> Result<SealPoint> {
    let _sealed_path = wal.rotate_and_seal().await?;
    let tail_mac = wal.last_hmac_snapshot().await;
    Ok(SealPoint {
        seq: current_seq,
        tail_mac,
    })
}
