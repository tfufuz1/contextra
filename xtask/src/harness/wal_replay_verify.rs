//! Gate harness for verifying WAL replay crash consistency.

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalReplayFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct WalReplayOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<WalReplayFinding>,
}

pub fn run_wal_replay_verify(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;
    let mut budget_secs = 180u64;
    let mut base_rev = None;
    let mut head_rev = None;
    let mut custom_tests: Vec<String> = vec![];

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--json" => {
                json = true;
            }
            "--budget-secs" => {
                if i + 1 < args.len() {
                    if let Ok(b) = args[i + 1].parse() {
                        budget_secs = b;
                    }
                    i += 1;
                }
            }
            "--base" => {
                if i + 1 < args.len() {
                    let val = args[i + 1].trim();
                    if !val.is_empty() {
                        base_rev = Some(val.to_string());
                    }
                    i += 1;
                }
            }
            "--head" => {
                if i + 1 < args.len() {
                    let val = args[i + 1].trim();
                    if !val.is_empty() {
                        head_rev = Some(val.to_string());
                    }
                    i += 1;
                }
            }
            "--tests" => {
                if i + 1 < args.len() {
                    custom_tests = args[i + 1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let base = base_rev.unwrap_or_else(|| "HEAD~1".to_string());
    let head = head_rev.unwrap_or_else(|| "HEAD".to_string());

    let diff_output = Command::new("git")
        .args(["diff", "--name-only", &base, &head])
        .current_dir(&root)
        .output();

    let is_applicable = if let Ok(o) = diff_output {
        let diff_text = String::from_utf8_lossy(&o.stdout);
        diff_text.lines().any(|l| {
            l.starts_with("crates/contextra-store/")
                || l.starts_with("crates/contextra-checkpoint/")
                || l.starts_with("crates/contextra-mvcc/")
                || l.starts_with("crates/contextra-wire/")
        })
    } else {
        true
    };

    if !is_applicable {
        let output = WalReplayOutput {
            gate: "wal-replay-verify".to_string(),
            status: "not_applicable".to_string(),
            summary: "Diff does not touch WAL/store core crates".to_string(),
            findings: vec![],
        };
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
        } else {
            println!("wal-replay-verify: not_applicable (Diff touched no WAL/store crates)");
        }
        return 0;
    }

    let default_tests = vec![
        ("chaos_matrix", "chaos_wal_corruption_during_flush"),
        (
            "chaos_matrix",
            "chaos_manifest_corruption_during_compaction",
        ),
        ("chaos_matrix", "chaos_concurrent_crash_recovery_matrix"),
        ("crash_recovery", ""),
        ("crash_prefix_enumeration", ""),
    ];

    let mut findings = vec![];
    let mut has_error = false;

    if custom_tests.is_empty() {
        for (target, filter) in default_tests {
            let list_output = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "contextra-store",
                    "--test",
                    target,
                    "--",
                    "--list",
                ])
                .current_dir(&root)
                .output();

            if list_output.is_err() || !list_output.as_ref().unwrap().status.success() {
                has_error = true;
                findings.push(WalReplayFinding {
                    id: "UNKNOWN_TEST_TARGET".to_string(),
                    severity: "error".to_string(),
                    file: format!("crates/contextra-store/tests/{}.rs", target),
                    line: 0,
                    message: format!("Test target '{}' does not exist in contextra-store", target),
                    fix: "Provide a valid test target name".to_string(),
                });
                continue;
            }

            let mut cmd = Command::new("cargo");
            cmd.args(["test", "-p", "contextra-store", "--test", target]);
            if !filter.is_empty() {
                cmd.args(["--", filter, "--ignored", "--test-threads=1"]);
            }
            cmd.current_dir(&root);

            let start = std::time::Instant::now();
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    has_error = true;
                    findings.push(WalReplayFinding {
                        id: "TEST_SPAWN_FAILURE".to_string(),
                        severity: "error".to_string(),
                        file: format!("crates/contextra-store/tests/{}.rs", target),
                        line: 0,
                        message: format!("Failed to spawn test target '{}': {}", target, e),
                        fix: "Check cargo installation and toolchain".to_string(),
                    });
                    continue;
                }
            };

            let mut finished = false;
            while start.elapsed().as_secs() < budget_secs {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        finished = true;
                        if !status.success() {
                            findings.push(WalReplayFinding {
                                id: "WAL_REPLAY_TEST_FAILURE".to_string(),
                                severity: "error".to_string(),
                                file: format!("crates/contextra-store/tests/{}.rs", target),
                                line: 0,
                                message: format!(
                                    "WAL replay crash test '{}:{}' failed",
                                    target, filter
                                ),
                                fix: "Investigate crash recovery logic and state replay"
                                    .to_string(),
                            });
                        }
                        break;
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => break,
                }
            }

            if !finished {
                let _ = child.kill();
                has_error = true;
                findings.push(WalReplayFinding {
                    id: "TEST_TIMEOUT".to_string(),
                    severity: "error".to_string(),
                    file: format!("crates/contextra-store/tests/{}.rs", target),
                    line: 0,
                    message: format!(
                        "Test '{}:{}' exceeded budget of {}s",
                        target, filter, budget_secs
                    ),
                    fix: "Optimize test runtime or increase --budget-secs".to_string(),
                });
            }
        }
    } else {
        for test_name in &custom_tests {
            let list_output = Command::new("cargo")
                .args([
                    "test",
                    "-p",
                    "contextra-store",
                    "--test",
                    test_name,
                    "--",
                    "--list",
                ])
                .current_dir(&root)
                .output();

            if list_output.is_err() || !list_output.as_ref().unwrap().status.success() {
                has_error = true;
                findings.push(WalReplayFinding {
                    id: "UNKNOWN_TEST_TARGET".to_string(),
                    severity: "error".to_string(),
                    file: format!("crates/contextra-store/tests/{}.rs", test_name),
                    line: 0,
                    message: format!(
                        "Test target '{}' does not exist in contextra-store",
                        test_name
                    ),
                    fix: "Provide a valid test target name".to_string(),
                });
                continue;
            }

            let mut cmd = Command::new("cargo");
            cmd.args(["test", "-p", "contextra-store", "--test", test_name]);
            cmd.current_dir(&root);

            let start = std::time::Instant::now();
            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => {
                    has_error = true;
                    findings.push(WalReplayFinding {
                        id: "TEST_SPAWN_FAILURE".to_string(),
                        severity: "error".to_string(),
                        file: format!("crates/contextra-store/tests/{}.rs", test_name),
                        line: 0,
                        message: format!("Failed to spawn test '{}': {}", test_name, e),
                        fix: "Check cargo installation and toolchain".to_string(),
                    });
                    continue;
                }
            };

            let mut finished = false;
            while start.elapsed().as_secs() < budget_secs {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        finished = true;
                        if !status.success() {
                            findings.push(WalReplayFinding {
                                id: "WAL_REPLAY_TEST_FAILURE".to_string(),
                                severity: "error".to_string(),
                                file: format!("crates/contextra-store/tests/{}.rs", test_name),
                                line: 0,
                                message: format!("WAL replay crash test '{}' failed", test_name),
                                fix: "Investigate crash recovery logic and state replay"
                                    .to_string(),
                            });
                        }
                        break;
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => break,
                }
            }

            if !finished {
                let _ = child.kill();
                has_error = true;
                findings.push(WalReplayFinding {
                    id: "TEST_TIMEOUT".to_string(),
                    severity: "error".to_string(),
                    file: format!("crates/contextra-store/tests/{}.rs", test_name),
                    line: 0,
                    message: format!("Test '{}' exceeded budget of {}s", test_name, budget_secs),
                    fix: "Optimize test runtime or increase --budget-secs".to_string(),
                });
            }
        }
    }

    let status = if has_error {
        "error"
    } else if findings.iter().any(|f| f.severity == "error") {
        "fail"
    } else {
        "pass"
    };

    let summary = format!(
        "WAL replay verification status: {} with {} findings",
        status,
        findings.len()
    );

    let output = WalReplayOutput {
        gate: "wal-replay-verify".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: findings.clone(),
    };

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
    } else {
        println!("{}", summary);
        for f in &findings {
            println!(
                "  [{}] WAS: {} - {}",
                f.severity.to_uppercase(),
                f.id,
                f.message
            );
        }
    }

    match status {
        "pass" => 0,
        "fail" => 1,
        _ => 2,
    }
}

pub fn wal_replay_is_applicable(root: &Path, base: &str, head: &str) -> bool {
    let diff_output = Command::new("git")
        .args(["diff", "--name-only", base, head])
        .current_dir(root)
        .output();

    if let Ok(o) = diff_output {
        let diff_text = String::from_utf8_lossy(&o.stdout);
        diff_text.lines().any(|l| {
            l.starts_with("crates/contextra-store/")
                || l.starts_with("crates/contextra-checkpoint/")
                || l.starts_with("crates/contextra-mvcc/")
                || l.starts_with("crates/contextra-wire/")
        })
    } else {
        true
    }
}
