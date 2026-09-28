#[path = "../src/harness/fuzz_smoke.rs"]
mod fuzz_smoke;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_fuzz_smoke_harness() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output();

    let fuzz_dir = root.join("crates/contextra-store/fuzz/fuzz_targets");
    fs::create_dir_all(&fuzz_dir).unwrap();
    fs::write(fuzz_dir.join("target1.rs"), "fn main() {}\n").unwrap();

    let touched = fuzz_smoke::fuzz_smoke_get_touched_fuzz_crates(root, "HEAD", "HEAD").unwrap();
    assert!(touched.is_empty());

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let code = fuzz_smoke::run_fuzz_smoke(&args);
    assert_eq!(code, 0); // Not applicable returns 0
}
