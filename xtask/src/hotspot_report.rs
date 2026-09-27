use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotspotRisk {
    Normal,
    Elevated,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileHotspot {
    pub file: String,
    pub changes_in_window: usize,
    pub distinct_authors: usize,
    pub risk_level: HotspotRisk,
}

pub fn run_hotspot_report_impl(
    root: &Path,
    since_days: u32,
    top_n: usize,
    fail_on_critical: bool,
) -> Result<Vec<FileHotspot>, String> {
    let since_arg = format!("--since={} days ago", since_days);

    // Single git log call for crates/**/*.rs
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "log",
            &since_arg,
            "--name-only",
            "--format=%H",
            "--",
            "crates/**/*.rs",
        ])
        .output()
        .map_err(|e| format!("Failed to execute git log for hotspots: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut file_counts: HashMap<String, usize> = HashMap::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Skip commit hash lines (40 hex characters)
        if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }

        // It's a file path
        let normalized = trimmed.replace('\\', "/");
        if normalized.starts_with("crates/") && normalized.ends_with(".rs") {
            *file_counts.entry(normalized).or_insert(0) += 1;
        }
    }

    let session_regex = Regex::new(r"SESSION:\s*([0-9a-fA-F]+)")
        .map_err(|e| format!("Failed to compile session regex: {}", e))?;

    let mut hotspots = Vec::new();

    for (file_rel, changes_in_window) in file_counts {
        let full_path = root.join(&file_rel);

        let mut session_hashes = HashSet::new();
        if full_path.exists() {
            if let Ok(content) = fs::read_to_string(&full_path) {
                for caps in session_regex.captures_iter(&content) {
                    if let Some(mat) = caps.get(1) {
                        session_hashes.insert(mat.as_str().to_lowercase());
                    }
                }
            }
        }

        let distinct_authors = session_hashes.len();

        let risk_level = if changes_in_window > 50 {
            HotspotRisk::Critical
        } else if changes_in_window > 20 {
            HotspotRisk::Elevated
        } else {
            HotspotRisk::Normal
        };

        hotspots.push(FileHotspot {
            file: file_rel,
            changes_in_window,
            distinct_authors,
            risk_level,
        });
    }

    hotspots.sort_by(|a, b| {
        b.changes_in_window
            .cmp(&a.changes_in_window)
            .then_with(|| a.file.cmp(&b.file))
    });

    if top_n > 0 && hotspots.len() > top_n {
        hotspots.truncate(top_n);
    }

    if fail_on_critical
        && hotspots
            .iter()
            .any(|h| h.risk_level == HotspotRisk::Critical)
    {
        return Err(
            "Hotspot report failed: detected critical hotspot with >50 changes in window"
                .to_string(),
        );
    }

    Ok(hotspots)
}

pub fn run_hotspot_report(
    since_days: u32,
    top_n: usize,
    fail_on_critical: bool,
) -> Result<Vec<FileHotspot>, String> {
    let root = xtask::find_root_dir();
    run_hotspot_report_impl(&root, since_days, top_n, fail_on_critical)
}
