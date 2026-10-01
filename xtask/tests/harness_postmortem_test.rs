// xtask/tests/harness_postmortem_test.rs

#[path = "../src/harness/postmortem.rs"]
mod postmortem;

fn get_repo_root() -> std::path::PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return std::path::PathBuf::from(path_str);
            }
        }
    }
    std::path::PathBuf::from("..")
}

#[test]
fn test_postmortem_check() {
    let root = get_repo_root();
    let root_str = root.to_string_lossy();
    let args = vec![
        "postmortem".to_string(),
        "check".to_string(),
        "--root".to_string(),
        root_str.to_string(),
        "--json".to_string(),
    ];
    let exit = postmortem::run_postmortem(&args);
    assert_eq!(exit, 0);
}

#[test]
fn test_postmortem_new() {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    std::fs::create_dir_all(root.join("docs/postmortems")).unwrap();
    std::fs::write(
        root.join("docs/postmortems/_TEMPLATE.md"),
        "# Postmortem TEMPLATE: [PM-NNNN] [Titel]\n\n## Vorfall\n\n## Zeitachse\n\n## Ursache (5 Whys)\n\n## Wirkung\n\n## Neue Regel\n\n## Neuer Test/Gate\n- **Pfad**: `xtask/src/check_commit_diff_integrity.rs` \n\n## ADR-Link\n- `docs/decisions/ADR-035-governance-system-haertung-prozessregeln.md` \n\n## Verifikation\n"
    ).unwrap();

    let args = vec![
        "postmortem".to_string(),
        "new".to_string(),
        "--title".to_string(),
        "Test Incident".to_string(),
        "--root".to_string(),
        root.to_string_lossy().to_string(),
        "--json".to_string(),
    ];

    let exit = postmortem::run_postmortem(&args);
    assert_eq!(exit, 0);
    assert!(root.join("docs/postmortems/PM-0001-test-incident.md").exists());
}
