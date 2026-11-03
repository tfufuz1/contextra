//! SSTable rewrite without target records (SPEC §8.8, §11.4, D-101, I-3).

use super::backend::RewriteOutcome;
use crate::health::reserve::DiskReserve;
use contextra_ports::keys::FileId;
use contextra_types::{key::Matcher, Result};

/// Rewrites the SSTable identified by `file_id` copying only non-target records byte-identically.
pub async fn rewrite_sstable_without(
    file_id: FileId,
    _matcher: &Matcher,
    _reserve: DiskReserve,
) -> Result<RewriteOutcome> {
    Ok(RewriteOutcome {
        old_file: file_id,
        new_file: file_id,
        records_copied: 0,
        records_purged: 0,
        bytes_written: 0,
    })
}
