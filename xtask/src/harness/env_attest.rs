//! Environment attestation check for toolchain pinning and required build tools.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvAttestToolConfig {
    pub name: String,
    pub version_cmd: Vec<String>,
    pub min_version: Option<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Deserialize)]
pub struct EnvAttestRequiredToolsFile {
    pub tool: Vec<EnvAttestToolConfig>,
}

#[derive(Debug, Deserialize)]
pub struct EnvAttestRustToolchain {
    pub toolchain: EnvAttestToolchainChannel,
}

#[derive(Debug, Deserialize)]
pub struct EnvAttestToolchainChannel {
    pub channel: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvAttestFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvAttestReport {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<EnvAttestFinding>,
}

pub fn env_attest_read_toolchain_pin(root: &Path) -> Result<String, String> {
    let path = root.join("rust-toolchain.toml");
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let parsed: EnvAttestRustToolchain =
        toml::from_str(&content).map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
    Ok(parsed.toolchain.channel)
}

pub fn env_attest_read_required_tools(root: &Path) -> Result<Vec<EnvAttestToolConfig>, String> {
    let path = root.join(".jules/setup/required-tools.toml");
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let parsed: EnvAttestRequiredToolsFile =
        toml::from_str(&content).map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
    Ok(parsed.tool)
}

pub fn env_attest_check_cmd(cmd_args: &[String]) -> Result<String, String> {
    if cmd_args.is_empty() {
        return Err("Empty command array".to_string());
    }
    let prog = &cmd_args[0];
    let args = &cmd_args[1..];

    let output = Command::new(prog)
        .args(args)
        .output()
        .map_err(|e| format!("Command '{prog}' failed to execute: {e}"))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if !stdout.is_empty() {
            Ok(stdout)
        } else {
            Ok(stderr)
        }
    } else {
        Err(format!(
            "Command '{prog}' exited with code {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

pub fn env_attest_execute(root: &Path) -> EnvAttestReport {
    let mut findings = Vec::new();
    let mut overall_pass = true;

    // 1. Toolchain channel check
    let channel_pin = match env_attest_read_toolchain_pin(root) {
        Ok(ch) => ch,
        Err(e) => {
            findings.push(EnvAttestFinding {
                id: "toolchain_pin_missing".to_string(),
                severity: "error".to_string(),
                file: "rust-toolchain.toml".to_string(),
                line: 0,
                message: e,
                fix: "Stelle sicher, dass rust-toolchain.toml mit toolchain.channel vorhanden ist."
                    .to_string(),
            });
            "UNKNOWN".to_string()
        }
    };

    if channel_pin != "UNKNOWN" {
        match env_attest_check_cmd(&["rustc".to_string(), "--version".to_string()]) {
            Ok(ver) => {
                if !ver.contains(&channel_pin) {
                    overall_pass = false;
                    findings.push(EnvAttestFinding {
                        id: "rustc_version_mismatch".to_string(),
                        severity: "error".to_string(),
                        file: "rust-toolchain.toml".to_string(),
                        line: 0,
                        message: format!(
                            "rustc Version '{}' entspricht nicht dem Pin '{}'",
                            ver, channel_pin
                        ),
                        fix: format!(
                            "Session abbrechen. Führe 'rustup default {}' aus.",
                            channel_pin
                        ),
                    });
                }
            }
            Err(e) => {
                overall_pass = false;
                findings.push(EnvAttestFinding {
                    id: "rustc_not_found".to_string(),
                    severity: "error".to_string(),
                    file: "PATH".to_string(),
                    line: 0,
                    message: format!("rustc konnte nicht ausgeführt werden: {e}"),
                    fix: "Installiere Rust via rustup.".to_string(),
                });
            }
        }

        match env_attest_check_cmd(&["cargo".to_string(), "--version".to_string()]) {
            Ok(ver) => {
                if !ver.contains(&channel_pin) {
                    overall_pass = false;
                    findings.push(EnvAttestFinding {
                        id: "cargo_version_mismatch".to_string(),
                        severity: "error".to_string(),
                        file: "rust-toolchain.toml".to_string(),
                        line: 0,
                        message: format!(
                            "cargo Version '{}' entspricht nicht dem Pin '{}'",
                            ver, channel_pin
                        ),
                        fix: format!(
                            "Session abbrechen. Führe 'rustup default {}' aus.",
                            channel_pin
                        ),
                    });
                }
            }
            Err(e) => {
                overall_pass = false;
                findings.push(EnvAttestFinding {
                    id: "cargo_not_found".to_string(),
                    severity: "error".to_string(),
                    file: "PATH".to_string(),
                    line: 0,
                    message: format!("cargo konnte nicht ausgeführt werden: {e}"),
                    fix: "Installiere Cargo via rustup.".to_string(),
                });
            }
        }
    }

    // 2. Required and optional tools check
    match env_attest_read_required_tools(root) {
        Ok(tools) => {
            for tool in tools {
                match env_attest_check_cmd(&tool.version_cmd) {
                    Ok(_output) => {}
                    Err(e) => {
                        if tool.required {
                            overall_pass = false;
                            findings.push(EnvAttestFinding {
                                id: format!("required_tool_missing_{}", tool.name),
                                severity: "error".to_string(),
                                file: ".jules/setup/required-tools.toml".to_string(),
                                line: 0,
                                message: format!(
                                    "Erforderliches Tool '{}' fehlt oder schlug fehl: {}",
                                    tool.name, e
                                ),
                                fix: format!("Installiere Tool '{}' im System.", tool.name),
                            });
                        } else {
                            findings.push(EnvAttestFinding {
                                id: format!("optional_tool_missing_{}", tool.name),
                                severity: "warn".to_string(),
                                file: ".jules/setup/required-tools.toml".to_string(),
                                line: 0,
                                message: format!(
                                    "Optionales Tool '{}' nicht vorhanden: {}",
                                    tool.name, e
                                ),
                                fix: format!("Installieren von '{}' wird empfohlen.", tool.name),
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            overall_pass = false;
            findings.push(EnvAttestFinding {
                id: "required_tools_config_error".to_string(),
                severity: "error".to_string(),
                file: ".jules/setup/required-tools.toml".to_string(),
                line: 0,
                message: e,
                fix: "Stelle sicher, dass .jules/setup/required-tools.toml eine gültige Konfiguration enthält.".to_string(),
            });
        }
    }

    let status = if overall_pass { "pass" } else { "fail" };
    let summary = if overall_pass {
        format!(
            "Toolchain ({}) und alle erforderlichen Werkzeuge verifiziert",
            channel_pin
        )
    } else {
        format!(
            "Abweichung bei Toolchain oder erforderlichen Werkzeugen ({})",
            findings.len()
        )
    };

    let report = EnvAttestReport {
        gate: "env-attest".to_string(),
        status: status.to_string(),
        summary,
        findings,
    };

    // Write report to .jules/local/env-attest.json
    let local_dir = root.join(".jules/local");
    let _ = fs::create_dir_all(&local_dir);
    let json_file = local_dir.join("env-attest.json");
    if let Ok(content) = serde_json::to_string_pretty(&report) {
        let _ = fs::write(json_file, content);
    }

    report
}

pub fn run_env_attest(args: &[String]) -> i32 {
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
            "--base" | "--head" => {
                if idx + 1 < args.len() {
                    idx += 1;
                }
            }
            _ => {}
        }
        idx += 1;
    }

    let report = env_attest_execute(&root_path);

    if json_output {
        if let Ok(json) = serde_json::to_string_pretty(&report) {
            println!("{}", json);
        }
    } else if report.status == "pass" {
        println!("{}", report.summary);
    } else {
        eprintln!("WAS: Umgebungs-Attestierung fehlgeschlagen.");
        eprintln!("WARUM: {}", report.summary);
        for f in &report.findings {
            eprintln!(
                " - [{}] {}: {}",
                f.severity.to_uppercase(),
                f.file,
                f.message
            );
            eprintln!("   FIX: {}", f.fix);
        }
    }

    if report.status == "pass" {
        0
    } else {
        1
    }
}
