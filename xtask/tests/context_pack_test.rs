use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;

static TEST_LOCK: Mutex<()> = Mutex::new(());

pub fn find_root_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("CONTEXTRA_ROOT_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(manifest_dir);
        loop {
            let cargo_path = dir.join("Cargo.toml");
            if cargo_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&cargo_path) {
                    if content.contains("[workspace]") && content.contains("members") {
                        return dir;
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
    }
    PathBuf::from(".")
}

#[path = "../src/claim.rs"]
mod claim;

#[path = "../src/context_pack.rs"]
mod context_pack;

use context_pack::{run_context_pack, scan_critical_tags};

#[test]
fn test_scan_critical_tags_filtering() {
    let _guard = TEST_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let root = dir.path();

    let store_dir = root.join("crates/contextra-store/src");
    let vector_dir = root.join("crates/contextra-vector/src");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&vector_dir).unwrap();

    let tag1 = "// AI-TAG[v1][BLOCKER] Critical memory leak";
    let tag2 = "// AI-TAG[v1][CRITICAL] Disk full check missing";
    let tag3 = "// AI-TAG[v1][BLOCKER][RESOLVED] Already fixed issue";
    let tag4 = "// AI-TAG[v1][INFO] Normal info tag";

    fs::write(store_dir.join("lib.rs"), format!("{}\n{}\n", tag1, tag3)).unwrap();
    fs::write(vector_dir.join("lib.rs"), format!("{}\n{}\n", tag2, tag4)).unwrap();

    // All tags scan
    let all_tags = scan_critical_tags(root, None);
    assert_eq!(all_tags.len(), 2);
    assert!(all_tags.iter().any(|t| t.contains("Critical memory leak")));
    assert!(all_tags
        .iter()
        .any(|t| t.contains("Disk full check missing")));
    assert!(!all_tags.iter().any(|t| t.contains("RESOLVED")));

    // Filter by crate
    let store_tags = scan_critical_tags(root, Some("contextra-store"));
    assert_eq!(store_tags.len(), 1);
    assert!(store_tags[0].contains("Critical memory leak"));
}

#[test]
fn test_run_context_pack_fast_mode() {
    let _guard = TEST_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::env::set_var("CONTEXTRA_ROOT_DIR", root);

    let jules_dir = root.join(".jules");
    fs::create_dir_all(&jules_dir).unwrap();

    let mut db = claim::ClaimsDatabase::default();
    db.claims.push(claim::ClaimEntry {
        krate: "contextra-store".to_string(),
        issue: "TASK-1".to_string(),
        timestamp: "2026-09-27T10:00:00Z".to_string(),
        session_id: "s123".to_string(),
        active: true,
        expires_at: None,
        released_at: None,
    });
    db.save(&jules_dir.join("claims.json")).unwrap();

    let output_file = dir.path().join("out/pack.md");

    // Fast mode: recent_commits is empty
    let pack = run_context_pack(None, true, &output_file).unwrap();
    assert!(pack.recent_commits.is_empty());
    assert_eq!(pack.active_claims.len(), 1);
    assert_eq!(pack.active_claims[0].krate, "contextra-store");

    assert!(output_file.is_file());
    let md_content = fs::read_to_string(&output_file).unwrap();
    assert!(md_content.contains("# Context Pack"));
    assert!(md_content.contains("contextra-store"));

    std::env::remove_var("CONTEXTRA_ROOT_DIR");
}
