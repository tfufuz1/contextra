use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommitSummary {
    pub hash: String,
    pub date: String,
    pub subject: String,
    #[serde(default)]
    pub crates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPack {
    pub generated_at: String,
    pub session_hash: String,
    pub head_commit: String,
    pub open_critical_tags: Vec<String>,
    pub active_claims: Vec<xtask::claim::ClaimEntry>,
    pub recent_commits: Vec<CommitSummary>,
}

pub fn get_head_commit(root: &Path) -> String {
    Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

pub fn truncate_with_notice(text: &str, max_lines: usize, max_bytes: usize, path: &str) -> String {
    let mut lines = Vec::new();
    let mut total_bytes = 0;
    let mut truncated = false;

    for line in text.lines() {
        let line_len = line.len() + 1; // including newline
        if lines.len() >= max_lines || total_bytes + line_len > max_bytes {
            truncated = true;
            break;
        }
        lines.push(line);
        total_bytes += line_len;
    }

    let mut result = lines.join("\n");
    if truncated {
        if !result.is_empty() {
            result.push('\n');
        }
        result.push_str(&format!("... [gekürzt, Quelle: {}]", path));
    }
    result
}

pub fn extract_affected_crates_from_files<I, S>(files: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut crates = BTreeSet::new();
    for file in files {
        let path = file.as_ref();
        if path.starts_with("crates/") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                crates.insert(parts[1].to_string());
            }
        }
    }
    crates.into_iter().collect()
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

fn get_commit_history(root: &Path, limit: usize) -> Vec<CommitSummary> {
    let count_str = format!("-{}", limit);
    let output = Command::new("git")
        .current_dir(root)
        .args(["log", &count_str, "--format=%H|%aI|%s"])
        .output()
        .ok()
        .filter(|out| out.status.success());

    let mut commits = Vec::new();
    if let Some(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() == 3 {
                let hash = parts[0].to_string();
                let date = parts[1].to_string();
                let subject = parts[2].to_string();

                // Fetch files for this commit to find affected crates
                let files_out = Command::new("git")
                    .current_dir(root)
                    .args(["show", "--name-only", "--format=", &hash])
                    .output()
                    .ok()
                    .filter(|o| o.status.success());

                let crates = if let Some(fo) = files_out {
                    let f_str = String::from_utf8_lossy(&fo.stdout);
                    extract_affected_crates_from_files(f_str.lines())
                } else {
                    Vec::new()
                };

                commits.push(CommitSummary {
                    hash,
                    date,
                    subject,
                    crates,
                });
            }
        }
    }
    commits
}

pub fn generate_repo_state(root: &Path) -> String {
    let mut md = String::new();

    // (a) Header
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    let head_sha = get_head_commit(root);
    let short_head = if head_sha.len() >= 8 {
        &head_sha[..8]
    } else {
        &head_sha
    };

    let commit_date = Command::new("git")
        .current_dir(root)
        .args(["log", "-1", "--format=%cd", "--date=short"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let branch = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    md.push_str("# REPO_STATE\n\n");
    md.push_str(&format!("- **Generiert:** {}\n", now));
    md.push_str(&format!("- **HEAD SHA:** {}\n", head_sha));
    md.push_str(&format!("- **Commit-Datum:** {}\n", commit_date));
    md.push_str(&format!("- **Branch:** {}\n", branch));
    md.push_str("- **Modus:** Repo State\n");
    md.push_str(&format!("Frische-Anker: HEAD muss `{}` sein\n\n", head_sha));

    // (b) Tool-Matrix
    md.push_str("## Tool-Matrix\n");
    let attest_report = xtask::harness::env_attest::env_attest_execute(root);
    if attest_report.findings.is_empty() {
        md.push_str("✅ Alle erforderlichen Toolchain-Tools sind vorhanden und gepinnt.\n\n");
    } else {
        for f in &attest_report.findings {
            let status = if f.severity == "error" {
                "❌"
            } else {
                "⚠️"
            };
            md.push_str(&format!(
                "- {} **{}**: {} (Gate: {})\n",
                status, f.id, f.message, f.file
            ));
        }
        md.push('\n');
    }

    // (c) BASELINE.md Auszug
    md.push_str("## Baseline Auszug\n");
    let baseline_path = root.join(".jules/context/BASELINE.md");
    if baseline_path.exists() {
        if let Ok(content) = fs::read_to_string(&baseline_path) {
            let first_line = content.lines().next().unwrap_or("");
            if !first_line.contains(&short_head[..4]) && !first_line.contains(&head_sha[..6]) {
                md.push_str("⚠️ **Warnung:** BASELINE.md HEAD-SHA weicht vom aktuellen HEAD ab!\n");
            }
            md.push_str(&truncate_with_notice(
                &content,
                20,
                1500,
                ".jules/context/BASELINE.md",
            ));
            md.push_str("\n\n");
        } else {
            md.push_str("*(BASELINE.md konnte nicht gelesen werden)*\n\n");
        }
    } else {
        md.push_str("*(Keine BASELINE.md vorhanden)*\n\n");
    }

    // (d) Letzte 15 Commits mit betroffenen Crates
    md.push_str("## Letzte 15 Commits\n");
    let commits_15 = get_commit_history(root, 15);
    if commits_15.is_empty() {
        md.push_str("*(Keine Commits)*\n\n");
    } else {
        for c in &commits_15 {
            let crates_str = if c.crates.is_empty() {
                "core/other".to_string()
            } else {
                c.crates.join(", ")
            };
            let short = if c.hash.len() >= 8 {
                &c.hash[..8]
            } else {
                &c.hash
            };
            md.push_str(&format!(
                "- `{}` ({}) [{}] - {}\n",
                short, c.date, crates_str, c.subject
            ));
        }
        md.push('\n');
    }

    // (e) Churn der letzten 7 Tage je Crate
    md.push_str("## Crate Churn (letzte 7 Tage)\n");
    let churn_out = Command::new("git")
        .current_dir(root)
        .args(["log", "--since=7 days ago", "--name-only", "--format="])
        .output()
        .ok()
        .filter(|o| o.status.success());

    let mut churn_map: BTreeMap<String, usize> = BTreeMap::new();
    if let Some(out) = churn_out {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("crates/") {
                let parts: Vec<&str> = line.split('/').collect();
                if parts.len() >= 2 {
                    *churn_map.entry(parts[1].to_string()).or_default() += 1;
                }
            }
        }
    }

    if churn_map.is_empty() {
        md.push_str("*(Kein Crate-Churn in den letzten 7 Tagen)*\n\n");
    } else {
        for (c_name, count) in &churn_map {
            md.push_str(&format!("- **{}**: {} Dateiänderungen\n", c_name, count));
        }
        md.push('\n');
    }

    // (f) Parallele Arbeit
    md.push_str("## Parallele Arbeit (Remote Branches)\n");
    let branches_out = Command::new("git")
        .current_dir(root)
        .args([
            "for-each-ref",
            "--sort=-committerdate",
            "refs/remotes/origin",
            "--format=%(refname:short)|%(committerdate:short)",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success());

    let mut parallel_count = 0;
    let mut parallel_branches: Vec<(String, Vec<String>, Vec<String>)> = Vec::new(); // (branch_name, crates, files)

    if let Some(out) = branches_out {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if parallel_count >= 25 {
                break;
            }
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() == 2 {
                let ref_name = parts[0];
                if ref_name == "origin/main" || ref_name == "origin/HEAD" || ref_name == "origin" {
                    continue;
                }
                let branch_short = ref_name
                    .strip_prefix("origin/")
                    .unwrap_or(ref_name)
                    .to_string();

                let diff_files_out = Command::new("git")
                    .current_dir(root)
                    .args(["diff", "--name-only", "origin/main...", ref_name])
                    .output()
                    .ok()
                    .filter(|o| o.status.success());

                if let Some(df) = diff_files_out {
                    let f_str = String::from_utf8_lossy(&df.stdout);
                    let file_list: Vec<String> = f_str
                        .lines()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if file_list.is_empty() {
                        continue;
                    }
                    let crates = extract_affected_crates_from_files(&file_list);
                    let capped_files: Vec<String> = file_list.into_iter().take(8).collect();

                    parallel_branches.push((
                        branch_short.clone(),
                        crates.clone(),
                        capped_files.clone(),
                    ));
                    parallel_count += 1;

                    let jules_tag = if branch_short.contains("jules") {
                        " 🤖 [Jules Branch]"
                    } else {
                        ""
                    };
                    let crates_str = if crates.is_empty() {
                        "core/other".to_string()
                    } else {
                        crates.join(", ")
                    };
                    md.push_str(&format!(
                        "- **{}**{} (Crates: {})\n",
                        branch_short, jules_tag, crates_str
                    ));
                    for f in capped_files {
                        md.push_str(&format!("  - `{}`\n", f));
                    }
                }
            }
        }
    }

    if parallel_count == 0 {
        md.push_str("*(Keine parallelen Remote-Branches)*\n\n");
    } else {
        md.push('\n');
    }

    // (g) Offene PRs
    md.push_str("## Offene PRs\n");
    let pr_file = root.join(".jules/context/RECENT_PRS.md");
    if pr_file.exists() {
        if let Ok(content) = fs::read_to_string(&pr_file) {
            md.push_str(&content);
            if !content.ends_with('\n') {
                md.push('\n');
            }
            md.push('\n');
        } else {
            md.push_str("*(Keine PR-Informationen)*\n\n");
        }
    } else {
        md.push_str("*(Keine PR-Informationen)*\n\n");
    }

    // (h) Offene CRITICAL/BLOCKER-AI-Tags pro Crate gezählt
    md.push_str("## Offene AI-Tags\n");
    let critical_tags = scan_critical_tags(root, None);
    let mut tags_by_crate: BTreeMap<String, usize> = BTreeMap::new();
    for tag in &critical_tags {
        if tag.starts_with("crates/") {
            let parts: Vec<&str> = tag.split('/').collect();
            if parts.len() >= 2 {
                *tags_by_crate.entry(parts[1].to_string()).or_default() += 1;
            }
        } else {
            *tags_by_crate.entry("core/other".to_string()).or_default() += 1;
        }
    }

    if tags_by_crate.is_empty() {
        md.push_str("*(Keine offenen CRITICAL/BLOCKER AI-Tags)*\n\n");
    } else {
        md.push_str("### Zusammenfassung pro Crate:\n");
        for (c_name, count) in &tags_by_crate {
            md.push_str(&format!("- **{}**: {} Tags\n", c_name, count));
        }
        md.push_str("\n### Erste 10 Fundstellen:\n");
        for tag in critical_tags.iter().take(10) {
            md.push_str(&format!("- `{}`\n", tag));
        }
        md.push('\n');
    }

    // (i) Karten-Index aus .jules/tasks/*.toml
    md.push_str("## Karten-Index\n");
    let tasks_dir = root.join(".jules/tasks");
    let mut cards = Vec::new();

    if tasks_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&tasks_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|e| e == "toml") {
                    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                    if stem == "_TEMPLATE" {
                        continue;
                    }
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(value) = content.parse::<toml::Value>() {
                            let id = value
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(stem)
                                .to_string();
                            let krate = value
                                .get("crate")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let risk = value
                                .get("risk")
                                .and_then(|v| v.as_str())
                                .unwrap_or("none")
                                .to_string();
                            let tier = value.get("tier").and_then(|v| v.as_integer()).unwrap_or(0);
                            let scope = value
                                .get("scope")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|x| x.as_str())
                                        .collect::<Vec<&str>>()
                                        .join(", ")
                                })
                                .unwrap_or_default();

                            // Overlap check with parallel branches
                            let mut overlap = false;
                            if let Some(scope_arr) = value.get("scope").and_then(|v| v.as_array()) {
                                for s_val in scope_arr {
                                    if let Some(s_str) = s_val.as_str() {
                                        for (_b_name, _b_crates, b_files) in &parallel_branches {
                                            for b_file in b_files {
                                                if s_str == b_file
                                                    || b_file
                                                        .starts_with(s_str.trim_end_matches('*'))
                                                {
                                                    overlap = true;
                                                    break;
                                                }
                                            }
                                            if overlap {
                                                break;
                                            }
                                        }
                                    }
                                    if overlap {
                                        break;
                                    }
                                }
                            }

                            cards.push((id, krate, risk, tier, scope, overlap));
                        }
                    }
                }
            }
        }
    }

    cards.sort_by(|a, b| a.0.cmp(&b.0));

    if cards.is_empty() {
        md.push_str("*(Keine Task-Karten gefunden)*\n\n");
    } else {
        md.push_str("| ID | Crate | Risiko | Tier | Scope | Overlap |\n");
        md.push_str("|---|---|---|---|---|---|\n");
        for (id, krate, risk, tier, scope, overlap) in cards {
            let short_scope = if scope.len() > 40 {
                format!("{}...", &scope[..37])
            } else {
                scope
            };
            let flag = if overlap { "⚠ Overlap" } else { "OK" };
            md.push_str(&format!(
                "| {} | {} | {} | {} | {} | {} |\n",
                id, krate, risk, tier, short_scope, flag
            ));
        }
        md.push('\n');
    }

    // (j) Crate-Karte aus prompter-data.json
    md.push_str("## Crate-Karte\n");
    let prompter_path = root.join(".jules/prompter-data.json");
    if prompter_path.exists() {
        if let Ok(content) = fs::read_to_string(&prompter_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(map) = val.get("target_architecture").and_then(|v| v.as_object()) {
                    md.push_str("| Crate | Tier / Kat | AGENTS-Pfad |\n");
                    md.push_str("|---|---|---|\n");
                    for (c_name, kat) in map {
                        let agents_path = format!("crates/{}/AGENTS.md", c_name);
                        let exists_tag = if root.join(&agents_path).exists() {
                            "✅"
                        } else {
                            "❌"
                        };
                        md.push_str(&format!(
                            "| {} | {} | {} {} |\n",
                            c_name,
                            kat.as_str().unwrap_or("-"),
                            agents_path,
                            exists_tag
                        ));
                    }
                    md.push('\n');
                }
            }
        }
    } else {
        let crates_dir = root.join("crates");
        if crates_dir.is_dir() {
            md.push_str("| Crate | AGENTS-Pfad |\n|---|---|\n");
            if let Ok(entries) = fs::read_dir(&crates_dir) {
                let mut c_names = Vec::new();
                for entry in entries.filter_map(|e| e.ok()) {
                    if entry.path().is_dir() {
                        c_names.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
                c_names.sort();
                for c_name in c_names {
                    let agents_path = format!("crates/{}/AGENTS.md", c_name);
                    let exists_tag = if root.join(&agents_path).exists() {
                        "✅"
                    } else {
                        "❌"
                    };
                    md.push_str(&format!(
                        "| {} | {} {} |\n",
                        c_name, agents_path, exists_tag
                    ));
                }
            }
            md.push('\n');
        }
    }

    // (k) Frische-Drift
    md.push_str("## Frische-Drift\n");
    let working_state_path = root.join("WORKING_STATE.md");
    let mut ws_date = "unbekannt".to_string();
    if working_state_path.exists() {
        if let Ok(content) = fs::read_to_string(&working_state_path) {
            for line in content.lines().take(10) {
                if line.contains("Stand:") || line.contains("Datum:") || line.contains("2026-") {
                    ws_date = line.to_string();
                    break;
                }
            }
        }
    }
    md.push_str(&format!("- **WORKING_STATE.md Stand:** {}\n", ws_date));
    md.push_str(&format!("- **Jüngstes Commit-Datum:** {}\n\n", commit_date));

    md
}

pub fn run_context_pack(
    crate_filter: Option<&str>,
    fast: bool,
    output: &Path,
) -> Result<ContextPack, String> {
    let root = xtask::find_root_dir();
    let generated_at = chrono::Utc::now().to_rfc3339();

    let session_hash = std::env::var("CONTEXTRA_SESSION_HASH")
        .or_else(|_| std::env::var("JULES_SESSION_ID"))
        .unwrap_or_else(|_| "unknown".to_string());

    let head_commit = get_head_commit(&root);
    let open_critical_tags = scan_critical_tags(&root, crate_filter);

    let claims_path = root.join(".jules/claims.json");
    let claims_db = xtask::claim::ClaimsDatabase::load(&claims_path);
    let active_claims: Vec<xtask::claim::ClaimEntry> =
        claims_db.claims.into_iter().filter(|c| c.active).collect();

    let recent_commits = if fast {
        get_commit_history(&root, 5)
    } else {
        get_commit_history(&root, 10)
    };

    let pack = ContextPack {
        generated_at,
        session_hash,
        head_commit,
        open_critical_tags,
        active_claims,
        recent_commits,
    };

    let repo_state_content = generate_repo_state(&root);

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
            let crates_str = if commit.crates.is_empty() {
                "".to_string()
            } else {
                format!(" [{}]", commit.crates.join(", "))
            };
            md.push_str(&format!(
                "- `{}` ({}){} - {}\n",
                &commit.hash[..std::cmp::min(8, commit.hash.len())],
                commit.date,
                crates_str,
                commit.subject
            ));
        }
        md.push('\n');
    }

    md.push_str("---\n\n");
    md.push_str(&repo_state_content);

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
            "Kann Context Pack / REPO_STATE nicht nach {} schreiben: {}",
            output.display(),
            e
        )
    })?;

    Ok(pack)
}
