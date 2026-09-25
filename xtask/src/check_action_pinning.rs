use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Scans all `*.yml` (and `*.yaml`) files in `workflows_dir` and verifies that third-party
/// GitHub Actions are pinned to a 40-character hex commit SHA.
///
/// `dtolnay/rust-toolchain@...` is exempted as it follows its own toolchain pinning scheme.
pub fn run_check_action_pinning(workflows_dir: &Path) -> Result<Vec<String>, String> {
    if !workflows_dir.exists() {
        return Err(format!(
            "Workflows directory does not exist: {}",
            workflows_dir.display()
        ));
    }

    let sha_regex = Regex::new(r"^[0-9a-f]{40}$")
        .map_err(|e| format!("Failed to compile SHA regex: {}", e))?;
    let uses_regex = Regex::new(r"\buses:\s*([^\s#]+)")
        .map_err(|e| format!("Failed to compile uses regex: {}", e))?;

    let mut violations = Vec::new();

    for entry in WalkDir::new(workflows_dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() {
            let is_yml = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"));

            if is_yml {
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(e) => {
                        return Err(format!("Failed to read workflow file {}: {}", path.display(), e))
                    }
                };

                let file_path_str = path.to_string_lossy().replace('\\', "/");

                for (idx, line) in content.lines().enumerate() {
                    let line_trimmed = line.trim();

                    if line_trimmed.is_empty() || line_trimmed.starts_with('#') {
                        continue;
                    }

                    if let Some(captures) = uses_regex.captures(line_trimmed) {
                        if let Some(target_match) = captures.get(1) {
                            let raw_target = target_match.as_str();
                            let target = raw_target.trim_matches(&['\'', '"'][..]);

                            if let Some((action_path, action_ref)) = target.rsplit_once('@') {
                                // Exemption: dtolnay/rust-toolchain
                                if action_path == "dtolnay/rust-toolchain"
                                    || action_path.ends_with("/dtolnay/rust-toolchain")
                                {
                                    continue;
                                }

                                if !sha_regex.is_match(action_ref) {
                                    let line_num = idx + 1;
                                    violations.push(format!(
                                        "{}:{}: {}",
                                        file_path_str, line_num, line_trimmed
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_sha_pinned_action_accepted() {
        let dir = tempdir().expect("failed to create temp dir");
        let file_path = dir.path().join("workflow.yml");
        fs::write(
            &file_path,
            r#"
name: Test Workflow
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11
"#,
        )
        .expect("failed to write test workflow");

        let violations = run_check_action_pinning(dir.path()).expect("check failed");
        assert!(
            violations.is_empty(),
            "Expected no violations for SHA-pinned action, got: {:?}",
            violations
        );
    }

    #[test]
    fn test_tag_pinned_action_detected() {
        let dir = tempdir().expect("failed to create temp dir");
        let file_path = dir.path().join("workflow.yml");
        fs::write(
            &file_path,
            r#"
name: Test Workflow
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pypa/gh-action-pypi-publish@release/v1
"#,
        )
        .expect("failed to write test workflow");

        let violations = run_check_action_pinning(dir.path()).expect("check failed");
        assert_eq!(violations.len(), 2);
        assert!(violations[0].contains("actions/checkout@v4"));
        assert!(violations[1].contains("pypa/gh-action-pypi-publish@release/v1"));
    }

    #[test]
    fn test_dtolnay_rust_toolchain_exempted() {
        let dir = tempdir().expect("failed to create temp dir");
        let file_path = dir.path().join("workflow.yml");
        fs::write(
            &file_path,
            r#"
name: Test Workflow
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: dtolnay/rust-toolchain@stable
      - uses: dtolnay/rust-toolchain@nightly
"#,
        )
        .expect("failed to write test workflow");

        let violations = run_check_action_pinning(dir.path()).expect("check failed");
        assert!(
            violations.is_empty(),
            "Expected no violations for dtolnay/rust-toolchain, got: {:?}",
            violations
        );
    }

    #[test]
    fn test_empty_or_comment_only_workflow() {
        let dir = tempdir().expect("failed to create temp dir");
        let empty_file = dir.path().join("empty.yml");
        let comment_file = dir.path().join("comments.yml");

        fs::write(&empty_file, "").expect("failed to write empty file");
        fs::write(
            &comment_file,
            r#"
# This is a comment-only workflow file
# - uses: actions/checkout@v4
# another comment line
"#,
        )
        .expect("failed to write comment file");

        let violations = run_check_action_pinning(dir.path()).expect("check failed");
        assert!(
            violations.is_empty(),
            "Expected no violations for empty or comment-only workflows, got: {:?}",
            violations
        );
    }
}
