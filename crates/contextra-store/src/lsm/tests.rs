pub use super::*;
pub use crate::compaction::{CompactionConfig, CompactionEngine};
pub use contextra_core::TOMBSTONE_BIT;
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
