#![allow(clippy::unwrap_used, clippy::expect_used)]

#[path = "support/canary.rs"]
mod canary;

use canary::CanaryOracle;
use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use tempfile::TempDir;

#[test]
fn canary_oracle_negative_test_finds_existing_markers() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let oracle = CanaryOracle::new();

    let marker_a = b"CANARY_MARKER_9999_EXISTING_DATA_001";
    let marker_wal = b"CANARY_WAL_ENTRY_8888_ACTIVE_LOG_002";
    let marker_sparse = b"CANARY_SPARSE_7777_OFFSET_REGION_003";

    // 1. Regular database payload file in subfolder
    let db_sub_dir = temp_dir.path().join("sstables");
    fs::create_dir_all(&db_sub_dir).expect("create sstable dir");
    let sstable_file = db_sub_dir.join("000001.sst");
    let mut file = File::create(&sstable_file).expect("create sstable file");
    file.write_all(b"HEADER_DUMMY_DATA_")
        .expect("write header");
    file.write_all(marker_a).expect("write marker a");
    file.write_all(b"_FOOTER_DUMMY_DATA").expect("write footer");
    file.flush().expect("flush sstable");

    // 2. WAL log file
    let wal_dir = temp_dir.path().join("wal");
    fs::create_dir_all(&wal_dir).expect("create wal dir");
    let wal_file = wal_dir.join("000001.wal");
    let mut file_wal = File::create(&wal_file).expect("create wal file");
    file_wal
        .write_all(b"WAL_HEADER_SEQ_100_")
        .expect("write wal header");
    file_wal.write_all(marker_wal).expect("write wal marker");
    file_wal
        .write_all(b"_WAL_RECORD_END")
        .expect("write wal record end");
    file_wal.flush().expect("flush wal");

    // 3. Sparse file with large zero region
    let sparse_file = temp_dir.path().join("sparse_001.dat");
    let mut file_sparse = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&sparse_file)
        .expect("open sparse file");
    file_sparse
        .seek(SeekFrom::Start(1024 * 1024))
        .expect("seek offset"); // 1 MiB sparse gap
    file_sparse
        .write_all(marker_sparse)
        .expect("write sparse marker");
    file_sparse.flush().expect("flush sparse");

    // NEGATIVTEST: All existing, non-deleted markers MUST be found by CanaryOracle
    assert_eq!(
        oracle
            .scan_file(&sstable_file, marker_a)
            .expect("scan sstable file"),
        1,
        "Negativtest failed: sstable marker was not found!"
    );

    assert_eq!(
        oracle
            .scan_file(&wal_file, marker_wal)
            .expect("scan wal file"),
        1,
        "Negativtest failed: WAL marker was not found!"
    );

    assert_eq!(
        oracle
            .scan_file(&sparse_file, marker_sparse)
            .expect("scan sparse file"),
        1,
        "Negativtest failed: Sparse region marker was not found!"
    );

    // Directory-wide recursive verification
    assert!(
        oracle
            .contains_marker(temp_dir.path(), marker_a)
            .expect("scan dir for marker a"),
        "Negativtest failed: Marker A not found in directory!"
    );
    assert!(
        oracle
            .contains_marker(temp_dir.path(), marker_wal)
            .expect("scan dir for marker wal"),
        "Negativtest failed: WAL marker not found in directory!"
    );
    assert!(
        oracle
            .contains_marker(temp_dir.path(), marker_sparse)
            .expect("scan dir for marker sparse"),
        "Negativtest failed: Sparse marker not found in directory!"
    );
}

#[test]
fn canary_oracle_positive_test_confirms_complete_erasure() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let oracle = CanaryOracle::new();

    let marker_deleted = b"CANARY_FORGOTTEN_MARKER_5555_TO_BE_SHREDDED";
    let marker_retained = b"CANARY_RETAINED_MARKER_6666_STAY_IN_STORAGE";

    let data_dir = temp_dir.path().join("data");
    fs::create_dir_all(&data_dir).expect("create data dir");

    let f1_path = data_dir.join("block_01.dat");
    let mut f1 = File::create(&f1_path).expect("create block 01");
    f1.write_all(marker_deleted)
        .expect("write marker to block 01");
    f1.flush().expect("flush f1");

    let f2_path = data_dir.join("block_02.dat");
    let mut f2 = File::create(&f2_path).expect("create block 02");
    f2.write_all(marker_retained)
        .expect("write retained marker to block 02");
    f2.flush().expect("flush f2");

    // Precondition verification: both markers exist
    assert!(
        oracle
            .contains_marker(temp_dir.path(), marker_deleted)
            .expect("precondition scan deleted"),
        "Precondition failed: deleted marker not present before forget"
    );
    assert!(
        oracle
            .contains_marker(temp_dir.path(), marker_retained)
            .expect("precondition scan retained"),
        "Precondition failed: retained marker not present before forget"
    );

    // Perform 'forget' simulation on marker_deleted (zeroing/shredding raw file bytes & removing file)
    {
        let mut f1_shred = OpenOptions::new()
            .write(true)
            .open(&f1_path)
            .expect("open for shredding");
        f1_shred
            .write_all(&vec![0u8; marker_deleted.len()])
            .expect("zero out marker");
        f1_shred.flush().expect("flush zeroed file");
    }
    fs::remove_file(&f1_path).expect("remove forgotten file");

    // POSITIVTEST: Forgotten marker MUST NOT be found anywhere in directory or raw disk image
    let deleted_found = oracle
        .contains_marker(temp_dir.path(), marker_deleted)
        .expect("post-forget scan deleted");
    assert!(
        !deleted_found,
        "Positivtest failed: Residual forgotten canary marker found after forget operation!"
    );

    // Retained marker MUST still be found
    let retained_found = oracle
        .contains_marker(temp_dir.path(), marker_retained)
        .expect("post-forget scan retained");
    assert!(
        retained_found,
        "Positivtest verification failed: Retained marker missing after unrelated forget!"
    );
}

#[test]
fn canary_oracle_straddle_chunk_boundary_test() {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let oracle = CanaryOracle::new();

    let marker = b"STRADDLE_CANARY_MARKER_1234567890";
    let chunk_size = 64 * 1024; // 64 KiB matching oracle chunk size

    let straddle_file = temp_dir.path().join("boundary_straddle.dat");
    let mut file = File::create(&straddle_file).expect("create straddle file");

    // Write chunk_size - 10 padding bytes so the marker straddles the 64 KiB chunk boundary
    let padding = vec![0x41u8; chunk_size - 10];
    file.write_all(&padding).expect("write padding");
    file.write_all(marker).expect("write straddled marker");
    file.write_all(&padding).expect("write trailing padding");
    file.flush().expect("flush straddle file");

    let count = oracle
        .scan_file(&straddle_file, marker)
        .expect("scan straddle file");
    assert_eq!(
        count, 1,
        "Boundary straddling canary marker was not detected across chunk boundary!"
    );
}
