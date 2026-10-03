use std::fs;
use std::path::PathBuf;

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

#[test]
fn test_working_state_has_no_volatile_fields() {
    let root = find_root_dir();
    let ws_path = root.join("WORKING_STATE.md");
    let content = fs::read_to_string(&ws_path).unwrap_or_default();

    assert!(
        !content.contains("- Letzter Commit:"),
        "WORKING_STATE.md still contains volatile field 'Letzter Commit'"
    );
    assert!(
        !content.contains("- Letzter Merge:"),
        "WORKING_STATE.md still contains volatile field 'Letzter Merge'"
    );
    assert!(
        !content.contains("- Aktive Claims:"),
        "WORKING_STATE.md still contains volatile field 'Aktive Claims'"
    );
    assert!(
        !content.contains("- Prompter-Data-Abstand:"),
        "WORKING_STATE.md still contains volatile field 'Prompter-Data-Abstand'"
    );
}

#[path = "../src/claim.rs"]
mod claim;

#[path = "../src/session_init.rs"]
mod session_init;

use session_init::run_session_init;

#[test]
fn test_session_init_has_continuity_fields() {
    let res = run_session_init(None, None, false).expect("session_init should succeed");

    assert!(
        !res.last_commit.is_empty(),
        "SessionInitResult should contain last_commit"
    );
    assert!(
        !res.last_merge.is_empty(),
        "SessionInitResult should contain last_merge"
    );
    assert!(
        !res.prompter_distance.is_empty(),
        "SessionInitResult should contain prompter_distance"
    );
}
