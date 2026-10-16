//! Aggregate Gate Guard (scope-guard, protected-paths, gate-weakening, diff-budget).

use serde::{Deserialize, Serialize};
use std::env;
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
pub struct GateGuardResult {
    pub gate: String,
    pub status: String, // "pass" or "fail"
    pub summary: String,
    pub constituents: Vec<ConstituentResult>,
}

pub fn run_gate_guard(args: &[String]) -> i32 {
    let mut root_dir = detect_repo_root();
    let mut base_rev = "origin/main".to_string();
    let mut head_rev = "HEAD".to_string();
    let mut card_path: Option<String> = None;
    let mut card_mode: Option<String> = None;
    let mut is_json = false;
    let mut run_all = false;

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
            "--card" => {
                if i + 1 < args.len() {
                    card_path = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--card-mode" => {
                if i + 1 < args.len() {
                    card_mode = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--all" => {
                run_all = true;
            }
            "--changed" => {
                run_all = false;
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

    let constituents = vec![
        "scope-guard",
        "protected-paths",
        "gate-weakening",
        "diff-budget",
    ];

    let mut results = Vec::new();
    let mut overall_fail = false;

    let default_card_mode = if card_path.is_some() { "required" } else { "optional" };
    let cm_val = card_mode.as_deref().unwrap_or(default_card_mode);

    for name in constituents {
        let mut sub_args = vec![
            "--root".to_string(),
            root_dir.to_string_lossy().to_string(),
            "--base".to_string(),
            base_rev.clone(),
            "--head".to_string(),
            head_rev.clone(),
            "--card-mode".to_string(),
            cm_val.to_string(),
            "--json".to_string(),
        ];

        if let Some(ref card) = card_path {
            sub_args.push("--card".to_string());
            sub_args.push(card.clone());
        }

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
        "gate-guard: Mindestens ein Bestandteil ist fehlgeschlagen."
    } else {
        "gate-guard: Alle Bestandteile erfolgreich bestanden."
    };

    if is_json {
        let gate_res = GateGuardResult {
            gate: "gate-guard".to_string(),
            status: final_status.to_string(),
            summary: final_summary.to_string(),
            constituents: results,
        };
        if let Ok(json_str) = serde_json::to_string(&gate_res) {
            println!("{}", json_str);
        }
    } else {
        println!("=== Gate gate-guard: {} ===", final_status.to_uppercase());
        for r in &results {
            println!("{:<20} {}", r.status, r.name);
        }
    }

    if overall_fail { 1 } else { 0 }
}

fn execute_xtask_subcmd(cmd: &str, args: &[String], root: &Path) -> (i32, String, String) {
    if let Ok(out) = Command::new("git").args(["rev-parse", "--show-toplevel"]).output() {
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
        Err(e) => (2, "".to_string(), format!("Prozessaufruf fehlgeschlagen: {e}")),
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
