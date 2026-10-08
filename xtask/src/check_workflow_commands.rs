//! Subcommand to verify that all xtask subcommands referenced in GitHub workflow files
//! actually exist as match arms in `xtask/src/main.rs` or as harness modules in `xtask/src/harness/*.rs`.

use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Helper function to strip line comments (`//...`) and block comments (`/*...*/`) from source text.
pub fn strip_comments(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_block_comment = false;

    while let Some(c) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if c == '/' {
            if chars.peek() == Some(&'*') {
                chars.next();
                in_block_comment = true;
                continue;
            } else if chars.peek() == Some(&'/') {
                // Skip line comment until newline
                while let Some(&next) = chars.peek() {
                    if next == '\n' {
                        break;
                    }
                    chars.next();
                }
                continue;
            }
        }

        output.push(c);
    }

    output
}

/// Extracts all valid xtask subcommand names defined in `xtask/src/main.rs` (match arms)
/// and `xtask/src/cli/mod.rs` (`COMMAND_DISPATCH_TABLE`).
pub fn extract_valid_subcommands(
    main_rs_content: &str,
    cli_mod_content: Option<&str>,
) -> HashSet<String> {
    let mut valid_commands = HashSet::new();

    // 1. Extract match arms from main.rs (comment-free)
    let clean_main = strip_comments(main_rs_content);
    let str_regex = Regex::new(r#""([a-z0-9_-]+)""#).expect("Valid regex");

    for line in clean_main.lines() {
        if let Some((patterns, _)) = line.split_once("=>") {
            for caps in str_regex.captures_iter(patterns) {
                if let Some(cmd) = caps.get(1) {
                    let s = cmd.as_str();
                    if !s.is_empty() {
                        valid_commands.insert(s.to_string());
                    }
                }
            }
        }
    }

    // 2. Extract COMMAND_DISPATCH_TABLE entries from cli/mod.rs (comment-free)
    if let Some(cli_content) = cli_mod_content {
        let clean_cli = strip_comments(cli_content);
        if let Some((_, after)) = clean_cli.split_once("COMMAND_DISPATCH_TABLE") {
            let bytes = after.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i] == b'"' {
                    let start = i + 1;
                    i += 1;
                    while i < bytes.len() && bytes[i] != b'"' {
                        i += 1;
                    }
                    if i < bytes.len() {
                        let candidate = &after[start..i];
                        let rest = after[i + 1..].trim_start();
                        if rest.starts_with(',')
                            && !candidate.is_empty()
                            && candidate
                                .chars()
                                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
                        {
                            valid_commands.insert(candidate.to_string());
                        }
                    }
                }
                i += 1;
            }
        }
    }

    valid_commands
}

/// Extracts harness subcommands from `xtask/src/harness/*.rs` plus builtins.
pub fn extract_harness_subcommands(harness_dir: &Path) -> HashSet<String> {
    let mut harness_cmds = HashSet::new();
    harness_cmds.insert("harness-list".to_string());
    harness_cmds.insert("harness-help".to_string());

    if harness_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(harness_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem != "mod" {
                            harness_cmds.insert(stem.replace('_', "-"));
                        }
                    }
                }
            }
        }
    }

    harness_cmds
}

/// Structure representing a subcommand invocation found in a workflow file.
#[derive(Debug, PartialEq, Eq)]
pub struct WorkflowCommandInvocation {
    pub file_path: String,
    pub line_number: usize,
    pub command: String,
}

/// Parses workflow YAML content to find `cargo run -p xtask -- <cmd>` and `cargo xtask <cmd>` invocations.
pub fn parse_workflow_invocations(
    file_path: &str,
    content: &str,
) -> Vec<WorkflowCommandInvocation> {
    let mut invocations = Vec::new();
    let cmd_regex =
        Regex::new(r"(?:cargo\s+run\s+-p\s+xtask\s+--\s+|cargo\s+xtask\s+)([a-z0-9_-]+)")
            .expect("Valid regex");

    for (idx, line) in content.lines().enumerate() {
        for captures in cmd_regex.captures_iter(line) {
            if let Some(cmd_match) = captures.get(1) {
                invocations.push(WorkflowCommandInvocation {
                    file_path: file_path.to_string(),
                    line_number: idx + 1,
                    command: cmd_match.as_str().to_string(),
                });
            }
        }
    }

    invocations
}

/// Runs the workflow command sync check against the repository root.
pub fn run_check_workflow_commands(root: &Path) -> bool {
    println!("=== Running Meta-Gate: check-workflow-commands ===");

    let main_rs_path = root.join("xtask/src/main.rs");
    let main_rs_content = match fs::read_to_string(&main_rs_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Failed to read {}: {}", main_rs_path.display(), e);
            return false;
        }
    };

    let cli_mod_path = root.join("xtask/src/cli/mod.rs");
    let cli_mod_content = fs::read_to_string(&cli_mod_path).ok();

    let mut valid_subcommands =
        extract_valid_subcommands(&main_rs_content, cli_mod_content.as_deref());
    let harness_subcommands = extract_harness_subcommands(&root.join("xtask/src/harness"));
    valid_subcommands.extend(harness_subcommands);

    if valid_subcommands.is_empty() {
        eprintln!(
            "❌ No valid xtask subcommands extracted from {}",
            main_rs_path.display()
        );
        return false;
    }

    println!(
        "Found {} valid xtask subcommands (main.rs + harness)",
        valid_subcommands.len()
    );

    let workflows_dir = root.join(".github/workflows");
    if !workflows_dir.exists() {
        eprintln!(
            "⚠️ Workflows directory does not exist: {}",
            workflows_dir.display()
        );
        return true;
    }

    let mut all_invocations = Vec::new();

    for entry in WalkDir::new(&workflows_dir)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    let relative_path = path
                        .strip_prefix(root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();

                    match fs::read_to_string(path) {
                        Ok(content) => {
                            let invs = parse_workflow_invocations(&relative_path, &content);
                            all_invocations.extend(invs);
                        }
                        Err(e) => {
                            eprintln!("⚠️ Could not read workflow file {}: {}", relative_path, e);
                        }
                    }
                }
            }
        }
    }

    let mut missing_count = 0;

    for inv in &all_invocations {
        if !valid_subcommands.contains(&inv.command) {
            eprintln!(
                "❌ {}:{}: Unknown xtask subcommand '{}'",
                inv.file_path, inv.line_number, inv.command
            );
            missing_count += 1;
        }
    }

    if missing_count > 0 {
        eprintln!(
            "❌ [META-GATE FAILED]: {} unknown xtask subcommand invocation(s) found in workflows",
            missing_count
        );
        false
    } else {
        println!(
            "✅ [META-GATE PASSED]: Verified {} xtask subcommand invocation(s) across workflows",
            all_invocations.len()
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_valid_workflow() {
        let dir = tempdir().expect("tempdir creation");
        let root = dir.path();

        let xtask_src = root.join("xtask/src");
        fs::create_dir_all(&xtask_src).expect("create_dir_all");
        let main_rs_content = r#"
fn main() {
    match subcommand {
        "sync-docs" => {}
        "check-compile" => {}
        "check-dag" => {}
        _ => {}
    }
}
"#;
        fs::write(xtask_src.join("main.rs"), main_rs_content).expect("write main.rs");

        let workflows_dir = root.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("create workflows dir");
        let workflow_content = r#"
name: Test Workflow
jobs:
  test:
    steps:
      - name: Step 1
        run: cargo run -p xtask -- check-compile
      - name: Step 2
        run: cargo xtask check-dag
"#;
        fs::write(workflows_dir.join("test.yml"), workflow_content).expect("write workflow.yml");

        assert!(run_check_workflow_commands(root));
    }

    #[test]
    fn test_valid_cli_mod_workflow() {
        let dir = tempdir().expect("tempdir creation");
        let root = dir.path();

        let xtask_src = root.join("xtask/src");
        let cli_dir = xtask_src.join("cli");
        fs::create_dir_all(&cli_dir).expect("create_dir_all");
        let main_rs_content = r#"
fn main() {}
"#;
        let cli_mod_content = r#"
pub const COMMAND_DISPATCH_TABLE: &[(&str, fn(&[String]) -> i32)] = &[
    ("check-compile", run_check_compile),
    ("check-dag", run_check_dag),
];
"#;
        fs::write(xtask_src.join("main.rs"), main_rs_content).expect("write main.rs");
        fs::write(cli_dir.join("mod.rs"), cli_mod_content).expect("write cli/mod.rs");

        let workflows_dir = root.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("create workflows dir");
        let workflow_content = r#"
name: Test Workflow
jobs:
  test:
    steps:
      - name: Step 1
        run: cargo run -p xtask -- check-compile
      - name: Step 2
        run: cargo xtask check-dag
"#;
        fs::write(workflows_dir.join("test.yml"), workflow_content).expect("write workflow.yml");

        assert!(run_check_workflow_commands(root));
    }

    #[test]
    fn test_command_in_comment_not_counted() {
        let main_rs_content = r#"
fn main() {
    // "comment-only-cmd" => {}
    /* "block-comment-cmd" => {} */
    match subcommand {
        "valid-cmd" => {}
        _ => {}
    }
}
"#;
        let cli_mod_content = r#"
// ("comment-registry-cmd", run_func),
pub static COMMAND_DISPATCH_TABLE: &[(&str, fn(&[String]) -> i32)] = &[
    ("valid-registry-cmd", run_func),
];
"#;
        let valid = extract_valid_subcommands(main_rs_content, Some(cli_mod_content));
        assert!(valid.contains("valid-cmd"));
        assert!(valid.contains("valid-registry-cmd"));
        assert!(!valid.contains("comment-only-cmd"));
        assert!(!valid.contains("block-comment-cmd"));
        assert!(!valid.contains("comment-registry-cmd"));
    }

    #[test]
    fn test_command_in_match_arm_counted() {
        let main_rs_content = r#"
fn main() {
    match cmd {
        "match-arm-one" | "match-arm-two" => { do_something(); }
        "match-arm-three" => { do_other(); }
        _ => {}
    }
}
"#;
        let valid = extract_valid_subcommands(main_rs_content, None);
        assert!(valid.contains("match-arm-one"));
        assert!(valid.contains("match-arm-two"));
        assert!(valid.contains("match-arm-three"));
        assert_eq!(valid.len(), 3);
    }

    #[test]
    fn test_unknown_command_fails() {
        let dir = tempdir().expect("tempdir creation");
        let root = dir.path();

        let xtask_src = root.join("xtask/src");
        fs::create_dir_all(&xtask_src).expect("create_dir_all");
        let main_rs_content = r#"
fn main() {
    match subcommand {
        "known-cmd" => {}
        _ => {}
    }
}
"#;
        fs::write(xtask_src.join("main.rs"), main_rs_content).expect("write main.rs");

        let workflows_dir = root.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("create workflows dir");
        let workflow_content = r#"
name: Test Workflow
jobs:
  test:
    steps:
      - name: Step 1
        run: cargo run -p xtask -- unknown-cmd
"#;
        fs::write(workflows_dir.join("test.yml"), workflow_content).expect("write workflow.yml");

        assert!(!run_check_workflow_commands(root));
    }

    #[test]
    fn test_valid_harness_workflow() {
        let dir = tempdir().expect("tempdir creation");
        let root = dir.path();

        let xtask_src = root.join("xtask/src");
        let harness_dir = xtask_src.join("harness");
        fs::create_dir_all(&harness_dir).expect("create_dir_all");
        let main_rs_content = r#"
fn main() {
    match subcommand {
        "sync-docs" => {}
        _ => {}
    }
}
"#;
        fs::write(xtask_src.join("main.rs"), main_rs_content).expect("write main.rs");
        fs::write(
            harness_dir.join("dummy_check.rs"),
            "//! Dummy summary\npub fn run_dummy_check(_args: &[String]) -> i32 { 0 }",
        )
        .expect("write dummy harness");

        let workflows_dir = root.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("create workflows dir");
        let workflow_content = r#"
name: Test Workflow
jobs:
  test:
    steps:
      - name: Step 1
        run: cargo xtask dummy-check
      - name: Step 2
        run: cargo xtask harness-list
"#;
        fs::write(workflows_dir.join("test.yml"), workflow_content).expect("write workflow.yml");

        assert!(run_check_workflow_commands(root));
    }

    #[test]
    fn test_invalid_workflow() {
        let dir = tempdir().expect("tempdir creation");
        let root = dir.path();

        let xtask_src = root.join("xtask/src");
        fs::create_dir_all(&xtask_src).expect("create_dir_all");
        let main_rs_content = r#"
fn main() {
    match subcommand {
        "sync-docs" => {}
        "check-compile" => {}
        _ => {}
    }
}
"#;
        fs::write(xtask_src.join("main.rs"), main_rs_content).expect("write main.rs");

        let workflows_dir = root.join(".github/workflows");
        fs::create_dir_all(&workflows_dir).expect("create workflows dir");
        let workflow_content = r#"
name: Test Workflow
jobs:
  test:
    steps:
      - name: Step 1
        run: cargo run -p xtask -- check-compile
      - name: Step 2
        run: cargo run -p xtask -- non-existent-command
"#;
        fs::write(workflows_dir.join("test.yml"), workflow_content).expect("write workflow.yml");

        assert!(!run_check_workflow_commands(root));
    }
}
