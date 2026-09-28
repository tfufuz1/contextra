//! Task-card management and validation gate for Jules agent tasks.

use chrono::Datelike;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct TaskCardBudget {
    pub files: u32,
    pub lines: u32,
    pub iterations: u32,
}

#[derive(Debug, Deserialize)]
pub struct TaskCard {
    pub id: String,
    pub title: String,
    pub crate_name: Option<String>,
    #[serde(rename = "crate")]
    pub crate_field: Option<String>,
    pub tier: Option<u32>,
    pub risk: String,
    pub goal: String,
    pub non_goals: Vec<String>,
    pub scope: Vec<String>,
    pub forbidden: Vec<String>,
    pub invariants: Vec<String>,
    pub acceptance: Vec<String>,
    pub evidence_required: Vec<String>,
    pub budget: TaskCardBudget,
    pub protected_change: Option<String>,
}

impl TaskCard {
    pub fn get_crate(&self) -> &str {
        if let Some(ref c) = self.crate_name {
            c.as_str()
        } else if let Some(ref c) = self.crate_field {
            c.as_str()
        } else {
            ""
        }
    }
}

pub struct TaskCardFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub fix: String,
}

pub fn task_card_find_root(start: &Path) -> PathBuf {
    let mut current = start.to_path_buf();
    loop {
        if current.join("Cargo.toml").exists() && current.join("capabilities.toml").exists() {
            return current;
        }
        if !current.pop() {
            return start.to_path_buf();
        }
    }
}

pub fn task_card_get_protected_paths() -> Vec<&'static str> {
    vec![
        ".github/**",
        "xtask/**",
        "capabilities.toml",
        "AGENTS.md",
        "rust-toolchain.toml",
        "deny.toml",
        ".jules/setup/**",
    ]
}

pub fn task_card_run_new(root: &Path, crate_name: &str, title: &str) -> Result<PathBuf, String> {
    let tasks_dir = root.join(".jules/tasks");
    fs::create_dir_all(&tasks_dir).map_err(|e| e.to_string())?;

    let year = chrono::Local::now().year();
    let mut max_seq = 0;

    if let Ok(entries) = fs::read_dir(&tasks_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("T-{}-", year)) && name.ends_with(".toml") {
                let seq_str = &name[7..name.len() - 5];
                if let Ok(seq) = seq_str.parse::<u32>() {
                    if seq > max_seq {
                        max_seq = seq;
                    }
                }
            }
        }
    }

    let next_id = format!("T-{}-{:04}", year, max_seq + 1);
    let template_path = root.join(".jules/tasks/_TEMPLATE.toml");
    let mut template_content = if template_path.exists() {
        fs::read_to_string(&template_path).map_err(|e| e.to_string())?
    } else {
        r#"id = "{ID}"
title = "{TITLE}"
crate = "{CRATE}"
tier = 1
risk = "none"
goal = "Goal"
non_goals = []
scope = ["crates/{CRATE}/src/**"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Zero-Panic"]
acceptance = ["cargo test -p {CRATE}"]
evidence_required = ["Test evidence"]

[budget]
files = 5
lines = 200
iterations = 5
"#
        .to_string()
    };

    template_content = template_content.replace("T-2026-0001", &next_id);
    template_content = template_content.replace("Kurzer, prägnanter Titel der Aufgabe", title);
    template_content = template_content.replace("contextra-core", crate_name);

    let card_path = tasks_dir.join(format!("{}.toml", next_id));
    fs::write(&card_path, template_content).map_err(|e| e.to_string())?;

    Ok(card_path)
}

pub fn task_card_lint_file(root: &Path, file_path: &Path) -> Vec<TaskCardFinding> {
    let mut findings = Vec::new();
    let file_str = file_path.to_string_lossy().to_string();

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            findings.push(TaskCardFinding {
                id: "file-read-error".to_string(),
                severity: "error".to_string(),
                file: file_str,
                line: 0,
                message: format!("Datei konnte nicht gelesen werden: {}", e),
                fix: "Pfad prüfen".to_string(),
            });
            return findings;
        }
    };

    let card: TaskCard = match toml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            findings.push(TaskCardFinding {
                id: "toml-parse-error".to_string(),
                severity: "error".to_string(),
                file: file_str,
                line: 0,
                message: format!("TOML-Syntaxfehler: {}", e),
                fix: "TOML-Syntax korrigieren".to_string(),
            });
            return findings;
        }
    };

    if !card.id.starts_with("T-") {
        findings.push(TaskCardFinding {
            id: "invalid-id-format".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: format!("ID '{}' muss mit 'T-' beginnen", card.id),
            fix: "ID-Format T-YYYY-NNNN verwenden".to_string(),
        });
    }

    if card.title.trim().is_empty() {
        findings.push(TaskCardFinding {
            id: "empty-title".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: "Titel darf nicht leer sein".to_string(),
            fix: "Sinnvollen Titel angeben".to_string(),
        });
    }

    if card.goal.trim().is_empty() {
        findings.push(TaskCardFinding {
            id: "empty-goal".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: "Goal darf nicht leer sein".to_string(),
            fix: "Zielbeschreibung eintragen".to_string(),
        });
    }

    if card.invariants.is_empty() {
        findings.push(TaskCardFinding {
            id: "empty-invariants".to_string(),
            severity: "warn".to_string(),
            file: file_str.clone(),
            line: 0,
            message: "Keine Invarianten definiert".to_string(),
            fix: "Mindestens eine Invariante eintragen".to_string(),
        });
    }

    let _ = &card.non_goals;

    let krate = card.get_crate();
    if krate.is_empty() {
        findings.push(TaskCardFinding {
            id: "missing-crate".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: "Feld 'crate' fehlt oder ist leer".to_string(),
            fix: "Crate-Namen eintragen".to_string(),
        });
    } else {
        let caps_path = root.join("capabilities.toml");
        if caps_path.exists() {
            if let Ok(caps_content) = fs::read_to_string(&caps_path) {
                let section = format!("[crates.{}]", krate);
                if !caps_content.contains(&section) {
                    findings.push(TaskCardFinding {
                        id: "invalid-crate".to_string(),
                        severity: "error".to_string(),
                        file: file_str.clone(),
                        line: 0,
                        message: format!("Crate '{}' nicht in capabilities.toml gefunden", krate),
                        fix: "Echten Crate-Namen aus capabilities.toml wählen".to_string(),
                    });
                }
            }
        }

        let prompter_path = root.join(".jules/prompter-tiers.toml");
        if prompter_path.exists() {
            if let Ok(prompter_content) = fs::read_to_string(&prompter_path) {
                let section = format!("[crate_overrides.{}]", krate);
                if prompter_content.contains(&section) {
                    if let Some(tier_val) = card.tier {
                        let tier_str = format!("tier = \"{}\"", tier_val);
                        if !prompter_content.contains(&tier_str) {
                            findings.push(TaskCardFinding {
                                id: "tier-mismatch".to_string(),
                                severity: "warn".to_string(),
                                file: file_str.clone(),
                                line: 0,
                                message: format!(
                                    "Tier {} weicht von prompter-tiers.toml ab",
                                    tier_val
                                ),
                                fix: "Tier an prompter-tiers.toml anpassen".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    if card.scope.is_empty() {
        findings.push(TaskCardFinding {
            id: "empty-scope".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: "Scope darf nicht leer sein".to_string(),
            fix: "Erlaubte Pfade eintragen".to_string(),
        });
    }

    for pattern in &card.scope {
        if pattern == "**" || pattern == "crates/**" || pattern == "crates/*" {
            findings.push(TaskCardFinding {
                id: "full-glob-scope".to_string(),
                severity: "error".to_string(),
                file: file_str.clone(),
                line: 0,
                message: format!("Scope-Muster '{}' ist als Vollglob zu breit", pattern),
                fix: "Spezifischeres Pfadmuster angeben".to_string(),
            });
        }
    }

    let scope_set: HashSet<_> = card.scope.iter().collect();
    for f in &card.forbidden {
        if scope_set.contains(f) {
            findings.push(TaskCardFinding {
                id: "scope-forbidden-overlap".to_string(),
                severity: "error".to_string(),
                file: file_str.clone(),
                line: 0,
                message: format!("Pfad '{}' ist sowohl in scope als auch in forbidden", f),
                fix: "Konflikt zwischen scope und forbidden auflösen".to_string(),
            });
        }
    }

    if card.protected_change.is_none() {
        let forbidden_set: HashSet<_> = card.forbidden.iter().map(|s| s.as_str()).collect();
        for protected in task_card_get_protected_paths() {
            if !forbidden_set.contains(protected) {
                findings.push(TaskCardFinding {
                    id: "missing-protected-path".to_string(),
                    severity: "error".to_string(),
                    file: file_str.clone(),
                    line: 0,
                    message: format!("Geschützter Pfad '{}' fehlt in forbidden", protected),
                    fix: format!(
                        "'{}' zu forbidden hinzufügen oder protected_change setzen",
                        protected
                    ),
                });
            }
        }
    }

    if card.acceptance.is_empty() {
        findings.push(TaskCardFinding {
            id: "empty-acceptance".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: "Acceptance-Befehle dürfen nicht leer sein".to_string(),
            fix: "Mindestens einen Akzeptanzbefehl hinzufügen".to_string(),
        });
    }

    for cmd in &card.acceptance {
        if !cmd.starts_with("cargo ") && !cmd.starts_with("just ") {
            findings.push(TaskCardFinding {
                id: "invalid-acceptance-cmd".to_string(),
                severity: "error".to_string(),
                file: file_str.clone(),
                line: 0,
                message: format!(
                    "Akzeptanzbefehl '{}' muss mit 'cargo ' oder 'just ' beginnen",
                    cmd
                ),
                fix: "Befehl mit cargo oder just anfangen lassen".to_string(),
            });
        }
    }

    if card.risk != "none" && card.evidence_required.is_empty() {
        findings.push(TaskCardFinding {
            id: "missing-evidence".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: format!("Bei risk='{}' ist evidence_required Pflicht", card.risk),
            fix: "Beleganforderungen eintragen".to_string(),
        });
    }

    if card.budget.files > 25 {
        findings.push(TaskCardFinding {
            id: "budget-files-exceeded".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: format!(
                "Budget files ({}) übersteigt Maximum (25)",
                card.budget.files
            ),
            fix: "files <= 25 setzen".to_string(),
        });
    }
    if card.budget.lines > 1000 {
        findings.push(TaskCardFinding {
            id: "budget-lines-exceeded".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: format!(
                "Budget lines ({}) übersteigt Maximum (1000)",
                card.budget.lines
            ),
            fix: "lines <= 1000 setzen".to_string(),
        });
    }
    if card.budget.iterations > 12 {
        findings.push(TaskCardFinding {
            id: "budget-iterations-exceeded".to_string(),
            severity: "error".to_string(),
            file: file_str.clone(),
            line: 0,
            message: format!(
                "Budget iterations ({}) übersteigt Maximum (12)",
                card.budget.iterations
            ),
            fix: "iterations <= 12 setzen".to_string(),
        });
    }

    findings
}

pub fn task_card_lint_pr(root: &Path) -> (&'static str, Vec<TaskCardFinding>) {
    let mut findings = Vec::new();
    let labels = std::env::var("PR_LABELS").unwrap_or_default();
    let has_jules_label = labels.split(',').any(|l| l.trim() == "jules");

    if !has_jules_label {
        return ("not_applicable", Vec::new());
    }

    let card_id_opt = std::env::var("PR_BODY").ok().and_then(|body| {
        for line in body.lines() {
            if line.contains("Task-Karte:") || line.contains("Task:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    if part.starts_with("T-") {
                        return Some(part.to_string());
                    }
                }
            }
        }
        None
    });

    if let Some(card_id) = card_id_opt {
        let card_path = root.join(format!(".jules/tasks/{}.toml", card_id));
        if !card_path.exists() {
            findings.push(TaskCardFinding {
                id: "missing-pr-card-file".to_string(),
                severity: "error".to_string(),
                file: card_path.to_string_lossy().to_string(),
                line: 0,
                message: format!("Referenzierte Task-Karte '{}' existiert nicht", card_id),
                fix: "Task-Karte anlegen oder PR-Text korrigieren".to_string(),
            });
            return ("fail", findings);
        }
        let file_findings = task_card_lint_file(root, &card_path);
        let has_errors = file_findings.iter().any(|f| f.severity == "error");
        findings.extend(file_findings);
        if has_errors {
            ("fail", findings)
        } else {
            ("pass", findings)
        }
    } else {
        findings.push(TaskCardFinding {
            id: "adhoc-task-without-card".to_string(),
            severity: "error".to_string(),
            file: "PR_BODY".to_string(),
            line: 0,
            message: "Ad-hoc-Task ohne Karte: PR hat Label 'jules', aber keine referenzierte Karte"
                .to_string(),
            fix: "Task-Karte im PR-Text mit T-YYYY-NNNN referenzieren".to_string(),
        });
        ("fail", findings)
    }
}

pub fn run_task_card(args: &[String]) -> i32 {
    let mut root_dir: Option<PathBuf> = None;
    let mut json_output = false;
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--root" && i + 1 < args.len() {
            root_dir = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i] == "--json" {
            json_output = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    let root = root_dir.unwrap_or_else(|| task_card_find_root(Path::new(".")));

    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    match cmd {
        "new" => {
            let mut crate_name = "contextra-core".to_string();
            let mut title = "Neue Task".to_string();
            let mut idx = 1;
            while idx < args.len() {
                if args[idx] == "--crate" && idx + 1 < args.len() {
                    crate_name = args[idx + 1].clone();
                    idx += 2;
                } else if args[idx] == "--title" && idx + 1 < args.len() {
                    title = args[idx + 1].clone();
                    idx += 2;
                } else {
                    idx += 1;
                }
            }

            match task_card_run_new(&root, &crate_name, &title) {
                Ok(path) => {
                    if json_output {
                        let json = serde_json::json!({
                            "gate": "task-card",
                            "status": "pass",
                            "summary": format!("Task-Karte erzeugt unter {}", path.display()),
                            "findings": []
                        });
                        println!("{}", json);
                    } else {
                        println!("Task-Karte erzeugt unter {}", path.display());
                    }
                    0
                }
                Err(e) => {
                    if json_output {
                        let json = serde_json::json!({
                            "gate": "task-card",
                            "status": "error",
                            "summary": format!("Fehler beim Erzeugen: {}", e),
                            "findings": []
                        });
                        println!("{}", json);
                    } else {
                        eprintln!("🔴 Fehler beim Erzeugen der Task-Karte: {}", e);
                    }
                    2
                }
            }
        }
        "lint" => {
            let is_pr = args.iter().any(|a| a == "--pr");
            let (status, findings) = if is_pr {
                task_card_lint_pr(&root)
            } else {
                let file_arg = args.iter().skip(1).find(|a| !a.starts_with("--"));
                if let Some(file_path) = file_arg {
                    let path = Path::new(file_path);
                    let file_findings = task_card_lint_file(&root, path);
                    let ok = !file_findings.iter().any(|f| f.severity == "error");
                    let st = if ok { "pass" } else { "fail" };
                    (st, file_findings)
                } else {
                    (
                        "fail",
                        vec![TaskCardFinding {
                            id: "missing-file-arg".to_string(),
                            severity: "error".to_string(),
                            file: "".to_string(),
                            line: 0,
                            message: "Keine Datei zum Linten angegeben".to_string(),
                            fix: "task-card lint <datei> aufrufen".to_string(),
                        }],
                    )
                }
            };

            let json_findings: Vec<serde_json::Value> = findings
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "id": f.id,
                        "severity": f.severity,
                        "file": f.file,
                        "line": f.line,
                        "message": f.message,
                        "fix": f.fix
                    })
                })
                .collect();

            if json_output {
                let json = serde_json::json!({
                    "gate": "task-card",
                    "status": status,
                    "summary": format!("Task-Card Linting: {}", status),
                    "findings": json_findings
                });
                println!("{}", json);
            } else if status == "pass" || status == "not_applicable" {
                println!("🟢 Task-Card Lint: {}", status);
            } else {
                println!("🔴 Task-Card Lint fehlgeschlagen:");
                for f in &findings {
                    println!(
                        "  - [{}] {}: {} (Fix: {})",
                        f.severity, f.id, f.message, f.fix
                    );
                }
            }

            if status == "pass" || status == "not_applicable" {
                0
            } else {
                1
            }
        }
        _ => {
            eprintln!(
                "Verwendung: task-card new --crate <c> --title <t> | task-card lint <datei|--pr>"
            );
            2
        }
    }
}
