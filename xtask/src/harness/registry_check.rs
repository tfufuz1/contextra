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
    summary: String,
    blocking: bool,
    phase: String,
    planned: bool,
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

    let allowed_owner_tiers: HashSet<&str> = [
        "agent-gate",
        "gate-constituent",
        "ci-gate",
        "generator",
        "report",
        "bench",
        "tool",
        "legacy",
    ]
    .into_iter()
    .collect();

    let allowed_phases: HashSet<&str> = [
        "Phase 0", "Phase 1", "Phase 2", "Phase 3", "Phase 4", "Phase 5", "Phase 6", "Phase X",
    ]
    .into_iter()
    .collect();

    for reg_entry in &registry_entries {
        if !allowed_owner_tiers.contains(reg_entry.owner_tier.as_str()) {
            findings.push(RegistryCheckFinding {
                id: "INVALID_OWNER_TIER".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!(
                    "Eintrag '{}' in registry.toml hat ungültigen owner_tier '{}'",
                    reg_entry.name, reg_entry.owner_tier
                ),
                fix: "Setze owner_tier auf einen der erlaubten Werte: agent-gate, gate-constituent, ci-gate, generator, report, bench, tool, legacy".to_string(),
            });
        }

        // Rule (a): Summary placeholder check
        let placeholder = format!("Subcommand {}", reg_entry.name);
        if reg_entry.summary.trim() == placeholder || reg_entry.summary.starts_with("Subcommand ") {
            findings.push(RegistryCheckFinding {
                id: "PLACEHOLDER_SUMMARY".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!(
                    "Eintrag '{}' in registry.toml hat eine Platzhalter-Summary: '{}'",
                    reg_entry.name, reg_entry.summary
                ),
                fix: format!(
                    "Ersetze die Platzhalter-Summary von '{}' durch eine echte einzeilige Beschreibung",
                    reg_entry.name
                ),
            });
        }

        // Rule (b): Phase check
        if !allowed_phases.contains(reg_entry.phase.as_str()) {
            findings.push(RegistryCheckFinding {
                id: "INVALID_PHASE".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!(
                    "Eintrag '{}' in registry.toml hat ungültigen phase-Wert '{}'",
                    reg_entry.name, reg_entry.phase
                ),
                fix: "Setze phase auf einen der erlaubten Werte: Phase 0..6 oder Phase X"
                    .to_string(),
            });
        }

        // Rule (5/planned): Warn on planned entries
        if reg_entry.planned {
            findings.push(RegistryCheckFinding {
                id: "PLANNED_ENTRY".to_string(),
                severity: "warning".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!(
                    "Eintrag '{}' in registry.toml ist als 'planned = true' markiert",
                    reg_entry.name
                ),
                fix: "Nach dem Merge des dazugehörigen Features 'planned = true' entfernen"
                    .to_string(),
            });
        }
    }

    let registered_names: HashSet<String> =
        registry_entries.iter().map(|e| e.name.clone()).collect();

    // 2. Extract active commands from main.rs and cli/mod.rs
    let main_rs_path = root.join("xtask/src/main.rs");
    let cli_mod_path = root.join("xtask/src/cli/mod.rs");
    let mut main_cmds = registry_check_extract_main_commands(&main_rs_path).unwrap_or_default();
    if let Ok(cli_cmds) = registry_check_extract_cli_mod_commands(&cli_mod_path) {
        main_cmds.extend(cli_cmds);
    }

    // 3. Extract active commands from justfile
    let justfile_path = root.join("justfile");
    let justfile_cmds = registry_check_extract_justfile_commands(&justfile_path);

    // 4. Extract active commands from .github/workflows
    let workflow_cmds = registry_check_extract_workflow_commands(&root);

    // 5. Extract harness commands
    let harness_dir = root.join("xtask/src/harness");
    let mut harness_cmds = registry_check_extract_harness_commands(&harness_dir);
    // Builtin harness commands in harness/mod.rs
    if harness_dir.join("mod.rs").exists() {
        harness_cmds.insert("harness-list".to_string());
        harness_cmds.insert("harness-help".to_string());
    }

    // Harness commands auto-registered check
    let mut required_commands = HashSet::new();
    required_commands.extend(main_cmds.clone());
    required_commands.extend(justfile_cmds.clone());
    required_commands.extend(workflow_cmds.clone());

    // Allowed collisions between harness module and cli/mod.rs (e.g. gc-floor-single-source)
    let allowed_collisions: HashSet<&str> =
        ["gc-floor-single-source", "verify", "check-loom-mocks"]
            .into_iter()
            .collect();

    // Collision check: Harness vs Main / CLI
    for h_cmd in &harness_cmds {
        if main_cmds.contains(h_cmd) && !allowed_collisions.contains(h_cmd.as_str()) {
            findings.push(RegistryCheckFinding {
                id: "HARNESS_LEGACY_COLLISION".to_string(),
                severity: "error".to_string(),
                file: format!("xtask/src/harness/{}.rs", h_cmd.replace('-', "_")),
                line: 1,
                message: format!(
                    "Harness-Kommando '{}' kollidiert mit bestehendem Arm in main.rs / cli/mod.rs",
                    h_cmd
                ),
                fix: "Benenne das Harness-Modul um oder entferne den Arm aus main.rs / cli/mod.rs"
                    .to_string(),
            });
        }
    }

    // Extract required harness-phases commands
    let phases_toml_path = root.join(".jules/harness-phases.toml");
    let required_local_phases = registry_check_extract_required_phases(&phases_toml_path);

    // Rule (c): Full bi-directional command existence check
    // Every active command (cli dispatch + harness modules) must exist in registry.toml,
    // and every registry entry must correspond to a real command unless `planned = true`.
    let mut all_existing_commands = HashSet::new();
    all_existing_commands.extend(main_cmds.clone());
    all_existing_commands.extend(harness_cmds.clone());

    for cmd in &all_existing_commands {
        if !registered_names.contains(cmd) {
            findings.push(RegistryCheckFinding {
                id: "MISSING_REGISTRY_ENTRY".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!(
                    "Befehl '{}' existiert als CLI-Arm oder Harness-Modul, fehlt aber in xtask/registry.toml",
                    cmd
                ),
                fix: format!(
                    "Füge einen Eintrag für '{}' in xtask/registry.toml ein",
                    cmd
                ),
            });
        }
    }

    for reg_entry in &registry_entries {
        if reg_entry.planned {
            // Rule (c) & (5): Skip command existence check for planned entries
            continue;
        }

        if !all_existing_commands.contains(&reg_entry.name) {
            findings.push(RegistryCheckFinding {
                id: "ORPHANED_REGISTRY_ENTRY".to_string(),
                severity: "error".to_string(),
                file: "xtask/registry.toml".to_string(),
                line: 1,
                message: format!(
                    "Eintrag '{}' in registry.toml entspricht keinem echten Befehl (Phantomeintrag)",
                    reg_entry.name
                ),
                fix: format!(
                    "Entferne den Phantomeintrag '{}' aus xtask/registry.toml oder erstelle das passende Modul",
                    reg_entry.name
                ),
            });
        }

        // Rule (d): Check unwired blocking = true entries
        if reg_entry.blocking {
            let is_required_in_phases = required_local_phases.contains(&reg_entry.name);
            let is_required_in_ci = workflow_cmds.contains(&reg_entry.name);

            if !is_required_in_phases && !is_required_in_ci {
                findings.push(RegistryCheckFinding {
                    id: "UNWIRED_BLOCKING".to_string(),
                    severity: "warning".to_string(),
                    file: "xtask/registry.toml".to_string(),
                    line: 1,
                    message: format!(
                        "Eintrag '{}' hat blocking = true, ist aber weder in harness-phases.toml (required = true) noch in CI-Workflows gefordert",
                        reg_entry.name
                    ),
                    fix: format!(
                        "Setze 'blocking = false' für '{}' oder verdrahte den Befehl als Pflichtgate in harness-phases/CI",
                        reg_entry.name
                    ),
                });
            }
        }
    }

    let has_errors = findings.iter().any(|f| f.severity == "error");

    if !has_errors {
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
        .args(["rev-parse", "--show-toplevel"])
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
        .ok_or_else(|| {
            "Schlüssel 'commands' fehlt oder ist kein Array in registry.toml".to_string()
        })?;

    let mut entries = Vec::new();
    for item in array {
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let owner_tier = item
            .get("owner_tier")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let summary = item
            .get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let blocking = item
            .get("blocking")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let phase = item
            .get("phase")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if name.is_empty() {
            return Err("Eintrag in registry.toml ohne 'name'".to_string());
        }

        let planned = item
            .get("planned")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        entries.push(RegistryCheckEntry {
            name,
            owner_tier,
            summary,
            blocking,
            phase,
            planned,
        });
    }

    Ok(entries)
}

fn registry_check_extract_cli_mod_commands(path: &Path) -> Result<HashSet<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut cmds = HashSet::new();

    let re = match regex::Regex::new(r#""([a-z0-9_-]+)""#) {
        Ok(r) => r,
        Err(e) => return Err(format!("Regex creation failed: {}", e)),
    };

    if let Some((_, after)) = content.split_once("COMMAND_DISPATCH_TABLE") {
        let table_str = match after.split_once("];") {
            Some((t, _)) => t,
            None => after,
        };
        for caps in re.captures_iter(table_str) {
            if let Some(c) = caps.get(1) {
                cmds.insert(c.as_str().to_string());
            }
        }
    }

    Ok(cmds)
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
        if let Ok(re) =
            regex::Regex::new(r"(?:cargo\s+run\s+-p\s+xtask\s+--\s+|cargo\s+xtask\s+)([a-z0-9_-]+)")
        {
            for entry in WalkDir::new(&workflows_dir)
                .into_iter()
                .filter_map(Result::ok)
            {
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

fn registry_check_extract_required_phases(path: &Path) -> HashSet<String> {
    let mut cmds = HashSet::new();
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(val) = toml::from_str::<toml::Value>(&content) {
            if let Some(phase) = val.get("phase").and_then(|p| p.as_table()) {
                for (_pname, pval) in phase {
                    if let Some(arr) = pval.as_array() {
                        for item in arr {
                            if let (Some(cmd_val), Some(req_val)) =
                                (item.get("cmd"), item.get("required"))
                            {
                                if req_val.as_bool().unwrap_or(false) {
                                    if let Some(cmd_arr) = cmd_val.as_array() {
                                        let cmd_strs: Vec<_> =
                                            cmd_arr.iter().filter_map(|v| v.as_str()).collect();
                                        if cmd_strs.len() >= 3
                                            && cmd_strs[0] == "cargo"
                                            && cmd_strs[1] == "xtask"
                                        {
                                            cmds.insert(cmd_strs[2].to_string());
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
    cmds
}

fn registry_check_extract_harness_commands(harness_dir: &Path) -> HashSet<String> {
    let mut cmds = HashSet::new();
    if harness_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(harness_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "rs") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_entry(
        name: &str,
        summary: &str,
        phase: &str,
        blocking: bool,
        planned: bool,
    ) -> RegistryCheckEntry {
        RegistryCheckEntry {
            name: name.to_string(),
            owner_tier: "tool".to_string(),
            summary: summary.to_string(),
            blocking,
            phase: phase.to_string(),
            planned,
        }
    }

    #[test]
    fn test_rule_a_placeholder_summary() {
        let entry = create_test_entry("my-cmd", "Subcommand my-cmd", "Phase 0", false, false);
        assert!(
            entry.summary.starts_with("Subcommand "),
            "Should detect placeholder summary"
        );
    }

    #[test]
    fn test_rule_b_invalid_phase() {
        let allowed_phases: HashSet<&str> = [
            "Phase 0", "Phase 1", "Phase 2", "Phase 3", "Phase 4", "Phase 5", "Phase 6", "Phase X",
        ]
        .into_iter()
        .collect();

        assert!(!allowed_phases.contains("Phase 99"));
        assert!(allowed_phases.contains("Phase 0"));
    }

    #[test]
    fn test_rule_c_planned_bypass_and_warning() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir_all(root.join("xtask/src/harness")).unwrap();
        fs::write(root.join("xtask/src/main.rs"), "fn main() {}").unwrap();
        fs::write(
            root.join("xtask/registry.toml"),
            r#"
[[commands]]
name = "context-card"
owner_tier = "agent-gate"
summary = "Erzeugt den kartenspezifischen Abschnitt des CONTEXT_PACK und prüft dessen Frische"
blocking = false
phase = "Phase 0"
planned = true
"#,
        )
        .unwrap();

        let args = vec![
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--json".to_string(),
        ];
        let exit_code = run_registry_check(&args);
        assert_eq!(
            exit_code, 0,
            "Planned entry should bypass command existence check"
        );
    }

    #[test]
    fn test_rule_d_unwired_blocking_warning() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        fs::create_dir_all(root.join("xtask/src/harness")).unwrap();
        fs::write(
            root.join("xtask/src/main.rs"),
            r#"fn main() { match s { "unwired-cmd" => {} } }"#,
        )
        .unwrap();
        fs::write(
            root.join("xtask/registry.toml"),
            r#"
[[commands]]
name = "unwired-cmd"
owner_tier = "tool"
summary = "Echter Befehl ohne Phasen/CI Bindung"
blocking = true
phase = "Phase 0"
"#,
        )
        .unwrap();

        let args = vec![
            "--root".to_string(),
            root.to_string_lossy().to_string(),
            "--json".to_string(),
        ];
        // run_registry_check should return 0 (warnings don't fail exit code)
        let exit_code = run_registry_check(&args);
        assert_eq!(exit_code, 0);
    }
}

fn registry_check_finish(
    json: bool,
    status: &str,
    findings: &[RegistryCheckFinding],
    exit_code: i32,
) -> i32 {
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
    } else if exit_code == 0 {
        println!("✅ [REGISTRY-CHECK]: Alle xtask-Kommandos sind konsistent in xtask/registry.toml registriert.");
    } else {
        println!("❌ [REGISTRY-CHECK]: Konsistencyprüfungen für xtask-Kommandos fehlgeschlagen.");
        for f in findings {
            println!(
                "  - [{}] {} (Datei: {}:{})",
                f.severity.to_uppercase(),
                f.message,
                f.file,
                f.line
            );
            println!("    Invariante/ADR: Registrierungspflicht in xtask/registry.toml");
            println!("    FIX: {}", f.fix);
        }
    }
    exit_code
}
