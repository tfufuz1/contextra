//! Module: workspace_verify
//! Central replacement for `verify_workspace.sh`.
//!
//! Iterates bottom-up over workspace crates calculated from `cargo metadata --no-deps --format-version=1`
//! using `serde_json`. Runs check/clippy/test depending on mode (Fast, Full, Audit) and outputs structured
//! diagnostic reports (`summary.md`, `diagnostics.jsonl`, `FEHLERBERICHT.md`).

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerifyMode {
    Fast,
    Full,
    Audit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceVerifyConfig {
    pub mode: VerifyMode,
    pub only: Vec<String>,
    pub resume_from: Option<String>,
    pub stop_on_fail: bool,
    pub run_clippy: bool,
}

impl Default for WorkspaceVerifyConfig {
    fn default() -> Self {
        Self {
            mode: VerifyMode::Full,
            only: Vec::new(),
            resume_from: None,
            stop_on_fail: false,
            run_clippy: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepResult {
    Pass,
    Fail(String),
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagLevel,
    pub file: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrateVerifyResult {
    pub crate_name: String,
    pub check: StepResult,
    pub clippy: StepResult,
    pub test: StepResult,
    pub duration_secs: u64,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    name: String,
    dependencies: Vec<MetadataDependency>,
}

#[derive(Deserialize)]
struct MetadataDependency {
    name: String,
}

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: HashSet<String>,
}

#[derive(Deserialize)]
struct CargoJsonMessage {
    reason: String,
    message: Option<CargoJsonDiagnostic>,
}

#[derive(Deserialize)]
struct CargoJsonDiagnostic {
    level: String,
    message: String,
    code: Option<CargoJsonCode>,
    spans: Vec<CargoJsonSpan>,
}

#[derive(Deserialize)]
struct CargoJsonCode {
    code: String,
}

#[derive(Deserialize)]
struct CargoJsonSpan {
    file_name: String,
    line_start: usize,
    column_start: usize,
    is_primary: bool,
}

fn find_root_dir() -> PathBuf {
    if let Ok(cargo_manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(cargo_manifest);
        if path.file_name().and_then(|s| s.to_str()) == Some("xtask") {
            if let Some(parent) = path.parent() {
                return parent.to_path_buf();
            }
        }
        return path;
    }
    let mut curr = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if curr.join("Cargo.toml").exists() && curr.join("capabilities.toml").exists() {
            return curr;
        }
        if !curr.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

pub fn get_bottom_up_crate_order_from_metadata(json_str: &str) -> Result<Vec<String>, String> {
    let metadata: CargoMetadata = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse cargo metadata JSON: {}", e))?;

    let workspace_packages: HashMap<String, Vec<String>> = metadata
        .packages
        .iter()
        .filter(|p| {
            metadata.workspace_members.contains(&p.name)
                || metadata
                    .workspace_members
                    .iter()
                    .any(|m| m.contains(&p.name))
        })
        .map(|p| {
            let deps = p.dependencies.iter().map(|d| d.name.clone()).collect();
            (p.name.clone(), deps)
        })
        .collect();

    let mut crate_depths: HashMap<String, usize> = HashMap::new();

    fn compute_depth(
        pkg: &str,
        ws_pkgs: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
    ) -> usize {
        if visited.contains(pkg) {
            return 0; // Avoid cycles
        }
        visited.insert(pkg.to_string());

        let mut max_dep_depth = 0;
        if let Some(deps) = ws_pkgs.get(pkg) {
            for dep in deps {
                if ws_pkgs.contains_key(dep) {
                    let d = compute_depth(dep, ws_pkgs, visited);
                    if d + 1 > max_dep_depth {
                        max_dep_depth = d + 1;
                    }
                }
            }
        }
        visited.remove(pkg);
        max_dep_depth
    }

    for pkg_name in workspace_packages.keys() {
        let mut visited = HashSet::new();
        let depth = compute_depth(pkg_name, &workspace_packages, &mut visited);
        crate_depths.insert(pkg_name.clone(), depth);
    }

    let mut crate_list: Vec<(String, usize)> = crate_depths.into_iter().collect();
    crate_list.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

    Ok(crate_list.into_iter().map(|(name, _)| name).collect())
}

pub fn get_bottom_up_crate_order(root: &Path) -> Result<Vec<String>, String> {
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["metadata", "--no-deps", "--format-version=1"])
        .output()
        .map_err(|e| format!("Failed to execute cargo metadata: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "cargo metadata command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    get_bottom_up_crate_order_from_metadata(&json_str)
}

fn parse_cargo_json_diagnostics(stdout_str: &str) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    for line in stdout_str.lines() {
        if let Ok(msg) = serde_json::from_str::<CargoJsonMessage>(line) {
            if msg.reason == "compiler-message" {
                if let Some(diag) = msg.message {
                    let level = match diag.level.as_str() {
                        "error" => DiagLevel::Error,
                        _ => DiagLevel::Warning,
                    };
                    let code = diag.code.map(|c| c.code);

                    let primary_span = diag.spans.iter().find(|s| s.is_primary);
                    let (file, line_num, col_num) = if let Some(span) = primary_span {
                        (
                            span.file_name.clone(),
                            Some(span.line_start),
                            Some(span.column_start),
                        )
                    } else {
                        ("unknown".to_string(), None, None)
                    };

                    diags.push(Diagnostic {
                        level,
                        file,
                        line: line_num,
                        column: col_num,
                        message: diag.message,
                        code,
                    });
                }
            }
        }
    }
    diags
}

pub fn run_workspace_verify(
    config: WorkspaceVerifyConfig,
    output_dir: &Path,
) -> Result<Vec<CrateVerifyResult>, String> {
    let root = find_root_dir();
    run_workspace_verify_in_root(&root, config, output_dir)
}

pub fn run_workspace_verify_in_root(
    root: &Path,
    config: WorkspaceVerifyConfig,
    output_dir: &Path,
) -> Result<Vec<CrateVerifyResult>, String> {
    fs::create_dir_all(output_dir).map_err(|e| {
        format!(
            "Failed to create output dir {}: {}",
            output_dir.display(),
            e
        )
    })?;

    let crate_order = get_bottom_up_crate_order(root)?;
    let mut results = Vec::new();
    let mut resuming = config.resume_from.is_some();
    let mut total_failed = false;

    for crate_name in crate_order {
        if let Some(ref resume_target) = config.resume_from {
            if resuming {
                if &crate_name == resume_target {
                    resuming = false;
                } else {
                    continue;
                }
            }
        }

        if !config.only.is_empty() && !config.only.contains(&crate_name) {
            continue;
        }

        let start = Instant::now();
        let mut crate_diags = Vec::new();

        // Step 1: cargo check
        let check_output = Command::new("cargo")
            .current_dir(root)
            .args([
                "check",
                "-p",
                &crate_name,
                "--locked",
                "--message-format=json",
            ])
            .output()
            .map_err(|e| format!("Failed to execute cargo check for {}: {}", crate_name, e))?;

        let check_stdout = String::from_utf8_lossy(&check_output.stdout);
        let check_stderr = String::from_utf8_lossy(&check_output.stderr);
        crate_diags.extend(parse_cargo_json_diagnostics(&check_stdout));

        let check_res = if check_output.status.success() {
            StepResult::Pass
        } else {
            StepResult::Fail(check_stderr.to_string())
        };

        let mut clippy_res = StepResult::Skip;
        let mut test_res = StepResult::Skip;

        if matches!(check_res, StepResult::Pass) {
            // Step 2: cargo clippy (Full or Audit mode)
            if (config.mode == VerifyMode::Full || config.mode == VerifyMode::Audit)
                && config.run_clippy
            {
                let clippy_output = Command::new("cargo")
                    .current_dir(root)
                    .args([
                        "clippy",
                        "-p",
                        &crate_name,
                        "--locked",
                        "--message-format=json",
                        "--",
                        "-D",
                        "warnings",
                    ])
                    .output()
                    .map_err(|e| {
                        format!("Failed to execute cargo clippy for {}: {}", crate_name, e)
                    })?;

                let clippy_stdout = String::from_utf8_lossy(&clippy_output.stdout);
                let clippy_stderr = String::from_utf8_lossy(&clippy_output.stderr);
                crate_diags.extend(parse_cargo_json_diagnostics(&clippy_stdout));

                clippy_res = if clippy_output.status.success() {
                    StepResult::Pass
                } else {
                    StepResult::Fail(clippy_stderr.to_string())
                };
            }

            // Step 3: cargo test (Full or Audit mode)
            if (config.mode == VerifyMode::Full || config.mode == VerifyMode::Audit)
                && matches!(clippy_res, StepResult::Pass | StepResult::Skip)
            {
                let test_output = Command::new("cargo")
                    .current_dir(root)
                    .args(["test", "-p", &crate_name, "--locked", "--quiet"])
                    .output()
                    .map_err(|e| {
                        format!("Failed to execute cargo test for {}: {}", crate_name, e)
                    })?;

                let test_stderr = String::from_utf8_lossy(&test_output.stderr);
                test_res = if test_output.status.success() {
                    StepResult::Pass
                } else {
                    StepResult::Fail(test_stderr.to_string())
                };
            }
        }

        let duration_secs = start.elapsed().as_secs();
        let has_fail = matches!(check_res, StepResult::Fail(_))
            || matches!(clippy_res, StepResult::Fail(_))
            || matches!(test_res, StepResult::Fail(_));

        if has_fail {
            total_failed = true;
        }

        results.push(CrateVerifyResult {
            crate_name: crate_name.clone(),
            check: check_res,
            clippy: clippy_res,
            test: test_res,
            duration_secs,
            diagnostics: crate_diags,
        });

        if has_fail && config.stop_on_fail {
            break;
        }
    }

    // Write output reports
    write_summary_md(output_dir, &results)?;
    write_diagnostics_jsonl(output_dir, &results)?;
    write_fehlerbericht_md(output_dir, &results)?;

    if total_failed {
        Err(format!(
            "Workspace verification completed with failures. Details written to {}",
            output_dir.display()
        ))
    } else {
        Ok(results)
    }
}

fn write_summary_md(output_dir: &Path, results: &[CrateVerifyResult]) -> Result<(), String> {
    let mut content = String::new();
    content.push_str("# Workspace Verification Summary\n\n");
    content.push_str("| Crate | Check | Clippy | Test | Duration (s) |\n");
    content.push_str("|---|---|---|---|---|\n");

    for r in results {
        let fmt_step = |s: &StepResult| match s {
            StepResult::Pass => "✅ Pass",
            StepResult::Fail(_) => "❌ Fail",
            StepResult::Skip => "➖ Skip",
        };
        content.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            r.crate_name,
            fmt_step(&r.check),
            fmt_step(&r.clippy),
            fmt_step(&r.test),
            r.duration_secs
        ));
    }

    fs::write(output_dir.join("summary.md"), content)
        .map_err(|e| format!("Failed to write summary.md: {}", e))
}

fn write_diagnostics_jsonl(output_dir: &Path, results: &[CrateVerifyResult]) -> Result<(), String> {
    let mut content = String::new();
    for r in results {
        for diag in &r.diagnostics {
            if let Ok(json_line) = serde_json::to_string(diag) {
                content.push_str(&json_line);
                content.push('\n');
            }
        }
    }
    fs::write(output_dir.join("diagnostics.jsonl"), content)
        .map_err(|e| format!("Failed to write diagnostics.jsonl: {}", e))
}

fn write_fehlerbericht_md(output_dir: &Path, results: &[CrateVerifyResult]) -> Result<(), String> {
    let mut content = String::new();
    content.push_str("# FEHLERBERICHT\n\n");

    let failed_results: Vec<_> = results
        .iter()
        .filter(|r| {
            matches!(r.check, StepResult::Fail(_))
                || matches!(r.clippy, StepResult::Fail(_))
                || matches!(r.test, StepResult::Fail(_))
        })
        .collect();

    if failed_results.is_empty() {
        content.push_str("Keine Fehler aufgetreten. Alle Schritte erfolgreich.\n");
    } else {
        for r in failed_results {
            content.push_str(&format!("## Crate: {}\n\n", r.crate_name));
            if let StepResult::Fail(msg) = &r.check {
                content.push_str("### Check Failure\n```\n");
                content.push_str(msg);
                content.push_str("\n```\n\n");
            }
            if let StepResult::Fail(msg) = &r.clippy {
                content.push_str("### Clippy Failure\n```\n");
                content.push_str(msg);
                content.push_str("\n```\n\n");
            }
            if let StepResult::Fail(msg) = &r.test {
                content.push_str("### Test Failure\n```\n");
                content.push_str(msg);
                content.push_str("\n```\n\n");
            }
        }
    }

    fs::write(output_dir.join("FEHLERBERICHT.md"), content)
        .map_err(|e| format!("Failed to write FEHLERBERICHT.md: {}", e))
}
