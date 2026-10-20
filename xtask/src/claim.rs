// Contextra — Claim / Reservation Mechanism Gate
//
// Koordiniert parallele Agenten-Sessions, um Mehrfach-Implementierungen (z.B. ConfigFingerprint #1627, #1634, #1645)
// zu verhindern.
// Schema:
//   cargo xtask claim --crate <CRATE> [--issue <TASK_ID>] [--dry-run]
//   cargo xtask claim --release --crate <CRATE> [--force] [--dry-run]
//   cargo xtask claim --help

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::find_root_dir;

/// Globales Standard-Parallelitäts-Limit für gleichzeitig aktive Claims.
pub const MAX_ACTIVE_CLAIMS: usize = 3;

/// Standard TTL in Stunden für Claims.
pub const DEFAULT_TTL_HOURS: i64 = 4;

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

    /// Speichert die Claims-Datenbank atomar via Temp-Datei und Rename.
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

        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, json)
            .map_err(|e| format!("Kann {} nicht schreiben: {}", tmp_path.display(), e))?;
        fs::rename(&tmp_path, path).map_err(|e| {
            format!(
                "Kann {} nicht nach {} umbenennen: {}",
                tmp_path.display(),
                path.display(),
                e
            )
        })?;
        Ok(())
    }

    pub fn find_active_claim_at(&self, krate: &str, now: DateTime<Utc>) -> Option<&ClaimEntry> {
        self.claims.iter().find(|c| {
            if c.krate != krate || !c.active {
                return false;
            }
            // TTL-Check: abgelaufene Claims werden als inaktiv behandelt
            if let Some(exp) = &c.expires_at {
                if let Ok(exp_dt) = exp.parse::<DateTime<Utc>>() {
                    if now > exp_dt {
                        return false; // TTL abgelaufen — kein Conflict
                    }
                }
            }
            true
        })
    }

    pub fn find_active_claim(&self, krate: &str) -> Option<&ClaimEntry> {
        self.find_active_claim_at(krate, Utc::now())
    }

    /// Zählt alle aktuell aktiven, nicht-abgelaufenen Claims über alle Crates hinweg.
    pub fn count_active_claims_at(&self, now: DateTime<Utc>) -> usize {
        self.claims
            .iter()
            .filter(|c| {
                if !c.active {
                    return false;
                }
                if let Some(exp) = &c.expires_at {
                    if let Ok(exp_dt) = exp.parse::<DateTime<Utc>>() {
                        if now > exp_dt {
                            return false;
                        }
                    }
                }
                true
            })
            .count()
    }

    pub fn count_active_claims(&self) -> usize {
        self.count_active_claims_at(Utc::now())
    }
}

/// Prüft alle Claims in `.jules/claims.json` auf TTL-Ablauf zu einem bestimmten Zeitpunkt `now`.
/// Setzt abgelaufene aktive Claims auf `active = false` und befüllt `released_at` mit Timestamp + Suffix `[TTL-EXPIRED]`.
/// Speichert die Datenbank nur zurück, wenn Claims abgelaufen sind.
pub fn expire_stale_claims_at(root: &Path, now: DateTime<Utc>) -> usize {
    let claims_path = root.join(".jules/claims.json");
    let mut db = ClaimsDatabase::load(&claims_path);
    let mut expired_count = 0;

    for entry in db.claims.iter_mut() {
        if !entry.active {
            continue;
        }
        if let Some(exp) = &entry.expires_at {
            if let Ok(exp_dt) = exp.parse::<DateTime<Utc>>() {
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

/// Prüft alle Claims in `.jules/claims.json` auf TTL-Ablauf zum aktuellen Zeitpunkt.
pub fn expire_stale_claims(root: &Path) -> usize {
    expire_stale_claims_at(root, Utc::now())
}

/// Erstellt oder überschreibt den Session-Snapshot in `.jules/SESSION.md`.
pub fn write_session_snapshot(
    root: &Path,
    krate: &str,
    issue: &str,
    phase: &str,
    notes: Option<&str>,
) -> Result<(), String> {
    write_session_snapshot_at(root, krate, issue, phase, notes, Utc::now())
}

pub fn write_session_snapshot_at(
    root: &Path,
    krate: &str,
    issue: &str,
    phase: &str,
    notes: Option<&str>,
    now: DateTime<Utc>,
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

    let timestamp = now.to_rfc3339();
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

fn print_claim_help() {
    println!(
        "cargo xtask claim — Reserviert oder gibt ein Crate für synchrone Agenten-Sessions frei.\n\n\
        VERWENDUNG:\n\
        \x20   cargo xtask claim --crate <CRATE> [--issue <TASK_ID>] [--dry-run]\n\
        \x20   cargo xtask claim --release --crate <CRATE> [--force] [--dry-run]\n\
        \x20   cargo xtask claim --help\n\n\
        OPTIONEN:\n\
        \x20   --crate <CRATE>     Name des Crates (z.B. contextra-store)\n\
        \x20   --issue <TASK_ID>   ID des Tasks/Issues (Standard: UNSPECIFIED)\n\
        \x20   --release           Gibt einen bestehenden Claim frei\n\
        \x20   --force             Erzwingt das Aufheben eines fremden Claims bei --release\n\
        \x20   --dry-run           Führt Prüfungen durch, ohne Änderungen zu speichern\n\
        \x20   -h, --help          Zeigt diese Hilfe an"
    );
}

/// Gibt einen aktiven Claim frei.
pub fn run_release_local_at(args: &[String], now: DateTime<Utc>, root_override: Option<&Path>) -> bool {
    let mut krate = String::new();
    let mut force = false;
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
            "--force" => {
                force = true;
            }
            "--dry-run" => {
                dry_run = true;
            }
            _ => {}
        }
        i += 1;
    }

    if krate.is_empty() {
        eprintln!("❌ Parameter --crate <CRATE> für --release erforderlich.");
        return false;
    }

    let root = root_override.map(|p| p.to_path_buf()).unwrap_or_else(find_root_dir);
    expire_stale_claims_at(&root, now);

    let claims_path = root.join(".jules/claims.json");
    let mut db = ClaimsDatabase::load(&claims_path);
    let current_session = std::env::var("JULES_SESSION_ID").unwrap_or_else(|_| "local".to_string());

    let mut matching_index = None;
    for (idx, entry) in db.claims.iter().enumerate() {
        if entry.krate == krate && entry.active {
            // Check TTL
            if let Some(exp) = &entry.expires_at {
                if let Ok(exp_dt) = exp.parse::<DateTime<Utc>>() {
                    if now > exp_dt {
                        continue;
                    }
                }
            }
            matching_index = Some(idx);
            break;
        }
    }

    let idx = match matching_index {
        Some(idx) => idx,
        None => {
            println!("ℹ️ Kein aktiver Claim für Crate '{}' gefunden.", krate);
            return true;
        }
    };

    let existing_session = db.claims[idx].session_id.clone();
    if existing_session != current_session && !force {
        eprintln!(
            "❌ KONFLIKT: Claim für Crate '{}' gehört Session '{}', nicht der aktuellen Session '{}'. Freigabe verweigert (Verwende --force für fremde Claims).",
            krate, existing_session, current_session
        );
        return false;
    }

    if dry_run {
        println!(
            "{{\"status\": \"released\", \"crate\": \"{}\", \"force\": {}, \"dry_run\": true}}",
            krate, force
        );
        return true;
    }

    let released_suffix = if existing_session != current_session {
        "[RELEASED-FORCE]"
    } else {
        "[RELEASED]"
    };
    let released_str = format!("{} {}", now.to_rfc3339(), released_suffix);

    let entry = &mut db.claims[idx];
    entry.active = false;
    entry.released_at = Some(released_str);

    if let Err(e) = db.save(&claims_path) {
        eprintln!("❌ Fehler beim Speichern der Freigabe: {}", e);
        return false;
    }

    println!("✅ Claim für '{}' freigegeben.", krate);
    true
}

pub fn run_release_local(args: &[String]) -> bool {
    run_release_local_at(args, Utc::now(), None)
}

pub fn run_claim(args: &[String]) -> bool {
    run_claim_at(args, Utc::now(), None)
}

pub fn run_claim_at(args: &[String], now: DateTime<Utc>, root_override: Option<&Path>) -> bool {
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_claim_help();
        return true;
    }

    if args.contains(&"--release".to_string()) {
        return run_release_local_at(args, now, root_override);
    }

    run_claim_local_at(args, now, root_override)
}

fn run_claim_local_at(args: &[String], now: DateTime<Utc>, root_override: Option<&Path>) -> bool {
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

    let root = root_override.map(|p| p.to_path_buf()).unwrap_or_else(find_root_dir);
    expire_stale_claims_at(&root, now);

    let claims_path = root.join(".jules/claims.json");
    let mut db = ClaimsDatabase::load(&claims_path);

    let current_session = std::env::var("JULES_SESSION_ID").unwrap_or_else(|_| "local".to_string());

    if let Some(existing) = db.find_active_claim_at(&krate, now) {
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
    let active_count = db.count_active_claims_at(now);
    if active_count >= max_claims {
        eprintln!(
            "⚠️ KONFLIKT: Globales Parallelitäts-Limit von {} aktiven Claims überschritten (aktuell {} aktive Claims).",
            max_claims, active_count
        );
        if !dry_run {
            return false;
        }
    }

    let expires_at = (now + Duration::hours(DEFAULT_TTL_HOURS)).to_rfc3339();
    let entry = ClaimEntry {
        krate: krate.clone(),
        issue: issue.clone(),
        timestamp: now.to_rfc3339(),
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

    if let Err(e) =
        write_session_snapshot_at(&root, &krate, &issue, "Phase 1: Exploration & Claim", None, now)
    {
        eprintln!("⚠️ Fehler beim Schreiben des Session-Snapshots: {}", e);
    }

    println!(
        "✅ Claim erfolgreich registriert: Crate '{}' für Issue '{}' gesperrt.",
        krate, issue
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        let now = Utc::now();
        let past_exp = (now - Duration::hours(1)).to_rfc3339();
        db.claims.push(ClaimEntry {
            krate: "contextra-store".to_string(),
            issue: "EXP-1".to_string(),
            timestamp: "2026-09-08T18:00:00Z".to_string(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some(past_exp),
            released_at: None,
        });

        assert!(db.find_active_claim_at("contextra-store", now).is_none());

        let future_exp = (now + Duration::hours(1)).to_rfc3339();
        db.claims.push(ClaimEntry {
            krate: "contextra-embed".to_string(),
            issue: "EXP-2".to_string(),
            timestamp: now.to_rfc3339(),
            session_id: "s2".to_string(),
            active: true,
            expires_at: Some(future_exp),
            released_at: None,
        });

        assert_eq!(
            db.find_active_claim_at("contextra-embed", now).unwrap().issue,
            "EXP-2"
        );
    }

    #[test]
    fn test_release_local() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let root = dir.path();
        let claims_path = root.join(".jules/claims.json");
        let now = Utc::now();

        std::env::set_var("JULES_SESSION_ID", "local");
        let mut db = ClaimsDatabase::default();
        db.claims.push(ClaimEntry {
            krate: "contextra-test-release".to_string(),
            issue: "REL-1".to_string(),
            timestamp: now.to_rfc3339(),
            session_id: "local".to_string(),
            active: true,
            expires_at: Some((now + Duration::hours(4)).to_rfc3339()),
            released_at: None,
        });
        db.save(&claims_path).unwrap();

        let args = vec![
            "--release".to_string(),
            "--crate".to_string(),
            "contextra-test-release".to_string(),
        ];
        let ok = run_claim_at(&args, now, Some(root));
        assert!(ok);

        let loaded = ClaimsDatabase::load(&claims_path);
        assert!(!loaded.claims[0].active);
        assert!(loaded.claims[0].released_at.is_some());
        assert!(loaded.claims[0]
            .released_at
            .as_ref()
            .unwrap()
            .contains("[RELEASED]"));
        std::env::remove_var("JULES_SESSION_ID");
    }

    #[test]
    fn test_claim_empty_crate_returns_false() {
        let args = vec!["--issue".to_string(), "ISSUE-1".to_string()];
        assert!(!run_claim(&args));
    }

    #[test]
    fn test_expire_stale_claims() {
        let dir = tempdir().unwrap();
        let jules_dir = dir.path().join(".jules");
        fs::create_dir_all(&jules_dir).unwrap();
        let claims_path = jules_dir.join("claims.json");
        let now = Utc::now();

        let past_exp = (now - Duration::hours(1)).to_rfc3339();
        let future_exp = (now + Duration::hours(2)).to_rfc3339();

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
            timestamp: now.to_rfc3339(),
            session_id: "s2".to_string(),
            active: true,
            expires_at: Some(future_exp),
            released_at: None,
        });
        db.save(&claims_path).unwrap();

        let expired_count = expire_stale_claims_at(dir.path(), now);
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

        let active = &reloaded.claims[1];
        assert_eq!(active.krate, "contextra-active");
        assert!(active.active);
        assert!(active.released_at.is_none());
    }

    #[test]
    fn test_count_active_claims() {
        let mut db = ClaimsDatabase::default();
        let now = Utc::now();
        let future_exp = (now + Duration::hours(2)).to_rfc3339();
        let past_exp = (now - Duration::hours(1)).to_rfc3339();

        db.claims.push(ClaimEntry {
            krate: "crate1".to_string(),
            issue: "ISSUE-1".to_string(),
            timestamp: now.to_rfc3339(),
            session_id: "s1".to_string(),
            active: true,
            expires_at: Some(future_exp.clone()),
            released_at: None,
        });
        db.claims.push(ClaimEntry {
            krate: "crate2".to_string(),
            issue: "ISSUE-2".to_string(),
            timestamp: now.to_rfc3339(),
            session_id: "s2".to_string(),
            active: true,
            expires_at: Some(future_exp.clone()),
            released_at: None,
        });
        db.claims.push(ClaimEntry {
            krate: "crate3".to_string(),
            issue: "ISSUE-3".to_string(),
            timestamp: now.to_rfc3339(),
            session_id: "s3".to_string(),
            active: false,
            expires_at: Some(future_exp),
            released_at: Some("2026-09-18T10:00:00Z [RELEASED]".to_string()),
        });
        db.claims.push(ClaimEntry {
            krate: "crate4".to_string(),
            issue: "ISSUE-4".to_string(),
            timestamp: now.to_rfc3339(),
            session_id: "s4".to_string(),
            active: true,
            expires_at: Some(past_exp),
            released_at: None,
        });

        assert_eq!(db.count_active_claims_at(now), 2);
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

    #[test]
    fn test_claim_duplicate_rejected_for_different_session() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let root = dir.path();

        let t0 = Utc::now();
        let claim_args = vec![
            "--crate".to_string(),
            "contextra-store".to_string(),
            "--issue".to_string(),
            "ISSUE-1".to_string(),
        ];

        // Session 1 claims contextra-store
        std::env::set_var("JULES_SESSION_ID", "sess-1");
        assert!(run_claim_at(&claim_args, t0, Some(root)));

        // Session 2 attempts claim on contextra-store -> REJECTED
        std::env::set_var("JULES_SESSION_ID", "sess-2");
        assert!(!run_claim_at(&claim_args, t0, Some(root)));

        std::env::remove_var("JULES_SESSION_ID");
    }

    #[test]
    fn test_claim_release_owner_vs_foreign_force() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let root = dir.path();
        let t0 = Utc::now();

        std::env::set_var("JULES_SESSION_ID", "owner-sess");
        let claim_args = vec![
            "--crate".to_string(),
            "contextra-core".to_string(),
            "--issue".to_string(),
            "ISSUE-100".to_string(),
        ];
        assert!(run_claim_at(&claim_args, t0, Some(root)));

        // Foreign session attempts release without --force -> REJECTED
        std::env::set_var("JULES_SESSION_ID", "foreign-sess");
        let rel_args = vec![
            "--release".to_string(),
            "--crate".to_string(),
            "contextra-core".to_string(),
        ];
        assert!(!run_claim_at(&rel_args, t0, Some(root)));

        // Foreign session attempts release with --force -> SUCCEEDS
        let rel_force_args = vec![
            "--release".to_string(),
            "--crate".to_string(),
            "contextra-core".to_string(),
            "--force".to_string(),
        ];
        assert!(run_claim_at(&rel_force_args, t0, Some(root)));

        let claims_path = root.join(".jules/claims.json");
        let db = ClaimsDatabase::load(&claims_path);
        assert!(!db.claims[0].active);
        assert!(db.claims[0]
            .released_at
            .as_ref()
            .unwrap()
            .contains("[RELEASED-FORCE]"));

        std::env::remove_var("JULES_SESSION_ID");
    }

    #[test]
    fn test_claim_dry_run_does_not_modify_database() {
        let dir = tempdir().unwrap();
        let root = dir.path();
        let claims_path = root.join(".jules/claims.json");
        let t0 = Utc::now();

        let dry_args = vec![
            "--crate".to_string(),
            "contextra-dry".to_string(),
            "--issue".to_string(),
            "DRY-1".to_string(),
            "--dry-run".to_string(),
        ];

        assert!(run_claim_at(&dry_args, t0, Some(root)));
        assert!(!claims_path.exists());
    }

    #[test]
    fn test_claim_ttl_expiration_reclaims_crate() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = tempdir().unwrap();
        let root = dir.path();

        let t0 = Utc::now();
        std::env::set_var("JULES_SESSION_ID", "sess-old");
        let claim_args = vec![
            "--crate".to_string(),
            "contextra-exp".to_string(),
            "--issue".to_string(),
            "EXP-10".to_string(),
        ];
        assert!(run_claim_at(&claim_args, t0, Some(root)));

        // Time advances by 5 hours (> 4h TTL)
        let t1 = t0 + Duration::hours(5);
        std::env::set_var("JULES_SESSION_ID", "sess-new");

        // New session claims same crate after TTL expiration -> SUCCEEDS
        assert!(run_claim_at(&claim_args, t1, Some(root)));

        let claims_path = root.join(".jules/claims.json");
        let db = ClaimsDatabase::load(&claims_path);
        assert_eq!(db.claims.len(), 2);
        assert!(!db.claims[0].active);
        assert!(db.claims[0]
            .released_at
            .as_ref()
            .unwrap()
            .contains("[TTL-EXPIRED]"));
        assert!(db.claims[1].active);
        assert_eq!(db.claims[1].session_id, "sess-new");

        std::env::remove_var("JULES_SESSION_ID");
    }
}
