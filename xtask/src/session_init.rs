use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInitResult {
    pub session_hash: String,
    pub timestamp: String,
    pub open_blockers: usize,
    pub active_claim_count: usize,
    pub claimed_crate: Option<String>,
    pub env_file_path: String,
    pub last_commit: String,
    pub last_merge: String,
    pub prompter_distance: String,
}

pub fn generate_session_hash(now: &str) -> String {
    let hash = blake3::hash(now.as_bytes()).to_hex().to_string();
    hash[..8].to_string()
}

fn count_open_blockers(root: &Path) -> usize {
    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return 0;
    }

    let mut count = 0;
    for entry in WalkDir::new(&crates_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            for line in content.lines() {
                if line.contains("AI-TAG[")
                    && (line.contains("BLOCKER") || line.contains("CRITICAL"))
                    && !line.contains("RESOLVED")
                {
                    count += 1;
                }
            }
        }
    }
    count
}

fn get_last_commit(root: &Path) -> String {
    let root_str = root.to_str().unwrap_or(".");
    std::process::Command::new("git")
        .args(["-C", root_str, "log", "--oneline", "-1"])
        .output()
        .ok()
        .filter(|o| o.status.success() && !o.stdout.is_empty())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unbekannt".to_string())
}

fn get_last_merge(root: &Path) -> String {
    let root_str = root.to_str().unwrap_or(".");
    std::process::Command::new("git")
        .args(["-C", root_str, "log", "--oneline", "-1", "--merges"])
        .output()
        .ok()
        .filter(|o| o.status.success() && !o.stdout.is_empty())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "keine".to_string())
}

fn get_prompter_distance(root: &Path) -> String {
    let root_str = root.to_str().unwrap_or(".");
    let prompter_data_path = root.join(".jules/prompter-data.json");
    let prompter_head_opt = if prompter_data_path.is_file() {
        fs::read_to_string(&prompter_data_path)
            .ok()
            .and_then(|c| serde_json::from_str::<serde_json::Value>(&c).ok())
            .and_then(|v| {
                v.get("head")
                    .and_then(|h| h.as_str())
                    .map(|s| s.to_string())
            })
    } else {
        None
    };

    if let Some(ref p_head) = prompter_head_opt {
        let rev_list_opt = std::process::Command::new("git")
            .args([
                "-C",
                root_str,
                "rev-list",
                "--count",
                &format!("{}..HEAD", p_head),
            ])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        if let Some(count_str) = rev_list_opt {
            format!("{} Commit(s)", count_str)
        } else {
            "unbekannt".to_string()
        }
    } else {
        "unbekannt (kein prompter-data.json)".to_string()
    }
}

pub fn run_session_init(
    crate_name: Option<&str>,
    task_description: Option<&str>,
    write_env_file: bool,
) -> Result<SessionInitResult, String> {
    let root = crate::find_root_dir();
    let timestamp = chrono::Utc::now().to_rfc3339();
    let session_hash = generate_session_hash(&timestamp);

    crate::claim::expire_stale_claims(&root);

    let open_blockers = count_open_blockers(&root);

    let claims_path = root.join(".jules/claims.json");

    if let Some(krate) = crate_name {
        let mut db = crate::claim::ClaimsDatabase::load(&claims_path);
        let current_session = session_hash.clone();

        if let Some(existing) = db.find_active_claim(krate) {
            if existing.session_id != current_session {
                return Err(format!(
                    "⚠️ KONFLIKT: Crate '{}' ist bereits aktiv reserviert durch Issue '{}' (Session '{}').",
                    krate, existing.issue, existing.session_id
                ));
            }
        } else {
            let issue_str = task_description.unwrap_or("UNSPECIFIED");
            let expires_at = (chrono::Utc::now() + chrono::Duration::hours(4)).to_rfc3339();
            let entry = crate::claim::ClaimEntry {
                krate: krate.to_string(),
                issue: issue_str.to_string(),
                timestamp: timestamp.clone(),
                session_id: current_session,
                active: true,
                expires_at: Some(expires_at),
                released_at: None,
            };
            db.claims.push(entry);
            db.save(&claims_path)?;

            let _ = crate::claim::write_session_snapshot(
                &root,
                krate,
                issue_str,
                "Phase 1: Exploration & Claim",
                None,
            );
        }
    }

    let db = crate::claim::ClaimsDatabase::load(&claims_path);
    let active_claim_count = db.count_active_claims();

    let last_commit = get_last_commit(&root);
    let last_merge = get_last_merge(&root);
    let prompter_distance = get_prompter_distance(&root);

    println!("Session-Kontinuität:");
    println!("- SESSION_HASH: {}", session_hash);
    println!("- Letzter Commit: {}", last_commit);
    println!("- Letzter Merge: {}", last_merge);
    println!("- Offene BLOCKER: {}", open_blockers);
    println!("- Aktive Claims: {}", active_claim_count);
    println!("- Prompter-Data-Abstand: {}", prompter_distance);

    let env_file = root.join(".jules/session.env");
    let env_file_path = env_file.display().to_string();

    if write_env_file {
        if let Some(parent) = env_file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Kann Verzeichnis {} nicht erstellen: {}",
                    parent.display(),
                    e
                )
            })?;
        }
        let env_content = format!(
            "export SESSION_HASH={}\nexport SESSION_TS={}\nexport CONTEXTRA_CLAIM_CRATE={}\n",
            session_hash,
            timestamp,
            crate_name.unwrap_or("")
        );
        fs::write(&env_file, env_content).map_err(|e| {
            format!(
                "Kann Environment-Datei {} nicht schreiben: {}",
                env_file_path, e
            )
        })?;
    }

    Ok(SessionInitResult {
        session_hash,
        timestamp,
        open_blockers,
        active_claim_count,
        claimed_crate: crate_name.map(|s| s.to_string()),
        env_file_path,
        last_commit,
        last_merge,
        prompter_distance,
    })
}
