// Test for harness module loop-guard

#[path = "../src/harness/loop_guard.rs"]
mod loop_guard;

use std::fs;

fn setup_temp_root() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp root dir")
}

#[test]
fn test_loop_guard_normalization_and_hashing() {
    let err1 = "error at /home/jules/repo/crates/contextra-core/src/lib.rs:142:15 on 2026-09-28T12:00:00Z with ptr 0x7fff1234abcd [1/8]";
    let err2 = "error at /tmp/workspace/crates/contextra-core/src/lib.rs:99:1 on 2026-09-29T14:30:00Z with ptr 0x12345678 [2/8]";

    let norm1 = loop_guard::loop_guard_normalize_error(err1);
    let norm2 = loop_guard::loop_guard_normalize_error(err2);

    assert_eq!(norm1, norm2);

    let hash1 = loop_guard::loop_guard_hash_error(&norm1);
    let hash2 = loop_guard::loop_guard_hash_error(&norm2);

    assert_eq!(hash1, hash2);
}

#[test]
fn test_loop_guard_record_repetition_and_check() {
    let root_dir = setup_temp_root();
    let root = root_dir.path();

    // First record
    let log1 = root.join("error1.log");
    fs::write(&log1, "compile error at /app/src/lib.rs:10:5 ptr 0xabc").unwrap();

    let (hash1, count1) =
        loop_guard::loop_guard_record(root, Some("clippy"), Some(&log1), None).unwrap();
    assert_eq!(count1, 1);
    assert!(!hash1.is_empty());

    // Check should pass on 1st error
    assert_eq!(loop_guard::loop_guard_check(root, 6), Ok(()));
    let check_code1 = loop_guard::run_loop_guard(&[
        "check".to_string(),
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
    ]);
    assert_eq!(check_code1, 0);

    // Second record with same normalized error
    let log2 = root.join("error2.log");
    fs::write(
        &log2,
        "compile error at /tmp/other/src/lib.rs:99:1 ptr 0xdef",
    )
    .unwrap();

    let (hash2, count2) =
        loop_guard::loop_guard_record(root, Some("clippy"), Some(&log2), None).unwrap();
    assert_eq!(hash1, hash2);
    assert_eq!(count2, 2);

    // Check should fail on 2nd error repetition
    assert!(loop_guard::loop_guard_check(root, 6).is_err());
    let check_code2 = loop_guard::run_loop_guard(&[
        "check".to_string(),
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
    ]);
    assert_eq!(check_code2, 1);
}

#[test]
fn test_loop_guard_file_edits_and_max_limit() {
    let root_dir = setup_temp_root();
    let root = root_dir.path();

    let file_path = "crates/contextra-core/src/lib.rs";

    // Record 3 edits
    for _ in 0..3 {
        loop_guard::loop_guard_record(root, None, None, Some(file_path)).unwrap();
    }

    assert_eq!(loop_guard::loop_guard_check(root, 5), Ok(()));

    // Record 3 more edits -> total 6 edits
    for _ in 0..3 {
        loop_guard::loop_guard_record(root, None, None, Some(file_path)).unwrap();
    }

    // With max 5 edits, check must fail
    assert!(loop_guard::loop_guard_check(root, 5).is_err());
}

#[test]
fn test_loop_guard_stop_and_escalate_report() {
    let root_dir = setup_temp_root();
    let root = root_dir.path();

    let log_file = root.join("sample.log");
    let mut lines = Vec::new();
    for i in 1..=50 {
        lines.push(format!("log line {}", i));
    }
    fs::write(&log_file, lines.join("\n")).unwrap();

    loop_guard::loop_guard_record(root, Some("test"), Some(&log_file), Some("src/main.rs"))
        .unwrap();

    let stop_code = loop_guard::run_loop_guard(&[
        "stop".to_string(),
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
        "--reason".to_string(),
        "Repeated clippy failure after 2 attempts".to_string(),
    ]);
    assert_eq!(stop_code, 1);

    let escalate_md = root.join(".jules/local/ESCALATE.md");
    assert!(escalate_md.exists());

    let content = fs::read_to_string(&escalate_md).unwrap();
    assert!(content.contains("Repeated clippy failure after 2 attempts"));
    assert!(content.contains("src/main.rs"));
    assert!(content.contains("log line 50"));
    assert!(!content.contains("log line 5\n")); // First lines should be truncated to max 40 lines
}

#[test]
fn test_loop_guard_reset() {
    let root_dir = setup_temp_root();
    let root = root_dir.path();

    let log_file = root.join("err.log");
    fs::write(&log_file, "error 1").unwrap();
    loop_guard::loop_guard_record(root, None, Some(&log_file), None).unwrap();

    let state_file = loop_guard::loop_guard_get_state_path(root);
    assert!(state_file.exists());

    let reset_code = loop_guard::run_loop_guard(&[
        "reset".to_string(),
        "--root".to_string(),
        root.to_str().unwrap().to_string(),
    ]);
    assert_eq!(reset_code, 0);
    assert!(!state_file.exists());
}
