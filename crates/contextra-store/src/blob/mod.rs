pub mod orphan;
pub mod pointer;
pub mod reader;
pub mod writer;

pub use orphan::list_orphans;
pub use pointer::BlobPointer;
pub use reader::BlobReader;
pub use writer::{BlobWriter, BLOB_HEADER_SIZE};

/// Default threshold in bytes (64 KiB = 65,536 bytes) to determine whether a payload should be stored as a blob.
pub const DEFAULT_BLOB_THRESHOLD: u64 = 65_536;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_blob_writer_reader_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let blob_path = dir.path().join("segment_0001.blob");

        let key_id = [0x42u8; 32];
        let mut writer = BlobWriter::create(&blob_path, 1, key_id)
            .await
            .expect("create writer");

        let payload1 = b"Hello, world! This is a large blob payload.";
        let payload2 = b"Second payload stored in the same blob segment file.";

        let ptr1 = writer.append(payload1).await.expect("append 1");
        let ptr2 = writer.append(payload2).await.expect("append 2");

        writer.flush_durable().await.expect("flush writer");

        let reader = BlobReader::open(&blob_path).await.expect("open reader");

        let read1 = reader.read(&ptr1).await.expect("read 1");
        assert_eq!(read1, payload1);

        let read2 = reader.read(&ptr2).await.expect("read 2");
        assert_eq!(read2, payload2);
    }

    #[tokio::test]
    async fn test_blob_digest_corruption_returns_error() {
        let dir = tempdir().expect("tempdir");
        let blob_path = dir.path().join("segment_corrupt.blob");

        let mut writer = BlobWriter::create(&blob_path, 1, [0u8; 32])
            .await
            .expect("create writer");

        let payload = b"Sensitive payload that will be corrupted";
        let mut ptr = writer.append(payload).await.expect("append");
        writer.flush_durable().await.expect("flush");

        // Fault injection: corrupt digest in pointer
        ptr.blake3[0] ^= 0xFF;

        let reader = BlobReader::open(&blob_path).await.expect("open reader");
        let read_res = reader.read(&ptr).await;

        assert!(
            read_res.is_err(),
            "Digest mismatch must return checksum error"
        );
    }

    #[test]
    fn test_blob_orphan_detection() {
        let dir = tempdir().expect("tempdir");
        let live_blob = dir.path().join("live.blob");
        let orphan_blob = dir.path().join("orphan.blob");

        std::fs::write(&live_blob, b"live").expect("write live");
        std::fs::write(&orphan_blob, b"orphan").expect("write orphan");

        let mut live_set = HashSet::new();
        live_set.insert(PathBuf::from("live.blob"));

        let orphans = list_orphans(dir.path(), &live_set).expect("list_orphans");
        assert_eq!(orphans.len(), 1);
        assert_eq!(orphans[0], orphan_blob);
    }
}
