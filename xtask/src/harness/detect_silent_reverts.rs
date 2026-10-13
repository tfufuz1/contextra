//! Harness Modul zur Erkennung von stiller Rückgängigmachung von Code (detect-silent-reverts).

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const MIN_LINE_LEN: usize = 25;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub author: String,
    pub ts: i64,
    pub subject: String,
    pub pr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: String,
    pub clobberer: String,
    pub clobberer_subject: String,
    pub victim: String,
    pub victim_subject: String,
    pub file: String,
    pub lost_lines: usize,
    pub added_lines: usize,
    pub hours_apart: f64,
    pub dangling: Vec<String>,
    pub declared_lost: Vec<String>,
    pub sample: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectSilentRevertsReport {
    pub commits: usize,
    pub findings: Vec<Finding>,
}

fn run_git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute git: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(err.trim().to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn norm_line(line: &str) -> String {
    let mut s = line.trim().to_string();
    while s.contains("  ") {
        s = s.replace("  ", " ");
    }
    s
}

fn is_interesting(line: &str) -> bool {
    let n = norm_line(line);
    if n.len() < MIN_LINE_LEN {
        return false;
    }
    if n.starts_with("//")
        || n.starts_with("///")
        || n.starts_with('*')
        || n.starts_with("/*")
        || n.starts_with("#[")
        || n.starts_with("use ")
    {
        return false;
    }
    true
}

fn short_str(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(3)).collect();
        format!("{}...", truncated)
    }
}

fn parse_log(
    repo: &Path,
    since: &str,
    range: Option<&str>,
) -> Result<(Vec<CommitInfo>, Vec<HashMap<String, (Vec<String>, Vec<String>)>>), String> {
    let pr_re = Regex::new(r"\(\#(\d+)\)").map_err(|e| e.to_string())?;
    let fmt = "@@C\t%H\t%an\t%at\t%s";

    let mut git_args = vec![
        "log",
        "--no-merges",
        "--reverse",
        "-p",
        "--unified=0",
        "--no-color",
    ];

    let fmt_arg;
    let range_arg;
    let since_arg;

    if let Some(r) = range {
        range_arg = r.to_string();
        git_args.push(&range_arg);
    } else {
        since_arg = format!("--since={}", since);
        git_args.push(&since_arg);
    }

    fmt_arg = format!("--format={}", fmt);
    git_args.push(&fmt_arg);
    git_args.extend_from_slice(&["--", "*.rs"]);

    let out = run_git(repo, &git_args)?;

    let mut commits: Vec<CommitInfo> = Vec::new();
    let mut per_commit: Vec<HashMap<String, (Vec<String>, Vec<String>)>> = Vec::new();

    let mut cur_file: Option<String> = None;

    for line in out.lines() {
        if let Some(stripped) = line.strip_prefix("@@C\t") {
            let parts: Vec<&str> = stripped.splitn(4, '\t').collect();
            if parts.len() == 4 {
                let sha = parts[0].to_string();
                let author = parts[1].to_string();
                let ts: i64 = parts[2].parse().unwrap_or(0);
                let subject = parts[3].to_string();
                let pr = pr_re
                    .captures(&subject)
                    .map(|c| c[1].to_string())
                    .unwrap_or_default();

                commits.push(CommitInfo {
                    sha,
                    author,
                    ts,
                    subject,
                    pr,
                });
                per_commit.push(HashMap::new());
                cur_file = None;
            }
        } else if let Some(last_commit) = per_commit.last_mut() {
            if let Some(file_path) = line.strip_prefix("+++ b/") {
                cur_file = Some(file_path.to_string());
                last_commit.entry(file_path.to_string()).or_default();
            } else if line.starts_with("+++ ") {
                cur_file = None;
            } else if let Some(ref cf) = cur_file {
                if line.starts_with('+') && !line.starts_with("+++") {
                    last_commit.get_mut(cf).unwrap().0.push(line[1..].to_string());
                } else if line.starts_with('-') && !line.starts_with("---") {
                    last_commit.get_mut(cf).unwrap().1.push(line[1..].to_string());
                }
            }
        }
    }

    Ok((commits, per_commit))
}

fn symbols_in(lines: &[String], decl_re: &Regex) -> std::collections::HashSet<String> {
    let mut s = std::collections::HashSet::new();
    for l in lines {
        for cap in decl_re.captures_iter(l) {
            if let Some(m) = cap.get(2) {
                s.insert(m.as_str().to_string());
            }
        }
    }
    s
}

fn worth_checking(n: &str) -> bool {
    let upper_pascal = Regex::new(r"^[A-Z][A-Za-z0-9]{3,}$").unwrap();
    let upper_snake = Regex::new(r"^[A-Z][A-Z0-9_]{3,}$").unwrap();

    upper_pascal.is_match(n)
        || upper_snake.is_match(n)
        || (n.contains('_') && n.chars().count() >= 8)
}

fn strip_comments_and_strings(code_line: &str) -> String {
    let str_re = Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap();
    let stripped = str_re.replace_all(code_line, "\"\"");
    if let Some(idx) = stripped.find("//") {
        stripped[..idx].to_string()
    } else {
        stripped.to_string()
    }
}

fn head_state(repo: &Path, names: &std::collections::HashSet<String>) -> HashMap<String, (bool, bool)> {
    let mut res = HashMap::new();

    let mut sorted_names: Vec<&String> = names.iter().collect();
    sorted_names.sort();

    for n in sorted_names {
        if !worth_checking(n) {
            continue;
        }

        let grep_raw = run_git(repo, &["grep", "-n", "-w", "-I", "-e", n, "HEAD", "--", "*.rs"])
            .unwrap_or_default();

        let mut used = false;
        for line in grep_raw.lines() {
            let parts: Vec<&str> = line.splitn(4, ':').collect();
            if parts.len() == 4 {
                let code_part = strip_comments_and_strings(parts[3]);
                let word_re = Regex::new(&format!(r"\b{}\b", regex::escape(n))).unwrap();
                if word_re.is_match(&code_part) {
                    used = true;
                    break;
                }
            }
        }

        if !used {
            res.insert(n.clone(), (false, false));
            continue;
        }

        let decl_pat = format!(r"(fn|struct|enum|trait|type|const|static|union)\s+{}\b", regex::escape(n));
        let decl_raw = run_git(repo, &["grep", "-I", "-l", "-E", &decl_pat, "HEAD", "--", "*.rs"])
            .unwrap_or_default();

        let declared = !decl_raw.trim().is_empty();
        res.insert(n.clone(), (true, declared));
    }

    res
}

pub fn run_detect_silent_reverts(args: &[String]) -> i32 {
    let decl_re = Regex::new(
        r"\b(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?(fn|struct|enum|trait|type|const|static|union)\s+([A-Za-z_]\w*)"
    ).unwrap();

    let mut repo_root = PathBuf::from(".");
    let mut since = "14 days ago".to_string();
    let mut range: Option<String> = None;
    let mut lookback: usize = 8;
    let mut min_lost: usize = 15;
    let mut window_h: f64 = 48.0;
    let mut fail_on = "kritisch".to_string();
    let mut format_opt = "text".to_string();
    let mut out_dir: Option<PathBuf> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo-root" => {
                if i + 1 < args.len() {
                    repo_root = PathBuf::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--since" => {
                if i + 1 < args.len() {
                    since = args[i + 1].clone();
                    i += 1;
                }
            }
            "--range" => {
                if i + 1 < args.len() {
                    range = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--lookback" => {
                if i + 1 < args.len() {
                    lookback = args[i + 1].parse().unwrap_or(8);
                    i += 1;
                }
            }
            "--min-lost" => {
                if i + 1 < args.len() {
                    min_lost = args[i + 1].parse().unwrap_or(15);
                    i += 1;
                }
            }
            "--window-h" => {
                if i + 1 < args.len() {
                    window_h = args[i + 1].parse().unwrap_or(48.0);
                    i += 1;
                }
            }
            "--fail-on" => {
                if i + 1 < args.len() {
                    fail_on = args[i + 1].to_lowercase();
                    i += 1;
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format_opt = args[i + 1].to_lowercase();
                    i += 1;
                }
            }
            "--out-dir" => {
                if i + 1 < args.len() {
                    out_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let (commits, per_commit) = match parse_log(&repo_root, &since, range.as_deref()) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("[error] detect-silent-reverts: {}", e);
            return 1;
        }
    };

    let mut recent: HashMap<String, VecDeque<(usize, std::collections::HashSet<String>)>> =
        HashMap::new();

    struct Candidate {
        ci: usize,
        pci: usize,
        file: String,
        hit: Vec<String>,
        lost_n: usize,
        add_n: usize,
        hrs: f64,
    }

    let mut cand: Vec<Candidate> = Vec::new();

    for (ci, files) in per_commit.iter().enumerate() {
        let mut all_added = std::collections::HashSet::new();
        for (_f, (a, _r)) in files {
            for l in a {
                if is_interesting(l) {
                    all_added.insert(norm_line(l));
                }
            }
        }

        for (f, (added, removed)) in files {
            let lost: Vec<String> = removed
                .iter()
                .filter(|l| is_interesting(l))
                .map(|l| norm_line(l))
                .filter(|n| !all_added.contains(n))
                .collect();

            if !lost.is_empty() {
                let lost_set: std::collections::HashSet<String> = lost.iter().cloned().collect();
                if let Some(recent_queue) = recent.get(f) {
                    for (pci, padded) in recent_queue {
                        if *pci == ci {
                            continue;
                        }
                        let hit: Vec<String> = lost_set.intersection(padded).cloned().collect();
                        if hit.len() >= 3 && commits[*pci].pr != commits[ci].pr {
                            let hrs = (commits[ci].ts - commits[*pci].ts) as f64 / 3600.0;
                            cand.push(Candidate {
                                ci,
                                pci: *pci,
                                file: f.clone(),
                                hit,
                                lost_n: lost.len(),
                                add_n: added.len(),
                                hrs,
                            });
                        }
                    }
                }
            }

            let queue = recent.entry(f.clone()).or_insert_with(|| VecDeque::with_capacity(lookback));
            if queue.len() >= lookback {
                queue.pop_front();
            }
            let added_set: std::collections::HashSet<String> = added
                .iter()
                .filter(|l| is_interesting(l))
                .map(|l| norm_line(l))
                .collect();
            queue.push_back((ci, added_set));
        }
    }

    let mut all_names = std::collections::HashSet::new();
    for c in &cand {
        all_names.extend(symbols_in(&c.hit, &decl_re));
    }

    let state = head_state(&repo_root, &all_names);

    let mut findings: Vec<Finding> = Vec::new();
    for c in cand {
        let mut declared: Vec<String> = symbols_in(&c.hit, &decl_re).into_iter().collect();
        declared.sort();

        let mut dangling: Vec<String> = Vec::new();
        for n in &declared {
            if state.get(n) == Some(&(true, false)) {
                dangling.push(n.clone());
            }
        }

        let severity = if !dangling.is_empty() {
            "KRITISCH".to_string()
        } else if c.hit.len() >= min_lost && c.lost_n > c.add_n && c.hrs.abs() <= window_h {
            "HOCH".to_string()
        } else {
            "INFO".to_string()
        };

        let sample = c.hit.iter().take(3).map(|h| short_str(h, 110)).collect();

        findings.push(Finding {
            severity,
            clobberer: commits[c.ci].sha[..8.min(commits[c.ci].sha.len())].to_string(),
            clobberer_subject: commits[c.ci].subject.clone(),
            victim: commits[c.pci].sha[..8.min(commits[c.pci].sha.len())].to_string(),
            victim_subject: commits[c.pci].subject.clone(),
            file: c.file,
            lost_lines: c.hit.len(),
            added_lines: c.add_n,
            hours_apart: (c.hrs * 10.0).round() / 10.0,
            dangling,
            declared_lost: declared.into_iter().take(8).collect(),
            sample,
        });
    }

    findings.sort_by(|a, b| {
        let rank = |sev: &str| match sev {
            "KRITISCH" => 0,
            "HOCH" => 1,
            _ => 2,
        };
        rank(&a.severity)
            .cmp(&rank(&b.severity))
            .then_with(|| b.lost_lines.cmp(&a.lost_lines))
    });

    let kritisch_count = findings.iter().filter(|f| f.severity == "KRITISCH").count();
    let hoch_count = findings.iter().filter(|f| f.severity == "HOCH").count();
    let info_count = findings.iter().filter(|f| f.severity == "INFO").count();

    if format_opt == "json" {
        let report = DetectSilentRevertsReport {
            commits: commits.len(),
            findings: findings.clone(),
        };
        let json_str = serde_json::to_string_pretty(&report).unwrap_or_default();
        if let Some(ref od) = out_dir {
            let _ = fs::create_dir_all(od);
            let _ = fs::write(od.join("detect_silent_reverts.json"), &json_str);
        }
        println!("{}", json_str);
    } else {
        let mut md = String::new();
        md.push_str("# Audit: Stille Rückgängigmachung / Merge-Überschreibung\n\n");
        md.push_str(&format!(
            "Geprüft: **{}** Commits ({})\n\n",
            commits.len(),
            range.as_deref().unwrap_or(&since)
        ));
        md.push_str(&format!(
            "- 🔴 KRITISCH (hängendes Symbol am HEAD): **{}**\n- 🟠 HOCH (großer Verlust fremden Codes, kurzer Abstand): **{}**\n- ⚪ INFO: **{}**\n\n",
            kritisch_count, hoch_count, info_count
        ));

        for (sev, icon) in [("KRITISCH", "🔴"), ("HOCH", "🟠")] {
            let items: Vec<&Finding> = findings.iter().filter(|f| f.severity == sev).collect();
            if items.is_empty() {
                continue;
            }
            md.push_str(&format!("## {} {}\n\n", icon, sev));
            md.push_str("| Überschreibender Commit | Überschriebener Commit | Datei | verlorene Zeilen | Abstand | Hängende Symbole |\n|---|---|---|---|---|---|\n");
            for f in items {
                let dang = if f.dangling.is_empty() {
                    "—".to_string()
                } else {
                    f.dangling
                        .iter()
                        .map(|d| format!("`{}`", d))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                md.push_str(&format!(
                    "| `{}` {} | `{}` {} | `{}` | {} | {} h | {} |\n",
                    f.clobberer,
                    short_str(&f.clobberer_subject, 55),
                    f.victim,
                    short_str(&f.victim_subject, 55),
                    f.file,
                    f.lost_lines,
                    f.hours_apart,
                    dang
                ));
            }
            md.push_str("\n");
        }

        if info_count > 0 {
            md.push_str(&format!(
                "*{} INFO-Treffer (überwiegend legitime Umschreibungen) nur im JSON-Format.*\n\n",
                info_count
            ));
        }

        if let Some(ref od) = out_dir {
            let _ = fs::create_dir_all(od);
            let _ = fs::write(od.join("detect_silent_reverts.md"), &md);
        }
        println!("{}", md);
    }

    if fail_on == "none" {
        return 0;
    }

    let has_fail = if fail_on == "kritisch" {
        kritisch_count > 0
    } else {
        kritisch_count > 0 || hoch_count > 0
    };

    if has_fail {
        2
    } else {
        0
    }
}
