use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq)]
pub struct CommitEntry {
    pub hash: String,
    pub date: String,
    pub raw_message: String,
    pub scope: String,
    pub subject: String,
    pub normalized_subject: String,
}

pub fn normalize_subject(subject: &str) -> String {
    let lower = subject.to_lowercase();
    // Remove ADR references like (ADR-067) or ADR-067
    let adr_re = Regex::new(r"(?i)\(?adr-\d+\)?").unwrap();
    let no_adr = adr_re.replace_all(&lower, "");

    // Replace non-alphabetic characters with space
    let clean: String = no_adr
        .chars()
        .map(|c| if c.is_alphabetic() { c } else { ' ' })
        .collect();

    clean.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_conventional_commit(raw_message: &str) -> (String, String) {
    let re =
        Regex::new(r"^(?i)[a-z0-9_-]+(?:\((?P<scope>[^)]+)\))?!?:\s*(?P<subject>.*)$").unwrap();
    if let Some(caps) = re.captures(raw_message.trim()) {
        let scope = caps
            .name("scope")
            .map(|m| m.as_str().trim().to_lowercase())
            .unwrap_or_default();
        let subject = caps
            .name("subject")
            .map(|m| m.as_str().trim())
            .unwrap_or_default()
            .to_string();
        (scope, subject)
    } else {
        (String::new(), raw_message.trim().to_string())
    }
}

pub fn get_trigrams(s: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    if s.is_empty() {
        return set;
    }
    let padded = format!("  {}  ", s);
    let chars: Vec<char> = padded.chars().collect();
    if chars.len() >= 3 {
        for window in chars.windows(3) {
            set.insert(window.iter().collect::<String>());
        }
    }
    set
}

pub fn normalized_similarity(a: &str, b: &str) -> f64 {
    let norm_a = normalize_subject(a);
    let norm_b = normalize_subject(b);

    if norm_a.is_empty() && norm_b.is_empty() {
        return 1.0;
    }
    if norm_a.is_empty() || norm_b.is_empty() {
        return 0.0;
    }

    let tri_a = get_trigrams(&norm_a);
    let tri_b = get_trigrams(&norm_b);

    let intersection = tri_a.intersection(&tri_b).count();
    let union = tri_a.union(&tri_b).count();

    if union == 0 {
        return 0.0;
    }

    intersection as f64 / union as f64
}

pub fn parse_override_exception(pr_body: &str) -> Option<String> {
    let re =
        Regex::new(r"(?i)(?:Supersedes|Fixes-Regression-Of):\s*\b([a-fA-F0-9]{7,40})\b").unwrap();
    re.captures(pr_body).map(|caps| caps[1].to_string())
}

pub fn parse_git_log_output(output: &str) -> Vec<CommitEntry> {
    let mut entries = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let hash = parts[0].trim().to_string();
            let date = parts[1].trim().to_string();
            let raw_message = parts[2].trim().to_string();

            let (scope, subject) = parse_conventional_commit(&raw_message);
            let normalized_subject = normalize_subject(&subject);

            entries.push(CommitEntry {
                hash,
                date,
                raw_message,
                scope,
                subject,
                normalized_subject,
            });
        }
    }
    entries
}

pub fn get_historical_commits() -> Result<Vec<CommitEntry>, String> {
    let output = Command::new("git")
        .args(["log", "--since=30 days ago", "--format=%H%x09%ci%x09%s"])
        .output()
        .map_err(|e| format!("Failed to execute git log: {}", e))?;

    if !output.status.success() {
        return Err("git log command returned non-zero exit status".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_git_log_output(&stdout))
}

pub fn get_remote_branch_commits() -> Vec<CommitEntry> {
    let output = Command::new("git")
        .args([
            "log",
            "--remotes",
            "--not",
            "HEAD",
            "--format=%H%x09%ci%x09%s",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return parse_git_log_output(&stdout);
        }
    }
    Vec::new()
}

pub fn get_current_head_sha() -> String {
    Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub fn fetch_open_prs_from_github(token: &str) -> Result<Vec<CommitEntry>, String> {
    let url = "https://api.github.com/repos/tfufuz1/memfuse/pulls?state=open&per_page=100";
    let output = Command::new("curl")
        .args([
            "-s",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-H",
            "User-Agent: memfuse-xtask",
            "-H",
            "Accept: application/vnd.github+json",
            url,
        ])
        .output()
        .map_err(|e| format!("curl command failed: {}", e))?;

    if !output.status.success() {
        return Err(format!("curl returned exit status {}", output.status));
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let prs: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

    let arr = match prs.as_array() {
        Some(a) => a,
        None => return Err(format!("GitHub API response is not an array: {}", body)),
    };

    let mut entries = Vec::new();
    let current_head = get_current_head_sha();

    for pr in arr {
        let head_sha = pr["head"]["sha"].as_str().unwrap_or_default();
        if !current_head.is_empty() && head_sha == current_head {
            continue;
        }

        let title = pr["title"].as_str().unwrap_or_default();
        if !title.is_empty() {
            let (scope, subject) = parse_conventional_commit(title);
            let normalized_subject = normalize_subject(&subject);
            let date = pr["created_at"].as_str().unwrap_or_default().to_string();
            let hash = if !head_sha.is_empty() {
                head_sha.to_string()
            } else {
                format!("PR#{}", pr["number"])
            };

            entries.push(CommitEntry {
                hash,
                date,
                raw_message: title.to_string(),
                scope,
                subject,
                normalized_subject,
            });
        }
    }

    Ok(entries)
}

pub fn get_new_commits() -> Result<Vec<CommitEntry>, String> {
    let output = Command::new("git")
        .args(["log", "origin/main..HEAD", "--format=%H%x09%ci%x09%s"])
        .output();

    let stdout = match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).to_string();
            if s.trim().is_empty() {
                // Fallback to last commit if origin/main..HEAD is empty
                let fallback = Command::new("git")
                    .args(["log", "-1", "--format=%H%x09%ci%x09%s"])
                    .output()
                    .map_err(|e| format!("Failed to execute git log fallback: {}", e))?;
                String::from_utf8_lossy(&fallback.stdout).to_string()
            } else {
                s
            }
        }
        _ => {
            let fallback = Command::new("git")
                .args(["log", "-1", "--format=%H%x09%ci%x09%s"])
                .output()
                .map_err(|e| format!("Failed to execute git log fallback: {}", e))?;
            String::from_utf8_lossy(&fallback.stdout).to_string()
        }
    };

    Ok(parse_git_log_output(&stdout))
}

#[derive(Debug, PartialEq)]
pub struct DuplicateHit {
    pub new_commit: CommitEntry,
    pub historical_commit: CommitEntry,
    pub similarity: f64,
}

pub fn evaluate_duplicate_hits(
    new_commits: &[CommitEntry],
    candidate_commits: &[CommitEntry],
) -> Vec<DuplicateHit> {
    let mut hits = Vec::new();
    let mut seen_pairs = HashSet::new();

    for new_c in new_commits {
        if new_c.normalized_subject.is_empty() {
            continue;
        }

        for cand_c in candidate_commits {
            if new_c.hash == cand_c.hash {
                continue;
            }

            if new_c.scope == cand_c.scope {
                let similarity = normalized_similarity(&new_c.subject, &cand_c.subject);
                if similarity >= 0.85 {
                    let pair_key = if new_c.hash < cand_c.hash {
                        format!("{}:{}", new_c.hash, cand_c.hash)
                    } else {
                        format!("{}:{}", cand_c.hash, new_c.hash)
                    };
                    if seen_pairs.insert(pair_key) {
                        hits.push(DuplicateHit {
                            new_commit: new_c.clone(),
                            historical_commit: cand_c.clone(),
                            similarity,
                        });
                    }
                }
            }
        }
    }

    hits
}

fn parse_diff_file_path(line: &str) -> Option<PathBuf> {
    if let Some(stripped) = line.strip_prefix("+++ ") {
        let p = stripped.trim();
        if p == "/dev/null" {
            return None;
        }
        if let Some(rest) = p.strip_prefix("b/") {
            Some(PathBuf::from(rest))
        } else if let Some(rest) = p.strip_prefix("a/") {
            Some(PathBuf::from(rest))
        } else {
            Some(PathBuf::from(p))
        }
    } else {
        None
    }
}

/// Extracts newly added top-level Rust symbol names per changed file path from a unified diff text.
///
/// # Heuristic & Limitations
/// This function uses a regex heuristic operating on added diff lines (`+` prefix, ignoring `+++` headers).
/// Known limitations:
/// - Macro-generated symbols are not detected.
/// - Multiline function/struct/enum/trait signatures with newlines before the identifier name may be missed.
/// - Non-standard formatting or non-Rust code may lead to undetected or false symbols.
pub fn extract_changed_files_and_symbols(
    diff_text: &str,
) -> HashMap<PathBuf, HashSet<String>> {
    let mut map: HashMap<PathBuf, HashSet<String>> = HashMap::new();
    let mut current_file: Option<PathBuf> = None;

    let fn_re = Regex::new(
        r#"^\+\s*(?:pub(?:\([^)]+\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+(?:"[^"]+"\s+)?)*fn\s+([a-zA-Z_]\w*)"#,
    )
    .unwrap();
    let struct_re = Regex::new(r#"^\+\s*(?:pub(?:\([^)]+\))?\s+)?struct\s+([a-zA-Z_]\w*)"#).unwrap();
    let enum_re = Regex::new(r#"^\+\s*(?:pub(?:\([^)]+\))?\s+)?enum\s+([a-zA-Z_]\w*)"#).unwrap();
    let trait_re = Regex::new(r#"^\+\s*(?:pub(?:\([^)]+\))?\s+)?trait\s+([a-zA-Z_]\w*)"#).unwrap();

    for line in diff_text.lines() {
        if line.starts_with("+++ ") {
            current_file = parse_diff_file_path(line);
            if let Some(ref path) = current_file {
                map.entry(path.clone()).or_default();
            }
            continue;
        }

        if line.starts_with("--- ") || line.starts_with("diff --git") {
            continue;
        }

        if line.starts_with('+') && !line.starts_with("+++") {
            if let Some(ref file_path) = current_file {
                let symbol = if let Some(caps) = fn_re.captures(line) {
                    caps.get(1).map(|m| m.as_str().to_string())
                } else if let Some(caps) = struct_re.captures(line) {
                    caps.get(1).map(|m| m.as_str().to_string())
                } else if let Some(caps) = enum_re.captures(line) {
                    caps.get(1).map(|m| m.as_str().to_string())
                } else if let Some(caps) = trait_re.captures(line) {
                    caps.get(1).map(|m| m.as_str().to_string())
                } else {
                    None
                };

                if let Some(sym) = symbol {
                    map.entry(file_path.clone()).or_default().insert(sym);
                }
            }
        }
    }

    map
}

/// Computes the intersection of newly introduced symbols for shared file paths.
///
/// Returns a list of `(PathBuf, HashSet<String>)` tuples for files where the symbol intersection
/// is non-empty.
pub fn compute_symbol_overlap(
    current: &HashMap<PathBuf, HashSet<String>>,
    other: &HashMap<PathBuf, HashSet<String>>,
) -> Vec<(PathBuf, HashSet<String>)> {
    let mut overlap = Vec::new();

    for (path, current_symbols) in current {
        if let Some(other_symbols) = other.get(path) {
            let intersection: HashSet<String> = current_symbols
                .intersection(other_symbols)
                .cloned()
                .collect();
            if !intersection.is_empty() {
                overlap.push((path.clone(), intersection));
            }
        }
    }

    overlap
}

#[derive(Debug, Clone)]
pub struct BranchDiffEntry {
    pub ref_id: String,
    pub raw_message: String,
    pub symbol_map: HashMap<PathBuf, HashSet<String>>,
}

pub fn get_remote_branch_diffs() -> Vec<BranchDiffEntry> {
    let mut diff_entries = Vec::new();

    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)%09%(objectname)%09%(contents:subject)",
            "refs/remotes/",
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let ref_name = parts[0].trim();
                    let commit_hash = parts[1].trim();
                    let subject = parts.get(2).copied().unwrap_or_default().trim();

                    if ref_name.ends_with("/HEAD")
                        || ref_name == "origin/main"
                        || ref_name == "main"
                    {
                        continue;
                    }

                    let mb_output = Command::new("git")
                        .args(["merge-base", "origin/main", commit_hash])
                        .output()
                        .or_else(|_| {
                            Command::new("git")
                                .args(["merge-base", "main", commit_hash])
                                .output()
                        });

                    let merge_base = match mb_output {
                        Ok(o) if o.status.success() => {
                            String::from_utf8_lossy(&o.stdout).trim().to_string()
                        }
                        _ => continue,
                    };

                    if merge_base.is_empty() {
                        continue;
                    }

                    let diff_output = Command::new("git")
                        .args(["diff", &format!("{}..{}", merge_base, commit_hash)])
                        .output();

                    if let Ok(diff_out) = diff_output {
                        if diff_out.status.success() {
                            let diff_text = String::from_utf8_lossy(&diff_out.stdout);
                            let symbol_map = extract_changed_files_and_symbols(&diff_text);
                            if !symbol_map.is_empty() {
                                diff_entries.push(BranchDiffEntry {
                                    ref_id: format!(
                                        "{} ({})",
                                        ref_name,
                                        &commit_hash[..8.min(commit_hash.len())]
                                    ),
                                    raw_message: subject.to_string(),
                                    symbol_map,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    diff_entries
}

pub fn fetch_open_pr_diffs_from_github(
    token: &str,
) -> Result<Vec<BranchDiffEntry>, String> {
    let url = "https://api.github.com/repos/tfufuz1/memfuse/pulls?state=open&per_page=100";
    let output = Command::new("curl")
        .args([
            "-s",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-H",
            "User-Agent: memfuse-xtask",
            "-H",
            "Accept: application/vnd.github+json",
            url,
        ])
        .output()
        .map_err(|e| format!("curl command failed when listing open PRs: {}", e))?;

    if !output.status.success() {
        return Err(format!("curl returned exit status {} when listing open PRs", output.status));
    }

    let body = String::from_utf8_lossy(&output.stdout);
    let prs: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Failed to parse GitHub API PR list response: {}", e))?;

    let arr = match prs.as_array() {
        Some(a) => a,
        None => return Err(format!("GitHub API response is not an array: {}", body)),
    };

    let mut entries = Vec::new();
    let current_head = get_current_head_sha();
    let mut attempt_count = 0;
    let mut fail_count = 0;

    for pr in arr {
        let head_sha = pr["head"]["sha"].as_str().unwrap_or_default();
        if !current_head.is_empty() && head_sha == current_head {
            continue;
        }

        let pr_number = match pr["number"].as_u64() {
            Some(n) => n,
            None => continue,
        };

        attempt_count += 1;
        let title = pr["title"].as_str().unwrap_or_default();
        let pr_url = format!("https://api.github.com/repos/tfufuz1/memfuse/pulls/{}", pr_number);

        let diff_output = Command::new("curl")
            .args([
                "-s",
                "-H",
                &format!("Authorization: Bearer {}", token),
                "-H",
                "User-Agent: memfuse-xtask",
                "-H",
                "Accept: application/vnd.github.v3.diff",
                &pr_url,
            ])
            .output();

        match diff_output {
            Ok(out) if out.status.success() => {
                let diff_text = String::from_utf8_lossy(&out.stdout);
                let symbol_map = extract_changed_files_and_symbols(&diff_text);
                let ref_id = if !head_sha.is_empty() {
                    format!("PR#{} ({})", pr_number, &head_sha[..8.min(head_sha.len())])
                } else {
                    format!("PR#{}", pr_number)
                };

                entries.push(BranchDiffEntry {
                    ref_id,
                    raw_message: title.to_string(),
                    symbol_map,
                });
            }
            Ok(out) => {
                fail_count += 1;
                eprintln!(
                    "⚠️ Gate 12: Failed to fetch diff for PR#{} (curl exit status {}). Skipping single PR diff.",
                    pr_number, out.status
                );
            }
            Err(e) => {
                fail_count += 1;
                eprintln!(
                    "⚠️ Gate 12: Failed to execute curl for PR#{} diff: {}. Skipping single PR diff.",
                    pr_number, e
                );
            }
        }
    }

    if attempt_count > 0 && fail_count == attempt_count {
        return Err(format!(
            "Failed to fetch diffs for any open PRs from GitHub REST API (0/{} succeeded)",
            attempt_count
        ));
    }

    Ok(entries)
}

pub fn get_current_diff() -> String {
    let mb_output = Command::new("git")
        .args(["merge-base", "origin/main", "HEAD"])
        .output()
        .or_else(|_| {
            Command::new("git")
                .args(["merge-base", "main", "HEAD"])
                .output()
        });

    if let Ok(out) = mb_output {
        if out.status.success() {
            let mb = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !mb.is_empty() {
                if let Ok(diff_out) = Command::new("git")
                    .args(["diff", &format!("{}..HEAD", mb)])
                    .output()
                {
                    if diff_out.status.success() {
                        let diff_str = String::from_utf8_lossy(&diff_out.stdout).to_string();
                        if !diff_str.trim().is_empty() {
                            return diff_str;
                        }
                    }
                }
            }
        }
    }

    if let Ok(diff_out) = Command::new("git")
        .args(["diff", "HEAD~1..HEAD"])
        .output()
    {
        if diff_out.status.success() {
            return String::from_utf8_lossy(&diff_out.stdout).to_string();
        }
    }

    String::new()
}

pub fn check_duplicate_intent() -> Result<(), String> {
    println!("=== Gate 12: Duplicate-PR-Intent-Detection ===");

    let mut candidate_commits = get_historical_commits()?;
    let remote_commits = get_remote_branch_commits();
    candidate_commits.extend(remote_commits);

    let is_ci = env::var("MEMFUSE_CI").map(|v| v == "true").unwrap_or(false)
        || env::var("GITHUB_ACTIONS").is_ok();
    let token = env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty());

    let mut pr_diff_candidates = Vec::new();

    if let Some(tok) = token {
        match fetch_open_prs_from_github(&tok) {
            Ok(pr_commits) => {
                println!(
                    "ℹ️ Gate 12: Fetched {} open PR entries via GitHub REST API.",
                    pr_commits.len()
                );
                candidate_commits.extend(pr_commits);
            }
            Err(e) => {
                let msg = format!("❌ Gate 12: GitHub REST API call failed: {}", e);
                eprintln!("{}", msg);
                if is_ci {
                    return Err(format!("GitHub API call failed in CI context: {}", e));
                }
            }
        }

        match fetch_open_pr_diffs_from_github(&tok) {
            Ok(diffs) => {
                pr_diff_candidates.extend(diffs);
            }
            Err(e) => {
                let msg = format!("❌ Gate 12: GitHub REST API PR diff call failed: {}", e);
                eprintln!("{}", msg);
                if is_ci {
                    return Err(format!("GitHub API PR diff call failed in CI context: {}", e));
                }
            }
        }
    } else if is_ci {
        eprintln!("⚠️ Gate 12: GITHUB_TOKEN missing in CI environment — cannot query open PRs via GitHub REST API.");
    }

    let remote_diffs = get_remote_branch_diffs();
    pr_diff_candidates.extend(remote_diffs);

    let new_commits = get_new_commits()?;
    let pr_body = env::var("MEMFUSE_PR_BODY").unwrap_or_default();
    let override_hash = parse_override_exception(&pr_body);

    let hits = evaluate_duplicate_hits(&new_commits, &candidate_commits);

    // Subject similarity check first
    if !hits.is_empty() && override_hash.is_none() {
        eprintln!("❌ Gate 12: Potential duplicate PR intent detected without valid 'Supersedes:' or 'Fixes-Regression-Of:' reference!");
        for hit in &hits {
            eprintln!(
                "  - New Commit: {} ({})",
                hit.new_commit.raw_message, hit.new_commit.hash
            );
            eprintln!(
                "    Matches Historical/Open Branch Commit: {} ({})",
                hit.historical_commit.raw_message, hit.historical_commit.hash
            );
            eprintln!(
                "    Scope: '{}', Similarity: {:.2}%",
                hit.new_commit.scope,
                hit.similarity * 100.0
            );
        }
        eprintln!("\n💡 To override if this intentional re-implementation/refactor, add 'Supersedes: <commit-hash>' or 'Fixes-Regression-Of: <commit-hash>' to your PR description.");
        return Err("Duplicate PR intent detected".to_string());
    }

    // Symbol overlap check
    let current_diff_text = get_current_diff();
    let current_symbol_map = extract_changed_files_and_symbols(&current_diff_text);

    let mut symbol_collisions = Vec::new();

    if !current_symbol_map.is_empty() {
        for candidate in &pr_diff_candidates {
            let overlaps = compute_symbol_overlap(&current_symbol_map, &candidate.symbol_map);
            if !overlaps.is_empty() {
                symbol_collisions.push((candidate.clone(), overlaps));
            }
        }
    }

    if !symbol_collisions.is_empty() {
        if let Some(ref ref_hash) = override_hash {
            println!(
                "ℹ️ Gate 12: Symbol overlap detected, but valid override exception found in PR body (Supersedes/Fixes-Regression-Of: {}).",
                ref_hash
            );
            for (candidate, overlaps) in &symbol_collisions {
                println!(
                    "  [INFO] Symbol overlap with {} ('{}'):",
                    candidate.ref_id, candidate.raw_message
                );
                for (file, symbols) in overlaps {
                    let mut sorted_syms: Vec<_> = symbols.iter().cloned().collect();
                    sorted_syms.sort();
                    println!(
                        "    - File: {} | Symbols: {}",
                        file.display(),
                        sorted_syms.join(", ")
                    );
                }
            }
        } else {
            eprintln!("❌ Gate 12: Potential duplicate PR intent detected via Symbol-Overlap without valid 'Supersedes:' or 'Fixes-Regression-Of:' reference!");
            for (candidate, overlaps) in &symbol_collisions {
                eprintln!(
                    "  - Colliding Branch/PR: {} ('{}')",
                    candidate.ref_id, candidate.raw_message
                );
                for (file, symbols) in overlaps {
                    let mut sorted_syms: Vec<_> = symbols.iter().cloned().collect();
                    sorted_syms.sort();
                    eprintln!(
                        "    Colliding File: {} | Overlapping Symbols: {}",
                        file.display(),
                        sorted_syms.join(", ")
                    );
                }
            }
            eprintln!("\n💡 To override if this intentional re-implementation/refactor, add 'Supersedes: <commit-hash>' or 'Fixes-Regression-Of: <commit-hash>' to your PR description.");
            return Err("Duplicate PR symbol overlap detected".to_string());
        }
    }

    if !hits.is_empty() && override_hash.is_some() {
        println!(
            "ℹ️ Gate 12: Potential duplicate intent detected, but valid override exception found in PR body (Supersedes/Fixes-Regression-Of: {}).",
            override_hash.as_ref().unwrap()
        );
        for hit in &hits {
            println!(
                "  [INFO] New commit '{}' ({}) is {:.2}% similar to historical commit '{}' ({}) [scope: '{}']",
                hit.new_commit.raw_message,
                hit.new_commit.hash,
                hit.similarity * 100.0,
                hit.historical_commit.raw_message,
                hit.historical_commit.hash,
                hit.new_commit.scope
            );
        }
    }

    println!("✅ Gate 12: No duplicate PR intent detected.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_intent_similarity() {
        let a = "calibrate PathRAG sufficiency threshold to 0.1";
        let b = "calibrate PathRAG sufficiency threshold to 0.1";
        let sim = normalized_similarity(a, b);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "Expected ~1.0 similarity, got {}",
            sim
        );
    }

    #[test]
    fn test_identical_intent_with_adr_similarity() {
        let a = "calibrate PathRAG sufficiency threshold to 0.1 (ADR-067)";
        let b = "calibrate PathRAG sufficiency threshold to 0.1";
        let sim = normalized_similarity(a, b);
        assert!(
            (sim - 1.0).abs() < 1e-6,
            "Expected ~1.0 similarity despite ADR reference, got {}",
            sim
        );
    }

    #[test]
    fn test_different_intent_similarity() {
        let a = "calibrate PathRAG sufficiency threshold to 0.1";
        let b = "implement bidirectional PPR damping factor";
        let sim = normalized_similarity(a, b);
        assert!(
            sim < 0.85,
            "Expected similarity significantly lower than 0.85, got {}",
            sim
        );
    }

    #[test]
    fn test_parse_override_exception_valid() {
        let pr_body = "This PR improves PPR damping.\n\nSupersedes: a1b2c3d\n";
        let parsed = parse_override_exception(pr_body);
        assert_eq!(parsed, Some("a1b2c3d".to_string()));

        let pr_body_2 = "Fixes-Regression-Of: 4b8a9c0d1e2f3a4b5c6d7e8f";
        let parsed_2 = parse_override_exception(pr_body_2);
        assert_eq!(parsed_2, Some("4b8a9c0d1e2f3a4b5c6d7e8f".to_string()));
    }

    #[test]
    fn test_parse_override_exception_without_valid_hash() {
        let pr_body = "This PR supersedes prior work.\nSupersedes:\nFixes-Regression-Of: xyz";
        let parsed = parse_override_exception(pr_body);
        assert_eq!(
            parsed, None,
            "Empty or non-hex hash after keyword must not be accepted"
        );

        let pr_body_short_hash = "Supersedes: 123456"; // 6 chars < 7 chars
        let parsed_short = parse_override_exception(pr_body_short_hash);
        assert_eq!(
            parsed_short, None,
            "Hash shorter than 7 chars must not be accepted"
        );
    }

    #[test]
    fn test_parse_conventional_commit() {
        let msg = "feat(memfuse-core): add new vector index";
        let (scope, subject) = parse_conventional_commit(msg);
        assert_eq!(scope, "memfuse-core");
        assert_eq!(subject, "add new vector index");

        let msg_no_scope = "fix: resolve deadlock in store";
        let (scope_none, subject_none) = parse_conventional_commit(msg_no_scope);
        assert_eq!(scope_none, "");
        assert_eq!(subject_none, "resolve deadlock in store");
    }

    #[test]
    fn test_regression_incident_duplicate_intent_detected_concurrent_branches() {
        // Incident 2026-09-13 commit pairs:
        // Pair 1: a238752e (#2373) vs 486fe929 (#2378)
        let pr1_commit1 = CommitEntry {
            hash: "a238752e".to_string(),
            date: "2026-09-13T17:43:00Z".to_string(),
            raw_message:
                "fix(store): replace dirty-read with IntentLockManager in put_if_absent (#2373)"
                    .to_string(),
            scope: "store".to_string(),
            subject: "replace dirty-read with IntentLockManager in put_if_absent (#2373)"
                .to_string(),
            normalized_subject: normalize_subject(
                "replace dirty-read with IntentLockManager in put_if_absent",
            ),
        };
        let pr2_commit1 = CommitEntry {
            hash: "486fe929".to_string(),
            date: "2026-09-13T17:58:00Z".to_string(),
            raw_message:
                "fix(store): replace dirty-read with IntentLockManager in put_if_absent (#2378)"
                    .to_string(),
            scope: "store".to_string(),
            subject: "replace dirty-read with IntentLockManager in put_if_absent (#2378)"
                .to_string(),
            normalized_subject: normalize_subject(
                "replace dirty-read with IntentLockManager in put_if_absent",
            ),
        };

        // Pair 2: 6b40d84a (#2372) vs 916a20a0 (#2377)
        let pr1_commit2 = CommitEntry {
            hash: "6b40d84a".to_string(),
            date: "2026-09-13T17:43:00Z".to_string(),
            raw_message:
                "refactor(store): extract advance_visibility and apply_mem_updates (#2372)"
                    .to_string(),
            scope: "store".to_string(),
            subject: "extract advance_visibility and apply_mem_updates (#2372)".to_string(),
            normalized_subject: normalize_subject(
                "extract advance_visibility and apply_mem_updates",
            ),
        };
        let pr2_commit2 = CommitEntry {
            hash: "916a20a0".to_string(),
            date: "2026-09-13T17:58:00Z".to_string(),
            raw_message:
                "refactor(store): extract advance_visibility and apply_mem_updates (#2377)"
                    .to_string(),
            scope: "store".to_string(),
            subject: "extract advance_visibility and apply_mem_updates (#2377)".to_string(),
            normalized_subject: normalize_subject(
                "extract advance_visibility and apply_mem_updates",
            ),
        };

        let new_commits = vec![pr2_commit1.clone(), pr2_commit2.clone()];
        let candidate_commits = vec![pr1_commit1.clone(), pr1_commit2.clone()];

        let hits = evaluate_duplicate_hits(&new_commits, &candidate_commits);

        assert_eq!(
            hits.len(),
            2,
            "Expected 2 duplicate intent hits for the incident commit pairs"
        );

        let hit1 = hits
            .iter()
            .find(|h| h.new_commit.hash == "486fe929")
            .unwrap();
        assert_eq!(hit1.historical_commit.hash, "a238752e");
        assert!(hit1.similarity >= 0.85);

        let hit2 = hits
            .iter()
            .find(|h| h.new_commit.hash == "916a20a0")
            .unwrap();
        assert_eq!(hit2.historical_commit.hash, "6b40d84a");
        assert!(hit2.similarity >= 0.85);
    }

    #[test]
    fn test_symbol_overlap_same_file_same_symbol_detected() {
        let diff_a = r#"
--- a/crates/memfuse-store/src/sstable.rs
+++ b/crates/memfuse-store/src/sstable.rs
@@ -100,6 +100,12 @@
+fn binary_search_in_block(data: &[u8]) -> usize {
+    // implementation A
+    42
+}
"#;

        let diff_b = r#"
--- a/crates/memfuse-store/src/sstable.rs
+++ b/crates/memfuse-store/src/sstable.rs
@@ -100,6 +100,12 @@
+pub fn binary_search_in_block(block_data: &[u8], key: &[u8]) -> Option<usize> {
+    // implementation B with different body
+    None
+}
"#;

        let map_a = extract_changed_files_and_symbols(diff_a);
        let map_b = extract_changed_files_and_symbols(diff_b);

        let overlaps = compute_symbol_overlap(&map_a, &map_b);
        assert_eq!(overlaps.len(), 1, "Expected 1 colliding file");
        let (file, symbols) = &overlaps[0];
        assert_eq!(file, &PathBuf::from("crates/memfuse-store/src/sstable.rs"));
        assert!(symbols.contains("binary_search_in_block"));
    }

    #[test]
    fn test_symbol_overlap_same_file_different_symbols_no_collision() {
        let diff_a = r#"
--- a/crates/memfuse-store/src/sstable.rs
+++ b/crates/memfuse-store/src/sstable.rs
@@ -100,6 +100,10 @@
+fn binary_search_in_block(data: &[u8]) -> usize {
+    42
+}
"#;

        let diff_b = r#"
--- a/crates/memfuse-store/src/sstable.rs
+++ b/crates/memfuse-store/src/sstable.rs
@@ -200,6 +200,10 @@
+pub fn linear_scan_block(data: &[u8]) -> usize {
+    0
+}
"#;

        let map_a = extract_changed_files_and_symbols(diff_a);
        let map_b = extract_changed_files_and_symbols(diff_b);

        let overlaps = compute_symbol_overlap(&map_a, &map_b);
        assert!(
            overlaps.is_empty(),
            "Different symbols in same file must not trigger collision"
        );
    }

    #[test]
    fn test_symbol_overlap_different_files_same_symbol_no_collision() {
        let diff_a = r#"
--- a/crates/memfuse-store/src/sstable.rs
+++ b/crates/memfuse-store/src/sstable.rs
@@ -100,6 +100,10 @@
+fn helper_fn() -> bool {
+    true
+}
"#;

        let diff_b = r#"
--- a/crates/memfuse-store/src/wal.rs
+++ b/crates/memfuse-store/src/wal.rs
@@ -100,6 +100,10 @@
+fn helper_fn() -> bool {
+    false
+}
"#;

        let map_a = extract_changed_files_and_symbols(diff_a);
        let map_b = extract_changed_files_and_symbols(diff_b);

        let overlaps = compute_symbol_overlap(&map_a, &map_b);
        assert!(
            overlaps.is_empty(),
            "Same symbol in different files must not trigger collision"
        );
    }

    #[test]
    fn test_regression_incident_sstable_symbol_overlap_detected() {
        let diff_pr1 = r#"
diff --git a/crates/memfuse-store/src/sstable.rs b/crates/memfuse-store/src/sstable.rs
--- a/crates/memfuse-store/src/sstable.rs
+++ b/crates/memfuse-store/src/sstable.rs
@@ -540,6 +540,15 @@ impl SstableReader {
+pub struct BlockCache {
+    shards: Vec<RwLock<LruCache<(u64, u64), Bytes>>>,
+}
+
+fn binary_search_in_block(
+    block_data: &[u8],
+    offsets_start: usize,
+    num_offsets: usize,
+    key: &[u8],
+) -> Result<Option<usize>> {
+    Ok(None)
+}
"#;

        let diff_pr2 = r#"
diff --git a/crates/memfuse-store/src/sstable.rs b/crates/memfuse-store/src/sstable.rs
--- a/crates/memfuse-store/src/sstable.rs
+++ b/crates/memfuse-store/src/sstable.rs
@@ -180,6 +180,15 @@ impl SstableReader {
+pub struct BlockCache {
+    shards: [BlockCacheShard; BLOCK_CACHE_SHARDS],
+}
+
+fn binary_search_in_block(
+    block_data: &[u8],
+    offsets_start: usize,
+    num_offsets: usize,
+    key: &[u8],
+) -> Result<Option<(usize, usize)>> {
+    Ok(None)
+}
"#;

        let map_pr1 = extract_changed_files_and_symbols(diff_pr1);
        let map_pr2 = extract_changed_files_and_symbols(diff_pr2);

        let overlaps = compute_symbol_overlap(&map_pr1, &map_pr2);

        assert_eq!(overlaps.len(), 1, "Must detect symbol overlap in sstable.rs");
        let (file, symbols) = &overlaps[0];
        assert_eq!(file, &PathBuf::from("crates/memfuse-store/src/sstable.rs"));
        assert!(symbols.contains("BlockCache"), "Must contain BlockCache");
        assert!(
            symbols.contains("binary_search_in_block"),
            "Must contain binary_search_in_block"
        );
    }
}
