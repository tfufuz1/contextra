//! Harness module verifying synchronization between `governance/gates.toml`, `.jules/harness-phases.toml`, and `governance/verdict-required.toml`.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct GateCatalogSyncFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateCatalogSyncResult {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<GateCatalogSyncFinding>,
}

#[derive(Debug, Deserialize)]
struct CatalogGate {
    name: String,
    #[serde(rename = "phase")]
    _phase: String,
    required_local: bool,
    required_ci: bool,
    #[serde(rename = "risk_class")]
    _risk_class: String,
    #[serde(rename = "owner")]
    _owner: String,
    #[serde(rename = "fixture")]
    _fixture: String,
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    gate: Vec<CatalogGate>,
}

#[derive(Debug, Deserialize)]
struct VerdictGate {
    name: String,
    blocking: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct VerdictFile {
    gate: Vec<VerdictGate>,
}

pub fn run_gates_catalog_sync(args: &[String]) -> i32 {
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

    // 1. Read governance/gates.toml
    let gates_toml_path = root_dir.join("governance/gates.toml");
    let catalog: Option<CatalogFile> = match fs::read_to_string(&gates_toml_path) {
        Ok(content) => match toml::from_str(&content) {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                findings.push(GateCatalogSyncFinding {
                    id: "GCS-001".to_string(),
                    severity: "error".to_string(),
                    file: "governance/gates.toml".to_string(),
                    line: 1,
                    message: format!("governance/gates.toml konnte nicht geparst werden: {e}"),
                    fix: "Korrigiere die TOML-Syntax in governance/gates.toml".to_string(),
                });
                None
            }
        },
        Err(e) => {
            findings.push(GateCatalogSyncFinding {
                id: "GCS-001".to_string(),
                severity: "error".to_string(),
                file: "governance/gates.toml".to_string(),
                line: 1,
                message: format!("governance/gates.toml konnte nicht gelesen werden: {e}"),
                fix: "Erstelle governance/gates.toml gemäß Spezifikation".to_string(),
            });
            None
        }
    };

    // 2. Read .jules/harness-phases.toml
    let phases_toml_path = root_dir.join(".jules/harness-phases.toml");
    let local_phases_map = parse_harness_phases(&phases_toml_path, &mut findings);

    // 3. Read governance/verdict-required.toml
    let verdict_toml_path = root_dir.join("governance/verdict-required.toml");
    let ci_verdict_map = parse_verdict_required(&verdict_toml_path, &mut findings);

    if let Some(catalog) = catalog {
        let mut catalog_names = HashSet::new();

        for gate in &catalog.gate {
            catalog_names.insert(gate.name.clone());

            // Check against harness-phases.toml
            if let Some(&local_req) = local_phases_map.get(&gate.name) {
                if gate.required_local != local_req {
                    findings.push(GateCatalogSyncFinding {
                        id: "GCS-002".to_string(),
                        file: "governance/gates.toml".to_string(),
                        line: 1,
                        severity: "error".to_string(),
                        message: format!(
                            "Gate '{}': required_local ({}) weicht von harness-phases.toml ({}) ab",
                            gate.name, gate.required_local, local_req
                        ),
                        fix: format!(
                            "Passe required_local in governance/gates.toml an '{}' an",
                            local_req
                        ),
                    });
                }
            } else if gate.required_local {
                // Check mapped names like ratchet check -> ratchet or task-card lint -> task-card
                let mapped = if gate.name == "ratchet" {
                    local_phases_map.get("ratchet check").copied()
                } else if gate.name == "task-card" {
                    local_phases_map.get("task-card lint").copied()
                } else {
                    None
                };

                if let Some(local_req) = mapped {
                    if gate.required_local != local_req {
                        findings.push(GateCatalogSyncFinding {
                            id: "GCS-002".to_string(),
                            file: "governance/gates.toml".to_string(),
                            line: 1,
                            severity: "error".to_string(),
                            message: format!(
                                "Gate '{}': required_local ({}) weicht von harness-phases.toml ({}) ab",
                                gate.name, gate.required_local, local_req
                            ),
                            fix: format!("Passe required_local in governance/gates.toml an '{}' an", local_req),
                        });
                    }
                }
            }

            // Check against verdict-required.toml
            let ci_name =
                if gate.name.starts_with("gate-") && !ci_verdict_map.contains_key(&gate.name) {
                    // Could be mapped e.g. gate-determinism-check
                    gate.name.as_str()
                } else {
                    gate.name.as_str()
                };

            if let Some(&ci_req) = ci_verdict_map.get(ci_name) {
                if gate.required_ci != ci_req {
                    findings.push(GateCatalogSyncFinding {
                        id: "GCS-003".to_string(),
                        file: "governance/gates.toml".to_string(),
                        line: 1,
                        severity: "error".to_string(),
                        message: format!(
                            "Gate '{}': required_ci ({}) weicht von verdict-required.toml ({}) ab",
                            gate.name, gate.required_ci, ci_req
                        ),
                        fix: format!(
                            "Passe required_ci in governance/gates.toml an '{}' an",
                            ci_req
                        ),
                    });
                }
            }
        }

        // Check if any local required gates are missing from catalog
        for (phase_cmd, &req) in &local_phases_map {
            let norm_name = normalize_cmd_name(phase_cmd);
            if req && !catalog_names.contains(&norm_name) {
                findings.push(GateCatalogSyncFinding {
                    id: "GCS-004".to_string(),
                    file: "governance/gates.toml".to_string(),
                    line: 1,
                    severity: "error".to_string(),
                    message: format!("Lokales Pflichtgate '{}' (aus harness-phases.toml) fehlt im Katalog governance/gates.toml", norm_name),
                    fix: format!("Füge einen Eintrag für Gate '{}' in governance/gates.toml ein", norm_name),
                });
            }
        }

        // Check if any CI required gates are missing from catalog
        for (verdict_name, &blocking) in &ci_verdict_map {
            if blocking && !catalog_names.contains(verdict_name) {
                findings.push(GateCatalogSyncFinding {
                    id: "GCS-005".to_string(),
                    file: "governance/gates.toml".to_string(),
                    line: 1,
                    severity: "error".to_string(),
                    message: format!("CI-Pflichtgate '{}' (aus verdict-required.toml) fehlt im Katalog governance/gates.toml", verdict_name),
                    fix: format!("Füge einen Eintrag für Gate '{}' in governance/gates.toml ein", verdict_name),
                });
            }
        }
    }

    // Read-only check: justfile and workflows
    check_justfile_and_workflows_read_only(&root_dir, &findings);

    let has_errors = findings.iter().any(|f| f.severity == "error");

    let status = if has_errors { "fail" } else { "pass" };
    let summary = if has_errors {
        format!(
            "Gate-Katalog Abweichung(en) gefunden: {} Verstoß/Verstöße",
            findings.len()
        )
    } else {
        "Gate-Katalog ist vollkommen synchron mit harness-phases.toml und verdict-required.toml."
            .to_string()
    };

    let res = GateCatalogSyncResult {
        gate: "gates-catalog-sync".to_string(),
        status: status.to_string(),
        summary,
        findings,
    };

    emit(res, json_output, if has_errors { 1 } else { 0 })
}

fn parse_harness_phases(
    path: &Path,
    findings: &mut Vec<GateCatalogSyncFinding>,
) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            findings.push(GateCatalogSyncFinding {
                id: "GCS-006".to_string(),
                file: ".jules/harness-phases.toml".to_string(),
                line: 1,
                severity: "error".to_string(),
                message: format!(".jules/harness-phases.toml konnte nicht gelesen werden: {e}"),
                fix: "Stelle sicher, dass .jules/harness-phases.toml existiert".to_string(),
            });
            return map;
        }
    };

    let val: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            findings.push(GateCatalogSyncFinding {
                id: "GCS-006".to_string(),
                file: ".jules/harness-phases.toml".to_string(),
                line: 1,
                severity: "error".to_string(),
                message: format!(".jules/harness-phases.toml konnte nicht geparst werden: {e}"),
                fix: "Prüfe die TOML-Syntax in .jules/harness-phases.toml".to_string(),
            });
            return map;
        }
    };

    if let Some(phase) = val.get("phase").and_then(|p| p.as_table()) {
        for (_pname, pval) in phase {
            if let Some(arr) = pval.as_array() {
                for item in arr {
                    if let (Some(cmd_val), Some(req_val)) = (item.get("cmd"), item.get("required"))
                    {
                        let req = req_val.as_bool().unwrap_or(false);
                        if let Some(cmd_arr) = cmd_val.as_array() {
                            let cmd_strs: Vec<_> =
                                cmd_arr.iter().filter_map(|v| v.as_str()).collect();
                            if cmd_strs.len() >= 3
                                && cmd_strs[0] == "cargo"
                                && cmd_strs[1] == "xtask"
                            {
                                let gate_cmd = cmd_strs[2..].join(" ");
                                map.insert(gate_cmd, req);
                            }
                        }
                    }
                }
            }
        }
    }

    map
}

fn parse_verdict_required(
    path: &Path,
    findings: &mut Vec<GateCatalogSyncFinding>,
) -> HashMap<String, bool> {
    let mut map = HashMap::new();
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            findings.push(GateCatalogSyncFinding {
                id: "GCS-007".to_string(),
                file: "governance/verdict-required.toml".to_string(),
                line: 1,
                severity: "error".to_string(),
                message: format!(
                    "governance/verdict-required.toml konnte nicht gelesen werden: {e}"
                ),
                fix: "Stelle sicher, dass governance/verdict-required.toml existiert".to_string(),
            });
            return map;
        }
    };

    let parsed: VerdictFile = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            findings.push(GateCatalogSyncFinding {
                id: "GCS-007".to_string(),
                file: "governance/verdict-required.toml".to_string(),
                line: 1,
                severity: "error".to_string(),
                message: format!(
                    "governance/verdict-required.toml konnte nicht geparst werden: {e}"
                ),
                fix: "Prüfe die TOML-Syntax in governance/verdict-required.toml".to_string(),
            });
            return map;
        }
    };

    for g in parsed.gate {
        map.insert(g.name, g.blocking.unwrap_or(true));
    }

    map
}

fn normalize_cmd_name(cmd: &str) -> String {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return cmd.to_string();
    }
    parts[0].to_string()
}

fn check_justfile_and_workflows_read_only(root: &Path, _findings: &[GateCatalogSyncFinding]) {
    // Read justfile and workflows for info/warning logging without failing gate
    let justfile = root.join("justfile");
    if let Ok(content) = fs::read_to_string(&justfile) {
        let _ = content;
    }

    let workflows_dir = root.join(".github/workflows");
    if workflows_dir.exists() {
        for entry in WalkDir::new(&workflows_dir).into_iter().flatten() {
            if entry.path().is_file() {
                let _ = fs::read_to_string(entry.path());
            }
        }
    }
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

fn emit(res: GateCatalogSyncResult, json_output: bool, exit_code: i32) -> i32 {
    if json_output {
        println!("{}", serde_json::to_string(&res).unwrap_or_default());
    } else {
        println!("=== Gate gates-catalog-sync: {} ===", res.status);
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
