//! Gate harness for auditing nested async locks, await holding lock lints, and lock hierarchy.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LockAuditFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LockAuditOutput {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<LockAuditFinding>,
}

pub fn lock_audit_get_touched_crates(
    root: &Path,
    base: &str,
    head: &str,
) -> Result<BTreeSet<String>, String> {
    let diff_output = Command::new("git")
        .args(["diff", "--name-only", base, head])
        .current_dir(root)
        .output();

    let mut touched = BTreeSet::new();
    if let Ok(o) = diff_output {
        let diff_text = String::from_utf8_lossy(&o.stdout);
        for line in diff_text.lines() {
            if line.starts_with("crates/") {
                let parts: Vec<&str> = line.split('/').collect();
                if parts.len() >= 2 {
                    touched.insert(parts[1].to_string());
                }
            }
        }
    }
    Ok(touched)
}

pub fn lock_audit_scan_file_fallback(
    file_path: &Path,
    rel_path: &str,
    findings: &mut Vec<LockAuditFinding>,
) {
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut lock_lines = Vec::new();

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        if line.contains(".lock().await") {
            lock_lines.push(line_num);
        }
    }

    if lock_lines.len() >= 2 {
        findings.push(LockAuditFinding {
            id: "NESTED_LOCK_AWAIT".to_string(),
            severity: "error".to_string(),
            file: rel_path.to_string(),
            line: lock_lines[1],
            message: format!(
                "Multiple lock().await calls detected in proximity (first at line {})",
                lock_lines[0]
            ),
            fix: "Enforce strict lock ordering or release earlier lock prior to acquiring next"
                .to_string(),
        });
    }
}

pub fn lock_audit_check_agents_md(
    crate_dir: &Path,
    crate_name: &str,
    findings: &mut Vec<LockAuditFinding>,
) {
    let agents_path = crate_dir.join("AGENTS.md");
    if !agents_path.exists() {
        return;
    }

    if let Ok(content) = fs::read_to_string(&agents_path) {
        let mut in_lock_section = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') && trimmed.to_lowercase().contains("lock") {
                in_lock_section = true;
                continue;
            }
            if in_lock_section && trimmed.starts_with('#') {
                in_lock_section = false;
            }

            if in_lock_section && (trimmed.contains('>') || trimmed.contains("->")) {
                findings.push(LockAuditFinding {
                    id: "LOCK_HIERARCHY_INFO".to_string(),
                    severity: "warn".to_string(),
                    file: format!("crates/{}/AGENTS.md", crate_name),
                    line: 1,
                    message: format!("Documented lock hierarchy found for {}: {}", crate_name, trimmed),
                    fix: "Ensure code respects lock hierarchy sequence".to_string(),
                });
            }
        }
    }
}

pub fn run_lock_audit(args: &[String]) -> i32 {
    let mut root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut json = false;
    let mut base_rev = None;
    let mut head_rev = None;

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
            "--base" => {
                if i + 1 < args.len() {
                    base_rev = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--head" => {
                if i + 1 < args.len() {
                    head_rev = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let base = base_rev.unwrap_or_else(|| "HEAD~1".to_string());
    let head = head_rev.unwrap_or_else(|| "HEAD".to_string());

    let touched_crates = match lock_audit_get_touched_crates(&root, &base, &head) {
        Ok(c) => c,
        Err(e) => {
            let out = LockAuditOutput {
                gate: "lock-audit".to_string(),
                status: "error".to_string(),
                summary: e,
                findings: vec![],
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            }
            return 2;
        }
    };

    if touched_crates.is_empty() {
        let output = LockAuditOutput {
            gate: "lock-audit".to_string(),
            status: "not_applicable".to_string(),
            summary: "No workspace crates touched in diff".to_string(),
            findings: vec![],
        };
        if json {
            println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
        } else {
            println!("lock-audit: not_applicable (no crates touched)");
        }
        return 0;
    }

    let mut findings = Vec::new();
    let has_sg = Command::new("sg").arg("--version").output().is_ok()
        || Command::new("ast-grep").arg("--version").output().is_ok();

    for crate_name in &touched_crates {
        let crate_dir = root.join("crates").join(crate_name);
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        if has_sg && root.join("xtask/src/detect_nested_locks.yml").exists() {
            let sg_tool = if Command::new("sg").arg("--version").output().is_ok() {
                "sg"
            } else {
                "ast-grep"
            };
            let rule_path = root.join("xtask/src/detect_nested_locks.yml");
            let sg_output = Command::new(sg_tool)
                .args(["scan", "--rule", rule_path.to_str().unwrap_or_default()])
                .current_dir(&src_dir)
                .output();

            if let Ok(o) = sg_output {
                if !o.stdout.is_empty() {
                    let text = String::from_utf8_lossy(&o.stdout);
                    for line in text.lines() {
                        if !line.trim().is_empty() {
                            findings.push(LockAuditFinding {
                                id: "NESTED_LOCK_SG".to_string(),
                                severity: "error".to_string(),
                                file: format!("crates/{}/src", crate_name),
                                line: 0,
                                message: line.to_string(),
                                fix: "Refactor nested lock().await invocations".to_string(),
                            });
                        }
                    }
                }
            }
        } else {
            for entry in walkdir::WalkDir::new(&src_dir)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file()
                    && entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
                {
                    let rel = match entry.path().strip_prefix(&root) {
                        Ok(p) => p.to_string_lossy().to_string(),
                        Err(_) => entry.path().to_string_lossy().to_string(),
                    };
                    lock_audit_scan_file_fallback(entry.path(), &rel, &mut findings);
                }
            }
        }

        let clippy_output = Command::new("cargo")
            .args([
                "clippy",
                "-p",
                crate_name,
                "--",
                "-D",
                "clippy::await_holding_lock",
            ])
            .current_dir(&root)
            .output();

        if let Ok(o) = clippy_output {
            if !o.status.success() {
                let stderr = String::from_utf8_lossy(&o.stderr);
                if stderr.contains("await_holding_lock") {
                    findings.push(LockAuditFinding {
                        id: "AWAIT_HOLDING_LOCK".to_string(),
                        severity: "error".to_string(),
                        file: format!("crates/{}", crate_name),
                        line: 0,
                        message: format!("clippy::await_holding_lock violation detected in crate {}", crate_name),
                        fix: "Do not hold synchronous parking_lot Mutex/RwLock guard across await points".to_string(),
                    });
                }
            }
        }

        lock_audit_check_agents_md(&crate_dir, crate_name, &mut findings);
    }

    let has_errors = findings.iter().any(|f| f.severity == "error");
    let status = if has_errors { "fail" } else { "pass" };
    let summary = format!(
        "Lock audit completed across {} touched crates: {} findings",
        touched_crates.len(),
        findings.len()
    );

    let output = LockAuditOutput {
        gate: "lock-audit".to_string(),
        status: status.to_string(),
        summary: summary.clone(),
        findings: findings.clone(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!("{}", summary);
        for f in &findings {
            println!("  [{}] {}: {}", f.severity.to_uppercase(), f.file, f.message);
        }
    }

    if status == "fail" {
        1
    } else {
        0
    }
}
