#[path = "../src/harness/mutants_diff.rs"]
mod mutants_diff;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_mutants_diff_harness() {
    let temp = TempDir::new().unwrap();
    let root = temp.path();

    let _ = std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output();

    let prompter_dir = root.join(".jules");
    fs::create_dir_all(&prompter_dir).unwrap();

    let prompter_toml = r#"
[crate_overrides.contextra-core]
tier = "1"
"#;
    fs::write(prompter_dir.join("prompter-tiers.toml"), prompter_toml).unwrap();

    let score = mutants_diff::mutants_diff_get_expected_score(root, "contextra-core");
    assert!((score - 0.60).abs() < f64::EPSILON);

    let touched =
        mutants_diff::mutants_diff_get_tier1_touched_crates(root, "HEAD", "HEAD").unwrap();
    assert!(touched.is_empty());

    let args = vec![
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let code = mutants_diff::run_mutants_diff(&args);
    assert_eq!(code, 0); // Not applicable returns 0
}
