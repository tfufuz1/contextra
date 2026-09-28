//! Jules REST API dispatcher gate and payload builder.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub trait JulesHttpTransport {
    fn post(&self, url: &str, header: &str, api_key: &str, body: &str) -> Result<String, String>;
}

pub struct CurlHttpTransport;

impl JulesHttpTransport for CurlHttpTransport {
    fn post(&self, url: &str, header: &str, api_key: &str, body: &str) -> Result<String, String> {
        let auth_header = format!("{}: {}", header, api_key);
        let output = Command::new("curl")
            .args([
                "-s",
                "-S",
                "-X",
                "POST",
                "-H",
                &auth_header,
                "-H",
                "Content-Type: application/json",
                "-d",
                body,
                url,
            ])
            .output()
            .map_err(|e| e.to_string())?;

        let res = String::from_utf8_lossy(&output.stdout).to_string();
        if !output.status.success() {
            let err_raw = String::from_utf8_lossy(&output.stderr).to_string();
            let redacted = jules_dispatch_redact_key(&err_raw, api_key);
            return Err(format!("curl error: {}", redacted));
        }
        Ok(res)
    }
}

pub fn jules_dispatch_redact_key(text: &str, api_key: &str) -> String {
    if api_key.is_empty() {
        text.to_string()
    } else {
        text.replace(api_key, "[REDACTED_API_KEY]")
    }
}

pub fn jules_dispatch_find_root(start: &Path) -> PathBuf {
    let mut current = start.to_path_buf();
    loop {
        if current.join("Cargo.toml").exists() && current.join("capabilities.toml").exists() {
            return current;
        }
        if !current.pop() {
            return start.to_path_buf();
        }
    }
}

pub fn jules_dispatch_build_payload(
    root: &Path,
    card_path: &Path,
) -> Result<(serde_json::Value, String), String> {
    let card_findings = task_card_lint_file_local(root, card_path);
    if card_findings.iter().any(|f| f.severity == "error") {
        let msgs: Vec<String> = card_findings
            .iter()
            .filter(|f| f.severity == "error")
            .map(|f| f.message.clone())
            .collect();
        return Err(format!("Task-Karte ungültig: {}", msgs.join("; ")));
    }

    let api_toml_path = root.join(".jules/jules-api.toml");
    let api_content = fs::read_to_string(&api_toml_path)
        .map_err(|_| format!("Konnte {} nicht lesen", api_toml_path.display()))?;

    if api_content.contains("AUTO_CREATE_PR") {
        return Err(
            "FEHLER: 'AUTO_CREATE_PR' ist in .jules/jules-api.toml STRENG VERBOTEN!".to_string(),
        );
    }

    let preamble_path = root.join(".jules/PREAMBLE.md");
    let preamble = fs::read_to_string(&preamble_path).unwrap_or_default();

    let card_content = fs::read_to_string(card_path)
        .map_err(|_| format!("Konnte Task-Karte {} nicht lesen", card_path.display()))?;

    let prompt = format!("{}\n\n### TASK CARD ###\n{}", preamble, card_content);

    let payload = serde_json::json!({
        "prompt": prompt,
        "requirePlanApproval": true,
        "automationMode": "MANUAL"
    });

    let base_url = "https://jules.googleapis.com/v1alpha";
    let sessions_ep = "/sessions";
    let full_url = format!("{}{}", base_url, sessions_ep);

    Ok((payload, full_url))
}

struct CardFindingLocal {
    pub severity: String,
    pub message: String,
}

fn task_card_lint_file_local(_root: &Path, file_path: &Path) -> Vec<CardFindingLocal> {
    let mut findings = Vec::new();
    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            findings.push(CardFindingLocal {
                severity: "error".to_string(),
                message: format!("Datei konnte nicht gelesen werden: {}", e),
            });
            return findings;
        }
    };

    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    if let Err(e) = parsed {
        findings.push(CardFindingLocal {
            severity: "error".to_string(),
            message: format!("TOML-Syntaxfehler in Task-Karte: {}", e),
        });
    }
    findings
}

pub fn jules_dispatch_execute<T: JulesHttpTransport>(
    root: &Path,
    card_path: &Path,
    send: bool,
    transport: &T,
) -> (i32, serde_json::Value) {
    let (payload, url) = match jules_dispatch_build_payload(root, card_path) {
        Ok(res) => res,
        Err(e) => {
            let json = serde_json::json!({
                "gate": "jules-dispatch",
                "status": "error",
                "summary": e,
                "findings": []
            });
            return (2, json);
        }
    };

    if !send {
        let json = serde_json::json!({
            "gate": "jules-dispatch",
            "status": "pass",
            "summary": "Trockenlauf erfolgreich (Payload erstellt, nicht gesendet)",
            "url": url,
            "payload": payload,
            "findings": []
        });
        return (0, json);
    }

    let api_key = match std::env::var("JULES_API_KEY") {
        Ok(k) if !k.trim().is_empty() => k,
        _ => {
            let json = serde_json::json!({
                "gate": "jules-dispatch",
                "status": "error",
                "summary": "Umgebungsvariable JULES_API_KEY fehlt oder ist leer",
                "findings": []
            });
            return (2, json);
        }
    };

    let body_str = serde_json::to_string(&payload).unwrap_or_default();
    match transport.post(&url, "X-Goog-Api-Key", &api_key, &body_str) {
        Ok(res_text) => {
            let redacted_res = jules_dispatch_redact_key(&res_text, &api_key);
            let parsed: serde_json::Value = serde_json::from_str(&redacted_res)
                .unwrap_or_else(|_| serde_json::json!({ "raw": redacted_res }));

            let session_id = parsed
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("session-unknown")
                .replace("sessions/", "");

            let session_file_dir = root.join(".jules/local/sessions");
            let _ = fs::create_dir_all(&session_file_dir);
            let session_file = session_file_dir.join(format!("{}.json", session_id));

            let session_data = serde_json::json!({
                "session_id": session_id,
                "created_at": chrono::Local::now().to_rfc3339(),
                "card_path": card_path.to_string_lossy(),
                "revision_counter": 0,
                "response": parsed
            });

            let _ = fs::write(
                &session_file,
                serde_json::to_string_pretty(&session_data).unwrap_or_default(),
            );

            let json = serde_json::json!({
                "gate": "jules-dispatch",
                "status": "pass",
                "summary": format!("Session gestartet: {}", session_id),
                "session_id": session_id,
                "findings": []
            });
            (0, json)
        }
        Err(e) => {
            let redacted_err = jules_dispatch_redact_key(&e, &api_key);
            let json = serde_json::json!({
                "gate": "jules-dispatch",
                "status": "error",
                "summary": format!("API-Fehler: {}", redacted_err),
                "findings": []
            });
            (2, json)
        }
    }
}

pub fn run_jules_dispatch(args: &[String]) -> i32 {
    let mut root_dir: Option<PathBuf> = None;
    let mut json_output = false;
    let mut card_path_opt: Option<PathBuf> = None;
    let mut send = false;
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--root" && i + 1 < args.len() {
            root_dir = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i] == "--card" && i + 1 < args.len() {
            card_path_opt = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if args[i] == "--send" {
            send = true;
            i += 1;
        } else if args[i] == "--json" {
            json_output = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    let root = root_dir.unwrap_or_else(|| jules_dispatch_find_root(Path::new(".")));
    let card_path = match card_path_opt {
        Some(p) => p,
        None => {
            eprintln!("Usage: jules-dispatch --card <datei> [--send]");
            return 2;
        }
    };

    let transport = CurlHttpTransport;
    let (code, json_res) = jules_dispatch_execute(&root, &card_path, send, &transport);

    if json_output {
        println!("{}", json_res);
    } else {
        let summary = json_res
            .get("summary")
            .and_then(|s| s.as_str())
            .unwrap_or("");
        if code == 0 {
            println!("🟢 {}", summary);
            if !send {
                if let Some(p) = json_res.get("payload") {
                    println!("--- PAYLOAD TROCKENLAUF ---");
                    println!("{}", serde_json::to_string_pretty(p).unwrap_or_default());
                }
            }
        } else {
            eprintln!("🔴 {}", summary);
        }
    }

    code
}
