// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Formatted execution table printing and summary writing for `cargo xtask verify`.

use crate::verify::evidence::{StepEvidence, StepStatus, VerificationSummary};
use std::fs;
use std::path::Path;

pub fn print_summary_and_save(run_dir: &Path, summary: &VerificationSummary) {
    let summary_json_path = run_dir.join("summary.json");
    if let Ok(json_str) = serde_json::to_string_pretty(summary) {
        let _ = fs::write(summary_json_path, json_str);
    }

    println!("\n==========================================================================");
    println!(" Contextra Verification Evidence Summary");
    println!(
        " Run ID: {}   Tier: {}   Status: {}",
        summary.run_id, summary.tier, summary.overall_status
    );
    println!("==========================================================================");
    println!(
        "{:<22} | {:<20} | {:<10} | {:<10} | {:<12}",
        "Step ID", "Status", "Exit Code", "Duration", "Log SHA-256 (prefix)"
    );
    println!("--------------------------------------------------------------------------");

    for step in &summary.steps {
        let status_colored = match step.status {
            StepStatus::Passed => format!("\x1b[32m{}\x1b[0m", step.status),
            StepStatus::Failed => format!("\x1b[31m{}\x1b[0m", step.status),
            StepStatus::SkippedToolMissing => format!("\x1b[33m{}\x1b[0m", step.status),
            StepStatus::SkippedByFlag => format!("\x1b[31m{}\x1b[0m", step.status),
            StepStatus::Error => format!("\x1b[35m{}\x1b[0m", step.status),
        };

        let sha_short = if step.log_sha256.len() >= 12 {
            &step.log_sha256[..12]
        } else {
            &step.log_sha256
        };

        println!(
            "{:<22} | {:<20} | {:<10} | {:<8}ms | {:<12}",
            step.step_id, status_colored, step.exit_code, step.duration_ms, sha_short
        );
    }
    println!("==========================================================================\n");
}
