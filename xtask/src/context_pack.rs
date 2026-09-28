use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitSummary {
    pub hash: String,
    pub date: String,
    pub subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPack {
    pub generated_at: String,
    pub session_hash: String,
    pub head_commit: String,
    pub open_critical_tags: Vec<String>,
    pub active_claims: Vec<crate::claim::ClaimEntry>,
    pub recent_commits: Vec<CommitSummary>,
}

pub fn scan_critical_tags(root: &Path, crate_filter: Option<&str>) -> Vec<String> {
    let scan_dir = match crate_filter {
        Some(filter) => {
            if filter.starts_with("crates/") {
                root.join(filter)
            } else {
                root.join("crates").join(filter)
            }
        }
        None => root.join("crates"),
    };

    if !scan_dir.exists() {
        return Vec::new();
    }

    let mut matches = Vec::new();

    for entry in WalkDir::new(&scan_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();
        if let Ok(content) = fs::read_to_string(path) {
            let relative_path = path.strip_prefix(root).unwrap_or(path);
            for (line_idx, line) in content.lines().enumerate() {
                if line.contains("AI-TAG[")
                    && (line.contains("BLOCKER") || line.contains("CRITICAL"))
                    && !line.contains("RESOLVED")
                {
                    matches.push(format!(
                        "{}:{}: {}",
                        relative_path.display(),
                        line_idx + 1,
                        line.trim()
                    ));
                }
            }
        }
    }

    matches.sort();
    matches
}

pub fn run_context_pack(
    crate_filter: Option<&str>,
    fast: bool,
    output: &Path,
) -> Result<ContextPack, String> {
    let root = crate::find_root_dir();
    let generated_at = chrono::Utc::now().to_rfc3339();

    let session_hash = std::env::var("CONTEXTRA_SESSION_HASH")
        .or_else(|_| std::env::var("JULES_SESSION_ID"))
        .unwrap_or_else(|_| "unknown".to_string());

    let head_commit = Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let open_critical_tags = scan_critical_tags(&root, crate_filter);

    let claims_path = root.join(".jules/claims.json");
    let claims_db = crate::claim::ClaimsDatabase::load(&claims_path);
    let active_claims: Vec<crate::claim::ClaimEntry> =
        claims_db.claims.into_iter().filter(|c| c.active).collect();

    let recent_commits = if fast {
        Vec::new()
    } else {
        Command::new("git")
            .current_dir(&root)
            .args(["log", "-10", "--format=%H|%aI|%s"])
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.splitn(3, '|').collect();
                        if parts.len() == 3 {
                            Some(CommitSummary {
                                hash: parts[0].to_string(),
                                date: parts[1].to_string(),
                                subject: parts[2].to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let pack = ContextPack {
        generated_at,
        session_hash,
        head_commit,
        open_critical_tags,
        active_claims,
        recent_commits,
    };

    let mut md = String::new();
    md.push_str("# Context Pack\n\n");
    md.push_str(&format!("- **Generated At:** {}\n", pack.generated_at));
    md.push_str(&format!("- **Session Hash:** {}\n", pack.session_hash));
    md.push_str(&format!("- **HEAD Commit:** {}\n\n", pack.head_commit));

    md.push_str("## Active Claims\n");
    if pack.active_claims.is_empty() {
        md.push_str("*(None)*\n\n");
    } else {
        for claim in &pack.active_claims {
            md.push_str(&format!(
                "- **{}** (Issue: {}, Session: {}, Since: {})\n",
                claim.krate, claim.issue, claim.session_id, claim.timestamp
            ));
        }
        md.push('\n');
    }

    md.push_str("## Open Critical AI Tags\n");
    if pack.open_critical_tags.is_empty() {
        md.push_str("*(None)*\n\n");
    } else {
        for tag in &pack.open_critical_tags {
            md.push_str(&format!("- `{}`\n", tag));
        }
        md.push('\n');
    }

    md.push_str("## Recent Commits\n");
    if pack.recent_commits.is_empty() {
        md.push_str("*(None or Fast Mode)*\n\n");
    } else {
        for commit in &pack.recent_commits {
            md.push_str(&format!(
                "- `{}` ({}) - {}\n",
                &commit.hash[..std::cmp::min(8, commit.hash.len())],
                commit.date,
                commit.subject
            ));
        }
        md.push('\n');
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Kann Verzeichnis {} nicht erstellen: {}",
                parent.display(),
                e
            )
        })?;
    }

    fs::write(output, &md).map_err(|e| {
        format!(
            "Kann Context Pack nicht nach {} schreiben: {}",
            output.display(),
            e
        )
    })?;

    Ok(pack)
}
