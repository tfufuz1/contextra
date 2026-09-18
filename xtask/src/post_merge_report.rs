// xtask/src/post_merge_report.rs
//
// Post-Merge Verification & Reporting Subcommand
// Executes workspace verification (cargo check, cargo clippy, cargo test),
// records report summary into results/, and appends baseline stats to docs/unwrap_baseline_history.jsonl.
//
// NOTE REGARDING UNWRAP BASELINE HISTORY VS RATCHET GATE:
// `docs/unwrap_baseline_history.jsonl` is used strictly for historical reporting and trend tracking.
// The active CI enforcement gate (`check-unwrap-baseline` / `check-unwrap-ratchet`) evaluates live state
// against `.unwrap-baseline.json`. Modifying or appending to `docs/unwrap_baseline_history.jsonl` does not
// alter gate pass/fail criteria.

use chrono::Utc;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::check_unwrap_baseline_trend::{append_history_entry, load_tier1_crates};
use crate::check_unwrap_ratchet::scan_unwrap_expect_occurrences;

pub fn run_post_merge_report(root: &Path) -> bool {
    println!("=== Running xtask post-merge-report ===");

    let utc_now = Utc::now();
    let timestamp_str = utc_now.format("%Y%m%d_%H%M%S").to_string();
    let iso_ts = utc_now.to_rfc3339();

    let commit_sha = match Command::new("git").args(["rev-parse", "HEAD"]).output() {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    };

    let commit_short = if commit_sha.len() >= 7 {
        &commit_sha[..7]
    } else {
        &commit_sha
    };

    println!("Commit SHA: {} ({})", commit_sha, commit_short);
    println!("Timestamp:  {}", iso_ts);

    // 1. Run workspace cargo check
    println!("\n[1/3] Executing cargo check --workspace ...");
    let check_status = Command::new("cargo")
        .args(["check", "--workspace"])
        .status();
    let check_passed = matches!(check_status, Ok(st) if st.success());

    // 2. Run workspace cargo clippy
    println!("\n[2/3] Executing cargo clippy --workspace -- -D warnings ...");
    let clippy_status = Command::new("cargo")
        .args(["clippy", "--workspace", "--", "-D", "warnings"])
        .status();
    let clippy_passed = matches!(clippy_status, Ok(st) if st.success());

    // 3. Run workspace cargo test
    println!("\n[3/3] Executing cargo test --workspace ...");
    let test_status = Command::new("cargo").args(["test", "--workspace"]).status();
    let test_passed = matches!(test_status, Ok(st) if st.success());

    // Write summary report to results/
    let results_dir = root.join("results");
    if let Err(e) = fs::create_dir_all(&results_dir) {
        eprintln!("❌ Failed to create results directory: {}", e);
        return false;
    }

    let report_filename = format!("post_merge_{}_{}.summary", timestamp_str, commit_short);
    let report_path = results_dir.join(&report_filename);

    let summary_content = format!(
        "# MemFuse Post-Merge Report — {}\n\
         Commit: {}\n\
         Timestamp: {}\n\
         \n\
         CHECK:  {}\n\
         CLIPPY: {}\n\
         TEST:   {}\n",
        iso_ts,
        commit_sha,
        iso_ts,
        if check_passed { "PASS" } else { "FAIL" },
        if clippy_passed { "PASS" } else { "FAIL" },
        if test_passed { "PASS" } else { "FAIL" },
    );

    if let Err(e) = fs::write(&report_path, &summary_content) {
        eprintln!(
            "❌ Failed to write report file {}: {}",
            report_path.display(),
            e
        );
        return false;
    }

    println!(
        "\n📄 Post-merge summary report created at {}",
        report_path.display()
    );

    // 4. Update docs/unwrap_baseline_history.jsonl
    println!("\nUpdating docs/unwrap_baseline_history.jsonl ...");
    match scan_unwrap_expect_occurrences(root) {
        Ok(current_entries) => {
            let tier1_crates = load_tier1_crates(root);
            if let Err(e) = append_history_entry(root, &current_entries, &tier1_crates) {
                eprintln!("⚠️ Failed to append entry to unwrap history: {}", e);
            } else {
                println!(
                    "✅ History entry appended with {} unwrap/expect occurrences.",
                    current_entries.len()
                );
            }
        }
        Err(e) => {
            eprintln!(
                "⚠️ Failed to scan unwrap occurrences for history update: {}",
                e
            );
        }
    }

    let overall_success = check_passed && clippy_passed && test_passed;
    if overall_success {
        println!("\n✅ Post-merge verification PASSED.");
    } else {
        eprintln!("\n❌ Post-merge verification FAILED.");
    }

    overall_success
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_zero_panic_in_post_merge_report_source() {
        let source = include_str!("post_merge_report.rs");
        let unwrap_pattern = concat!(".", "unwrap()");
        let expect_pattern = concat!(".", "expect(");
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }
            assert!(
                !trimmed.contains(unwrap_pattern),
                "post_merge_report.rs must not contain unwrap calls"
            );
            assert!(
                !trimmed.contains(expect_pattern),
                "post_merge_report.rs must not contain expect calls"
            );
        }
    }
}
