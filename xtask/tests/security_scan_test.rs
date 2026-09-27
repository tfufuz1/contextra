//! Tests for security_scan module.

#[path = "../src/security_scan.rs"]
mod security_scan;

use security_scan::{run_security_scan_in_root, SecurityPattern};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_security_scan_synthetic_fixture() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();

    let crates_dir = root.join("crates").join("security-test-crate").join("src");
    fs::create_dir_all(&crates_dir).unwrap();

    let lib_rs_content = r#"
use std::process::Command;

pub fn shell_bad() {
    let _ = Command::new("sh").arg("-c").arg("echo hi"); // ShellInterpolation
}

pub async fn async_fs_bad() {
    let _ = std::fs::read("file.txt"); // StdFsInAsync without spawn_blocking
}

pub async fn async_fs_good() {
    tokio::task::spawn_blocking(|| {
        let _ = std::fs::read("file.txt"); // StdFsInAsync with spawn_blocking
    }).await.unwrap();
}

pub fn secret_bad() {
    let my_api_key = "super_secret_token_123456"; // HardcodedSecret (> 8 chars)
    let dummy_key = "changeme"; // Ignored placeholder
}
"#;

    fs::write(crates_dir.join("lib.rs"), lib_rs_content).unwrap();

    let findings = run_security_scan_in_root(root, None, None).unwrap();

    assert_eq!(findings.len(), 3);

    let shell_finding = findings
        .iter()
        .find(|f| f.pattern == SecurityPattern::ShellInterpolation)
        .unwrap();
    assert_eq!(shell_finding.line, 5);

    let async_finding = findings
        .iter()
        .find(|f| f.pattern == SecurityPattern::StdFsInAsync)
        .unwrap();
    assert_eq!(async_finding.line, 9);

    let secret_finding = findings
        .iter()
        .find(|f| f.pattern == SecurityPattern::HardcodedSecret)
        .unwrap();
    assert_eq!(secret_finding.line, 20);
}
