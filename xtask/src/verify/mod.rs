// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Implementation of single-command verification pipeline `cargo xtask verify`.

pub mod evidence;
pub mod report;
pub mod steps;

use chrono::Utc;
use evidence::{StepEvidence, StepStatus, VerificationSummary};
use report::print_summary_and_save;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use steps::StepRunner;

pub fn get_default_members(root: &Path) -> Vec<String> {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(members) = val
                    .get("workspace_default_members")
                    .and_then(|m| m.as_array())
                {
                    let mut list = Vec::new();
                    for m in members {
                        if let Some(id) = m.as_str() {
                            let crate_name = id.split_whitespace().next().unwrap_or(id);
                            list.push(crate_name.to_string());
                        }
                    }
                    if !list.is_empty() {
                        return list;
                    }
                }
            }
        }
    }

    vec![
        "contextra-types".to_string(),
        "contextra-sys".to_string(),
        "contextra-crypto".to_string(),
        "contextra-store".to_string(),
        "contextra-mvcc".to_string(),
        "contextra-vector".to_string(),
        "contextra-engine".to_string(),
    ]
}

pub fn run_verify(args: &[String]) -> i32 {
    let mut tier = "kernel".to_string();
    let mut allow_skips = Vec::new();
    let mut root = crate::find_root_dir();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--tier" => {
                if i + 1 < args.len() {
                    tier = args[i + 1].clone();
                    i += 1;
                }
            }
            "--allow-skip" => {
                if i + 1 < args.len() {
                    allow_skips.push(args[i + 1].clone());
                    i += 1;
                }
            }
            "--root" => {
                if i + 1 < args.len() {
                    root = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let run_id = format!("{}", Utc::now().format("%Y%m%d_%H%M%S"));
    let rel_run_dir = format!("target/evidence/{}", run_id);
    let run_dir = root.join(&rel_run_dir);

    if let Err(e) = fs::create_dir_all(&run_dir) {
        eprintln!(
            "Failed to create evidence directory {}: {}",
            run_dir.display(),
            e
        );
        return 2;
    }

    let toolchain_info = Command::new("rustc")
        .arg("-Vv")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "rustc unknown".to_string());

    let commit_sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let runner = StepRunner {
        root: &root,
        run_dir: &run_dir,
        rel_run_dir: &rel_run_dir,
        toolchain_info: &toolchain_info,
        commit_sha: &commit_sha,
        allow_skips: &allow_skips,
    };

    let mut steps_evidence: Vec<StepEvidence> = Vec::new();

    // 1. fmt --check
    steps_evidence.push(runner.run_step(
        "fmt",
        &["cargo", "fmt", "--all", "--", "--check"],
        None,
        None,
    ));

    // 2. clippy
    steps_evidence.push(runner.run_step(
        "clippy",
        &["cargo", "clippy", "--all-targets", "--", "-D", "warnings"],
        None,
        None,
    ));

    // 3. check for kernel crates (default-members)
    let default_members = get_default_members(&root);
    let mut check_args = vec!["cargo", "check", "--all-targets"];
    for m in &default_members {
        check_args.push("-p");
        check_args.push(m.as_str());
    }
    steps_evidence.push(runner.run_step("check", &check_args, None, None));

    // 4. nextest / test
    let has_nextest = Command::new("cargo")
        .args(["nextest", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_nextest {
        steps_evidence.push(runner.run_step(
            "test",
            &["cargo", "nextest", "run", "--all-targets"],
            None,
            None,
        ));
    } else {
        eprintln!("⚠️ cargo-nextest is missing, falling back to cargo test --all-targets");
        steps_evidence.push(runner.run_step(
            "test",
            &["cargo", "test", "--all-targets"],
            None,
            None,
        ));
    }

    // 5. doctrine-scan
    steps_evidence.push(runner.run_step(
        "doctrine-scan",
        &["cargo", "xtask", "doctrine-scan"],
        None,
        None,
    ));

    // 6. check-ring-layering
    steps_evidence.push(runner.run_step(
        "check-ring-layering",
        &["cargo", "xtask", "check-ring-layering"],
        None,
        None,
    ));

    // 7. check-unsafe-islands
    steps_evidence.push(runner.run_step(
        "check-unsafe-islands",
        &["cargo", "xtask", "check-unsafe-islands"],
        None,
        None,
    ));

    // 8. test_integrity
    steps_evidence.push(runner.run_step(
        "test_integrity",
        &["cargo", "xtask", "test-integrity"],
        None,
        None,
    ));

    // 9. check-loom-mocks
    steps_evidence.push(runner.run_step(
        "check-loom-mocks",
        &["cargo", "xtask", "check-loom-mocks"],
        None,
        None,
    ));

    if tier == "full" {
        // 10. miri on wire and sys
        steps_evidence.push(runner.run_step(
            "miri-wire",
            &["cargo", "+nightly", "miri", "test", "-p", "contextra-wire"],
            None,
            Some("miri"),
        ));
        steps_evidence.push(runner.run_step(
            "miri-sys",
            &["cargo", "+nightly", "miri", "test", "-p", "contextra-sys"],
            None,
            Some("miri"),
        ));

        // 11. loom-run
        steps_evidence.push(runner.run_step(
            "loom-run",
            &["cargo", "xtask", "loom-run"],
            None,
            None,
        ));

        // 12. fuzz-smoke
        steps_evidence.push(runner.run_step(
            "fuzz-smoke",
            &["cargo", "xtask", "fuzz-smoke"],
            None,
            Some("cargo-fuzz"),
        ));

        // 13. cargo mutants list
        steps_evidence.push(runner.run_step(
            "mutants-list",
            &["cargo", "mutants", "--list"],
            None,
            Some("cargo-mutants"),
        ));
    }

    let overall_status = if steps_evidence.iter().any(|s| {
        s.status == StepStatus::Failed
            || s.status == StepStatus::Error
            || s.status == StepStatus::SkippedToolMissing
    }) {
        StepStatus::Failed
    } else {
        StepStatus::Passed
    };

    let summary = VerificationSummary {
        run_id,
        tier,
        timestamp_utc: Utc::now().to_rfc3339(),
        overall_status: overall_status.clone(),
        steps: steps_evidence,
    };

    print_summary_and_save(&run_dir, &summary);

    if overall_status == StepStatus::Passed {
        0
    } else {
        1
    }
}
