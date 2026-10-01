//! Diff Budget Gate: Prüft das Diff gegen Budget-Grenzen aus der Task-Karte oder Standard-Schwellenwerte.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffBudgetFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiffBudgetGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<DiffBudgetFinding>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DiffBudgetCardBudget {
    pub files: Option<usize>,
    pub lines: Option<usize>,
    pub crates: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DiffBudgetCard {
    pub budget: Option<DiffBudgetCardBudget>,
}

pub fn run_diff_budget(args: &[String]) -> i32 {
    let mut root_dir = diff_budget_default_root();
    let mut base = String::new();
    let mut head = String::from("HEAD");
    let mut card_arg: Option<String> = None;
    let mut body_file: Option<String> = None;
    let mut json_output = false;

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
                    base = args[i + 1].clone();
                    i += 1;
                }
            }
            "--head" => {
                if i + 1 < args.len() {
                    head = args[i + 1].clone();
                    i += 1;
                }
            }
            "--card" => {
                if i + 1 < args.len() {
                    card_arg = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--body-file" => {
                if i + 1 < args.len() {
                    body_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--json" => {
                json_output = true;
            }
            _ => {}
        }
        i += 1;
    }

    if base.is_empty() {
        base = diff_budget_git_merge_base(&root_dir, &head);
    }

    // Load card if available
    let card_file_path = if let Some(ref c) = card_arg {
        Some(diff_budget_resolve_card_path(&root_dir, c))
    } else {
        let git_log = diff_budget_git_log_trailers(&root_dir, &base, &head);
        if let Some(id) = diff_budget_find_card_id(&git_log) {
            Some(diff_budget_resolve_card_path(&root_dir, &id))
        } else {
            let body_text = if let Some(ref bf) = body_file {
                fs::read_to_string(bf).unwrap_or_default()
            } else {
                env::var("PR_BODY").unwrap_or_default()
            };
            diff_budget_find_card_id(&body_text)
                .map(|id| diff_budget_resolve_card_path(&root_dir, &id))
        }
    };

    let card_budget = card_file_path
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|c| toml::from_str::<DiffBudgetCard>(&c).ok())
        .and_then(|c| c.budget);

    let max_files = card_budget.as_ref().and_then(|b| b.files).unwrap_or(15);
    let max_lines = card_budget.as_ref().and_then(|b| b.lines).unwrap_or(600);
    let max_crates = card_budget.as_ref().and_then(|b| b.crates).unwrap_or(2);

    let (file_changes, total_lines, crate_counts) =
        diff_budget_get_numstat(&root_dir, &base, &head);

    let mut findings = Vec::new();

    if file_changes.len() > max_files {
        findings.push(DiffBudgetFinding {
            id: "DB-001".to_string(),
            severity: "error".to_string(),
            file: "".to_string(),
            line: 0,
            message: format!(
                "Anzahl geänderter Dateien ({}) überschreitet Budget ({})",
                file_changes.len(),
                max_files
            ),
            fix: diff_budget_format_split_recommendation(&crate_counts),
        });
    }

    if total_lines > max_lines {
        findings.push(DiffBudgetFinding {
            id: "DB-002".to_string(),
            severity: "error".to_string(),
            file: "".to_string(),
            line: 0,
            message: format!(
                "Anzahl geänderter Zeilen ({}) überschreitet Budget ({})",
                total_lines, max_lines
            ),
            fix: diff_budget_format_split_recommendation(&crate_counts),
        });
    }

    if crate_counts.len() > max_crates {
        findings.push(DiffBudgetFinding {
            id: "DB-003".to_string(),
            severity: "error".to_string(),
            file: "".to_string(),
            line: 0,
            message: format!(
                "Anzahl berührter Crates ({}) überschreitet Budget ({})",
                crate_counts.len(),
                max_crates
            ),
            fix: diff_budget_format_split_recommendation(&crate_counts),
        });
    }

    if findings.is_empty() {
        let res = DiffBudgetGateResult {
            gate: "diff-budget".to_string(),
            status: "pass".to_string(),
            summary: format!(
                "Diff-Budget eingehalten: {} Dateien, {} Zeilen, {} Crates",
                file_changes.len(),
                total_lines,
                crate_counts.len()
            ),
            findings: vec![],
        };
        diff_budget_emit(res, json_output, 0)
    } else {
        let res = DiffBudgetGateResult {
            gate: "diff-budget".to_string(),
            status: "fail".to_string(),
            summary: format!("{} Budget-Überschreitung(en) gefunden", findings.len()),
            findings,
        };
        diff_budget_emit(res, json_output, 1)
    }
}

fn diff_budget_format_split_recommendation(crate_counts: &HashMap<String, usize>) -> String {
    let mut parts: Vec<_> = crate_counts.iter().collect();
    parts.sort_by(|a, b| b.1.cmp(a.1));
    let crate_str = parts
        .iter()
        .map(|(c, count)| format!("{} ({} Zeilen)", c, count))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "Empfehlung: Task teilen nach Crates. Betroffene Crates: {}",
        if crate_str.is_empty() {
            "keine"
        } else {
            &crate_str
        }
    )
}

fn diff_budget_emit(res: DiffBudgetGateResult, json_output: bool, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", serde_json::to_string(&res).unwrap_or_default());
    } else {
        println!("=== Gate diff-budget: {} ===", res.status);
        println!("{}", res.summary);
        for f in &res.findings {
            println!(
                "[{}] {}: {}\n  Fix: {}",
                f.severity.to_uppercase(),
                f.file,
                f.message,
                f.fix
            );
        }
    }
    exit_code
}

fn diff_budget_default_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return PathBuf::from(path);
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn diff_budget_git_merge_base(root: &Path, head: &str) -> String {
    let output = Command::new("git")
        .current_dir(root)
        .args(["merge-base", head, "origin/main"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    let output_head = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", &format!("{}^", head)])
        .output();
    if let Ok(out) = output_head {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    head.to_string()
}

fn diff_budget_git_log_trailers(root: &Path, base: &str, head: &str) -> String {
    let range = if base == head {
        head.to_string()
    } else {
        format!("{}..{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["log", &range])
        .output();
    if let Ok(out) = output {
        String::from_utf8_lossy(&out.stdout).to_string()
    } else {
        String::new()
    }
}

fn diff_budget_find_card_id(text: &str) -> Option<String> {
    let re = Regex::new(r"(?i)Task-Card:\s*([a-zA-Z0-9_\-\./]+)").ok()?;
    re.captures(text).map(|cap| cap[1].to_string())
}

fn diff_budget_resolve_card_path(root: &Path, id_or_path: &str) -> PathBuf {
    if id_or_path.ends_with(".toml") {
        root.join(id_or_path)
    } else {
        root.join(".jules/tasks")
            .join(format!("{}.toml", id_or_path))
    }
}

pub fn diff_budget_is_excluded(file: &str, root: &Path) -> bool {
    if file == "Cargo.lock" || file.ends_with("/Cargo.lock") {
        return true;
    }
    if file.starts_with("docs/generated/") || file.contains("/docs/generated/") {
        return true;
    }
    if file.ends_with(".snap") {
        return true;
    }
    // Check if file has @generated in first 5 lines
    let full_path = root.join(file);
    if let Ok(content) = fs::read_to_string(&full_path) {
        for line in content.lines().take(5) {
            if line.contains("@generated") {
                return true;
            }
        }
    }
    false
}

pub fn diff_budget_get_numstat(
    root: &Path,
    base: &str,
    head: &str,
) -> (HashSet<String>, usize, HashMap<String, usize>) {
    let range = if base == head {
        format!("{}^..{}", head, head)
    } else {
        format!("{}...{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "--numstat", &range])
        .output();

    let mut changed_files = HashSet::new();
    let mut total_lines = 0;
    let mut crate_counts: HashMap<String, usize> = HashMap::new();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let added: usize = parts[0].parse().unwrap_or(0);
                let removed: usize = parts[1].parse().unwrap_or(0);
                let file = parts[2].to_string();

                if diff_budget_is_excluded(&file, root) {
                    continue;
                }

                changed_files.insert(file.clone());
                let lines = added + removed;
                total_lines += lines;

                if file.starts_with("crates/") {
                    let segments: Vec<&str> = file.split('/').collect();
                    if segments.len() >= 2 {
                        let crate_name = segments[1].to_string();
                        *crate_counts.entry(crate_name).or_insert(0) += lines;
                    }
                }
            }
        }
    }

    (changed_files, total_lines, crate_counts)
}
