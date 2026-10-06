//! Aggregate Gate Arch (check-dag, check-ring-layering, check-vetoes, check-duplicate-symbols, check-unsafe-islands, check-ring0-async-purity, check-ring-capabilities-consistency).

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConstituentResult {
    pub name: String,
    pub status: String, // "OK", "FAIL", or "SKIP(...)"
    pub code: i32,
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateArchResult {
    pub gate: String,
    pub status: String, // "pass" or "fail"
    pub summary: String,
    pub constituents: Vec<ConstituentResult>,
}

pub fn run_gate_arch(args: &[String]) -> i32 {
    let mut root_dir = detect_repo_root();
    let mut base_rev = "origin/main".to_string();
    let mut head_rev = "HEAD".to_string();
    let mut is_json = false;
    let mut run_all = false;
    let mut strict = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                if i + 1 < args.len() {
                    root_dir = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--base" => {
                if i + 1 < args.len() {
                    base_rev = args[i + 1].clone();
                    i += 1;
                }
            }
            "--head" => {
                if i + 1 < args.len() {
                    head_rev = args[i + 1].clone();
                    i += 1;
                }
            }
            "--all" => {
                run_all = true;
            }
            "--changed" => {
                run_all = false;
            }
            "--strict" => {
                strict = true;
            }
            "--json" => {
                is_json = true;
            }
            _ => {}
        }
        i += 1;
    }

    if base_rev == "origin/main" && !run_all {
        if let Some(mb) = get_git_merge_base(&root_dir, "origin/main", &head_rev) {
            base_rev = mb;
        } else if let Some(mb) = get_git_merge_base(&root_dir, "HEAD~1", &head_rev) {
            base_rev = mb;
        }
    }

    let changed_files = get_changed_files(&root_dir, &base_rev, &head_rev);

    let (trigger_unsafe, trigger_purity, trigger_caps) = if run_all || changed_files.is_none() {
        (true, true, true)
    } else if let Some(ref files) = changed_files {
        let has_unsafe = files.iter().any(|f| {
            f.ends_with("Cargo.toml")
                || f == "capabilities.toml"
                || (f.ends_with(".rs") && file_contains_unsafe(&root_dir.join(f)))
        });
        let has_ring0 = files.iter().any(|f| {
            f.ends_with("Cargo.toml")
                || f.starts_with("crates/contextra-types/")
                || f.starts_with("crates/contextra-ports/")
                || f.starts_with("crates/contextra-vector/")
                || f.starts_with("crates/contextra-text/")
                || f.starts_with("crates/contextra-graph/")
                || f.starts_with("crates/contextra-rank/")
                || f.starts_with("crates/contextra-adapt/")
                || f.starts_with("crates/contextra-crypto/")
                || f.starts_with("crates/contextra-wire/")
                || f.starts_with("crates/contextra-sys/")
                || f.starts_with("crates/contextra-simd/")
                || f.starts_with("crates/contextra-core/")
                || f.starts_with("crates/contextra-avv-generator/")
                || f.starts_with("crates/contextra-audit-export/")
        });
        let has_caps = files
            .iter()
            .any(|f| f == "capabilities.toml" || f.ends_with("Cargo.toml"));

        (has_unsafe, has_ring0, has_caps)
    } else {
        (true, true, true)
    };

    let strict_flag = if strict { vec!["--strict"] } else { vec![] };

    let items: Vec<(&str, Vec<&str>, bool, &str)> = vec![
        ("check-dag", vec![], true, ""),
        ("check-ring-layering", strict_flag.clone(), true, ""),
        ("check-vetoes", vec![], true, ""),
        ("check-duplicate-symbols", vec![], true, ""),
        (
            "check-unsafe-islands",
            strict_flag,
            trigger_unsafe,
            "no path match",
        ),
        (
            "check-ring0-async-purity",
            vec![],
            trigger_purity,
            "no path match",
        ),
        (
            "check-ring-capabilities-consistency",
            vec![],
            trigger_caps,
            "no path match",
        ),
    ];

    let mut results = Vec::new();
    let mut overall_fail = false;

    for (name, extra_flags, active, skip_reason) in items {
        if !active {
            results.push(ConstituentResult {
                name: name.to_string(),
                status: format!("SKIP({skip_reason})"),
                code: 0,
                summary: format!("Skipped: {skip_reason}"),
            });
            continue;
        }

        let mut sub_args: Vec<String> = extra_flags.iter().map(|s| s.to_string()).collect();
        sub_args.extend(vec![
            "--root".to_string(),
            root_dir.to_string_lossy().to_string(),
            "--base".to_string(),
            base_rev.clone(),
            "--head".to_string(),
            head_rev.clone(),
        ]);

        let (code, stdout, stderr) = execute_xtask_subcmd(name, &sub_args, &root_dir);

        let status_str = if code == 0 {
            "OK".to_string()
        } else {
            overall_fail = true;
            "FAIL".to_string()
        };

        let summary = parse_summary_from_output(&stdout, &stderr, code);

        results.push(ConstituentResult {
            name: name.to_string(),
            status: status_str,
            code,
            summary,
        });
    }

    let final_status = if overall_fail { "fail" } else { "pass" };
    let final_summary = if overall_fail {
        "gate-arch: Mindestens ein Bestandteil ist fehlgeschlagen."
    } else {
        "gate-arch: Alle Bestandteile erfolgreich bestanden."
    };

    if is_json {
        let gate_res = GateArchResult {
            gate: "gate-arch".to_string(),
            status: final_status.to_string(),
            summary: final_summary.to_string(),
            constituents: results,
        };
        if let Ok(json_str) = serde_json::to_string(&gate_res) {
            println!("{}", json_str);
        }
    } else {
        println!("=== Gate gate-arch: {} ===", final_status.to_uppercase());
        for r in &results {
            println!("{:<20} {}", r.status, r.name);
        }
    }

    if overall_fail {
        1
    } else {
        0
    }
}

fn execute_xtask_subcmd(cmd: &str, args: &[String], root: &Path) -> (i32, String, String) {
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if out.status.success() {
            let ws_root = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let xtask_bin = PathBuf::from(&ws_root).join("target/debug/xtask");
            if xtask_bin.exists() {
                let mut command = Command::new(&xtask_bin);
                command.arg(cmd);
                command.args(args);
                command.current_dir(root);
                if let Ok(res) = command.output() {
                    let stdout = String::from_utf8_lossy(&res.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&res.stderr).to_string();
                    let code = res.status.code().unwrap_or(2);
                    return (code, stdout, stderr);
                }
            }
        }
    }

    if let Ok(exe) = env::current_exe() {
        let name_str = exe.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if name_str.starts_with("xtask") {
            let mut command = Command::new(&exe);
            command.arg(cmd);
            command.args(args);
            command.current_dir(root);
            if let Ok(res) = command.output() {
                let stdout = String::from_utf8_lossy(&res.stdout).to_string();
                let stderr = String::from_utf8_lossy(&res.stderr).to_string();
                let code = res.status.code().unwrap_or(2);
                return (code, stdout, stderr);
            }
        }
    }

    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = Command::new(cargo);
    command.args(["xtask", cmd]);
    command.args(args);
    command.current_dir(root);

    match command.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let code = out.status.code().unwrap_or(2);
            (code, stdout, stderr)
        }
        Err(e) => (
            2,
            "".to_string(),
            format!("Prozessaufruf fehlgeschlagen: {e}"),
        ),
    }
}

fn get_git_merge_base(root: &Path, rev1: &str, rev2: &str) -> Option<String> {
    let out = Command::new("git")
        .args(["merge-base", rev1, rev2])
        .current_dir(root)
        .output()
        .ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    None
}

fn get_changed_files(root: &Path, base: &str, head: &str) -> Option<Vec<String>> {
    let out = Command::new("git")
        .args(["diff", "--name-only", base, head])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut files: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    if let Ok(untracked) = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard"])
        .current_dir(root)
        .output()
    {
        if untracked.status.success() {
            for l in String::from_utf8_lossy(&untracked.stdout).lines() {
                let trimmed = l.trim().to_string();
                if !trimmed.is_empty() && !files.contains(&trimmed) {
                    files.push(trimmed);
                }
            }
        }
    }

    Some(files)
}

fn file_contains_unsafe(path: &Path) -> bool {
    if let Ok(content) = fs::read_to_string(path) {
        content.contains("unsafe")
    } else {
        false
    }
}

fn detect_repo_root() -> PathBuf {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(o) = out {
        if o.status.success() {
            let path_str = String::from_utf8_lossy(&o.stdout).trim().to_string();
            return PathBuf::from(path_str);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn parse_summary_from_output(stdout: &str, stderr: &str, code: i32) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(stdout) {
        if let Some(s) = val.get("summary").and_then(|v| v.as_str()) {
            return s.to_string();
        }
    }
    if code == 0 {
        "Bestanden".to_string()
    } else {
        let combined = format!("{stdout}\n{stderr}");
        combined
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("Fehlgeschlagen")
            .to_string()
    }
}
