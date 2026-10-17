use contextra_store::blob::{list_orphans, BlobReader, BlobWriter};
use contextra_store::format::{check_openable, FileKind, FormatError, FormatHeader};
use contextra_store::manifest::{Manifest, ManifestEntry, ManifestFormatTracker};
use contextra_store::sstable::{BlockCache, SstableBuilder, SstableReader};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("compat")
        .join("corpus")
}

/// Generates or validates the deterministic golden corpus.
pub async fn ensure_golden_corpus() {
    let base = corpus_dir();

    // 1. Manifest v1
    let manifest_dir = base.join("manifest").join("v1");
    std::fs::create_dir_all(&manifest_dir).expect("create manifest corpus dir");
    let manifest_file = manifest_dir.join("golden_manifest_v1.manifest");
    if !manifest_file.exists() {
        let manifest = Manifest::open(&manifest_file)
            .await
            .expect("open corpus manifest");
        manifest
            .append(&ManifestEntry::Add {
                path: PathBuf::from("sst-golden-0001.sst"),
                max_tx: 100,
            })
            .await
            .expect("append add");
        manifest
            .append(&ManifestEntry::FormatVersionRecord {
                file_kind: 0,
                max_version: 4,
                migrated_from: 3,
            })
            .await
            .expect("append format version");
    }

    // 2. SSTable v4
    let sstable_dir = base.join("sstable").join("v4");
    std::fs::create_dir_all(&sstable_dir).expect("create sstable corpus dir");
    let sstable_file = sstable_dir.join("golden_sstable_v4.sst");
    if !sstable_file.exists() {
        let mut builder = SstableBuilder::create_with_version(&sstable_file, 4)
            .await
            .expect("create corpus sstable");
        for i in 1..=10 {
            let k = format!("key_{:04}", i);
            let v = format!("val_{:04}", i);
            builder
                .add(k.as_bytes(), v.as_bytes(), i as u64, 1)
                .await
                .expect("add corpus sst key");
        }
        builder.finish().await.expect("finish corpus sst");
    }

    // 3. Blob v1
    let blob_dir = base.join("blob").join("v1");
    std::fs::create_dir_all(&blob_dir).expect("create blob corpus dir");
    let blob_file = blob_dir.join("golden_blob_v1.blob");
    if !blob_file.exists() {
        let key_id = [0x07u8; 32];
        let mut writer = BlobWriter::create(&blob_file, 1, key_id)
            .await
            .expect("create corpus blob");
        let _ptr = writer
            .append(b"golden_corpus_blob_payload_v1_deterministic")
            .await
            .expect("append corpus blob");
        writer.flush_durable().await.expect("flush corpus blob");
    }
}

#[tokio::test]
async fn test_golden_corpus_open_and_verify() {
    ensure_golden_corpus().await;

    let base = corpus_dir();

    // Verify Manifest Corpus v1
    let manifest_file = base
        .join("manifest")
        .join("v1")
        .join("golden_manifest_v1.manifest");
    assert!(manifest_file.exists());
    let entries = Manifest::load(&manifest_file)
        .await
        .expect("load golden manifest");
    assert!(!entries.is_empty());
    let tracker = ManifestFormatTracker::from_entries(&entries);
    assert_eq!(tracker.max_sstable_version, 4);

    // Verify SSTable Corpus v4
    let sstable_file = base
        .join("sstable")
        .join("v4")
        .join("golden_sstable_v4.sst");
    assert!(sstable_file.exists());
    let bc = Arc::new(BlockCache::new(10));
    let reader = SstableReader::open(&sstable_file, bc)
        .await
        .expect("open golden sstable");
    assert_eq!(reader.format_version, 4);
    let val = reader.get(b"key_0001").await.expect("get").expect("exists");
    assert_eq!(val.0.as_ref(), b"val_0001");

    // Verify Blob Corpus v1
    let blob_file = base.join("blob").join("v1").join("golden_blob_v1.blob");
    assert!(blob_file.exists());
    let blob_reader = BlobReader::open(&blob_file)
        .await
        .expect("open golden blob");
    assert_eq!(blob_reader.path(), &blob_file);
}

#[tokio::test]
async fn test_negative_cases_magic_newer_version_downgrade() {
    // 1. Invalid Magic
    let header_bad_magic = FormatHeader::new(*b"BADM", 1);
    let err_magic = check_openable(FileKind::Sstable, &header_bad_magic, 4);
    assert!(matches!(err_magic, Err(FormatError::Corrupt { .. })));

    // 2. Newer Unsupported Version
    let header_newer = FormatHeader::new(*b"MFSX", 99);
    let err_newer = check_openable(FileKind::Sstable, &header_newer, 4);
    assert!(matches!(
        err_newer,
        Err(FormatError::UnsupportedNewerVersion { .. })
    ));

    // 3. Downgrade Attempt
    let tracker = ManifestFormatTracker {
        max_sstable_version: 4,
        ..Default::default()
    };
    let err_downgrade = tracker.verify_file_version(FileKind::Sstable, 2);
    assert!(matches!(
        err_downgrade,
        Err(FormatError::DowngradeAttempt { .. })
    ));
}

#[tokio::test]
async fn test_runtime_downgrade_protection_open_sstable_rejected() {
    let dir = tempdir().expect("tempdir");
    let manifest_path = dir.path().join("MANIFEST");

    // Write MANIFEST recording max SSTable version = 4
    let manifest = Manifest::open(&manifest_path).await.expect("open manifest");
    manifest
        .append(&ManifestEntry::FormatVersionRecord {
            file_kind: 0,
            max_version: 4,
            migrated_from: 0,
        })
        .await
        .expect("append record");

    // Create an SSTable file with version = 3 in the same directory
    let sst_path = dir.path().join("v3_downgrade.sst");
    let mut builder = SstableBuilder::create_with_version(&sst_path, 3)
        .await
        .expect("builder");
    builder.add(b"k1", b"v1", 1, 1).await.expect("add");
    builder.finish().await.expect("finish");

    // Opening SSTable v3 when MANIFEST records max = 4 must fail with DowngradeAttempt
    let bc = Arc::new(BlockCache::new(10));
    let res = SstableReader::open(&sst_path, bc).await;
    assert!(
        res.is_err(),
        "Opening lower version SSTable must be rejected by runtime downgrade protection"
    );
}

#[tokio::test]
async fn test_crash_blob_flushed_manifest_missing_orphan_detected() {
    let dir = tempdir().expect("tempdir");
    let blob_path = dir.path().join("segment_orphan.blob");

    let mut writer = BlobWriter::create(&blob_path, 1, [0u8; 32])
        .await
        .expect("create blob");
    writer.append(b"orphaned blob data").await.expect("append");
    writer.flush_durable().await.expect("flush");

    // Manifest live set does NOT contain segment_orphan.blob
    let live_set: HashSet<PathBuf> = HashSet::new();

    let orphans = list_orphans(dir.path(), &live_set).expect("list_orphans");
    assert_eq!(orphans.len(), 1);
    assert_eq!(orphans[0], blob_path);
}
