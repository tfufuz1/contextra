use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommitEntry {
    pub hash: String,
    pub date: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
    pub has_substantial_diff: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommitReport {
    pub total_commits: usize,
    pub shell_commits: Vec<ShellCommitEntry>,
}

fn parse_stat_line(line: &str) -> (usize, usize, usize) {
    let mut files_changed = 0;
    let mut insertions = 0;
    let mut deletions = 0;

    for part in line.split(',') {
        let part = part.trim();
        if part.contains("file") {
            if let Some(num) = part.split_whitespace().next() {
                files_changed = num.parse::<usize>().unwrap_or(0);
            }
        } else if part.contains("insertion") {
            if let Some(num) = part.split_whitespace().next() {
                insertions = num.parse::<usize>().unwrap_or(0);
            }
        } else if part.contains("deletion") {
            if let Some(num) = part.split_whitespace().next() {
                deletions = num.parse::<usize>().unwrap_or(0);
            }
        }
    }

    (files_changed, insertions, deletions)
}

fn ensure_unshallow(root: &Path) {
    // Check if shallow checkout
    let is_shallow = root.join(".git/shallow").exists();
    if is_shallow {
        let _ = Command::new("git")
            .current_dir(root)
            .args(["fetch", "--unshallow"])
            .output();
    }
}

pub fn run_shell_commit_audit_impl(
    root: &Path,
    since_days: Option<u32>,
    fail_on_count: Option<usize>,
) -> Result<ShellCommitReport, String> {
    ensure_unshallow(root);

    let since_arg = since_days.map(|d| format!("--since={} days ago", d));

    // Get total commit count
    let mut count_args = vec!["rev-list", "--count", "HEAD"];
    if let Some(ref sa) = since_arg {
        count_args.push(sa.as_str());
    }

    let count_output = Command::new("git")
        .current_dir(root)
        .args(&count_args)
        .output()
        .map_err(|e| format!("Failed to execute git rev-list: {}", e))?;

    let total_commits = if count_output.status.success() {
        String::from_utf8_lossy(&count_output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0)
    } else {
        0
    };

    // Find commits with exact subject "Shell-Commit"
    let mut log_args = vec!["log", "--format=%H|%cI|%s"];
    let branch_range;
    if let Some(ref sa) = since_arg {
        log_args.push(sa.as_str());
    } else {
        let has_origin_main = Command::new("git")
            .current_dir(root)
            .args(["rev-parse", "--verify", "origin/main"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if has_origin_main {
            branch_range = "origin/main..HEAD".to_string();
            log_args.push(&branch_range);
        }
    }

    let log_output = Command::new("git")
        .current_dir(root)
        .args(&log_args)
        .output()
        .map_err(|e| format!("Failed to execute git log: {}", e))?;

    if !log_output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&log_output.stderr)
        ));
    }

    let log_stdout = String::from_utf8_lossy(&log_output.stdout);
    let mut shell_commits = Vec::new();

    for line in log_stdout.lines() {
        let parts: Vec<&str> = line.splitn(3, '|').collect();
        if parts.len() < 3 {
            continue;
        }

        let hash = parts[0].trim();
        let date = parts[1].trim();
        let subject = parts[2].trim();

        if subject == "Shell-Commit" {
            // Run git show --stat for this commit
            let stat_output = Command::new("git")
                .current_dir(root)
                .args(["show", "--stat", "--oneline", hash])
                .output()
                .map_err(|e| format!("Failed to execute git show --stat on {}: {}", hash, e))?;

            let mut files_changed = 0;
            let mut insertions = 0;
            let mut deletions = 0;

            if stat_output.status.success() {
                let stat_stdout = String::from_utf8_lossy(&stat_output.stdout);
                for stat_line in stat_stdout.lines().rev() {
                    let trimmed = stat_line.trim();
                    if trimmed.contains("changed")
                        && (trimmed.contains("insertion") || trimmed.contains("deletion"))
                    {
                        let (f, i, d) = parse_stat_line(trimmed);
                        files_changed = f;
                        insertions = i;
                        deletions = d;
                        break;
                    }
                }
            }

            let has_substantial_diff = (insertions + deletions) > 50;

            shell_commits.push(ShellCommitEntry {
                hash: hash.to_string(),
                date: date.to_string(),
                files_changed,
                insertions,
                deletions,
                has_substantial_diff,
            });
        }
    }

    if let Some(fail_threshold) = fail_on_count {
        let substantial_count = shell_commits
            .iter()
            .filter(|c| c.has_substantial_diff)
            .count();
        if substantial_count > fail_threshold {
            return Err(format!(
                "Shell commit audit failed: found {} shell commits with substantial diff (>50 lines), exceeding threshold of {}",
                substantial_count, fail_threshold
            ));
        }
    }

    Ok(ShellCommitReport {
        total_commits,
        shell_commits,
    })
}

pub fn run_shell_commit_audit(
    since_days: Option<u32>,
    fail_on_count: Option<usize>,
) -> Result<ShellCommitReport, String> {
    let root = xtask::find_root_dir();
    run_shell_commit_audit_impl(&root, since_days, fail_on_count)
}
