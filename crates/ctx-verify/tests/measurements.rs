//! Throughput and memory verification for ctx-verify scan and report validation.
//!
//! SPEC: T-SCAN, T-REPORT, D-72

use ctx_verify::scan::execute_field_scan;
use std::fs::{self, File};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::time::Instant;
use tempfile::tempdir;

#[allow(clippy::disallowed_methods)]
#[test]
fn bench_field_scan_throughput() {
    let dir = tempdir().unwrap();
    let data_dir = dir.path().join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Create 10 files with 1 MiB clean data each (total 10 MiB)
    let chunk = vec![0x42u8; 1024 * 1024];
    for i in 0..10 {
        fs::write(data_dir.join(format!("segment_{:03}.dat", i)), &chunk).unwrap();
    }

    let mut pass_file = tempfile::NamedTempFile::new().unwrap();
    pass_file.write_all(b"bench_passphrase").unwrap();
    pass_file.flush().unwrap();
    let file_handle = File::open(pass_file.path()).unwrap();
    let fd = file_handle.as_raw_fd();

    let target_hex = "0102030405060708090a0b0c0d0e0f10";

    // Warm-up
    let _ = execute_field_scan(&data_dir, target_hex, fd);

    let start = Instant::now();
    let runs = 5;
    for _ in 0..runs {
        let res = execute_field_scan(&data_dir, target_hex, fd);
        assert_eq!(res, Ok(()));
    }
    let elapsed = start.elapsed();
    let total_bytes = (10 * 1024 * 1024 * runs) as f64;
    let mb_per_sec = (total_bytes / (1024.0 * 1024.0)) / elapsed.as_secs_f64();

    println!(
        "SCAN THROUGHPUT: {:.2} MB/s across {} runs ({} MiB total)",
        mb_per_sec,
        runs,
        10 * runs
    );
    assert!(mb_per_sec > 10.0, "Throughput must exceed 10 MB/s");
}
