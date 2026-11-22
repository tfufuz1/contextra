//! Module analyzing git diffs for gate weakening practices.

use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub struct GateWeakeningFinding {
    pub id: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

pub fn gate_weakening_parse_args(args: &[String]) -> (String, String, String, bool) {
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
        if let Ok(out) = Command::new("git")
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

pub fn gate_weakening_get_commits(
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

pub fn gate_weakening_get_commit_trailers(
    root: &str,
    commit_hash: &str,
) -> Result<Vec<(String, String)>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "log",
            "-1",
            "--format=%(trailers:key=Gate-Weakening)",
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

pub fn gate_weakening_analyze_diff(
    root: &str,
    base: &str,
    head: &str,
) -> Result<Vec<GateWeakeningFinding>, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["diff", "-U0", &format!("{base}...{head}")])
        .output()
        .map_err(|e| format!("git diff -U0 failed: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "git diff -U0 failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut findings = Vec::new();

    let mut current_file = String::new();
    let mut current_line = 0;

    // File-level net accounting for rust tests and assertions
    let mut test_diff_counts: HashMap<String, (i32, usize)> = HashMap::new(); // (net_test_diff, first_line)
    let mut assert_diff_counts: HashMap<String, (i32, usize)> = HashMap::new(); // (net_assert_diff, first_line)

    let hunk_re =
        Regex::new(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,\d+)? @@").map_err(|e| e.to_string())?;
    let retries_re = Regex::new(r"--retries\s+([1-9]\d*)").map_err(|e| e.to_string())?;
    let threshold_re = Regex::new(r"(coverage|threshold|min|score)\s*[:=]\s*(\d+(?:\.\d+)?)")
        .map_err(|e| e.to_string())?;
    let kv_re = Regex::new(r"([a-z_]+)\s*=\s*(\d+)").map_err(|e| e.to_string())?;

    let mut removed_thresholds: HashMap<String, f64> = HashMap::new();

    for raw_line in stdout.lines() {
        if raw_line.starts_with("--- a/") {
            continue;
        }
        if let Some(file_path) = raw_line.strip_prefix("+++ b/") {
            current_file = file_path.to_string();
            current_line = 0;
            continue;
        }

        if raw_line.starts_with("@@ ") {
            if let Some(caps) = hunk_re.captures(raw_line) {
                if let Ok(l) = caps[1].parse::<usize>() {
                    current_line = l;
                }
            }
            continue;
        }

        if raw_line.starts_with('+') && !raw_line.starts_with("+++") {
            let added = &raw_line[1..];

            // 1. Workflow checks
            if current_file.starts_with(".github/workflows/") {
                if added.contains("continue-on-error: true") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-001".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: 'continue-on-error: true' schwächt Pipeline-Fehler ab.".to_string(),
                        fix: "Entferne 'continue-on-error: true' und behebe die zugrunde liegende Ursache.".to_string(),
                    });
                }
                if added.contains("|| true") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-002".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: '|| true' unterdrückt Befehlsfehler.".to_string(),
                        fix: "Entferne '|| true' und erlaube dem Befehl bei Fehlern sauber fehlzuschlagen.".to_string(),
                    });
                }
                if added.contains("--retries") && retries_re.is_match(added) {
                    findings.push(GateWeakeningFinding {
                        id: "GW-003".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: Hinzugefügte Flaky-Test Retries (--retries > 0) kaschieren Instabilitäten.".to_string(),
                        fix: "Entferne Retries und repariere den Flaky Test deterministisch.".to_string(),
                    });
                }
                if added.contains("if: false") || added.contains("if: ${{ false }}") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-004".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: 'if: false' deaktiviert Workflow-Steps."
                            .to_string(),
                        fix: "Entferne die Deaktivierung des Steps.".to_string(),
                    });
                }

                if let Some(caps) = threshold_re.captures(added) {
                    let key = caps[1].to_string();
                    if let Ok(new_val) = caps[2].parse::<f64>() {
                        if let Some(&old_val) = removed_thresholds.get(&key) {
                            if new_val < old_val {
                                findings.push(GateWeakeningFinding {
                                    id: "GW-015".to_string(),
                                    file: current_file.clone(),
                                    line: current_line,
                                    message: format!("Falsche Abkürzung: Schwellenwert '{key}' wurde von {old_val} auf {new_val} gesenkt!"),
                                    fix: "Stelle den ursprünglichen Schwellenwert wieder her.".to_string(),
                                });
                            }
                        }
                    }
                }
            }

            // 2. Rust Code checks
            if current_file.ends_with(".rs") {
                if added.contains("#[allow(") || added.contains("#![allow(") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-005".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: '#[allow(...)]' schaltet Warnungen/Lints aus."
                            .to_string(),
                        fix: "Entferne #[allow(...)] und behebe die Warnung im Code.".to_string(),
                    });
                }
                if added.contains("#[ignore") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-006".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: '#[ignore]' ignoriert relevante Testfälle."
                            .to_string(),
                        fix: "Entferne #[ignore] und stelle sicher, dass der Test grün läuft."
                            .to_string(),
                    });
                }
                if added.contains("#[test]") {
                    let entry = test_diff_counts
                        .entry(current_file.clone())
                        .or_insert((0, current_line));
                    entry.0 += 1;
                }
                if added.contains("assert!")
                    || added.contains("assert_eq!")
                    || added.contains("assert_ne!")
                {
                    let entry = assert_diff_counts
                        .entry(current_file.clone())
                        .or_insert((0, current_line));
                    entry.0 += 1;
                }
            }

            // 3. Cargo / Lints & deny.toml checks
            if current_file == "deny.toml"
                && (added.contains("ignore")
                    || added.contains("unmaintained = \"allow\"")
                    || added.contains("yanked = \"allow\""))
            {
                findings.push(GateWeakeningFinding {
                    id: "GW-007".to_string(),
                    file: current_file.clone(),
                    line: current_line,
                    message:
                        "Falsche Abkürzung: Abschwächung der Security-Richtlinien in deny.toml."
                            .to_string(),
                    fix: "Rückgängig machen der Lockerung in deny.toml.".to_string(),
                });
            }

            if current_file.ends_with("Cargo.toml")
                && (added.contains("deny = \"warn\"")
                    || added.contains("deny = \"allow\"")
                    || added.contains("forbid = \"deny\""))
            {
                findings.push(GateWeakeningFinding {
                    id: "GW-008".to_string(),
                    file: current_file.clone(),
                    line: current_line,
                    message: "Falsche Abkürzung: Lint-Level Absenkung in Cargo.toml.".to_string(),
                    fix: "Wiederherstellen des ursprünglichen Lint-Levels in Cargo.toml."
                        .to_string(),
                });
            }

            // 4. Baseline checks
            if current_file == ".github/unwrap_baseline.txt" {
                if let Ok(val) = added.trim().parse::<u64>() {
                    let orig_output = Command::new("git")
                        .current_dir(root)
                        .args(["show", &format!("{base}:.github/unwrap_baseline.txt")])
                        .output();
                    if let Ok(o) = orig_output {
                        if o.status.success() {
                            if let Ok(orig_val) =
                                String::from_utf8_lossy(&o.stdout).trim().parse::<u64>()
                            {
                                if val > orig_val {
                                    findings.push(GateWeakeningFinding {
                                        id: "GW-009".to_string(),
                                        file: current_file.clone(),
                                        line: current_line,
                                        message: format!("Falsche Abkürzung: unwrap_baseline.txt wurde von {orig_val} auf {val} erhöht!"),
                                        fix: "Reduziere unwrap()-Aufrufe statt die Baseline zu erhöhen.".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            if current_file == "governance/ratchet.toml" {
                if let Some(caps) = kv_re.captures(added) {
                    let key = caps[1].to_string();
                    if let Ok(new_val) = caps[2].parse::<u64>() {
                        let orig_output = Command::new("git")
                            .current_dir(root)
                            .args(["show", &format!("{base}:governance/ratchet.toml")])
                            .output();
                        if let Ok(o) = orig_output {
                            if o.status.success() {
                                let orig_str = String::from_utf8_lossy(&o.stdout);
                                for orig_line in orig_str.lines() {
                                    if let Some(orig_caps) = kv_re.captures(orig_line) {
                                        if orig_caps[1] == key {
                                            if let Ok(orig_val) = orig_caps[2].parse::<u64>() {
                                                if (key != "forbid_unsafe_crates"
                                                    && new_val > orig_val)
                                                    || (key == "forbid_unsafe_crates"
                                                        && new_val < orig_val)
                                                {
                                                    findings.push(GateWeakeningFinding {
                                                        id: "GW-016".to_string(),
                                                        file: current_file.clone(),
                                                        line: current_line,
                                                        message: format!("Falsche Abkürzung: Ratchet Baseline '{key}' wurde von {orig_val} auf {new_val} aufgeweicht!"),
                                                        fix: "Stelle die ursprüngliche Ratchet-Baseline wieder her.".to_string(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            current_line += 1;
        } else if raw_line.starts_with('-') && !raw_line.starts_with("---") {
            let removed = &raw_line[1..];

            if current_file.starts_with(".github/workflows/") {
                if removed.contains("-D warnings") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-010".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: '-D warnings' wurde aus CI-Workflow entfernt."
                            .to_string(),
                        fix: "Füge '-D warnings' wieder in den Workflow ein.".to_string(),
                    });
                }
                if removed.trim().starts_with("needs:") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-011".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message:
                            "Falsche Abkürzung: 'needs:'-Abhängigkeit aus Workflow-Job entfernt."
                                .to_string(),
                        fix: "Stelle die Job-Abhängigkeiten im Workflow wieder her.".to_string(),
                    });
                }
                if let Some(caps) = threshold_re.captures(removed) {
                    let key = caps[1].to_string();
                    if let Ok(val) = caps[2].parse::<f64>() {
                        removed_thresholds.insert(key, val);
                    }
                }
            }

            if current_file.ends_with(".rs") {
                if removed.contains("#![forbid(unsafe_code)]") || removed.contains("#![deny(") {
                    findings.push(GateWeakeningFinding {
                        id: "GW-012".to_string(),
                        file: current_file.clone(),
                        line: current_line,
                        message: "Falsche Abkürzung: Unsafe-Sicherheitsattribut (#![forbid(unsafe_code)]) entfernt.".to_string(),
                        fix: "Stelle #![forbid(unsafe_code)] wieder her.".to_string(),
                    });
                }
                if removed.contains("#[test]") {
                    let entry = test_diff_counts
                        .entry(current_file.clone())
                        .or_insert((0, current_line));
                    entry.0 -= 1;
                }
                if removed.contains("assert!")
                    || removed.contains("assert_eq!")
                    || removed.contains("assert_ne!")
                {
                    let entry = assert_diff_counts
                        .entry(current_file.clone())
                        .or_insert((0, current_line));
                    entry.0 -= 1;
                }
            }
        }
    }

    // Evaluate net test and assert removals
    for (file, (net_diff, line)) in test_diff_counts {
        if net_diff < 0 {
            findings.push(GateWeakeningFinding {
                id: "GW-013".to_string(),
                file,
                line,
                message: format!(
                    "Falsche Abkürzung: Netto {} #[test]-Funktion(en) in Datei entfernt.",
                    net_diff.abs()
                ),
                fix: "Gelöschte Testfälle wiederherstellen oder gleichwertigen Ersatz schaffen."
                    .to_string(),
            });
        }
    }

    for (file, (net_diff, line)) in assert_diff_counts {
        if net_diff < 0 {
            findings.push(GateWeakeningFinding {
                id: "GW-014".to_string(),
                file,
                line,
                message: format!(
                    "Falsche Abkürzung: Netto {} Test-Assertion(en) (assert*!) in Datei entfernt.",
                    net_diff.abs()
                ),
                fix: "Entfernte Assertionen wiederherstellen.".to_string(),
            });
        }
    }

    Ok(findings)
}

pub fn run_gate_weakening(args: &[String]) -> i32 {
    let (root, base, head, json) = gate_weakening_parse_args(args);

    let findings = match gate_weakening_analyze_diff(&root, &base, &head) {
        Ok(f) => f,
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "gate-weakening",
                        "status": "error",
                        "summary": e,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [gate-weakening]: {e}");
            }
            return 2;
        }
    };

    if findings.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "gate-weakening",
                    "status": "pass",
                    "summary": "Keine Gate-Schwächungen im Diff erkannt.",
                    "findings": []
                })
            );
        } else {
            println!("PASS [gate-weakening]: Keine Gate-Schwächungen im Diff erkannt.");
        }
        return 0;
    }

    // Check exception rule: Trailer 'Gate-Weakening: ADR-NNN'
    let commits = match gate_weakening_get_commits(&root, &base, &head) {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "gate": "gate-weakening",
                        "status": "error",
                        "summary": e,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER [gate-weakening]: {e}");
            }
            return 2;
        }
    };

    let mut all_commits_have_valid_adr = true;
    let mut missing_adr_reason = String::new();

    if commits.is_empty() {
        all_commits_have_valid_adr = false;
        missing_adr_reason = "Keine Commits im Bereich gefunden.".to_string();
    } else {
        for commit in &commits {
            let trailers = match gate_weakening_get_commit_trailers(&root, commit) {
                Ok(t) => t,
                Err(e) => {
                    all_commits_have_valid_adr = false;
                    missing_adr_reason = e;
                    break;
                }
            };

            let mut commit_valid = false;
            for (key, val) in trailers {
                if key == "Gate-Weakening" {
                    let adr_path = Path::new(&root)
                        .join("docs/decisions")
                        .join(format!("{val}.md"));

                    let mut exists = adr_path.exists();
                    if !exists {
                        if let Ok(entries) =
                            walkdir::WalkDir::new(Path::new(&root).join("docs/decisions"))
                                .max_depth(1)
                                .into_iter()
                                .collect::<Result<Vec<_>, _>>()
                        {
                            for entry in entries {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name.starts_with(&format!("{val}-")) && name.ends_with(".md")
                                    {
                                        exists = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if exists {
                        commit_valid = true;
                        break;
                    } else {
                        missing_adr_reason =
                            format!("ADR-Datei für '{val}' nicht in docs/decisions/ gefunden.");
                    }
                }
            }

            if !commit_valid {
                all_commits_have_valid_adr = false;
                if missing_adr_reason.is_empty() {
                    missing_adr_reason = format!(
                        "Commit {commit} besitzt keinen 'Gate-Weakening: ADR-NNN' Trailer."
                    );
                }
                break;
            }
        }
    }

    if all_commits_have_valid_adr {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "gate": "gate-weakening",
                    "status": "pass",
                    "summary": "Gate-Schwächung mit gültigem ADR-Trailer genehmigt.",
                    "findings": []
                })
            );
        } else {
            println!("PASS [gate-weakening]: Gate-Schwächung mit gültiger ADR-Ausnahme genehmigt.");
        }
        return 0;
    }

    let summary = format!("Gate-Schwächung erkannt: {} Verstoß/Verstöße ohne gültige ADR-Ausnahme. Ursache: {missing_adr_reason}", findings.len());

    if json {
        let json_findings: Vec<serde_json::Value> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "id": f.id,
                    "severity": "error",
                    "file": f.file,
                    "line": f.line,
                    "message": f.message,
                    "fix": f.fix
                })
            })
            .collect();

        println!(
            "{}",
            serde_json::json!({
                "gate": "gate-weakening",
                "status": "fail",
                "summary": summary,
                "findings": json_findings
            })
        );
    } else {
        eprintln!("VERSTOSS [gate-weakening]: {summary}");
        for f in &findings {
            eprintln!("  WAS [{}] in {}:{}: {}", f.id, f.file, f.line, f.message);
            eprintln!(
                "  WARUM: Untergräbt Qualitätssicherung und Pipeline-Schutz (Invariante/ADR)."
            );
            eprintln!("  FIX: {}", f.fix);
        }
    }

    1
}
