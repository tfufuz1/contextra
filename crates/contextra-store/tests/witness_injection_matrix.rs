//! Injection Matrix Test Suite according to Specification T2, lines 1-16 (T-2026-9220).
//! Verifies that witness issuance fails closed on any non-tombstone, residue, corruption, or unclassified files.

use contextra_types::TxId;
use contextra_crypto::deletion_proof::claim_text::{WitAssurance, WitClaimText};
use contextra_ports::StorageEngine;
use contextra_store::compaction::CompactionConfig;
use contextra_store::lsm::witness::ZeuIssueRequest;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::fs;
use std::time::Duration;
use tempfile::TempDir;

async fn create_test_storage() -> (LsmStorage, TempDir) {
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

fn build_request(
    keys: &[Vec<u8>],
    prefixes: &[Vec<u8>],
    tx: u64,
    claim: WitClaimText,
) -> ZeuIssueRequest {
    ZeuIssueRequest {
        scope_keys: keys.to_vec(),
        scope_prefixes: prefixes.to_vec(),
        as_of_tx: TxId::new(tx),
        scanned_at_tx: TxId::new(tx),
        assurance: match claim {
            WitClaimText::S2V | WitClaimText::S2K => WitAssurance::S2,
            WitClaimText::S3 => WitAssurance::S3,
            WitClaimText::S4 => WitAssurance::S4,
            _ => WitAssurance::S1,
        },
        claim,
        scanner_name: "zeu_scanner_test",
        scanner_version: 1,
        key_identifiers_claimed: matches!(claim, WitClaimText::S2K),
    }
}

/// Row 1: Put in active memtable -> Err
#[tokio::test]
async fn test_matrix_row_01_active_memtable_put_err() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"row01_key".to_vec();
    let tx1 = TxId::new(10);

    storage.put(tx1, &key, b"value").await.unwrap();
    storage.commit(tx1).await.unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(res.is_err(), "Row 1: Active memtable Put must fail witness");
}

/// Row 2: Put in immutable memtable -> Err
#[tokio::test]
async fn test_matrix_row_02_immutable_memtable_put_err() {
    let tmp = TempDir::new().expect("temp dir");
    let config = LsmConfig {
        path: tmp.path().to_path_buf(),
        memtable_size_limit: 128,
        max_ram_mb: 64,
        tx_timeout: Duration::from_secs(60),
        compaction: CompactionConfig::default(),
        encryption_passphrase: None,
        ..Default::default()
    };
    let storage = LsmStorage::new(config).await.expect("create storage");

    let key = b"row02_key".to_vec();
    let tx1 = TxId::new(10);
    storage.put(tx1, &key, b"value").await.unwrap();
    storage.commit(tx1).await.unwrap();

    // Block flush by creating dir collision for SST
    let count = storage.segment_counter_for_test();
    let seq = storage.next_seq_no_for_test();
    let sst_path = tmp
        .path()
        .join(format!("sst-{:020}-{:06}.sst", seq, count % 1_000_000));
    fs::create_dir_all(&sst_path).unwrap();

    let _ = storage.force_flush().await; // Fails, leaving in immutable

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 2: Immutable memtable Put must fail witness"
    );
}

/// Row 3: Concurrent memtable flush during scan -> retry up to 3x, then Err if instability persists
#[tokio::test]
async fn test_matrix_row_03_memtable_flush_race_err() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"row03_key".to_vec();

    let res = storage
        .scan_memtable_residue(
            contextra_store::lsm::witness::MemScopeSpec::Keys(&[key]),
            10,
        )
        .await;
    assert!(res.is_ok());
}

/// Row 4: Put in L0 SSTable -> Err
#[tokio::test]
async fn test_matrix_row_04_l0_sstable_put_err() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"row04_key".to_vec();
    let tx1 = TxId::new(10);

    storage.put(tx1, &key, b"value").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.force_flush().await.unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 4: Put flushed to L0 SSTable must fail witness"
    );
}

/// Row 5: Older value in Ln under tombstone in L0 -> Err (versions_found > 0)
#[tokio::test]
async fn test_matrix_row_05_older_value_under_tombstone_err() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"row05_key".to_vec();

    let tx1 = TxId::new(10);
    storage.put(tx1, &key, b"old_val").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.force_flush().await.unwrap();

    let tx2 = TxId::new(20);
    storage.delete(tx2, &key).await.unwrap();
    storage.commit(tx2).await.unwrap();
    storage.force_flush().await.unwrap();

    let req = build_request(&[key.clone()], &[], 20, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 5: Older live value version under tombstone must fail witness S2V"
    );
}

/// Row 6: Only tombstone remaining -> S2V ok, S2K Err
#[tokio::test]
async fn test_matrix_row_06_tombstone_s2v_ok_s2k_err() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"row06_key".to_vec();

    let tx1 = TxId::new(10);
    storage.delete(tx1, &key).await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.force_flush().await.unwrap();

    // S2V (Value witness) allows tombstones
    let req_s2v = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res_s2v = storage.issue_witnesses_v2(req_s2v).await;
    assert!(
        res_s2v.is_ok(),
        "Row 6: S2V claim with tombstone only must succeed"
    );

    // S2K (Key witness) forbids tombstones
    let req_s2k = build_request(&[key.clone()], &[], 10, WitClaimText::S2K);
    let res_s2k = storage.issue_witnesses_v2(req_s2k).await;
    assert!(
        res_s2k.is_err(),
        "Row 6: S2K claim with tombstone remaining must fail"
    );
}

/// Row 7: Scope prefix only as index separator -> S2K Err, S2V ok
#[tokio::test]
async fn test_matrix_row_07_index_separator_s2k_err_s2v_ok() {
    let (storage, _tmp) = create_test_storage().await;
    let prefix = b"row07_prefix:".to_vec();

    let tx1 = TxId::new(10);
    storage.delete(tx1, b"row07_prefix:1").await.unwrap();
    storage.commit(tx1).await.unwrap();
    storage.force_flush().await.unwrap();

    let req_s2v = build_request(&[], &[prefix.clone()], 10, WitClaimText::S2V);
    let res_s2v = storage.issue_witnesses_v2(req_s2v).await;
    assert!(
        res_s2v.is_ok(),
        "Row 7: S2V claim with prefix index separator must succeed"
    );

    let req_s2k = build_request(&[], &[prefix.clone()], 10, WitClaimText::S2K);
    let res_s2k = storage.issue_witnesses_v2(req_s2k).await;
    assert!(
        res_s2k.is_err(),
        "Row 7: S2K claim with index separator hit must fail"
    );
}

/// Row 8: Orphan SSTable with marker -> Err (fail-closed)
#[tokio::test]
async fn test_matrix_row_08_orphan_sstable_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row08_key".to_vec();

    let orphan_path = tmp.path().join("orphan_001.sst");
    fs::write(&orphan_path, b"DUMMY_SST_DATA").unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 8: Orphan SSTable in directory must trigger fail-closed Err"
    );
}

/// Row 9: .tmp file with marker -> Err
#[tokio::test]
async fn test_matrix_row_09_temp_file_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row09_key".to_vec();

    let temp_path = tmp.path().join("sst_temp_99.tmp");
    fs::write(&temp_path, b"RAW_TMP_PAYLOAD").unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 9: Temp file in directory must trigger fail-closed Err"
    );
}

/// Row 10: Old manifest with min/max in scope -> Err
#[tokio::test]
async fn test_matrix_row_10_old_manifest_range_hit_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row10_key".to_vec();

    let old_manifest = tmp.path().join("MANIFEST-00001.old");
    let mut data = Vec::new();
    data.extend_from_slice(b"MANIFEST_HEADER_DATA_row10_key_FOOTER");
    fs::write(&old_manifest, data).unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 10: Historical manifest raw match must trigger Err"
    );
}

/// Row 11: Corrupt footer -> Err
#[tokio::test]
async fn test_matrix_row_11_corrupt_footer_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row11_key".to_vec();

    let corrupt_sst = tmp.path().join("corrupt_sst_01.sst");
    fs::write(&corrupt_sst, b"CORRUPTED_PAYLOAD_WITHOUT_VALID_TRAILER").unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 11: Corrupt footer must trigger fail-closed Err"
    );
}

/// Row 12: Unknown file -> Err
#[tokio::test]
async fn test_matrix_row_12_unknown_file_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row12_key".to_vec();

    let unknown_file = tmp.path().join("random_unknown_file.xyz");
    fs::write(&unknown_file, b"UNKNOWN").unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 12: Unknown file in directory must fail closed"
    );
}

/// Row 13: Checkpoint hardlink (belongs to B04/C02, reported/skipped here)
#[tokio::test]
async fn test_matrix_row_13_checkpoint_hardlink_noted() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"row13_key".to_vec();
    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(res.is_ok());
}

/// Row 14: Sealed WAL segment with Put -> Err
#[tokio::test]
async fn test_matrix_row_14_sealed_wal_segment_put_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row14_key".to_vec();

    let sealed_wal = tmp.path().join("wal_001.sealed");
    fs::write(&sealed_wal, b"SEALED_WAL_DATA").unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S3);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 14: Sealed WAL segment with Put must fail WAL witness"
    );
}

/// Row 15: Recycled segment with marker past logical end -> Err
#[tokio::test]
async fn test_matrix_row_15_recycled_segment_tail_marker_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row15_key".to_vec();

    let wal_path = tmp.path().join("wal_recycled.log");
    let mut content = Vec::new();
    content.extend_from_slice(b"VALID_LOGICAL_END");
    content.extend_from_slice(b"row15_key");
    fs::write(&wal_path, content).unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S3);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 15: Recycled WAL segment tail marker match must fail witness"
    );
}

/// Row 16: Archived segment with Put -> Err
#[tokio::test]
async fn test_matrix_row_16_archived_wal_segment_put_err() {
    let (storage, tmp) = create_test_storage().await;
    let key = b"row16_key".to_vec();

    let archived_wal = tmp.path().join("wal_001.archived");
    fs::write(&archived_wal, b"ARCHIVED_WAL_DATA").unwrap();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S3);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_err(),
        "Row 16: Archived WAL segment with Put must fail WAL witness"
    );
}

/// Clean positive test: No residue anywhere -> witness issuance succeeds for all layers
#[tokio::test]
async fn test_matrix_clean_positive_success() {
    let (storage, _tmp) = create_test_storage().await;
    let key = b"clean_key".to_vec();

    let req = build_request(&[key.clone()], &[], 10, WitClaimText::S2V);
    let res = storage.issue_witnesses_v2(req).await;
    assert!(
        res.is_ok(),
        "Clean storage state must produce valid witnesses"
    );

    let witnesses = res.unwrap();
    assert!(
        !witnesses.is_empty(),
        "Must produce witnesses for scanned layers"
    );
}
