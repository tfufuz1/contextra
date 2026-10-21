pub use super::*;
pub use crate::compaction::{CompactionConfig, CompactionEngine};
pub use contextra_types::TOMBSTONE_BIT;
pub use std::sync::Arc;
pub use std::time::Duration;
use tempfile::TempDir;

pub(crate) async fn test_storage() -> (LsmStorage, TempDir) {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 1024 * 1024,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");
    (storage, tmp)
}

#[cfg(test)]
mod commit_tests;
#[cfg(test)]
mod flush_tests;
#[cfg(test)]
mod mvcc_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod scan_tests;
#[cfg(test)]
mod w1_02_async_observer_tests;
#[cfg(test)]
mod w1_02_cancel_safety_tests;
#[cfg(test)]
mod w1_02_fence_tests;
#[cfg(test)]
mod w1_02_phantom_commit_tests;
#[cfg(test)]
mod w1_02_scan_hygiene_tests;
#[cfg(test)]
mod w1_02_seq_visibility_tests;
