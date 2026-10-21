// FILE-CONTEXT
// STAND: 2026-10-22
// ZWECK: Integration tests for RawSweep builder API and positive control in contextra-testkit.
// INVARIANTEN: Zero-Panic, pure std, fail-closed.

use contextra_testkit::deletion_oracle::rawsweep::RawSweep;
use tempfile::tempdir;

#[test]
fn raw_sweep_positive_control_detects_planted_marker() {
    let dir = tempdir().expect("failed to create temp dir");
    let marker = b"PLANTED_CANARY_MARKER_001";

    // Positive control must plant a marker and succeed
    let res = RawSweep::assert_detects_planted(dir.path(), marker);
    assert!(res.is_ok(), "assert_detects_planted failed: {:?}", res);
}

#[test]
fn raw_sweep_detects_marker_across_block_boundary() {
    let dir = tempdir().expect("failed to create temp dir");
    let marker = b"BOUNDARY_MARKER_XYZ123";

    // Create a file where marker spans across 1 MiB chunk boundary
    let file_path = dir.path().join("large_straddled_file.dat");
    let chunk_size = 1_048_576; // 1 MiB
    let offset = chunk_size - (marker.len() / 2);

    let mut data = vec![0u8; chunk_size * 2];
    data[offset..offset + marker.len()].copy_from_slice(marker);

    std::fs::write(&file_path, &data).expect("failed to write test file");

    let report = RawSweep::new(dir.path())
        .with_marker(marker)
        .run()
        .expect("sweep failed");

    assert!(
        !report.hits.is_empty(),
        "marker across block boundary was not detected"
    );
}

#[test]
fn raw_sweep_detects_encoded_variants() {
    let dir = tempdir().expect("failed to create temp dir");
    let raw_marker = b"SECRET_CANARY_STRING";

    // Hex variant of "SECRET_CANARY_STRING"
    // 'S' = 53, 'E' = 45, etc.
    let hex_marker = "5345435245545f43414e4152595f535452494e47".as_bytes();

    let file_path = dir.path().join("encoded_hex_file.txt");
    std::fs::write(&file_path, hex_marker).expect("failed to write hex file");

    let report = RawSweep::new(dir.path())
        .with_marker(raw_marker)
        .run()
        .expect("sweep failed");

    assert!(
        !report.hits.is_empty(),
        "hex encoded variant of marker was not detected by RawSweep"
    );
}
