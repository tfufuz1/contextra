//! Tests for candidate selection in compaction.

use super::*;
use crate::sstable::{create_block_cache, SstableBuilder};
use tempfile::TempDir;

#[test]
fn test_generate_sst_path_uniqueness() {
    let temp_dir = TempDir::new().unwrap();
    let counter = std::sync::atomic::AtomicU64::new(1);
    let path1 = crate::compaction::engine::install::generate_sst_path(
        temp_dir.path(),
        &std::sync::atomic::AtomicU64::new(1),
        &counter,
    )
    .unwrap();
    let path2 = crate::compaction::engine::install::generate_sst_path(
        temp_dir.path(),
        &std::sync::atomic::AtomicU64::new(2),
        &counter,
    )
    .unwrap();
    assert_ne!(path1, path2);
}
