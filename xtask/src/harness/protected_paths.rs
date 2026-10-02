//! Module enforcing protected paths restrictions on changed files.

use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct ProtectedPathsConfig {
    #[serde(default)]
    pub protected: Vec<ProtectedPathRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProtectedPathRule {
    pub glob: String,
    pub reason: String,
}

pub fn protected_paths_glob_to_regex(glob: &str) -> Result<Regex, String> {
    let mut regex_str = String::from("^");
    let chars: Vec<char> = glob.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    if i + 2 < chars.len() && chars[i + 2] == '/' {
                        regex_str.push_str("(?:.*/)?");
                        i += 3;
                    } else {
                        regex_str.push_str(".*");
                        i += 2;
                    }
                } else {
                    regex_str.push_str("[^/]*");
                    i += 1;
                }
            }
            '?' => {
                regex_str.push_str("[^/]");
                i += 1;
            }
            '.' | '(' | ')' | '+' | '|' | '^' | '$' | '@' | '%' | '{' | '}' | '[' | ']' | '\\' => {
                regex_str.push('\\');
                regex_str.push(chars[i]);
                i += 1;
            }
            c => {
                regex_str.push(c);
                i += 1;
            }
        }
    }
    regex_str.push('$');
    Regex::new(&regex_str).map_err(|e| format!("Fehler beim Compilieren des Globs '{glob}': {e}"))
}

pub fn protected_paths_parse_args(args: &[String]) -> (String, String, String, bool) {
    let mut root = String::new();
    let mut base = String::new();
    let mut head = String::from("HEAD");
    let mut json = false;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root = args[idx + 1].clone();
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            "--base" => {
                if idx + 1 < args.len() {
                    base = args[idx + 1].clone();
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            "--head" => {
                if idx + 1 < args.len() {
                    head = args[idx + 1].clone();
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            "--json" => {
                json = true;
                idx += 1;
            }
            _ => {
                idx += 1;
            }
        }
    }

    if root.is_empty() {
        if let Ok(out) = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
        {
            if out.status.success() {
                root = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
    }

    if base.is_empty() {
        let work_dir = if root.is_empty() { "." } else { &root };
        if let Ok(out) = Command::new("git")
            .current_dir(work_dir)
            .args(["merge-base", "HEAD", "origin/main"])
            .output()
        {
            if out.status.success() {
                base = String::from_utf8_lossy(&out.stdout).trim().to_string();
            }
        }
        if base.is_empty() {
            base = String::from("HEAD~1");
        }
    }

    (root, base, head, json)
}

pub fn check_shallow_repository(root: &str) -> Result<bool, String> {
    let work_dir = if root.is_empty() { "." } else { root };
    let output = Command::new("git")
        .current_dir(work_dir)
        .args(["rev-parse", "--is-shallow-repository"])
        .output()
        .map_err(|e| format!("git rev-parse --is-shallow-repository failed: {e}"))?;

    if output.status.success() {
        let is_shallow = String::from_utf8_lossy(&output.stdout).trim() == "true";
        Ok(is_shallow)
    } else {
        Err(format!(
            "git rev-parse --is-shallow-repository error: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

pub fn check_commit_valid(root: &str, rev: &str) -> Result<(), String> {
    let work_dir = if root.is_empty() { "." } else { root };
    let output = Command::new("git")
        .current_dir(work_dir)
        .args(["cat-file", "-e", &format!("{rev}^{{commit}}")])
        .output()
        .map_err(|e| format!("git cat-file check failed: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Basis-Commit '{rev}' konnte in Git nicht aufgelöst werden (kein origin/main oder ungültige Revision). Bitte 'git fetch origin main' ausführen."
        ))
    }
}

pub fn protected_paths_get_changed_files(
    root: &str,
    base: &str,
    head: &str,
) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "diff",
            "--name-status",
            "-z",
            "--find-renames",
            &format!("{base}...{head}"),
        ])
        .output()
        .map_err(|e| format!("git diff execution failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "git diff failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let mut files = Vec::new();
    let raw = output.stdout;
    let parts: Vec<&[u8]> = raw.split(|&b| b == 0).collect();

    let mut i = 0;
    while i < parts.len() {
        if parts[i].is_empty() {
            i += 1;
            continue;
        }
        let status_str = String::from_utf8_lossy(parts[i]);
        let status_code = status_str.chars().next().unwrap_or(' ');
        i += 1;

        if status_code == 'R' || status_code == 'C' {
            if i < parts.len() && !parts[i].is_empty() {
                files.push(String::from_utf8_lossy(parts[i]).to_string());
                i += 1;
            }
            if i < parts.len() && !parts[i].is_empty() {
                files.push(String::from_utf8_lossy(parts[i]).to_string());
                i += 1;
            }
        } else if i < parts.len() && !parts[i].is_empty() {
            files.push(String::from_utf8_lossy(parts[i]).to_string());
            i += 1;
        }
    }

    files.sort();
    files.dedup();
    Ok(files)
}

pub fn protected_paths_get_commits(
    root: &str,
    base: &str,
    head: &str,
) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["log", "--format=%H", &format!("{base}..{head}")])
        .output()
        .map_err(|e| format!("git log failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

pub fn protected_paths_get_commit_trailers(
    root: &str,
    commit_hash: &str,
) -> Result<Vec<(String, String)>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "log",
            "-1",
            "--format=%(trailers:key=Protected-Change)",
            commit_hash,
        ])
        .output()
        .map_err(|e| format!("git log trailer failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "git log trailer failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut trailers = Vec::new();
    for line in stdout.lines() {
        if let Some((k, v)) = line.split_once(':') {
            trailers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Ok(trailers)
}

pub fn parse_adr_status(content: &str) -> Result<String, String> {
    for line in content.lines() {
        let trimmed = line.trim();
        let stripped = if let Some(s) = trimmed.strip_prefix('*') {
            s.trim()
        } else if let Some(s) = trimmed.strip_prefix('-') {
            s.trim()
        } else {
            trimmed
        };
        let lower = stripped.to_lowercase();
        if lower.starts_with("status:")
            || lower.starts_with("**status**:")
            || lower.starts_with("**status:**")
        {
            if let Some((_, raw_val)) = stripped.split_once(':') {
                let status_val = raw_val
                    .trim()
                    .trim_matches(|c| c == '*' || c == '_' || c == '`')
                    .trim();
                return Ok(status_val.to_string());
            }
        }
    }
    Err("Kein 'Status:' Feld im Dateikopf der ADR gefunden.".to_string())
}

pub fn is_status_accepted(status_val: &str) -> bool {
    let lower = status_val.to_lowercase();
    let cleaned = lower.replace("✅", "").replace("✔", "").trim().to_string();

    cleaned == "accepted"
        || cleaned.starts_with("accepted ")
        || cleaned.starts_with("accepted(")
        || cleaned == "final"
        || cleaned.starts_with("final ")
        || cleaned.starts_with("final(")
}

pub fn validate_adr_on_base_commit(
    root: &str,
    base: &str,
    val: &str,
    changed_files: &[String],
) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["ls-tree", "-r", "--name-only", base, "docs/decisions"])
        .output()
        .map_err(|e| format!("git ls-tree failed for base {base}: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "ADR-Verzeichnis docs/decisions/ auf Basis-Commit '{base}' nicht gefunden."
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let prefix = format!("{val}-");
    let exact = format!("{val}.md");

    let mut matched_path: Option<String> = None;
    for line in stdout.lines() {
        let path_str = line.trim();
        if let Some(filename) = Path::new(path_str).file_name().and_then(|f| f.to_str()) {
            if filename == exact || (filename.starts_with(&prefix) && filename.ends_with(".md")) {
                matched_path = Some(path_str.to_string());
                break;
            }
        }
    }

    let adr_path = match matched_path {
        Some(p) => p,
        None => {
            return Err(format!(
                "ADR-Datei für '{val}' nicht auf Basis-Commit '{base}' in docs/decisions/ gefunden."
            ));
        }
    };

    if changed_files.contains(&adr_path) {
        return Err(format!(
            "ADR-Datei '{adr_path}' wurde im selben PR angelegt oder verändert (darf nicht im Diff basis...head enthalten sein)."
        ));
    }

    let cat_out = Command::new("git")
        .current_dir(root)
        .args(["cat-file", "-p", &format!("{base}:{adr_path}")])
        .output()
        .map_err(|e| format!("git cat-file failed for {base}:{adr_path}: {e}"))?;

    if !cat_out.status.success() {
        return Err(format!(
            "Konnte ADR-Inhalt für '{adr_path}' von Basis-Commit '{base}' nicht lesen."
        ));
    }

    let content = String::from_utf8_lossy(&cat_out.stdout);
    let status_val = parse_adr_status(&content)?;
    if !is_status_accepted(&status_val) {
        return Err(format!(
            "ADR '{val}' auf Basis-Commit hat Status '{status_val}', erforderlich ist 'accepted' oder 'final'."
        ));
    }

    Ok(adr_path)
}

pub fn run_protected_paths(args: &[String]) -> i32 {
    let (root, base, head, json) = protected_paths_parse_args(args);

    let work_dir = if root.is_empty() { "." } else { &root };

    // Check for shallow repository
    match check_shallow_repository(work_dir) {
        Ok(true) => {
            let msg = "Flaches Repository erkannt (shallow repository). Bitte führe 'git fetch --unshallow' bzw. 'git fetch origin main' aus.".to_string();
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "protected-paths",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [protected-paths]: {msg}");
            }
            return 2;
        }
        Ok(false) => {}
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "protected-paths",
                        "status": "error",
                        "summary": e,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [protected-paths]: {e}");
            }
            return 2;
        }
    }

    // Check base commit validity
    if let Err(e) = check_commit_valid(work_dir, &base) {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "protected-paths",
                    "status": "error",
                    "summary": e,
                    "findings": []
                })
            );
        } else {
            eprintln!("FEHLER [protected-paths]: {e}");
        }
        return 2;
    }

    let config_path = Path::new(work_dir).join("governance/protected-paths.toml");
    if !config_path.exists() {
        let msg = format!(
            "Konfigurationsdatei '{}' nicht gefunden.",
            config_path.display()
        );
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "protected-paths",
                    "status": "error",
                    "summary": msg,
                    "findings": []
                })
            );
        } else {
            eprintln!("FEHLER [protected-paths]: {msg}");
        }
        return 2;
    }

    let config_str = match fs::read_to_string(&config_path) {
        Ok(s) => s,
        Err(e) => {
            let msg = format!("Fehler beim Lesen von '{}': {e}", config_path.display());
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "protected-paths",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [protected-paths]: {msg}");
            }
            return 2;
        }
    };

    let config: ProtectedPathsConfig = match toml::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Fehler beim Parsen von '{}': {e}", config_path.display());
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "protected-paths",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [protected-paths]: {msg}");
            }
            return 2;
        }
    };

    // Verify self-protection of config and module
    let required_protected = [
        "governance/protected-paths.toml",
        "xtask/src/harness/protected_paths.rs",
    ];
    for req in required_protected {
        let mut covered = false;
        for rule in &config.protected {
            if let Ok(re) = protected_paths_glob_to_regex(&rule.glob) {
                if re.is_match(req) {
                    covered = true;
                    break;
                }
            }
        }
        if !covered {
            let msg = format!("Selbstschutz-Verstoß: Pflichtpfad '{req}' ist nicht in governance/protected-paths.toml erfasst!");
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "protected-paths",
                        "status": "fail",
                        "summary": msg,
                        "findings": [{
                            "id": "PP-SELF-PROTECT",
                            "severity": "error",
                            "file": "governance/protected-paths.toml",
                            "line": 0,
                            "message": msg,
                            "fix": format!("Füge eine Regel für '{req}' in governance/protected-paths.toml ein."),
                            "reason": "Pflicht-Selbstschutz-Pfad fehlt in Konfiguration."
                        }]
                    })
                );
            } else {
                eprintln!("VERSTOSS [protected-paths]: {msg}");
            }
            return 1;
        }
    }

    let changed_files = match protected_paths_get_changed_files(work_dir, &base, &head) {
        Ok(f) => f,
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "protected-paths",
                        "status": "error",
                        "summary": e,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [protected-paths]: {e}");
            }
            return 2;
        }
    };

    let mut violations = Vec::new();
    for file in &changed_files {
        for rule in &config.protected {
            if let Ok(re) = protected_paths_glob_to_regex(&rule.glob) {
                if re.is_match(file) {
                    violations.push((file.clone(), rule.glob.clone(), rule.reason.clone()));
                }
            }
        }
    }

    if violations.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "protected-paths",
                    "status": "pass",
                    "summary": "Keine geschützten Pfade verändert.",
                    "findings": []
                })
            );
        } else {
            println!("PASS [protected-paths]: Keine geschützten Pfade verändert.");
        }
        return 0;
    }

    // Check exception rules
    let commits = match protected_paths_get_commits(work_dir, &base, &head) {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "protected-paths",
                        "status": "error",
                        "summary": e,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [protected-paths]: {e}");
            }
            return 2;
        }
    };

    let pr_labels_env = env::var("PR_LABELS").unwrap_or_default();
    let pr_labels: Vec<&str> = pr_labels_env.split(',').map(|s| s.trim()).collect();
    let has_label = pr_labels.contains(&"protected-change");

    let mut all_commits_have_valid_adr = true;
    let mut missing_adr_reason = String::new();

    if commits.is_empty() {
        all_commits_have_valid_adr = false;
        missing_adr_reason = "Keine Commits im angegebenen Bereich gefunden.".to_string();
    } else {
        for commit in &commits {
            let trailers = match protected_paths_get_commit_trailers(work_dir, commit) {
                Ok(t) => t,
                Err(e) => {
                    all_commits_have_valid_adr = false;
                    missing_adr_reason = e;
                    break;
                }
            };

            let mut commit_valid = false;
            let mut commit_adr_found = false;
            for (key, val) in trailers {
                if key == "Protected-Change" {
                    commit_adr_found = true;
                    match validate_adr_on_base_commit(work_dir, &base, &val, &changed_files) {
                        Ok(_adr_path) => {
                            commit_valid = true;
                            break;
                        }
                        Err(err_msg) => {
                            missing_adr_reason = err_msg;
                        }
                    }
                }
            }

            if !commit_valid {
                all_commits_have_valid_adr = false;
                if !commit_adr_found && missing_adr_reason.is_empty() {
                    missing_adr_reason = format!(
                        "Commit {commit} besitzt keinen 'Protected-Change: ADR-NNN' Trailer."
                    );
                }
                break;
            }
        }
    }

    if all_commits_have_valid_adr && !has_label {
        missing_adr_reason = "PR-Label 'protected-change' fehlt in PR_LABELS.".to_string();
    }

    if all_commits_have_valid_adr && has_label {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "protected-paths",
                    "status": "pass",
                    "summary": "Geschützte Pfade wurden mit gültigem ADR-Trailer und PR-Label geändert.",
                    "findings": []
                })
            );
        } else {
            println!(
                "PASS [protected-paths]: Geschützte Pfade mit gültiger ADR-Ausnahme geändert."
            );
        }
        return 0;
    }

    let mut findings = Vec::new();
    for (file, glob, rule_reason) in &violations {
        let msg = format!("Geschützter Pfad '{file}' wurde verändert (Regel Glob: '{glob}'). Grund: {rule_reason}. Exception-Status: ADR-Trailer ok = {all_commits_have_valid_adr}, PR-Label 'protected-change' = {has_label}.");
        let fix = format!("Entferne die Änderungen an '{file}' ODER füge jedem Commit den Trailer 'Protected-Change: ADR-NNN' hinzu, verankere die ADR-Datei in docs/decisions/ auf dem Basis-Commit mit Status 'accepted' und setze das PR-Label 'protected-change'.");
        findings.push(serde_json::json!({
            "id": "PP-PROTECTED-PATH-VIOLATION",
            "severity": "error",
            "file": file,
            "line": 0,
            "message": msg,
            "fix": fix,
            "reason": missing_adr_reason
        }));
    }

    let summary = format!("Verstoß gegen geschützte Pfade: {} geschützte Datei(en) verändert ohne vollständige Ausnahme. Ursache: {missing_adr_reason}", violations.len());

    if json {
        println!(
            "{}",
            serde_json::json!({
                "gate": "protected-paths",
                "status": "fail",
                "summary": summary,
                "findings": findings
            })
        );
    } else {
        eprintln!("VERSTOSS [protected-paths]: {summary}");
        for (file, glob, rule_reason) in &violations {
            eprintln!("  WAS: Datei '{file}' verändert (Glob: '{glob}')");
            eprintln!("  WARUM: {rule_reason} (ADR/AGENTS.md Invariante)");
            eprintln!("  FIX: Commit Trailer 'Protected-Change: ADR-NNN', ADR in docs/decisions/ auf Basis-Commit verankern und PR-Label 'protected-change' setzen.");
            eprintln!("  URSACHE: {missing_adr_reason}");
        }
    }

    1
}
