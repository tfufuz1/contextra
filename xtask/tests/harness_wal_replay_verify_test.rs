#[path = "../src/harness/wal_replay_verify.rs"]
mod wal_replay_verify;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_wal_replay_verify_harness() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output();

    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    let mock_cargo = r#"#!/bin/sh
if [ "$1" = "test" ] && [ "$5" = "--list" ]; then
    if [ "$4" = "unknown_test" ]; then
        exit 1
    else
        echo "test_item: test"
        exit 0
    fi
fi
if [ "$1" = "test" ]; then
    exit 0
fi
exit 0
"#;
    let mock_path = bin_dir.join("cargo");
    fs::write(&mock_path, mock_cargo).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&mock_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&mock_path, perms).unwrap();
    }

    let orig_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), orig_path);
    std::env::set_var("PATH", new_path);

    let is_app = wal_replay_verify::wal_replay_is_applicable(root, "HEAD", "HEAD");
    assert!(!is_app);

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--base".to_string(),
        "HEAD".to_string(),
        "--head".to_string(),
        "HEAD".to_string(),
        "--json".to_string(),
        "--tests".to_string(),
        "unknown_test".to_string(),
    ];

    let code = wal_replay_verify::run_wal_replay_verify(&args);
    assert_eq!(code, 0); // Not applicable returns 0
}
