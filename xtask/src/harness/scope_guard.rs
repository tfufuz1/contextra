//! Scope Guard Gate: Prüft ob geänderte Dateien im zugewiesenen Scope liegen und keine forbidden Pfade berühren.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScopeGuardFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScopeGuardGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<ScopeGuardFinding>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ScopeGuardCard {
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub forbidden: Vec<String>,
}

pub fn scope_guard_glob_to_regex(glob: &str) -> String {
    let mut reg = String::from("^");
    let chars: Vec<char> = glob.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' {
            if i + 1 < chars.len() && chars[i + 1] == '*' {
                if i + 2 < chars.len() && chars[i + 2] == '/' {
                    reg.push_str("(?:.*/)?");
                    i += 3;
                } else {
                    reg.push_str(".*");
                    i += 2;
                }
            } else {
                reg.push_str("[^/]*");
                i += 1;
            }
        } else if chars[i] == '?' {
            reg.push_str("[^/]");
            i += 1;
        } else if r".+()[]{}^$\|".contains(chars[i]) {
            reg.push('\\');
            reg.push(chars[i]);
            i += 1;
        } else {
            reg.push(chars[i]);
            i += 1;
        }
    }
    reg.push('$');
    reg
}

pub fn scope_guard_matches_glob(file: &str, glob: &str) -> bool {
    let pattern = scope_guard_glob_to_regex(glob);
    if let Ok(re) = Regex::new(&pattern) {
        re.is_match(file)
    } else {
        false
    }
}

pub fn scope_guard_find_card_id(text: &str) -> Option<String> {
    let re = Regex::new(r"(?i)Task-Card:\s*([a-zA-Z0-9_\-\./]+)").ok()?;
    re.captures(text).map(|cap| cap[1].to_string())
}

pub fn scope_guard_resolve_card_path(root: &Path, id_or_path: &str) -> PathBuf {
    if id_or_path.ends_with(".toml") {
        root.join(id_or_path)
    } else {
        root.join(".jules/tasks").join(format!("{}.toml", id_or_path))
    }
}

pub fn run_scope_guard(args: &[String]) -> i32 {
    let mut root_dir = scope_guard_default_root();
    let mut base = String::new();
    let mut head = String::from("HEAD");
    let mut card_arg: Option<String> = None;
    let mut card_mode = String::from("required");
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
            "--card-mode" => {
                if i + 1 < args.len() {
                    card_mode = args[i + 1].clone();
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
        base = scope_guard_git_merge_base(&root_dir, &head);
    }

    let card_file_path = if let Some(ref c) = card_arg {
        Some(scope_guard_resolve_card_path(&root_dir, c))
    } else {
        // Look in git trailers
        let git_log = scope_guard_git_log_trailers(&root_dir, &base, &head);
        if let Some(id) = scope_guard_find_card_id(&git_log) {
            Some(scope_guard_resolve_card_path(&root_dir, &id))
        } else {
            // Look in PR body
            let body_text = if let Some(ref bf) = body_file {
                fs::read_to_string(bf).unwrap_or_default()
            } else {
                env::var("PR_BODY").unwrap_or_default()
            };
            scope_guard_find_card_id(&body_text)
                .map(|id| scope_guard_resolve_card_path(&root_dir, &id))
        }
    };

    let card = match card_file_path {
        Some(ref path) => match fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<ScopeGuardCard>(&content) {
                Ok(parsed) => Some(parsed),
                Err(e) => {
                    let res = ScopeGuardGateResult {
                        gate: "scope-guard".to_string(),
                        status: "error".to_string(),
                        summary: format!("Fehler beim Parsen der Task-Karte {:?}: {}", path, e),
                        findings: vec![],
                    };
                    return scope_guard_emit(res, json_output, 2);
                }
            },
            Err(_) => None,
        },
        None => None,
    };

    let card = match card {
        Some(c) => c,
        None => {
            if card_mode == "optional" {
                let res = ScopeGuardGateResult {
                    gate: "scope-guard".to_string(),
                    status: "not_applicable".to_string(),
                    summary: "Keine Task-Karte gefunden (Modus optional)".to_string(),
                    findings: vec![ScopeGuardFinding {
                        id: "SG-000".to_string(),
                        severity: "warn".to_string(),
                        file: "".to_string(),
                        line: 0,
                        message: "Keine Task-Karte gefunden".to_string(),
                        fix: "Task-Karte in .jules/tasks/ hinterlegen oder Task-Card Trailer angeben".to_string(),
                    }],
                };
                return scope_guard_emit(res, json_output, 0);
            } else {
                let res = ScopeGuardGateResult {
                    gate: "scope-guard".to_string(),
                    status: "error".to_string(),
                    summary: "Keine Task-Karte gefunden".to_string(),
                    findings: vec![ScopeGuardFinding {
                        id: "SG-000".to_string(),
                        severity: "error".to_string(),
                        file: "".to_string(),
                        line: 0,
                        message: "Keine Task-Karte gefunden".to_string(),
                        fix: "Task-Karte in .jules/tasks/ hinterlegen oder --card/Trailer angeben".to_string(),
                    }],
                };
                return scope_guard_emit(res, json_output, 2);
            }
        }
    };

    let changed_files = scope_guard_get_changed_files(&root_dir, &base, &head);

    let mut findings = Vec::new();
    for file in &changed_files {
        // Check forbidden first
        let is_forbidden = card.forbidden.iter().any(|g| scope_guard_matches_glob(file, g));
        let is_in_scope = card.scope.iter().any(|g| scope_guard_matches_glob(file, g));

        if is_forbidden {
            findings.push(ScopeGuardFinding {
                id: "SG-001".to_string(),
                severity: "error".to_string(),
                file: file.clone(),
                line: 0,
                message: format!("Datei '{}' ist explizit forbidden in der Task-Karte", file),
                fix: "Unter 'Out-of-scope Findings' im PR melden statt ändern".to_string(),
            });
        } else if !is_in_scope {
            findings.push(ScopeGuardFinding {
                id: "SG-002".to_string(),
                severity: "error".to_string(),
                file: file.clone(),
                line: 0,
                message: format!("Datei '{}' liegt außerhalb des Scopes der Task-Karte", file),
                fix: "Unter 'Out-of-scope Findings' im PR melden statt ändern".to_string(),
            });
        }
    }

    if findings.is_empty() {
        let res = ScopeGuardGateResult {
            gate: "scope-guard".to_string(),
            status: "pass".to_string(),
            summary: format!("Alle {} geänderten Dateien liegen im Scope", changed_files.len()),
            findings: vec![],
        };
        scope_guard_emit(res, json_output, 0)
    } else {
        let res = ScopeGuardGateResult {
            gate: "scope-guard".to_string(),
            status: "fail".to_string(),
            summary: format!("{} Scope-Verletzung(en) gefunden", findings.len()),
            findings,
        };
        scope_guard_emit(res, json_output, 1)
    }
}

fn scope_guard_emit(res: ScopeGuardGateResult, json_output: bool, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", serde_json::to_string(&res).unwrap_or_default());
    } else {
        println!("=== Gate scope-guard: {} ===", res.status);
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

fn scope_guard_default_root() -> PathBuf {
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

fn scope_guard_git_merge_base(root: &Path, head: &str) -> String {
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

fn scope_guard_git_log_trailers(root: &Path, base: &str, head: &str) -> String {
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

fn scope_guard_get_changed_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let range = if base == head {
        format!("{}^..{}", head, head)
    } else {
        format!("{}...{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-status", "-z", "--find-renames", &range])
        .output();

    let mut files = Vec::new();
    if let Ok(out) = output {
        let stdout = out.stdout;
        let parts: Vec<&[u8]> = stdout.split(|&b| b == 0).collect();
        let mut idx = 0;
        while idx < parts.len() {
            if parts[idx].is_empty() {
                idx += 1;
                continue;
            }
            let status_str = String::from_utf8_lossy(parts[idx]);
            let status_code = status_str.chars().next().unwrap_or(' ');
            idx += 1;
            if status_code == 'R' || status_code == 'C' {
                if idx < parts.len() {
                    let old_path = String::from_utf8_lossy(parts[idx]).to_string();
                    if !old_path.is_empty() {
                        files.push(old_path);
                    }
                    idx += 1;
                }
                if idx < parts.len() {
                    let new_path = String::from_utf8_lossy(parts[idx]).to_string();
                    if !new_path.is_empty() {
                        files.push(new_path);
                    }
                    idx += 1;
                }
            } else if idx < parts.len() {
                let file_path = String::from_utf8_lossy(parts[idx]).to_string();
                if !file_path.is_empty() {
                    files.push(file_path);
                }
                idx += 1;
            }
        }
    }
    files.sort();
    files.dedup();
    files
}
