//! Loop guard and escalation manager for iterative error prevention.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct LoopGuardState {
    pub error_counts: BTreeMap<String, u32>,
    pub error_logs: BTreeMap<String, String>,
    pub file_edits: BTreeMap<String, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoopGuardFinding {
    pub id: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub message: String,
    pub fix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoopGuardReport {
    pub gate: String,
    pub status: String,
    pub summary: String,
    pub findings: Vec<LoopGuardFinding>,
}

pub fn loop_guard_normalize_error(text: &str) -> String {
    let mut normalized = text.to_string();

    // Remove hex addresses: 0x[0-9a-fA-F]+ -> 0xHEX
    if let Ok(re) = Regex::new(r"0x[0-9a-fA-F]+") {
        normalized = re.replace_all(&normalized, "0xHEX").to_string();
    }

    // Remove ISO-8601 timestamps and time strings: YYYY-MM-DD, HH:MM:SS
    if let Ok(re) =
        Regex::new(r"\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?")
    {
        normalized = re.replace_all(&normalized, "<TIMESTAMP>").to_string();
    }
    if let Ok(re) = Regex::new(r"\b\d{2}:\d{2}:\d{2}\b") {
        normalized = re.replace_all(&normalized, "<TIME>").to_string();
    }

    // Remove file path prefixes: e.g. /app/, /home/.../, /tmp/.../
    if let Ok(re) = Regex::new(r"/(?:[a-zA-Z0-9_.-]+/)+") {
        normalized = re.replace_all(&normalized, "<PATH>/").to_string();
    }

    // Remove line and column numbers: :123:45 -> :0:0, :123 -> :0
    if let Ok(re) = Regex::new(r":\d+:\d+") {
        normalized = re.replace_all(&normalized, ":0:0").to_string();
    }
    if let Ok(re) = Regex::new(r":\d+\b") {
        normalized = re.replace_all(&normalized, ":0").to_string();
    }

    // Remove bracketed numbers: [1/8], [123], (456)
    if let Ok(re) = Regex::new(r"\[\d+/\d+\]|\[\d+\]|\(\d+\)") {
        normalized = re.replace_all(&normalized, "[N]").to_string();
    }

    // Collapse whitespace
    if let Ok(re) = Regex::new(r"\s+") {
        normalized = re.replace_all(&normalized, " ").to_string();
    }

    normalized.trim().to_string()
}

pub fn loop_guard_hash_error(normalized_text: &str) -> String {
    blake3::hash(normalized_text.as_bytes())
        .to_hex()
        .to_string()
}

pub fn loop_guard_get_state_path(root: &Path) -> PathBuf {
    root.join(".jules/local/loop-guard.json")
}

pub fn loop_guard_load_state(root: &Path) -> LoopGuardState {
    let path = loop_guard_get_state_path(root);
    if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        LoopGuardState::default()
    }
}

pub fn loop_guard_save_state(root: &Path, state: &LoopGuardState) -> Result<(), String> {
    let path = loop_guard_get_state_path(root);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Failed to serialize loop-guard state: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write state to {}: {e}", path.display()))
}

pub fn loop_guard_record_text(
    root: &Path,
    gate_name: Option<&str>,
    content: &str,
    edited_path: Option<&str>,
) -> Result<(String, u32), String> {
    let mut state = loop_guard_load_state(root);
    let mut recorded_hash = String::new();
    let mut count = 0;

    if let Some(path_str) = edited_path {
        let current_edits = state.file_edits.entry(path_str.to_string()).or_insert(0);
        *current_edits += 1;
    }

    if !content.is_empty() {
        let full_text = match gate_name {
            Some(name) if !name.is_empty() => format!("[{}] {}", name, content),
            _ => content.to_string(),
        };
        let norm = loop_guard_normalize_error(&full_text);
        let hash = loop_guard_hash_error(&norm);

        let entry = state.error_counts.entry(hash.clone()).or_insert(0);
        *entry += 1;
        count = *entry;
        recorded_hash = hash.clone();

        // Keep last 40 lines of original log for escalation report
        let lines: Vec<&str> = content.lines().collect();
        let start = if lines.len() > 40 {
            lines.len() - 40
        } else {
            0
        };
        let snippet = lines[start..].join("\n");
        state.error_logs.insert(hash, snippet);
    }

    loop_guard_save_state(root, &state)?;
    Ok((recorded_hash, count))
}

pub fn loop_guard_record(
    root: &Path,
    gate_name: Option<&str>,
    log_file: Option<&Path>,
    edited_path: Option<&str>,
) -> Result<(String, u32), String> {
    if let Some(log_p) = log_file {
        let content = fs::read_to_string(log_p)
            .map_err(|e| format!("Failed to read log file {}: {e}", log_p.display()))?;
        loop_guard_record_text(root, gate_name, &content, edited_path)
    } else {
        loop_guard_record_text(root, gate_name, "", edited_path)
    }
}

pub fn loop_guard_check(root: &Path, max_file_edits: u32) -> Result<(), String> {
    let state = loop_guard_load_state(root);

    for (hash, count) in &state.error_counts {
        if *count >= 2 {
            return Err(format!(
                "Repeated error detected! Error hash {} occurred {} times.",
                hash, count
            ));
        }
    }

    for (file, edits) in &state.file_edits {
        if *edits > max_file_edits {
            return Err(format!(
                "Maximum file edit limit exceeded for {}: {} edits (max {})",
                file, edits, max_file_edits
            ));
        }
    }

    Ok(())
}

pub fn loop_guard_stop(root: &Path, reason: &str) -> Result<(), String> {
    let state = loop_guard_load_state(root);

    let last_hash = state
        .error_counts
        .iter()
        .last()
        .map(|(h, _)| h.as_str())
        .unwrap_or("N/A");
    let last_log = state
        .error_logs
        .get(last_hash)
        .map(|s| s.as_str())
        .unwrap_or("Kein Log verfügbar.");

    let file_list: Vec<String> = state.file_edits.keys().cloned().collect();
    let file_str = if file_list.is_empty() {
        "Keine veränderten Dateien aufgezeichnet.".to_string()
    } else {
        file_list.join("\n- ")
    };

    let escalate_content = format!(
        "# ESCALATION REPORT\n\n\
         ## Reason\n{}\n\n\
         ## Error Hash\n{}\n\n\
         ## Last Log Snippet (max 40 lines)\n```\n{}\n```\n\n\
         ## Modified Files\n- {}\n\n\
         ## Recommended Next Step\n\
         Überprüfe die Invarianten, erstelle ein Minimalbeispiel und verändere nicht weiter blind den Code.\n",
        reason, last_hash, last_log, file_str
    );

    let local_dir = root.join(".jules/local");
    let _ = fs::create_dir_all(&local_dir);
    let escalate_file = local_dir.join("ESCALATE.md");

    fs::write(&escalate_file, escalate_content).map_err(|e| {
        format!(
            "Failed to write ESCALATE.md at {}: {e}",
            escalate_file.display()
        )
    })?;

    Ok(())
}

pub fn loop_guard_reset(root: &Path) -> Result<(), String> {
    let path = loop_guard_get_state_path(root);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| {
            format!(
                "Failed to remove loop-guard state file {}: {e}",
                path.display()
            )
        })?;
    }
    Ok(())
}

pub fn run_loop_guard(args: &[String]) -> i32 {
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
    let mut subcommand = None;
    let mut gate_name = None;
    let mut log_file = None;
    let mut edited_path = None;
    let mut max_file_edits = 6;
    let mut stop_reason = None;

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
            "--gate" => {
                if idx + 1 < args.len() {
                    gate_name = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "--log" => {
                if idx + 1 < args.len() {
                    log_file = Some(PathBuf::from(&args[idx + 1]));
                    idx += 1;
                }
            }
            "--edited" => {
                if idx + 1 < args.len() {
                    edited_path = Some(args[idx + 1].clone());
                    idx += 1;
                }
            }
            "--max-file-edits" => {
                if idx + 1 < args.len() {
                    if let Ok(v) = args[idx + 1].parse::<u32>() {
                        max_file_edits = v;
                    }
                    idx += 1;
                }
            }
            "--reason" => {
                if idx + 1 < args.len() {
                    stop_reason = Some(args[idx + 1].clone());
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
                    "gate": "loop-guard",
                    "status": "error",
                    "summary": "Missing subcommand for loop-guard",
                    "findings": []
                });
                println!("{}", out);
            } else {
                eprintln!("WAS: Fehlendes Unterkommando.\nWARUM: loop-guard verlangt 'record', 'check', 'stop' oder 'reset'.\nFIX: Unterkommando angeben.");
            }
            return 2;
        }
    };

    match sub {
        "record" => {
            match loop_guard_record(
                &root_path,
                gate_name.as_deref(),
                log_file.as_deref(),
                edited_path.as_deref(),
            ) {
                Ok((hash, count)) => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "loop-guard",
                            "status": "pass",
                            "summary": format!("Recorded error hash '{}' (count {})", hash, count),
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        println!("Recorded error hash '{}' (count {})", hash, count);
                    }
                    0
                }
                Err(e) => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "loop-guard",
                            "status": "error",
                            "summary": e,
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        eprintln!("WAS: Aufzeichnung im loop-guard fehlgeschlagen.\nWARUM: {e}\nFIX: Prüfe Logpfad und Schreibrechte.");
                    }
                    2
                }
            }
        }
        "check" => match loop_guard_check(&root_path, max_file_edits) {
            Ok(()) => {
                if json_output {
                    let out = serde_json::json!({
                        "gate": "loop-guard",
                        "status": "pass",
                        "summary": "Keine Schleife oder Edit-Überschreitung erkannt",
                        "findings": []
                    });
                    println!("{}", out);
                } else {
                    println!("Loop-Guard pass: Keine Schleife erkannt.");
                }
                0
            }
            Err(e) => {
                if json_output {
                    let out = serde_json::json!({
                        "gate": "loop-guard",
                        "status": "fail",
                        "summary": e,
                        "findings": [
                            {
                                "id": "loop_detected",
                                "severity": "error",
                                "file": ".jules/local/loop-guard.json",
                                "line": 0,
                                "message": e,
                                "fix": "Stoppe automatische Versuche, analysiere den Fehlerursprung und passe den Ansatz an."
                            }
                        ]
                    });
                    println!("{}", out);
                } else {
                    eprintln!("WAS: Wiederholter Fehler oder Edit-Limit überschritten.\nWARUM: {e}\nFIX: Stoppe die Schleife und erstelle einen Eskalationsbericht.");
                }
                1
            }
        },
        "stop" => {
            let reason = stop_reason
                .unwrap_or_else(|| "Manuelle Unterbrechung durch loop-guard stop".to_string());
            match loop_guard_stop(&root_path, &reason) {
                Ok(()) => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "loop-guard",
                            "status": "fail",
                            "summary": format!("Eskalationsbericht geschrieben unter .jules/local/ESCALATE.md: {}", reason),
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        println!("ESCALATE.md geschrieben: {}", reason);
                    }
                    1
                }
                Err(e) => {
                    if json_output {
                        let out = serde_json::json!({
                            "gate": "loop-guard",
                            "status": "error",
                            "summary": e,
                            "findings": []
                        });
                        println!("{}", out);
                    } else {
                        eprintln!("WAS: Schreiben von ESCALATE.md fehlgeschlagen.\nWARUM: {e}\nFIX: Prüfe Schreibrechte in .jules/local/.");
                    }
                    2
                }
            }
        }
        "reset" => match loop_guard_reset(&root_path) {
            Ok(()) => {
                if json_output {
                    let out = serde_json::json!({
                        "gate": "loop-guard",
                        "status": "pass",
                        "summary": "Loop-Guard Zustand zurückgesetzt",
                        "findings": []
                    });
                    println!("{}", out);
                } else {
                    println!("Loop-Guard Zustand zurückgesetzt.");
                }
                0
            }
            Err(e) => {
                if json_output {
                    let out = serde_json::json!({
                        "gate": "loop-guard",
                        "status": "error",
                        "summary": e,
                        "findings": []
                    });
                    println!("{}", out);
                } else {
                    eprintln!("WAS: Zurücksetzen fehlgeschlagen.\nWARUM: {e}\nFIX: Lösche .jules/local/loop-guard.json manuell.");
                }
                2
            }
        },
        _ => {
            if json_output {
                let out = serde_json::json!({
                    "gate": "loop-guard",
                    "status": "error",
                    "summary": format!("Unbekanntes Unterkommando: {}", sub),
                    "findings": []
                });
                println!("{}", out);
            } else {
                eprintln!("WAS: Unbekanntes Unterkommando '{}'.\nWARUM: Nur 'record', 'check', 'stop' und 'reset' werden unterstützt.\nFIX: Gültiges Unterkommando angeben.", sub);
            }
            2
        }
    }
}
