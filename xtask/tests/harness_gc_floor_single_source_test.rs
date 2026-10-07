#[path = "../src/harness/gc_floor_single_source.rs"]
mod gc_floor_single_source;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_gc_floor_single_source_detects_unauthorized_call() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let mvcc_dir = root.join("crates/contextra-mvcc/src");
    let store_dir = root.join("crates/contextra-store/src");
    fs::create_dir_all(&mvcc_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();

    // Legitimate call in floor.rs (should be ignored)
    fs::write(
        mvcc_dir.join("floor.rs"),
        r#"
fn calc(reg: &SnapshotRegistry) {
    let min = reg.min_active_seqno();
}
"#,
    )
    .unwrap();

    // Unauthorized call in store/src/bad.rs (should be flagged)
    fs::write(
        store_dir.join("bad.rs"),
        r#"
fn bad(reg: &SnapshotRegistry) {
    let min = reg.min_active_seqno();
}
"#,
    )
    .unwrap();

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let code = gc_floor_single_source::run_gc_floor_single_source(&args);
    assert_eq!(code, 1);
}

#[test]
fn test_gc_floor_single_source_baselined_call_passes() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let store_dir = root.join("crates/contextra-store/src");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(root.join("governance")).unwrap();

    fs::write(
        store_dir.join("bad.rs"),
        r#"
fn bad(reg: &SnapshotRegistry) {
    let min = reg.min_active_seqno();
}
"#,
    )
    .unwrap();

    let baseline = r#"[occurrences]
"crates/contextra-store/src/bad.rs:3:let min = reg.min_active_seqno();" = true
"#;
    fs::write(root.join("governance/gc-floor-single-source-baseline.toml"), baseline).unwrap();

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let code = gc_floor_single_source::run_gc_floor_single_source(&args);
    assert_eq!(code, 0);
}
