//! Harness Modul zur Erklaerung von Qualitätsgates und Invarianten.

use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_explain(args: &[String]) -> i32 {
    let mut root_dir = explain_find_repo_root();
    let mut use_json = false;
    let mut list_mode = false;
    let mut target_gate: Option<String> = None;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--json" => {
                use_json = true;
            }
            "--list" => {
                list_mode = true;
            }
            arg if !arg.starts_with('-') && target_gate.is_none() => {
                target_gate = Some(arg.to_string());
            }
            _ => {}
        }
        idx += 1;
    }

    let explain_dir = root_dir.join("docs/explain");
    if !explain_dir.exists() {
        let msg = "Explain-Verzeichnis docs/explain existiert nicht.";
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "explain",
                    "status": "error",
                    "summary": msg,
                    "findings": []
                })
            );
        } else {
            eprintln!("FEHLER: {}", msg);
        }
        return 2;
    }

    if list_mode || target_gate.is_none() {
        let mut gates = Vec::new();
        if let Ok(entries) = fs::read_dir(&explain_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        gates.push(stem.to_string());
                    }
                }
            }
        }
        gates.sort();

        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "explain",
                    "status": "pass",
                    "summary": format!("Verfuegbare Gates: {}", gates.join(", ")),
                    "findings": gates.iter().map(|g| json!({
                        "id": g,
                        "severity": "info",
                        "file": format!("docs/explain/{}.toml", g),
                        "line": 0,
                        "message": format!("Wissenseintrag fuer {}", g),
                        "fix": format!("cargo xtask explain {}", g)
                    })).collect::<Vec<_>>()
                })
            );
        } else {
            println!("Verfuegbare Gates (Wissenseintraege):");
            for g in &gates {
                println!(" - {}", g);
            }
        }
        return 0;
    }

    let gate_name = match target_gate {
        Some(ref name) => name.as_str(),
        None => "",
    };

    let file_path = explain_dir.join(format!("{}.toml", gate_name));
    if !file_path.exists() {
        let mut available = Vec::new();
        if let Ok(entries) = fs::read_dir(&explain_dir) {
            for entry in entries.flatten() {
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    available.push(stem.to_string());
                }
            }
        }
        available.sort();
        let similar: Vec<_> = available
            .iter()
            .filter(|g| g.contains(gate_name) || gate_name.contains(g.as_str()))
            .cloned()
            .collect();

        let msg = format!(
            "Unbekanntes Gate: '{}'. Aehnliche Gates: {}",
            gate_name,
            if similar.is_empty() {
                available.join(", ")
            } else {
                similar.join(", ")
            }
        );

        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "explain",
                    "status": "fail",
                    "summary": msg,
                    "findings": [{
                        "id": "explain-unknown",
                        "severity": "error",
                        "file": "",
                        "line": 0,
                        "message": msg,
                        "fix": "Nutze 'cargo xtask explain --list' fuer eine Übersicht aller Gates."
                    }]
                })
            );
        } else {
            eprintln!("VERSTOß: {}", msg);
        }
        return 1;
    }

    let content = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("Konnte Wissensdatei nicht lesen: {}", e);
            if use_json {
                println!(
                    "{}",
                    json!({
                        "gate": "explain",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER: {}", msg);
            }
            return 2;
        }
    };

    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    let table = match parsed {
        Ok(toml::Value::Table(t)) => t,
        _ => {
            let msg = "Ungueltiges TOML-Format in Wissensdatei.";
            if use_json {
                println!(
                    "{}",
                    json!({
                        "gate": "explain",
                        "status": "error",
                        "summary": msg,
                        "findings": []
                    })
                );
            } else {
                eprintln!("FEHLER: {}", msg);
            }
            return 2;
        }
    };

    let get_str = |key: &str| -> Option<String> {
        table
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    };

    let title = get_str("title");
    let what = get_str("what");
    let why = get_str("why");
    let fix = get_str("fix");
    let anti_fix = get_str("anti_fix");

    // Check mandatory fields
    if title.is_none() || what.is_none() || why.is_none() || fix.is_none() || anti_fix.is_none() {
        let msg = format!("Fehlendes Pflichtfeld in Wissensdatei 'docs/explain/{}.toml' (erforderlich: title, what, why, fix, anti_fix).", gate_name);
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "explain",
                    "status": "fail",
                    "summary": msg,
                    "findings": [{
                        "id": "explain-missing-fields",
                        "severity": "error",
                        "file": format!("docs/explain/{}.toml", gate_name),
                        "line": 0,
                        "message": msg,
                        "fix": "Ergänze alle Pflichtfelder in der TOML-Datei."
                    }]
                })
            );
        } else {
            eprintln!("VERSTOß: {}", msg);
        }
        return 1;
    }

    let title = title.unwrap_or_default();
    let what = what.unwrap_or_default();
    let why = why.unwrap_or_default();
    let fix = fix.unwrap_or_default();
    let anti_fix = anti_fix.unwrap_or_default();

    if use_json {
        println!(
            "{}",
            json!({
                "gate": "explain",
                "status": "pass",
                "summary": format!("Erklaerung fuer Gate '{}'", gate_name),
                "findings": [{
                    "id": gate_name,
                    "severity": "info",
                    "file": format!("docs/explain/{}.toml", gate_name),
                    "line": 0,
                    "message": format!("Titel: {}\nWas: {}\nWarum: {}\nFix: {}\nAnti-Fix: {}", title, what, why, fix, anti_fix),
                    "fix": fix
                }]
            })
        );
    } else {
        println!("=== Gate: {} ({}) ===", title, gate_name);
        println!("WAS IST PASSIERT:\n  {}\n", what);
        println!("WARUM GIBT ES DIE REGEL:\n  {}\n", why);
        println!("TYPISCHER FIX:\n  {}\n", fix);
        println!("FALSCHE ABKÜRZUNG (VERBOTEN):\n  {}\n", anti_fix);
    }

    0
}

fn explain_find_repo_root() -> PathBuf {
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return PathBuf::from(path_str);
            }
        }
    }
    PathBuf::from(".")
}
