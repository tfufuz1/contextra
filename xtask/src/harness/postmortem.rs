//! Harness Modul zur Verwaltung und Prüfung von Incident Postmortems (postmortem).

use serde_json::json;
use std::fs;
use std::path::PathBuf;

pub fn run_postmortem(args: &[String]) -> i32 {
    let mut root_dir = postmortem_find_repo_root();
    let mut mode = "check".to_string(); // "check" or "new"
    let mut title = "Unbenannter Incident".to_string();
    let mut use_json = false;

    let mut idx = 1;
    while idx < args.len() {
        match args[idx].as_str() {
            "new" => {
                mode = "new".to_string();
            }
            "check" => {
                mode = "check".to_string();
            }
            "--title" => {
                if idx + 1 < args.len() {
                    title = args[idx + 1].clone();
                    idx += 1;
                }
            }
            "--root" => {
                if idx + 1 < args.len() {
                    root_dir = PathBuf::from(&args[idx + 1]);
                    idx += 1;
                }
            }
            "--json" => {
                use_json = true;
            }
            _ => {}
        }
        idx += 1;
    }

    let pm_dir = root_dir.join("docs/postmortems");
    if !pm_dir.exists() {
        let msg = "Postmortem-Verzeichnis docs/postmortems existiert nicht.";
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "postmortem",
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

    if mode == "new" {
        let mut max_id = 0;
        if let Ok(entries) = fs::read_dir(&pm_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("PM-") {
                    let parts: Vec<_> = name.split('-').collect();
                    if parts.len() >= 2 {
                        if let Ok(id) = parts[1].parse::<u32>() {
                            if id > max_id {
                                max_id = id;
                            }
                        }
                    }
                }
            }
        }

        let next_id = max_id + 1;
        let slug = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>();
        let clean_slug = slug
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        let file_name = format!("PM-{:04}-{}.md", next_id, clean_slug);
        let new_file_path = pm_dir.join(&file_name);

        let template_path = pm_dir.join("_TEMPLATE.md");
        let template_content = if template_path.exists() {
            fs::read_to_string(&template_path).unwrap_or_default()
        } else {
            format!("# Postmortem TEMPLATE: [PM-{:04}] {}\n\n## Vorfall\n\n## Zeitachse\n\n## Ursache (5 Whys)\n\n## Wirkung\n\n## Neue Regel\n\n## Neuer Test/Gate\n\n## ADR-Link\n\n## Verifikation\n", next_id, title)
        };

        if let Err(e) = fs::write(&new_file_path, template_content) {
            let msg = format!("Konnte Postmortem nicht anlegen: {}", e);
            if use_json {
                println!(
                    "{}",
                    json!({
                        "gate": "postmortem",
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

        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "postmortem",
                    "status": "pass",
                    "summary": format!("Neues Postmortem angelegt: {}", file_name),
                    "findings": [{
                        "id": file_name,
                        "severity": "info",
                        "file": format!("docs/postmortems/{}", file_name),
                        "line": 0,
                        "message": format!("Neues Postmortem PM-{:04} erfolgreich erstellt.", next_id),
                        "fix": "Fülle alle Abschnitte des Postmortems aus."
                    }]
                })
            );
        } else {
            println!(
                "Neues Postmortem erfolgreich angelegt: docs/postmortems/{}",
                file_name
            );
        }
        return 0;
    }

    // Check mode
    let mut findings = Vec::new();
    let required_sections = [
        "## Vorfall",
        "## Zeitachse",
        "## Ursache (5 Whys)",
        "## Wirkung",
        "## Neue Regel",
        "## Neuer Test/Gate",
        "## ADR-Link",
        "## Verifikation",
    ];

    if let Ok(entries) = fs::read_dir(&pm_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with("PM-") && name.ends_with(".md") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for section in &required_sections {
                        if !content.contains(section) {
                            findings.push(json!({
                                "id": name,
                                "severity": "error",
                                "file": format!("docs/postmortems/{}", name),
                                "line": 0,
                                "message": format!("Fehlender Pflichtabschnitt '{}'", section),
                                "fix": format!("Ergänze den Abschnitt '{}' in docs/postmortems/{}", section, name)
                            }));
                        }
                    }

                    // Check path existence
                    for line in content.lines() {
                        if line.contains("`xtask/src/") || line.contains("`crates/") {
                            if let Some(start) = line.find('`') {
                                if let Some(end) = line[start + 1..].find('`') {
                                    let rel_path = &line[start + 1..start + 1 + end];
                                    let target_path = root_dir.join(rel_path);
                                    if !target_path.exists() {
                                        findings.push(json!({
                                            "id": name,
                                            "severity": "error",
                                            "file": format!("docs/postmortems/{}", name),
                                            "line": 0,
                                            "message": format!("Referenzierter Test-/Gate-Pfad {:?} existiert nicht", rel_path),
                                            "fix": format!("Korrigiere den Pfad in docs/postmortems/{}", name)
                                        }));
                                    }
                                }
                            }
                        }

                        if line.contains("`docs/decisions/ADR-") {
                            if let Some(start) = line.find('`') {
                                if let Some(end) = line[start + 1..].find('`') {
                                    let rel_path = &line[start + 1..start + 1 + end];
                                    let target_path = root_dir.join(rel_path);
                                    if !target_path.exists() {
                                        findings.push(json!({
                                            "id": name,
                                            "severity": "error",
                                            "file": format!("docs/postmortems/{}", name),
                                            "line": 0,
                                            "message": format!("Referenzierte ADR-Datei {:?} existiert nicht", rel_path),
                                            "fix": format!("Korrigiere die ADR-Referenz in docs/postmortems/{}", name)
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if findings.is_empty() {
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "postmortem",
                    "status": "pass",
                    "summary": "Alle Postmortems sind vollständig und valide.",
                    "findings": []
                })
            );
        } else {
            println!("Pass: Alle Postmortems sind vollständig.");
        }
        0
    } else {
        if use_json {
            println!(
                "{}",
                json!({
                    "gate": "postmortem",
                    "status": "fail",
                    "summary": format!("Postmortem-Prüfung ergab {} Verstöße.", findings.len()),
                    "findings": findings
                })
            );
        } else {
            eprintln!("VERSTOß: Postmortem-Prüfung fehlgeschlagen:");
            for f in &findings {
                eprintln!(
                    " - {}: {}",
                    f.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    f.get("message").and_then(|v| v.as_str()).unwrap_or("")
                );
            }
        }
        1
    }
}

fn postmortem_find_repo_root() -> PathBuf {
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
