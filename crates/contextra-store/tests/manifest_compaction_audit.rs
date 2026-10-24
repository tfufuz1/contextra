// Integration tests for store manifest and compaction audit findings.

use contextra_core::{ResourceBudget, ResourceTracker, SnapshotRegistry, TOMBSTONE_BIT};
use contextra_store::compaction::{CompactionConfig, CompactionEngine};
use contextra_store::manifest::ManifestEntry;
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_manifest_entry_unbounded_removed_count_allocation() {
    // Construct a Replace payload with removed_count = 0x3FFF_FFFF (1 billion)
    // but the payload itself is very short.
    let added_path = "sst-added.sst";
    let added_bytes = added_path.as_bytes();

    let mut payload = Vec::new();
    payload.push(3u8); // Replace op tag
    payload.extend_from_slice(&100u64.to_le_bytes()); // added_max_tx
    payload.extend_from_slice(&0u64.to_le_bytes()); // rank
    payload.extend_from_slice(&(added_bytes.len() as u32).to_le_bytes()); // added_path_len
    payload.extend_from_slice(added_bytes); // added_path
    payload.extend_from_slice(&0x3FFF_FFFFu32.to_le_bytes()); // removed_count = 1,073,741,823

    let crc = crc32fast::hash(&payload);

    let mut data = Vec::new();
    data.extend_from_slice(&crc.to_le_bytes());
    data.extend_from_slice(&payload);

    // This should fail gracefully with a Serialization error rather than
    // attempting a multi-gigabyte memory allocation or panicking.
    let result = ManifestEntry::from_bytes(&data);
    assert!(
        result.is_err(),
        "Expected deserialization error due to unbounded removed_count"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("removed_count")
            || err_msg.contains("invalid")
            || err_msg.contains("exceeds"),
        "Error message should mention invalid removed_count, got: {}",
        err_msg
    );
}

#[test]
fn test_manifest_entry_trailing_garbage_rejected() {
    let add_entry = ManifestEntry::Add {
        path: std::path::PathBuf::from("sst-1.sst"),
        max_tx: 42,
    };
    let mut valid_bytes = add_entry.to_bytes().unwrap();

    // Appending trailing garbage bytes to the payload section
    valid_bytes.extend_from_slice(b"EXTRA_GARBAGE_BYTES");

    // Recalculate total_payload_size and CRC over the modified payload
    let total_len = (valid_bytes.len() - 4) as u32;
    valid_bytes[0..4].copy_from_slice(&total_len.to_le_bytes());

    let payload = &valid_bytes[8..];
    let new_crc = crc32fast::hash(payload);
    valid_bytes[4..8].copy_from_slice(&new_crc.to_le_bytes());

    // Deserializing [crc: 4][payload + garbage] should be rejected because of unparsed trailing bytes
    let result = ManifestEntry::from_bytes(&valid_bytes[4..]);
    assert!(
        result.is_err(),
        "Expected deserialization failure due to trailing unparsed bytes"
    );
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("trailing") || err_msg.contains("unparsed"),
        "Error message should mention trailing bytes, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_compaction_retention_exact_min_snapshot_seq() {
    let dir = tempdir().unwrap();
    let cache = Arc::new(BlockCache::new(1024 * 1024));
    let snapshot_reg = Arc::new(SnapshotRegistry::new());
    let budget = Arc::new(ResourceTracker::new(ResourceBudget::default()));

    // Create SSTable with key "k1" at seq 100 and seq 50
    let sst1_path = dir.path().join("sst-1.sst");
    let mut b1 = SstableBuilder::create(&sst1_path).await.unwrap();
    b1.add(b"k1", b"v100", 100, 1).await.unwrap();
    b1.add(b"k1", b"v50", 50, 1).await.unwrap();
    b1.finish().await.unwrap();

    let r1 = Arc::new(
        SstableReader::open(&sst1_path, Arc::clone(&cache))
            .await
            .unwrap(),
    );

    let config = CompactionConfig::default();
    let engine =
        CompactionEngine::new(config, snapshot_reg, Arc::clone(&cache), None, budget, None);

    let output_path = dir.path().join("sst-out.sst");

    // Set min_snapshot_seq = 100.
    // Key "k1" at seq 100 is the highest version <= min_snapshot_seq (100).
    // Therefore, seq 100 is the floor version for min_snapshot_seq = 100.
    // Seq 50 is strictly older than min_snapshot_seq and older than the floor version, so it MUST be discarded.
    engine
        .merge_sstables(&[r1], &output_path, 100, true)
        .await
        .unwrap();

    let out_reader = Arc::new(
        SstableReader::open(&output_path, Arc::clone(&cache))
            .await
            .unwrap(),
    );
    let mut stream = out_reader.stream().await.unwrap();

    let mut entries = Vec::new();
    while let Some((k, v, seq, _tx)) = stream.next_entry().await.unwrap() {
        entries.push((k, v, seq & !TOMBSTONE_BIT));
    }

    assert_eq!(
        entries.len(),
        1,
        "Only floor version (seq 100) should be retained for min_snapshot_seq = 100, but got entries: {:?}",
        entries
    );
    assert_eq!(entries[0].2, 100);
}
