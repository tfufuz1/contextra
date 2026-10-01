//! Prüft Vollständigkeit und Konsistenz von xtask-Kommandos gegen `xtask/registry.toml`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug)]
struct RegistryCheckFinding {
    id: String,
    severity: String,
    file: String,
    line: usize,
    message: String,
    fix: String,
}

#[derive(Debug, Default)]
struct RegistryCheckArgs {
    root: Option<PathBuf>,
    _base: Option<String>,
    _head: Option<String>,
    json: bool,
}

#[derive(Debug)]
struct RegistryCheckEntry {
    name: String,
    owner_tier: String,
    _summary: String,
    _blocking: bool,
    _phase: String,
}

/// Runs the `registry-check` gate.
pub fn run_registry_check(args: &[String]) -> i32 {
    let parsed_args = match registry_check_parse_args(args) {
        Ok(a) => a,
        Err(err_msg) => {
            eprintln!("Fehler beim Parsen der Argumente: {}", err_msg);
            return 2;
        }
    };

    let root = match parsed_args.root {
        Some(r) => r,
        None => match registry_check_detect_root() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Fehler beim Ermitteln des Repository-Roots: {}", e);
                return 2;
            }
        },
    };

    let mut findings = Vec::new();

    // 1. Parse registry.toml
    let registry_path = root.join("xtask/registry.toml");
    let registry_entries = match registry_check_load_registry(&registry_path) {
        Ok(entries) => entries,
        Err(e) => {
            findings.push(RegistryCheckFinding {
                id: "REGISTRY_PARSE_ERROR".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!("registry.toml konnte nicht geparst werden: {}", e),
                fix: "Prüfe xtask/registry.toml auf gültiges TOML-Format".to_string(),
            });
            return registry_check_finish(parsed_args.json, "fail", &findings, 1);
        }
    };

    let registered_names: HashSet<String> = registry_entries.iter().map(|e| e.name.clone()).collect();

    // 2. Extract active commands from main.rs
    let main_rs_path = root.join("xtask/src/main.rs");
    let main_cmds = match registry_check_extract_main_commands(&main_rs_path) {
        Ok(cmds) => cmds,
        Err(e) => {
            findings.push(RegistryCheckFinding {
                id: "MAIN_RS_READ_ERROR".to_string(),
                severity: "error".to_string(),
                file: "xtask/src/main.rs".to_string(),
                line: 1,
                message: format!("xtask/src/main.rs konnte nicht gelesen werden: {}", e),
                fix: "Stelle sicher, dass xtask/src/main.rs existiert".to_string(),
            });
            return registry_check_finish(parsed_args.json, "error", &findings, 2);
        }
    };

    // 3. Extract active commands from justfile
    let justfile_path = root.join("justfile");
    let justfile_cmds = registry_check_extract_justfile_commands(&justfile_path);

    // 4. Extract active commands from .github/workflows
    let workflow_cmds = registry_check_extract_workflow_commands(&root);

    // 5. Extract harness commands
    let harness_dir = root.join("xtask/src/harness");
    let mut harness_cmds = registry_check_extract_harness_commands(&harness_dir);
    // Builtin harness commands
    harness_cmds.insert("harness-list".to_string());
    harness_cmds.insert("harness-help".to_string());

    // Harness commands auto-registered check
    let mut required_commands = HashSet::new();
    required_commands.extend(main_cmds.clone());
    required_commands.extend(justfile_cmds.clone());
    required_commands.extend(workflow_cmds.clone());

    // Collision check: Harness vs Main
    for h_cmd in &harness_cmds {
        if main_cmds.contains(h_cmd) {
            findings.push(RegistryCheckFinding {
                id: "HARNESS_LEGACY_COLLISION".to_string(),
                severity: "error".to_string(),
                file: format!("xtask/src/harness/{}.rs", h_cmd.replace('-', "_")),
                line: 1,
                message: format!("Harness-Kommando '{}' kollidiert mit bestehendem Arm in main.rs", h_cmd),
                fix: "Benenne das Harness-Modul um oder entferne den Arm aus main.rs".to_string(),
            });
        }
    }

    // Check missing registrations (excluding harness commands which are implicitly registered)
    for cmd in &required_commands {
        if !harness_cmds.contains(cmd) && !registered_names.contains(cmd) {
            findings.push(RegistryCheckFinding {
                id: "MISSING_REGISTRY_ENTRY".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!("Kommando '{}' wird verwendet, fehlt aber in xtask/registry.toml", cmd),
                fix: format!("Füge einen Eintrag für '{}' in xtask/registry.toml ein", cmd),
            });
        }
    }

    // Check orphaned entries in registry.toml
    let mut all_known_active = HashSet::new();
    all_known_active.extend(required_commands);
    all_known_active.extend(harness_cmds);

    for reg_entry in &registry_entries {
        if reg_entry.owner_tier == "legacy" && !all_known_active.contains(&reg_entry.name) {
            findings.push(RegistryCheckFinding {
                id: "ORPHANED_REGISTRY_ENTRY".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!("Eintrag '{}' in registry.toml ist verwaist (wird nirgends verwendet)", reg_entry.name),
                fix: format!("Entferne den verwaisten Eintrag '{}' aus xtask/registry.toml", reg_entry.name),
            });
        }
    }

    if findings.is_empty() {
        registry_check_finish(parsed_args.json, "pass", &findings, 0)
    } else {
        registry_check_finish(parsed_args.json, "fail", &findings, 1)
    }
}

fn registry_check_parse_args(args: &[String]) -> Result<RegistryCheckArgs, String> {
    let mut result = RegistryCheckArgs::default();
    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--json" {
            result.json = true;
        } else if arg == "--root" {
            idx += 1;
            if idx >= args.len() {
                return Err("Fehlender Wert für --root".to_string());
            }
            result.root = Some(PathBuf::from(&args[idx]));
        } else if let Some(val) = arg.strip_prefix("--root=") {
            result.root = Some(PathBuf::from(val));
        } else if arg == "--base" {
            idx += 1;
            if idx >= args.len() {
                return Err("Fehlender Wert für --base".to_string());
            }
            result._base = Some(args[idx].clone());
        } else if let Some(val) = arg.strip_prefix("--base=") {
            result._base = Some(val.to_string());
        } else if arg == "--head" {
            idx += 1;
            if idx >= args.len() {
                return Err("Fehlender Wert für --head".to_string());
            }
            result._head = Some(args[idx].clone());
        } else if let Some(val) = arg.strip_prefix("--head=") {
            result._head = Some(val.to_string());
        }
        idx += 1;
    }
    Ok(result)
}

fn registry_check_detect_root() -> Result<PathBuf, String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--show-toplevel"])
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(PathBuf::from(path_str))
        }
        _ => {
            if Path::new("Cargo.toml").exists() {
                Ok(PathBuf::from("."))
            } else {
                Err("Konnte Repository-Root nicht bestimmen".to_string())
            }
        }
    }
}

fn registry_check_load_registry(path: &Path) -> Result<Vec<RegistryCheckEntry>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value: toml::Value = toml::from_str(&content).map_err(|e| e.to_string())?;

    let array = value
        .get("commands")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "Schlüssel 'commands' fehlt oder ist kein Array in registry.toml".to_string())?;

    let mut entries = Vec::new();
    for item in array {
        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let owner_tier = item.get("owner_tier").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let summary = item.get("summary").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let blocking = item.get("blocking").and_then(|v| v.as_bool()).unwrap_or(false);
        let phase = item.get("phase").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if name.is_empty() {
            return Err("Eintrag in registry.toml ohne 'name'".to_string());
        }

        entries.push(RegistryCheckEntry {
            name,
            owner_tier,
            _summary: summary,
            _blocking: blocking,
            _phase: phase,
        });
    }

    Ok(entries)
}

fn registry_check_extract_main_commands(path: &Path) -> Result<HashSet<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut cmds = HashSet::new();

    let re = match regex::Regex::new(r#""([a-z0-9_-]+)""#) {
        Ok(r) => r,
        Err(e) => return Err(format!("Regex creation failed: {}", e)),
    };

    for line in content.lines() {
        if let Some((patterns, _)) = line.split_once("=>") {
            for caps in re.captures_iter(patterns) {
                if let Some(c) = caps.get(1) {
                    cmds.insert(c.as_str().to_string());
                }
            }
        }
    }

    Ok(cmds)
}

fn registry_check_extract_justfile_commands(path: &Path) -> HashSet<String> {
    let mut cmds = HashSet::new();
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(re) = regex::Regex::new(r"cargo\s+xtask\s+([a-z0-9_-]+)") {
            for line in content.lines() {
                for caps in re.captures_iter(line) {
                    if let Some(c) = caps.get(1) {
                        cmds.insert(c.as_str().to_string());
                    }
                }
            }
        }
    }
    cmds
}

fn registry_check_extract_workflow_commands(root: &Path) -> HashSet<String> {
    let mut cmds = HashSet::new();
    let workflows_dir = root.join(".github/workflows");
    if workflows_dir.exists() {
        if let Ok(re) = regex::Regex::new(r"(?:cargo\s+run\s+-p\s+xtask\s+--\s+|cargo\s+xtask\s+)([a-z0-9_-]+)") {
            for entry in WalkDir::new(&workflows_dir).into_iter().filter_map(Result::ok) {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(content) = fs::read_to_string(path) {
                        for line in content.lines() {
                            for caps in re.captures_iter(line) {
                                if let Some(c) = caps.get(1) {
                                    cmds.insert(c.as_str().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    cmds
}

fn registry_check_extract_harness_commands(harness_dir: &Path) -> HashSet<String> {
    let mut cmds = HashSet::new();
    if harness_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(harness_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |e| e == "rs") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if stem != "mod" {
                            cmds.insert(stem.replace('_', "-"));
                        }
                    }
                }
            }
        }
    }
    cmds
}

fn registry_check_finish(json: bool, status: &str, findings: &[RegistryCheckFinding], exit_code: i32) -> i32 {
    if json {
        let json_findings: Vec<_> = findings
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

        let output = serde_json::json!({
            "gate": "registry-check",
            "status": status,
            "summary": if exit_code == 0 { "Vollständigkeitsprüfung erfolgreich" } else { "Registry-Check fehlgeschlagen" },
            "findings": json_findings
        });
        println!("{}", output);
    } else {
        if exit_code == 0 {
            println!("✅ [REGISTRY-CHECK]: Alle xtask-Kommandos sind konsistent in xtask/registry.toml registriert.");
        } else {
            println!("❌ [REGISTRY-CHECK]: Konsistenzprüfungen für xtask-Kommandos fehlgeschlagen.");
            for f in findings {
                println!("  - [{}] {} (Datei: {}:{})", f.severity.to_uppercase(), f.message, f.file, f.line);
                println!("    Invariante/ADR: Registrierungspflicht in xtask/registry.toml");
                println!("    FIX: {}", f.fix);
            }
        }
    }
    exit_code
}
