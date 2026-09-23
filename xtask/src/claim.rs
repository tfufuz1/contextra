// Contextra — Claim / Reservation Mechanism Gate
//
// Koordiniert parallele Agenten-Sessions, um Mehrfach-Implementierungen (z.B. ConfigFingerprint #1627, #1634, #1645)
// zu verhindern.
// Schema:
//   cargo xtask claim --crate <CRATE> --issue <TASK_ID> [--dry-run]
//
// TODO(opt-RC-1/A-2): Implement --release/unclaim flag (`cargo xtask claim --release --crate <CRATE>`) and automated TTL expiry (4h default) check in claim database.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::find_root_dir;

/// Globales Standard-Parallelitäts-Limit für gleichzeitig aktive Claims.
pub const MAX_ACTIVE_CLAIMS: usize = 3;

/// Ermittelt das globale Parallelitäts-Limit (konfigurierbar über CONTEXTRA_MAX_ACTIVE_CLAIMS).
pub fn get_max_active_claims() -> usize {
    std::env::var("CONTEXTRA_MAX_ACTIVE_CLAIMS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(MAX_ACTIVE_CLAIMS)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimEntry {
    pub krate: String,
    pub issue: String,
    pub timestamp: String,
    #[serde(default)]
    pub session_id: String,
    #[serde(default = "default_active")]
    pub active: bool,
    /// ISO-8601 Ablaufzeitpunkt (Standard: timestamp + 4h). Fehlend = kein Expiry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// ISO-8601 Zeitpunkt der expliziten Freigabe.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released_at: Option<String>,
}

fn default_active() -> bool {
    true
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClaimsDatabase {
    pub claims: Vec<ClaimEntry>,
}

impl ClaimsDatabase {
    pub fn load(path: &Path) -> Self {
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(db) = serde_json::from_str::<ClaimsDatabase>(&content) {
                    return db;
                }
            }
        }
        ClaimsDatabase { claims: Vec::new() }
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Kann Verzeichnis {} nicht erstellen: {}",
                    parent.display(),
                    e
                )
            })?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Fehler bei Serialisierung der Claims: {}", e))?;
        fs::write(path, json)
            .map_err(|e| format!("Kann {} nicht schreiben: {}", path.display(), e))?;
        Ok(())
    }

    pub fn find_active_claim(&self, krate: &str) -> Option<&ClaimEntry> {
        let now = Utc::now();
        self.claims.iter().find(|c| {
            if c.krate != krate || !c.active {
                return false;
            }
            // TTL-Check: abgelaufene Claims werden als inaktiv behandelt
            if let Some(exp) = &c.expires_at {
                if let Ok(exp_dt) = exp.parse::<chrono::DateTime<Utc>>() {
                    if now > exp_dt {
                        return false; // TTL abgelaufen — kein Conflict
                    }
                }
            }
            true
        })
    }

    /// Zählt alle aktuell aktiven, nicht-abgelaufenen Claims über alle Crates hinweg.
    pub fn count_active_claims(&self) -> usize {
        let now = Utc::now();
        self.claims
            .iter()
            .filter(|c| {
                if !c.active {
                    return false;
                }
                if let Some(exp) = &c.expires_at {
                    if let Ok(exp_dt) = exp.parse::<chrono::DateTime<Utc>>() {
                        if now > exp_dt {
                            return false;
                        }
                    }
                }
                true
            })
            .count()
    }
}

/// Prüft alle Claims in `.jules/claims.json` auf TTL-Ablauf.
/// Setzt abgelaufene aktive Claims auf `active = false` und befüllt `released_at` mit Timestamp + Suffix `[TTL-EXPIRED]`.
/// Speichert die Datenbank nur zurück, wenn Claims abgelaufen sind.
pub fn expire_stale_claims(root: &Path) -> usize {
    let claims_path = root.join(".jules/claims.json");
    let mut db = ClaimsDatabase::load(&claims_path);
    let now = Utc::now();
    let mut expired_count = 0;

    for entry in db.claims.iter_mut() {
        if !entry.active {
            continue;
        }
        if let Some(exp) = &entry.expires_at {
            if let Ok(exp_dt) = exp.parse::<chrono::DateTime<Utc>>() {
                if now > exp_dt {
                    entry.active = false;
                    let ts_str = now.to_rfc3339();
                    entry.released_at = Some(format!("{} [TTL-EXPIRED]", ts_str));
                    expired_count += 1;
                }
            }
        }
    }

    if expired_count > 0 {
        if let Err(e) = db.save(&claims_path) {
            eprintln!("⚠️ Fehler beim Speichern abgelaufener Claims: {}", e);
        }
    }

    expired_count
}

/// Erstellt oder überschreibt den Session-Snapshot in `.jules/SESSION.md`.
///
/// Schema per Spezifikation (Abschnitt 4.2):
/// - Crate Scope
/// - Issue / Task ID
/// - Startzeit (ISO-8601 UTC)
/// - Letzte bekannte Phase (Phase 1–5 aus AGENTS.md §3)
/// - Notizen
pub fn write_session_snapshot(
    root: &Path,
    krate: &str,
    issue: &str,
    phase: &str,
    notes: Option<&str>,
) -> Result<(), String> {
    let session_path = root.join(".jules/SESSION.md");
    if let Some(parent) = session_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Kann Verzeichnis {} nicht erstellen: {}",
                parent.display(),
                e
            )
        })?;
    }

    let timestamp = Utc::now().to_rfc3339();
    let notes_str = notes.unwrap_or("Keine Notizen.");

    let content = format!(
        "# Jules Session Snapshot\n\n\
        - **Crate Scope:** {}\n\
        - **Issue / Task ID:** {}\n\
        - **Startzeit:** {}\n\
        - **Letzte bekannte Phase:** {}\n\
        - **Notizen:** {}\n",
        krate, issue, timestamp, phase, notes_str
    );

    fs::write(&session_path, content)
        .map_err(|e| format!("Kann {} nicht schreiben: {}", session_path.display(), e))?;

    Ok(())
}

/// Gibt einen aktiven Claim frei: setzt active=false und released_at.
pub fn run_release_local(args: &[String]) -> bool {
    let mut krate = String::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--crate" && i + 1 < args.len() {
            krate = args[i + 1].clone();
            i += 1;
        }
        i += 1;
    }
    if krate.is_empty() {
        eprintln!("❌ Parameter --crate <CRATE> für --release erforderlich.");
        return false;
    }
    let root = find_root_dir();
    let claims_path = root.join(".jules/claims.json");
    let mut db = ClaimsDatabase::load(&claims_path);
    let now_str = Utc::now().to_rfc3339();
    let released_str = format!("{} [RELEASED]", now_str);
    let mut found = false;
    for entry in db.claims.iter_mut() {
        if entry.krate == krate && entry.active {
            entry.active = false;
            entry.released_at = Some(released_str.clone());
            found = true;
        }
    }
    if !found {
        println!("ℹ️ Kein aktiver Claim für Crate '{}' gefunden.", krate);
        return true;
    }
    if let Err(e) = db.save(&claims_path) {
        eprintln!("❌ Fehler beim Speichern: {}", e);
        return false;
    }
    println!("✅ Claim für '{}' freigegeben.", krate);
    true
}

pub fn run_claim(args: &[String]) -> bool {
    if args.contains(&"--release".to_string()) {
        return run_release_local(args);
    }
    if std::env::var("GITHUB_TOKEN")
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false)
    {
        return run_claim_github(args);
    }
    run_claim_local(args)
}

fn run_claim_local(args: &[String]) -> bool {
    let mut krate = String::new();
    let mut issue = String::new();
    let mut dry_run = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--crate" => {
                if i + 1 < args.len() {
                    krate = args[i + 1].clone();
                    i += 1;
                }
            }
            "--issue" => {
                if i + 1 < args.len() {
                    issue = args[i + 1].clone();
                    i += 1;
                }
            }
            "--dry-run" => {
                dry_run = true;
            }
            _ => {}
        }
        i += 1;
    }

    if krate.is_empty() {
        eprintln!("❌ Parameter --crate <CRATE> erforderlich.");
        return false;
    }
    if issue.is_empty() {
        issue = "UNSPECIFIED".to_string();
    }

    let root = find_root_dir();
    let claims_path = root.join(".jules/claims.json");
    let mut db = ClaimsDatabase::load(&claims_path);

    let current_session = std::env::var("JULES_SESSION_ID").unwrap_or_else(|_| "local".to_string());

    if let Some(existing) = db.find_active_claim(&krate) {
        if existing.issue != issue || existing.session_id != current_session {
            eprintln!(
                "⚠️ KONFLIKT: Crate '{}' ist bereits aktiv reserviert durch Issue '{}' (Session '{}', seit {}).",
                krate, existing.issue, existing.session_id, existing.timestamp
            );
            if !dry_run {
                eprintln!(
                    "Bitte warten oder mit Entwickler abstimmen, um doppelte Arbeit zu verhindern."
                );
                return false;
            }
        } else {
            println!(
                "ℹ️ Crate '{}' ist bereits für Issue '{}' beansprucht (Session '{}').",
                krate, issue, current_session
            );
            return true;
        }
    }

    let max_claims = get_max_active_claims();
    let active_count = db.count_active_claims();
    if active_count >= max_claims {
        eprintln!(
            "⚠️ KONFLIKT: Globales Parallelitäts-Limit von {} aktiven Claims überschritten (aktuell {} aktive Claims).",
            max_claims, active_count
        );
        if !dry_run {
            return false;
        }
    }

    let expires_at = (Utc::now() + chrono::Duration::hours(4)).to_rfc3339();
    let entry = ClaimEntry {
        krate: krate.clone(),
        issue: issue.clone(),
        timestamp: Utc::now().to_rfc3339(),
        session_id: current_session,
        active: true,
        expires_at: Some(expires_at),
        released_at: None,
    };

    if dry_run {
        println!(
            "{{\"status\": \"claimed\", \"crate\": \"{}\", \"issue\": \"{}\", \"dry_run\": true}}",
            krate, issue
        );
        return true;
    }

    db.claims.push(entry);
    if let Err(e) = db.save(&claims_path) {
        eprintln!("❌ Fehler beim Speichern des Claims: {}", e);
        return false;
    }

    if let Err(e) = write_session_snapshot(
        &root,
        &krate,
        &issue,
        "Phase 1: Exploration & Claim",
        None,
    ) {
        eprintln!("⚠️ Fehler beim Schreiben des Session-Snapshots: {}", e);
    }

    println!(
        "✅ Claim erfolgreich registriert: Crate '{}' für Issue '{}' gesperrt.",
        krate, issue
    );
    true
}

pub fn run_claim_github(args: &[String]) -> bool {
    let token = match std::env::var("GITHUB_TOKEN") {
        Ok(t) if !t.trim().is_empty() => t,
        _ => return run_claim_local(args),
    };

    let mut krate = String::new();
    let mut issue = String::new();
    let mut session_hash = String::new();
    let mut dry_run = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--crate" => {
                if i + 1 < args.len() {
                    krate = args[i + 1].clone();
                    i += 1;
                }
            }
            "--issue" => {
                if i + 1 < args.len() {
                    issue = args[i + 1].clone();
                    i += 1;
                }
            }
            "--session" | "--session-hash" => {
                if i + 1 < args.len() {
                    session_hash = args[i + 1].clone();
                    i += 1;
                }
            }
            "--dry-run" => {
                dry_run = true;
            }
            _ => {}
        }
        i += 1;
    }

    if krate.is_empty() {
        eprintln!("❌ Parameter --crate <CRATE> erforderlich.");
        return false;
    }
    if issue.is_empty() {
        issue = "UNSPECIFIED".to_string();
    }
    if session_hash.is_empty() {
        session_hash = std::env::var("CONTEXTRA_SESSION_HASH")
            .or_else(|_| std::env::var("JULES_SESSION_ID"))
            .unwrap_or_else(|_| "local".to_string());
    }

    // Step c: Check existing issues with label "claim:<crate>" via GitHub REST API
    let check_url = format!(
        "https://api.github.com/repos/tfufuz1/contextra/issues?labels=claim:{}&state=open",
        krate
    );
    let output = std::process::Command::new("curl")
        .args([
            "-s",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-H",
            "User-Agent: contextra-xtask",
            "-H",
            "Accept: application/vnd.github+json",
            &check_url,
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let body = String::from_utf8_lossy(&out.stdout);
            if let Ok(issues) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(arr) = issues.as_array() {
                    if !arr.is_empty() {
                        eprintln!(
                            "⚠️ KONFLIKT: Offenes Issue mit Label 'claim:{}' existiert bereits auf GitHub.",
                            krate
                        );
                        if !dry_run {
                            return false;
                        }
                    }
                }
            }
        }
    }

    if dry_run {
        println!(
            "{{\"status\": \"claimed\", \"crate\": \"{}\", \"issue\": \"{}\", \"mode\": \"github\", \"dry_run\": true}}",
            krate, issue
        );
        return true;
    }

    // Step b: Create GitHub issue via REST API
    let create_url = "https://api.github.com/repos/tfufuz1/contextra/issues";
    let title = format!("CLAIM: {} — {}", krate, issue);
    let payload = serde_json::json!({
        "title": title,
        "body": session_hash,
        "labels": ["claimed", format!("claim:{}", krate)]
    });

    let payload_str = match serde_json::to_string(&payload) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "❌ Fehler bei JSON-Serialisierung für Issue-Erstellung: {}",
                e
            );
            return false;
        }
    };

    let post_output = std::process::Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-H",
            "User-Agent: contextra-xtask",
            "-H",
            "Accept: application/vnd.github+json",
            "-d",
            &payload_str,
            create_url,
        ])
        .output();

    match post_output {
        Ok(out) if out.status.success() => {
            let response_body = String::from_utf8_lossy(&out.stdout);
            if let Ok(json_res) = serde_json::from_str::<serde_json::Value>(&response_body) {
                if json_res.get("number").is_some() {
                    println!(
                        "✅ GitHub Claim-Issue erfolgreich erstellt: Crate '{}' für Issue '{}' gesperrt.",
                        krate, issue
                    );
                    return true;
                }
            }
            eprintln!(
                "❌ GitHub API Fehler bei Issue-Erstellung: {}",
                response_body
            );
            false
        }
        Ok(out) => {
            eprintln!(
                "❌ curl-Befehl fehlgeschlagen mit Status Code {}",
                out.status
            );
            false
        }
        Err(e) => {
            eprintln!("❌ Fehler beim Ausführen von curl: {}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_claims_db_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("claims.json");

        let mut db = ClaimsDatabase::default();
        db.claims.push(ClaimEntry {
            krate: "contextra-router".to_string(),
            issue: "ADR-063".to_string(),
            timestamp: "2026-09-08T18:00:00Z".to_string(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: None,
            released_at: None,
        });

        db.save(&path).unwrap();

        let loaded = ClaimsDatabase::load(&path);
        assert_eq!(loaded.claims.len(), 1);
        assert_eq!(
            loaded.find_active_claim("contextra-router").unwrap().issue,
            "ADR-063"
        );
        assert!(loaded.find_active_claim("contextra-core").is_none());
    }

    #[test]
    fn test_find_active_claim_ttl_expiry() {
        let mut db = ClaimsDatabase::default();
        let past_exp = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        db.claims.push(ClaimEntry {
            krate: "contextra-store".to_string(),
            issue: "EXP-1".to_string(),
            timestamp: "2026-09-08T18:00:00Z".to_string(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some(past_exp),
            released_at: None,
        });

        assert!(db.find_active_claim("contextra-store").is_none());

        let future_exp = (Utc::now() + chrono::Duration::hours(1)).to_rfc3339();
        db.claims.push(ClaimEntry {
            krate: "contextra-embed".to_string(),
            issue: "EXP-2".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s2".to_string(),
            active: true,
            expires_at: Some(future_exp),
            released_at: None,
        });

        assert_eq!(
            db.find_active_claim("contextra-embed").unwrap().issue,
            "EXP-2"
        );
    }

    #[test]
    fn test_release_local() {
        let dir = tempdir().unwrap();
        let claims_path = dir.path().join(".jules/claims.json");
        let mut db = ClaimsDatabase::default();
        db.claims.push(ClaimEntry {
            krate: "contextra-test-release".to_string(),
            issue: "REL-1".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some((Utc::now() + chrono::Duration::hours(4)).to_rfc3339()),
            released_at: None,
        });
        db.save(&claims_path).unwrap();

        // Check active claim initially
        assert!(db.find_active_claim("contextra-test-release").is_some());

        // Run release logic directly on the db
        for entry in db.claims.iter_mut() {
            if entry.krate == "contextra-test-release" && entry.active {
                entry.active = false;
                entry.released_at = Some(Utc::now().to_rfc3339());
            }
        }
        db.save(&claims_path).unwrap();

        let loaded = ClaimsDatabase::load(&claims_path);
        assert!(loaded.find_active_claim("contextra-test-release").is_none());
        assert!(!loaded.claims[0].active);
        assert!(loaded.claims[0].released_at.is_some());
    }

    #[test]
    fn test_claim_empty_crate_returns_false() {
        let args = vec!["--issue".to_string(), "ISSUE-1".to_string()];
        assert!(!run_claim(&args));
        assert!(!run_claim_github(&args));
    }

    #[test]
    fn test_claim_github_fallback_without_token() {
        // Ensure GITHUB_TOKEN is unset for fallback test
        std::env::remove_var("GITHUB_TOKEN");
        let args = vec![
            "--crate".to_string(),
            "contextra-test-fallback".to_string(),
            "--issue".to_string(),
            "TEST-FB".to_string(),
            "--dry-run".to_string(),
        ];
        assert!(run_claim_github(&args));
    }

    #[test]
    fn test_expire_stale_claims() {
        let dir = tempdir().unwrap();
        let jules_dir = dir.path().join(".jules");
        fs::create_dir_all(&jules_dir).unwrap();
        let claims_path = jules_dir.join("claims.json");

        let past_exp = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
        let future_exp = (Utc::now() + chrono::Duration::hours(2)).to_rfc3339();

        let mut db = ClaimsDatabase::default();
        db.claims.push(ClaimEntry {
            krate: "contextra-stale".to_string(),
            issue: "STALE-1".to_string(),
            timestamp: "2026-09-08T18:00:00Z".to_string(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some(past_exp),
            released_at: None,
        });
        db.claims.push(ClaimEntry {
            krate: "contextra-active".to_string(),
            issue: "ACTIVE-1".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s2".to_string(),
            active: true,
            expires_at: Some(future_exp),
            released_at: None,
        });
        db.save(&claims_path).unwrap();

        let expired_count = expire_stale_claims(dir.path());
        assert_eq!(expired_count, 1);

        let reloaded = ClaimsDatabase::load(&claims_path);
        assert_eq!(reloaded.claims.len(), 2);

        let stale = &reloaded.claims[0];
        assert_eq!(stale.krate, "contextra-stale");
        assert!(!stale.active);
        assert!(stale.released_at.is_some());
        assert!(stale
            .released_at
            .as_ref()
            .unwrap()
            .contains("[TTL-EXPIRED]"));
        assert_eq!(stale.issue, "STALE-1");
        assert_eq!(stale.session_id, "s1");

        let active = &reloaded.claims[1];
        assert_eq!(active.krate, "contextra-active");
        assert!(active.active);
        assert!(active.released_at.is_none());
    }

    #[test]
    fn test_count_active_claims() {
        let mut db = ClaimsDatabase::default();
        let future_exp = (Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
        let past_exp = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();

        db.claims.push(ClaimEntry {
            krate: "crate1".to_string(),
            issue: "ISSUE-1".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some(future_exp.clone()),
            released_at: None,
        });
        db.claims.push(ClaimEntry {
            krate: "crate2".to_string(),
            issue: "ISSUE-2".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s2".to_string(),
            active: true,
            expires_at: Some(future_exp.clone()),
            released_at: None,
        });
        db.claims.push(ClaimEntry {
            krate: "crate3".to_string(),
            issue: "ISSUE-3".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s3".to_string(),
            active: false,
            expires_at: Some(future_exp),
            released_at: Some("2026-09-18T10:00:00Z [RELEASED]".to_string()),
        });
        db.claims.push(ClaimEntry {
            krate: "crate4".to_string(),
            issue: "ISSUE-4".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s4".to_string(),
            active: true,
            expires_at: Some(past_exp),
            released_at: None,
        });

        assert_eq!(db.count_active_claims(), 2);
    }

    #[test]
    fn test_release_local_released_suffix() {
        let dir = tempdir().unwrap();
        let claims_path = dir.path().join(".jules/claims.json");
        let mut db = ClaimsDatabase::default();
        db.claims.push(ClaimEntry {
            krate: "contextra-test-explicit-release".to_string(),
            issue: "REL-2".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some((Utc::now() + chrono::Duration::hours(4)).to_rfc3339()),
            released_at: None,
        });
        db.save(&claims_path).unwrap();

        let now_str = Utc::now().to_rfc3339();
        let released_str = format!("{} [RELEASED]", now_str);
        for entry in db.claims.iter_mut() {
            if entry.krate == "contextra-test-explicit-release" && entry.active {
                entry.active = false;
                entry.released_at = Some(released_str.clone());
            }
        }
        db.save(&claims_path).unwrap();

        let loaded = ClaimsDatabase::load(&claims_path);
        assert!(!loaded.claims[0].active);
        assert!(loaded.claims[0]
            .released_at
            .as_ref()
            .unwrap()
            .contains("[RELEASED]"));
    }

    #[test]
    fn test_run_claim_local_concurrency_limit_exceeded() {
        std::env::set_var("CONTEXTRA_MAX_ACTIVE_CLAIMS", "2");
        let future_exp = (Utc::now() + chrono::Duration::hours(2)).to_rfc3339();
        let mut db = ClaimsDatabase::default();
        db.claims.push(ClaimEntry {
            krate: "c1".to_string(),
            issue: "I1".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some(future_exp.clone()),
            released_at: None,
        });
        db.claims.push(ClaimEntry {
            krate: "c2".to_string(),
            issue: "I2".to_string(),
            timestamp: Utc::now().to_rfc3339(),
            session_id: "s2".to_string(),
            active: true,
            expires_at: Some(future_exp),
            released_at: None,
        });

        assert_eq!(db.count_active_claims(), 2);
        assert_eq!(get_max_active_claims(), 2);

        std::env::remove_var("CONTEXTRA_MAX_ACTIVE_CLAIMS");
    }

    #[test]
    fn test_write_session_snapshot() {
        let dir = tempdir().unwrap();
        let res = write_session_snapshot(
            dir.path(),
            "xtask",
            "TEST-100",
            "Phase 1: Exploration & Claim",
            Some("Testing snapshot writing"),
        );
        assert!(res.is_ok());

        let session_file = dir.path().join(".jules/SESSION.md");
        assert!(session_file.is_file());
        let content = fs::read_to_string(&session_file).unwrap();

        assert!(content.contains("- **Crate Scope:** xtask"));
        assert!(content.contains("- **Issue / Task ID:** TEST-100"));
        assert!(content.contains("- **Letzte bekannte Phase:** Phase 1: Exploration & Claim"));
        assert!(content.contains("- **Notizen:** Testing snapshot writing"));
    }
}
