//! # RealFs Konformitätstest

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_storage::env::RealFs;
use contextra_storage::testing::conformance::run_fs_conformance_suite;
use std::fs;

#[test]
fn test_real_fs_conformance() {
    let temp_dir = std::env::temp_dir().join(format!("contextra_fs_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    let real_fs = RealFs::new();
    run_fs_conformance_suite(&real_fs, &temp_dir);

    let _ = fs::remove_dir_all(&temp_dir);
}
