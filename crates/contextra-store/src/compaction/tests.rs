use super::*;
use crate::sstable::{create_block_cache, BlockCache, SstableBuilder, SstableReader};
use contextra_core::{SnapshotRegistry, StorageEngine, TOMBSTONE_BIT};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;

pub(super) async fn create_test_sstable(
    dir: &std::path::Path,
    name: &str,
    entries: &[(&[u8], &[u8], u64)],
    bc: Arc<BlockCache>,
) -> Arc<SstableReader> {
    let path = dir.join(name);
    let mut builder = SstableBuilder::create(&path).await.unwrap();
    for &(k, v, seq) in entries {
        builder.add(k, v, seq, 0).await.unwrap();
    }
    builder.finish().await.unwrap();
    Arc::new(
        SstableReader::open_with_key_manager(&path, bc, None)
            .await
            .unwrap(),
    )
}

#[cfg(test)]
mod basic;

#[cfg(test)]
mod advanced;
