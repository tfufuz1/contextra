use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoomTestFile {
    pub crate_name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct LoomRunResult {
    pub discovered_files: Vec<LoomTestFile>,
    pub passed: bool,
    pub output: String,
    pub duration_secs: u64,
}

/// Discovers all `loom_*.rs` test files in `crates/*/tests/loom_*.rs` under the given root directory.
pub fn discover_loom_tests(root: &Path) -> Vec<LoomTestFile> {
    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Vec::new();
    }

    let mut discovered = Vec::new();

    for entry in WalkDir::new(&crates_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if let Ok(rel_path) = path.strip_prefix(root) {
            let components: Vec<_> = rel_path.components().map(|c| c.as_os_str()).collect();
            // Expected relative path structure: crates/<crate_name>/tests/loom_*.rs
            if components.len() >= 4 && components[0] == "crates" && components[2] == "tests" {
                if let (Some(file_name), Some(crate_name)) = (
                    path.file_name().and_then(|n| n.to_str()),
                    components[1].to_str(),
                ) {
                    if file_name.starts_with("loom_") && file_name.ends_with(".rs") {
                        discovered.push(LoomTestFile {
                            crate_name: crate_name.to_string(),
                            path: rel_path.to_string_lossy().to_string(),
                        });
                    }
                }
            }
        }
    }

    discovered.sort_by(|a, b| a.path.cmp(&b.path));
    discovered
}

/// Runs loom concurrency tests using `cargo test`.
/// Sets `RUSTFLAGS="--cfg loom"` on the child process Command.
/// Writes a report to `target/loom-report.md`.
pub fn run_loom(test_filter: Option<&str>, root: &Path) -> Result<LoomRunResult, String> {
    let discovered = discover_loom_tests(root);
    let start_time = Instant::now();

    let mut cmd = Command::new("cargo");
    cmd.current_dir(root);
    cmd.env("RUSTFLAGS", "--cfg loom");
    cmd.args(["test", "--locked"]);

    for file in &discovered {
        // Skip tests that cannot run under `RUSTFLAGS="--cfg loom"`:
        // - `loom_relate_n_ary`: contextra-db depends on contextra-engine which gates Collection with #[cfg(not(loom))]
        // - `loom_quantizer_race_test`: tokio runtime builder enable_all() panics under Loom mock runtime
        if file.path.contains("loom_relate_n_ary") || file.path.contains("loom_quantizer_race_test") {
            continue;
        }

        cmd.arg("-p").arg(&file.crate_name);
        if let Some(stem) = Path::new(&file.path).file_stem().and_then(|s| s.to_str()) {
            cmd.arg("--test").arg(stem);
        }
    }

    cmd.arg("--");
    if let Some(filter) = test_filter {
        cmd.arg(filter);
    }
    cmd.arg("--test-threads=1");

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute cargo test for loom: {e}"))?;

    let duration_secs = start_time.elapsed().as_secs();
    let passed = output.status.success();
    let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
    let combined_output = format!("{stdout_str}\n{stderr_str}");

    // Write target/loom-report.md
    let target_dir = root.join("target");
    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!("Warning: failed to create target directory for loom report: {e}");
    }

    let report_path = target_dir.join("loom-report.md");
    let mut report_content = String::new();
    report_content.push_str("# Loom Concurrency Test Report\n\n");
    report_content.push_str(&format!(
        "**Status**: {}\n**Duration**: {}s\n**Discovered Files**: {}\n\n",
        if passed { "PASSED" } else { "FAILED" },
        duration_secs,
        discovered.len()
    ));

    if discovered.is_empty() {
        report_content.push_str("⚠️ **WARNING**: No `loom_*.rs` test files were discovered under `crates/*/tests/`!\n\n");
    } else {
        report_content.push_str("| Crate | File | Status |\n");
        report_content.push_str("|---|---|---|\n");
        for file in &discovered {
            let status_str = if file.path.contains("loom_relate_n_ary") || file.path.contains("loom_quantizer_race_test") {
                "SKIPPED_NOT_LOOM"
            } else if passed {
                "PASSED"
            } else {
                "CHECK_OUTPUT"
            };
            report_content.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                file.crate_name, file.path, status_str
            ));
        }
        report_content.push('\n');
    }

    let _ = fs::write(&report_path, report_content);

    Ok(LoomRunResult {
        discovered_files: discovered,
        passed,
        output: combined_output,
        duration_secs,
    })
}
