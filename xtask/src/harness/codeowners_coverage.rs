//! Codeowners coverage harness module verifying protected paths against CODEOWNERS rules and checking phantom entries.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeownersCoverageFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeownersCoverageResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<CodeownersCoverageFinding>,
}

#[derive(Debug, Deserialize)]
struct ProtectedPath {
    glob: String,
    _reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProtectedPathsConfig {
    protected: Vec<ProtectedPath>,
}

pub fn run_codeowners_coverage(args: &[String]) -> i32 {
    let mut root_dir = default_root();
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
            "--json" => {
                json_output = true;
            }
            _ => {}
        }
        i += 1;
    }

    let mut findings = Vec::new();

    // 1. Read governance/protected-paths.toml
    let protected_paths_file = root_dir.join("governance/protected-paths.toml");
    let protected_globs = match fs::read_to_string(&protected_paths_file) {
        Ok(content) => match toml::from_str::<ProtectedPathsConfig>(&content) {
            Ok(config) => config
                .protected
                .into_iter()
                .map(|p| p.glob)
                .collect::<Vec<_>>(),
            Err(e) => {
                findings.push(CodeownersCoverageFinding {
                    id: "COC-001".to_string(),
                    severity: "error".to_string(),
                    file: "governance/protected-paths.toml".to_string(),
                    line: 1,
                    message: format!("protected-paths.toml konnte nicht geparst werden: {e}"),
                    fix: "Prüfe die TOML-Syntax in governance/protected-paths.toml".to_string(),
                });
                Vec::new()
            }
        },
        Err(e) => {
            findings.push(CodeownersCoverageFinding {
                id: "COC-001".to_string(),
                severity: "error".to_string(),
                file: "governance/protected-paths.toml".to_string(),
                line: 1,
                message: format!("protected-paths.toml konnte nicht gelesen werden: {e}"),
                fix: "Stelle sicher, dass governance/protected-paths.toml existiert".to_string(),
            });
            Vec::new()
        }
    };

    // 2. Read CODEOWNERS
    let codeowners_file = root_dir.join("CODEOWNERS");
    let codeowners_lines = match fs::read_to_string(&codeowners_file) {
        Ok(content) => content
            .lines()
            .enumerate()
            .map(|(idx, line)| (idx + 1, line.to_string()))
            .collect::<Vec<_>>(),
        Err(e) => {
            findings.push(CodeownersCoverageFinding {
                id: "COC-002".to_string(),
                severity: "error".to_string(),
                file: "CODEOWNERS".to_string(),
                line: 1,
                message: format!("CODEOWNERS konnte nicht gelesen werden: {e}"),
                fix: "Stelle sicher, dass CODEOWNERS existiert".to_string(),
            });
            Vec::new()
        }
    };

    let mut codeowners_patterns = Vec::new();

    for (line_num, line) in codeowners_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let path_pattern = parts[0].to_string();
            codeowners_patterns.push((line_num, path_pattern.clone()));

            // Check phantom paths (valid existence of pattern target)
            if !is_valid_codeowners_pattern(&root_dir, &path_pattern) {
                findings.push(CodeownersCoverageFinding {
                    id: "COC-003".to_string(),
                    severity: "error".to_string(),
                    file: "CODEOWNERS".to_string(),
                    line: line_num,
                    message: format!("Phantom-Pfad in CODEOWNERS: Pattern '{path_pattern}' matchet keine existierende Datei oder Verzeichnis"),
                    fix: format!("Entferne oder korrigiere den verwaisten Pfad '{path_pattern}' in CODEOWNERS"),
                });
            }
        }
    }

    // 3. Verify coverage: every protected glob in protected-paths.toml must be covered by CODEOWNERS pattern
    for glob_str in &protected_globs {
        let is_covered = codeowners_patterns
            .iter()
            .any(|(_, co_pat)| codeowners_matches_glob(&root_dir, co_pat, glob_str));

        if !is_covered {
            findings.push(CodeownersCoverageFinding {
                id: "COC-004".to_string(),
                severity: "error".to_string(),
                file: "governance/protected-paths.toml".to_string(),
                line: 1,
                message: format!(
                    "Geschütztes Glob '{glob_str}' ist nicht durch CODEOWNERS abgedeckt"
                ),
                fix: format!("Füge einen passenden Eintrag für '{glob_str}' in CODEOWNERS ein"),
            });
        }
    }

    let has_errors = findings.iter().any(|f| f.severity == "error");
    let status = if has_errors { "fail" } else { "pass" };
    let summary = if has_errors {
        format!(
            "CODEOWNERS-Abdeckung unvollständig: {} Verstoß/Verstöße gefunden",
            findings.len()
        )
    } else {
        "Alle geschützten Pfade sind vollständig in CODEOWNERS abgedeckt und keine Phantom-Pfade vorhanden.".to_string()
    };

    let res = CodeownersCoverageResult {
        gate: "codeowners-coverage".to_string(),
        status: status.to_string(),
        summary,
        findings,
    };

    emit(res, json_output, if has_errors { 1 } else { 0 })
}

fn glob_to_regex_pattern(glob: &str) -> String {
    let mut regex = String::from("^");
    let mut chars = glob.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '*' => {
                if chars.peek() == Some(&'*') {
                    chars.next();
                    if chars.peek() == Some(&'/') {
                        chars.next();
                        regex.push_str("(?:.*/)?");
                    } else {
                        regex.push_str(".*");
                    }
                } else {
                    regex.push_str("[^/]*");
                }
            }
            '?' => regex.push('.'),
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' | '[' | ']' | '\\' => {
                regex.push('\\');
                regex.push(c);
            }
            _ => regex.push(c),
        }
    }
    regex.push('$');
    regex
}

fn matches_glob_pattern(pattern_str: &str, candidate: &str) -> bool {
    let norm_pat = pattern_str.strip_prefix('/').unwrap_or(pattern_str);
    let norm_cand = candidate.strip_prefix('/').unwrap_or(candidate);

    if norm_pat == norm_cand {
        return true;
    }

    let pat_regex = glob_to_regex_pattern(norm_pat);
    if let Ok(re) = regex::Regex::new(&pat_regex) {
        if re.is_match(norm_cand) {
            return true;
        }
    }

    false
}

fn filtered_walkdir(root: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            if let Some(name) = e.file_name().to_str() {
                if name == "target" || name == ".git" {
                    return false;
                }
            }
            true
        })
        .flatten()
}

fn is_valid_codeowners_pattern(root: &Path, pat: &str) -> bool {
    let clean = pat.strip_prefix('/').unwrap_or(pat);
    let trimmed = clean.trim_end_matches('/');

    let target_path = root.join(trimmed);
    if target_path.exists() {
        return true;
    }

    // Check glob pattern matching existing files
    let mut matched = false;
    for entry in filtered_walkdir(root) {
        if let Ok(rel) = entry.path().strip_prefix(root) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if matches_glob_pattern(clean, &rel_str) {
                matched = true;
                break;
            }
        }
    }
    matched
}

fn codeowners_matches_glob(root: &Path, co_pat: &str, prot_glob: &str) -> bool {
    let norm_co = co_pat.strip_prefix('/').unwrap_or(co_pat);
    let norm_prot = prot_glob.strip_prefix('/').unwrap_or(prot_glob);

    if norm_co == norm_prot {
        return true;
    }

    let co_clean = norm_co.trim_end_matches('/');
    let prot_clean = norm_prot.trim_end_matches('/');

    if co_clean == prot_clean {
        return true;
    }

    if norm_co.ends_with("/**") || norm_co.ends_with("/*") {
        let co_prefix = norm_co
            .trim_end_matches("/**")
            .trim_end_matches("/*")
            .trim_end_matches('/');
        if norm_prot.starts_with(co_prefix) {
            return true;
        }
    }

    // Check if files matching prot_glob are a subset of files matching co_pat
    let mut prot_files = HashSet::new();
    let mut co_files = HashSet::new();

    for entry in filtered_walkdir(root) {
        if let Ok(rel) = entry.path().strip_prefix(root) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            if matches_glob_pattern(norm_prot, &rel_str) {
                prot_files.insert(rel_str.clone());
            }
            if matches_glob_pattern(norm_co, &rel_str) {
                co_files.insert(rel_str);
            }
        }
    }

    if !prot_files.is_empty() && prot_files.is_subset(&co_files) {
        return true;
    }

    false
}

fn default_root() -> PathBuf {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return PathBuf::from(path);
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn emit(res: CodeownersCoverageResult, json_output: bool, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", serde_json::to_string(&res).unwrap_or_default());
    } else {
        println!("=== Gate codeowners-coverage: {} ===", res.status);
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
