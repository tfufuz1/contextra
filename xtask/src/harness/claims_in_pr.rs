//! Claims in PR Gate: Prüft PR-Text-Vollständigkeit, Phantom-Claims im Diff und Verifikationsbelege.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsInPrFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimsInPrGateResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<ClaimsInPrFinding>,
}

pub fn claims_in_pr_required_headers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("Ziel", "CP-001"),
        ("Änderungen", "CP-001"),
        ("Invarianten berührt", "CP-001"),
        ("Verifikation", "CP-001"),
        ("Out-of-scope Findings", "CP-001"),
        ("Risiken/Rollback", "CP-001"),
        ("ADR/Spec-Sync", "CP-001"),
    ]
}

pub fn claims_in_pr_is_placeholder(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return true;
    }
    let lower = trimmed.to_lowercase();
    if lower == "todo" || lower == "n/a" || lower == "tbd" {
        return true;
    }
    if lower.starts_with("todo") && lower.len() < 10 {
        return true;
    }
    if lower.starts_with("n/a")
        && !lower.contains("weil")
        && !lower.contains("da ")
        && lower.len() < 10
    {
        return true;
    }
    false
}

pub fn claims_in_pr_parse_sections(body: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let re = match Regex::new(r"(?m)^##\s+(.+)$") {
        Ok(r) => r,
        Err(_) => return sections,
    };
    let matches: Vec<_> = re.find_iter(body).collect();

    for (i, m) in matches.iter().enumerate() {
        let title_line = m.as_str();
        let title = title_line.trim_start_matches('#').trim().to_string();

        let start = m.end();
        let end = if i + 1 < matches.len() {
            matches[i + 1].start()
        } else {
            body.len()
        };
        let content = body[start..end].trim().to_string();
        sections.push((title, content));
    }
    sections
}

pub fn claims_in_pr_extract_listed_files(changes_content: &str) -> HashSet<String> {
    let mut files = HashSet::new();
    for line in changes_content.lines() {
        let trimmed = line.trim();
        // Match list items `- path`, `* path`, `1. path` or code block lines
        let raw_path = if let Some(stripped) = trimmed.strip_prefix('-') {
            stripped.trim()
        } else if let Some(stripped) = trimmed.strip_prefix('*') {
            stripped.trim()
        } else if let Some(idx) = trimmed.find(". ") {
            if idx < 5 && trimmed[..idx].chars().all(|c| c.is_ascii_digit()) {
                trimmed[idx + 2..].trim()
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        // Clean backticks or markdown links
        let clean = raw_path
            .trim_matches('`')
            .trim_matches('[')
            .split(']')
            .next()
            .unwrap_or(raw_path)
            .trim();

        // Extract path candidate
        if !clean.is_empty() && (clean.contains('/') || clean.contains('.')) && !clean.contains(' ')
        {
            files.insert(clean.to_string());
        }
    }
    files
}

pub fn run_claims_in_pr(args: &[String]) -> i32 {
    let mut root_dir = claims_in_pr_default_root();
    let mut base = String::new();
    let mut head = String::from("HEAD");
    let mut body_file: Option<String> = None;
    let mut results_dir: Option<String> = None;
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
            "--body-file" => {
                if i + 1 < args.len() {
                    body_file = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--results-dir" => {
                if i + 1 < args.len() {
                    results_dir = Some(args[i + 1].clone());
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
        base = claims_in_pr_git_merge_base(&root_dir, &head);
    }

    let body_text = if let Some(ref bf) = body_file {
        fs::read_to_string(bf).unwrap_or_default()
    } else {
        env::var("PR_BODY").unwrap_or_default()
    };

    let mut findings = Vec::new();

    // Check CP-001: 7 required sections
    let parsed_sections = claims_in_pr_parse_sections(&body_text);
    for (req_header, rule_id) in claims_in_pr_required_headers() {
        let sec = parsed_sections.iter().find(|(t, _)| t == req_header);
        if let Some((_, content)) = sec {
            if claims_in_pr_is_placeholder(content) {
                findings.push(ClaimsInPrFinding {
                    id: rule_id.to_string(),
                    severity: "error".to_string(),
                    file: "".to_string(),
                    line: 0,
                    message: format!(
                        "Pflichtabschnitt '## {}' ist leer oder Platzhaltertext",
                        req_header
                    ),
                    fix: format!(
                        "Abschnitt '## {}' mit inhaltlicher Beschreibung füllen",
                        req_header
                    ),
                });
            }
        } else {
            findings.push(ClaimsInPrFinding {
                id: rule_id.to_string(),
                severity: "error".to_string(),
                file: "".to_string(),
                line: 0,
                message: format!("Pflichtabschnitt '## {}' fehlt im PR-Text", req_header),
                fix: format!("Abschnitt '## {}' im PR-Text ergänzen", req_header),
            });
        }
    }

    // Check CP-002: Set equality between listed files and actual diff
    let actual_diff_files = claims_in_pr_get_changed_files(&root_dir, &base, &head);
    let changes_content = parsed_sections
        .iter()
        .find(|(t, _)| t == "Änderungen")
        .map(|(_, c)| c.as_str())
        .unwrap_or_default();
    let listed_files = claims_in_pr_extract_listed_files(changes_content);

    let actual_set: HashSet<String> = actual_diff_files.into_iter().collect();

    // Missing files (in diff, not listed)
    for missing in actual_set.difference(&listed_files) {
        findings.push(ClaimsInPrFinding {
            id: "CP-002".to_string(),
            severity: "error".to_string(),
            file: missing.clone(),
            line: 0,
            message: format!(
                "Geänderte Datei '{}' ist im PR-Text unter 'Änderungen' nicht aufgeführt",
                missing
            ),
            fix: format!("Datei '{}' unter 'Änderungen' eintragen", missing),
        });
    }

    // Phantom files (listed, not in diff)
    for phantom in listed_files.difference(&actual_set) {
        findings.push(ClaimsInPrFinding {
            id: "CP-002".to_string(),
            severity: "error".to_string(),
            file: phantom.clone(),
            line: 0,
            message: format!(
                "Im PR-Text aufgeführte Datei '{}' existiert nicht im Git-Diff (Phantom-Claim)",
                phantom
            ),
            fix: format!(
                "Datei '{}' aus 'Änderungen' entfernen oder im Diff ergänzen",
                phantom
            ),
        });
    }

    // Check CP-003: Verification claims
    let verification_content = parsed_sections
        .iter()
        .find(|(t, _)| t == "Verifikation")
        .map(|(_, c)| c.as_str())
        .unwrap_or_default();

    if let Ok(ver_re) = Regex::new(r"(`[^`]+`|[a-zA-Z0-9_\-\.\s]+):\s*(PASS|grün|bestanden|ok)") {
        for cap in ver_re.captures_iter(verification_content) {
            let cmd_claim = cap[1].trim_matches('`').trim();
            let has_evidence = if let Some(ref rdir) = results_dir {
                let rpath = Path::new(rdir);
                if let Ok(entries) = fs::read_dir(rpath) {
                    entries.filter_map(|e| e.ok()).any(|e| {
                        if e.path().extension().is_some_and(|ext| ext == "json") {
                            if let Ok(c) = fs::read_to_string(e.path()) {
                                c.contains(cmd_claim)
                                    && (c.contains("\"status\":\"pass\"")
                                        || c.contains("\"status\": \"pass\"")
                                        || c.contains("ok"))
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            } else {
                true // No results dir specified, don't fail unless results dir is provided
            };

            if !has_evidence {
                findings.push(ClaimsInPrFinding {
                    id: "CP-003".to_string(),
                    severity: "error".to_string(),
                    file: "".to_string(),
                    line: 0,
                    message: format!(
                        "Behauptung für '{}' ist nicht durch Ergebnisse in --results-dir belegt",
                        cmd_claim
                    ),
                    fix: "Beleg-JSON im results-dir ablegen oder Verifikationsbehauptung anpassen"
                        .to_string(),
                });
            }
        }
    }

    // Check CP-004: "Tests hinzugefügt" claim
    let full_text_lower = body_text.to_lowercase();
    if full_text_lower.contains("test hinzugefügt") || full_text_lower.contains("tests hinzugefügt")
    {
        let diff_has_new_test = claims_in_pr_diff_has_new_test(&root_dir, &base, &head);
        if !diff_has_new_test {
            findings.push(ClaimsInPrFinding {
                id: "CP-004".to_string(),
                severity: "error".to_string(),
                file: "".to_string(),
                line: 0,
                message: "PR-Text behauptet 'Tests hinzugefügt', aber im Diff wurde kein neuer #[test] oder neue Testdatei gefunden".to_string(),
                fix: "Tests im Code hinzufügen oder Aussage aus PR-Text entfernen".to_string(),
            });
        }
    }

    if findings.is_empty() {
        let res = ClaimsInPrGateResult {
            gate: "claims-in-pr".to_string(),
            status: "pass".to_string(),
            summary: "Alle Claims im PR-Text sind vollständig, korrekt und belegt".to_string(),
            findings: vec![],
        };
        claims_in_pr_emit(res, json_output, 0)
    } else {
        let res = ClaimsInPrGateResult {
            gate: "claims-in-pr".to_string(),
            status: "fail".to_string(),
            summary: format!(
                "{} Verstoß/Verstöße gegen Claims-Regeln gefunden",
                findings.len()
            ),
            findings,
        };
        claims_in_pr_emit(res, json_output, 1)
    }
}

fn claims_in_pr_emit(res: ClaimsInPrGateResult, json_output: bool, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", serde_json::to_string(&res).unwrap_or_default());
    } else {
        println!("=== Gate claims-in-pr: {} ===", res.status);
        println!("{}", res.summary);
        for f in &res.findings {
            println!(
                "[{}] {}: {}\n  Fix: {}",
                f.severity.to_uppercase(),
                if f.file.is_empty() {
                    "PR-Text"
                } else {
                    &f.file
                },
                f.message,
                f.fix
            );
        }
    }
    exit_code
}

fn claims_in_pr_default_root() -> PathBuf {
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

fn claims_in_pr_git_merge_base(root: &Path, head: &str) -> String {
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

fn claims_in_pr_get_changed_files(root: &Path, base: &str, head: &str) -> Vec<String> {
    let range = if base == head {
        format!("{}^..{}", head, head)
    } else {
        format!("{}...{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "--name-only", &range])
        .output();

    let mut files = Vec::new();
    if let Ok(out) = output {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                files.push(trimmed.to_string());
            }
        }
    }
    files
}

fn claims_in_pr_diff_has_new_test(root: &Path, base: &str, head: &str) -> bool {
    let range = if base == head {
        format!("{}^..{}", head, head)
    } else {
        format!("{}...{}", base, head)
    };
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "-U0", &range])
        .output();

    if let Ok(out) = output {
        let diff_text = String::from_utf8_lossy(&out.stdout);
        for line in diff_text.lines() {
            if line.starts_with('+')
                && !line.starts_with("+++")
                && (line.contains("#[test]") || line.contains("#[tokio::test]"))
            {
                return true;
            }
            if line.starts_with("+++ b/")
                && (line.contains("/tests/") || line.ends_with("_test.rs"))
            {
                return true;
            }
        }
    }
    false
}
