//! Hard-gate audit ledger management command for worktree verification.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerGateFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerGateReport {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<LedgerGateFinding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LedgerDocument {
    pub tree_hash: String,
    pub head: String,
    pub generated_at: String,
    pub gates: Vec<LedgerGateReport>,
    pub overall_status: String,
}

pub fn ledger_compute_tree_hash(root: &Path) -> Result<String, String> {
    let temp_index = tempfile::NamedTempFile::new()
        .map_err(|e| format!("Failed to create temporary index file: {e}"))?;
    let temp_path = temp_index.path();

    let mut cmd_read = Command::new("git");
    cmd_read.env("GIT_INDEX_FILE", temp_path);
    cmd_read.current_dir(root);
    let read_out = cmd_read
        .args(["read-tree", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to run git read-tree: {e}"))?;
    if !read_out.status.success() {
        return Err(format!(
            "git read-tree failed: {}",
            String::from_utf8_lossy(&read_out.stderr)
        ));
    }

    let mut cmd_add = Command::new("git");
    cmd_add.env("GIT_INDEX_FILE", temp_path);
    cmd_add.current_dir(root);
    let add_out = cmd_add
        .args(["add", "-A"])
        .output()
        .map_err(|e| format!("Failed to run git add: {e}"))?;
    if !add_out.status.success() {
        return Err(format!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_out.stderr)
        ));
    }

    let mut cmd_tree = Command::new("git");
    cmd_tree.env("GIT_INDEX_FILE", temp_path);
    cmd_tree.current_dir(root);
    let tree_out = cmd_tree
        .args(["write-tree"])
        .output()
        .map_err(|e| format!("Failed to run git write-tree: {e}"))?;
    if !tree_out.status.success() {
        return Err(format!(
            "git write-tree failed: {}",
            String::from_utf8_lossy(&tree_out.stderr)
        ));
    }

    let hash = String::from_utf8_lossy(&tree_out.stdout).trim().to_string();
    if hash.is_empty() {
        Err("Empty tree hash produced".to_string())
    } else {
        Ok(hash)
    }
}

pub fn ledger_get_head_commit(root: &Path) -> Result<String, String> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to get HEAD commit: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(format!(
            "git rev-parse HEAD failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

pub fn ledger_write(
    root: &Path,
    results_dir: &Path,
    out_file: &Path,
) -> Result<LedgerDocument, String> {
    let tree_hash = ledger_compute_tree_hash(root)?;
    let head = ledger_get_head_commit(root)?;
    let generated_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let mut gates = Vec::new();
    let mut overall_ok = true;

    let entries = fs::read_dir(results_dir).map_err(|e| {
        format!(
            "Failed to read results directory {}: {e}",
            results_dir.display()
        )
    })?;

    let mut found_json = false;
    for entry_res in entries {
        let entry = entry_res.map_err(|e| format!("Failed to read dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            found_json = true;
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read gate JSON file {}: {e}", path.display()))?;
            let report: LedgerGateReport = serde_json::from_str(&content).map_err(|e| {
                format!("Invalid JSON schema in gate report {}: {e}", path.display())
            })?;
            if report.status != "pass" && report.status != "not_applicable" {
                overall_ok = false;
            }
            gates.push(report);
        }
    }

    if !found_json {
        return Err(format!(
            "No gate JSON result files found in {}",
            results_dir.display()
        ));
    }

    gates.sort_by(|a, b| a.gate.cmp(&b.gate));

    let doc = LedgerDocument {
        tree_hash,
        head,
        generated_at,
        gates,
        overall_status: if overall_ok {
            "PASS".to_string()
        } else {
            "FAIL".to_string()
        },
    };

    if let Some(parent) = out_file.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let json_bytes = serde_json::to_string_pretty(&doc)
        .map_err(|e| format!("Failed to serialize ledger document: {e}"))?;
    fs::write(out_file, json_bytes)
        .map_err(|e| format!("Failed to write ledger file {}: {e}", out_file.display()))?;

    Ok(doc)
}

pub fn ledger_verify(root: &Path, ledger_file: &Path) -> Result<LedgerDocument, String> {
    let content = fs::read_to_string(ledger_file)
        .map_err(|e| format!("Failed to read ledger file {}: {e}", ledger_file.display()))?;
    let doc: LedgerDocument = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid ledger JSON in {}: {e}", ledger_file.display()))?;

    let current_tree_hash = ledger_compute_tree_hash(root)?;
    if doc.tree_hash != current_tree_hash {
        return Err(format!(
            "Tree hash mismatch: ledger has '{}', current worktree has '{}'",
            doc.tree_hash, current_tree_hash
        ));
    }

    if doc.overall_status != "PASS" {
        return Err(format!(
            "Ledger overall status is '{}', expected 'PASS'",
            doc.overall_status
        ));
    }

    Ok(doc)
}

pub fn run_ledger(args: &[String]) -> i32 {
    let mut root_path = match Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        Ok(out) if out.status.success() => {
            PathBuf::from(String::from_utf8_lossy(&out.stdout).trim().to_string())
        }
        _ => PathBuf::from("."),
    };

    let mut json_output = false;
    let mut results_dir: Option<PathBuf> = None;
    let mut out_file: Option<PathBuf> = None;
    let mut ledger_file: Option<PathBuf> = None;
    let mut subcommand = None;

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        match arg.as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_path = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--json" => {
                json_output = true;
            }
            "--results-dir" => {
                if idx + 1 < args.len() {
                    results_dir = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--out" => {
                if idx + 1 < args.len() {
                    out_file = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--ledger" => {
                if idx + 1 < args.len() {
                    ledger_file = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--base" | "--head" => {
                if idx + 1 < args.len() {
                    idx += 1;
                }
            }
            s if !s.starts_with("--") && subcommand.is_none() => {
                subcommand = Some(s.to_string());
            }
            _ => {}
        }
        idx += 1;
    }

    let sub = match subcommand.as_deref() {
        Some(s) => s,
        None => {
            if json_output {
                let out = serde_json::json!({
                    "gate": "ledger",
                    "status": "error",
                    "summary": "Missing subcommand for ledger",
                    "findings": []
                });
                println!("{}", out);
            } else {
                eprintln!("WAS: Fehlendes Unterkommando.\nWARUM: ledger verlangt 'tree-hash', 'write' oder 'verify'.\nFIX: Unterkommando angeben, z. B. 'cargo xtask ledger tree-hash'.");
            }
            return 2;
        }
    };

    match sub {
        "tree-hash" => match ledger_compute_tree_hash(&root_path) {
            Ok(hash) => {
                if json_output {
                    let out = serde_json::json!({
                        "gate": "ledger",
                        "status": "pass",
                        "summary": format!("Tree hash: {}", hash),
                        "findings": []
                    });
                    println!("{}", out);
                } else {
                    println!("{}", hash);
                }
                0
            }
            Err(e) => {
                if json_output {
                    let out = serde_json::json!({
                        "gate": "ledger",
                        "status": "error",
                        "summary": e,
                        "findings": []
                    });
                    println!("{}", out);
                } else {
                    eprintln!("WAS: Tree-Hash-Berechnung fehlgeschlagen.\nWARUM: {e}\nFIX: Prüfe Git-Worktree und Repository-Status.");
                }
                2
            }
        },
        "write" => {
            let res_dir = match results_dir {
                Some(d) => d,
                None => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "ledger",
                            "status": "error",
                            "summary": "Missing --results-dir argument",
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        eprintln!("WAS: Argument --results-dir fehlt.\nWARUM: 'ledger write' benötigt das Verzeichnis mit den Gate-Ergebnissen.\nFIX: --results-dir <pfad> übergeben.");
                    }
                    return 2;
                }
            };
            let out_p = match out_file {
                Some(f) => f,
                None => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "ledger",
                            "status": "error",
                            "summary": "Missing --out argument",
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        eprintln!("WAS: Argument --out fehlt.\nWARUM: 'ledger write' benötigt den Zielpfad für das Ledger-JSON.\nFIX: --out <pfad> übergeben.");
                    }
                    return 2;
                }
            };

            match ledger_write(&root_path, &res_dir, &out_p) {
                Ok(doc) => {
                    let is_pass = doc.overall_status == "PASS";
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "ledger",
                            "status": if is_pass { "pass" } else { "fail" },
                            "summary": format!("Ledger geschrieben unter {} mit Gesamtstatus {}", out_p.display(), doc.overall_status),
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        println!(
                            "Ledger geschrieben: {} ({})",
                            out_p.display(),
                            doc.overall_status
                        );
                    }
                    0
                }
                Err(e) => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "ledger",
                            "status": "error",
                            "summary": e,
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        eprintln!("WAS: Ledger-Erstellung fehlgeschlagen.\nWARUM: {e}\nFIX: Prüfe --results-dir und Dateiberechtigungen.");
                    }
                    2
                }
            }
        }
        "verify" => {
            let leg_file = match ledger_file {
                Some(f) => f,
                None => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "ledger",
                            "status": "error",
                            "summary": "Missing --ledger argument",
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        eprintln!("WAS: Argument --ledger fehlt.\nWARUM: 'ledger verify' benötigt den Pfad zur Ledger-Datei.\nFIX: --ledger <pfad> übergeben.");
                    }
                    return 2;
                }
            };

            match ledger_verify(&root_path, &leg_file) {
                Ok(doc) => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "ledger",
                            "status": "pass",
                            "summary": format!("Ledger {} ist gültig und hat Status PASS", leg_file.display()),
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        println!(
                            "Ledger verifiziert: {} (Tree-Hash: {})",
                            leg_file.display(),
                            doc.tree_hash
                        );
                    }
                    0
                }
                Err(e) => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "ledger",
                            "status": "fail",
                            "summary": format!("Ledger-Verifikation fehlgeschlagen: {}", e),
                            "findings": [
                                {
                                    "id": "ledger_mismatch",
                                    "severity": "error",
                                    "file": leg_file.display().to_string(),
                                    "line": 0,
                                    "message": e,
                                    "fix": "Führe alle Gates erneut aus und erstelle ein frisches Ledger."
                                }
                            ]
                        });
                        println!("{}", out);
                    } else {
                        eprintln!("WAS: Ledger-Verifikation fehlgeschlagen.\nWARUM: {e}\nFIX: Führe die Gate-Prüfungen erneut aus und erstelle ein aktuelles Ledger.");
                    }
                    1
                }
            }
        }
        _ => {
            if json_output {
                let out = serde_json::json!({
                    "gate": "ledger",
                    "status": "error",
                    "summary": format!("Unbekanntes Unterkommando: {}", sub),
                    "findings": []
                });
                println!("{}", out);
            } else {
                eprintln!("WAS: Unbekanntes Unterkommando '{}'.\nWARUM: Nur 'tree-hash', 'write' und 'verify' werden unterstützt.\nFIX: Gültiges Unterkommando angeben.", sub);
            }
            2
        }
    }
}
