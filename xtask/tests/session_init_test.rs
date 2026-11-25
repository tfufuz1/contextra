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

#[path = "../src/session_init.rs"]
mod session_init;

use session_init::{generate_session_hash, run_session_init};

#[test]
fn test_generate_session_hash_length() {
    let now = "2026-09-27T12:00:00Z";
    let hash = generate_session_hash(now);
    assert_eq!(hash.len(), 8);
    // Hash is deterministic for same timestamp
    assert_eq!(hash, generate_session_hash(now));
}

#[test]
fn test_run_session_init_flow() {
    let _guard = TEST_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let root = dir.path();
    std::env::set_var("CONTEXTRA_ROOT_DIR", root);

    let store_dir = root.join("crates/contextra-store/src");
    fs::create_dir_all(&store_dir).unwrap();

    let blocker_tag = "// AI-TAG[v1][BLOCKER] Test blocker tag";
    fs::write(store_dir.join("lib.rs"), blocker_tag).unwrap();

    let res = run_session_init(Some("contextra-store"), Some("Refactor store WAL"), true).unwrap();

    assert_eq!(res.session_hash.len(), 8);
    assert_eq!(res.open_blockers, 1);
    assert_eq!(res.active_claim_count, 1);
    assert_eq!(res.claimed_crate, Some("contextra-store".to_string()));

    // Verify session.env created
    let env_path = root.join(".jules/session.env");
    assert!(env_path.is_file());
    let env_content = fs::read_to_string(&env_path).unwrap();
    assert!(env_content.contains(&format!("export SESSION_HASH={}", res.session_hash)));
    assert!(env_content.contains("export CONTEXTRA_CLAIM_CRATE=contextra-store"));

    // Verify SESSION.md snapshot created
    let session_md = root.join(".jules/SESSION.md");
    assert!(session_md.is_file());
    let md_content = fs::read_to_string(&session_md).unwrap();
    assert!(md_content.contains("contextra-store"));
    assert!(md_content.contains("Refactor store WAL"));

    std::env::remove_var("CONTEXTRA_ROOT_DIR");
}
