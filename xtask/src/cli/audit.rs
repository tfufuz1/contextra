#![allow(unused_imports, dead_code, unused_variables)]
use crate::*;
use std::path::{Path, PathBuf};
use std::process;

pub fn run_debt_audit(args: &[String]) -> i32 {
    if let Err(e) = gates::debt_audit::run_debt_audit() {
        eprintln!("{}", e);
        return 1;
    }
    0
}

pub fn run_init_audit_fix(args: &[String]) -> i32 {
    let hash = args.get(2).map(|s| s.as_str()).unwrap_or("HEAD");
    if let Err(e) = init_audit_fix::run_init_audit_fix(hash) {
        eprintln!("❌ init-audit-fix failed: {}", e);
        return 1;
    }
    0
}

pub fn run_shell_commit_audit(args: &[String]) -> i32 {
    let since_days = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--since="))
        .and_then(|val| val.trim_end_matches('d').parse::<u32>().ok());
    let fail_on_count = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--fail-on-count="))
        .and_then(|val| val.parse::<usize>().ok());
    match shell_commit_audit::run_shell_commit_audit(since_days, fail_on_count) {
        Ok(report) => {
            let substantial = report
                .shell_commits
                .iter()
                .filter(|c| c.has_substantial_diff)
                .count();
            println!(
                "✅ shell-commit-audit passed: total_commits={}, shell_commits={}, substantial={}",
                report.total_commits,
                report.shell_commits.len(),
                substantial
            );
        }
        Err(e) => {
            eprintln!("❌ shell-commit-audit failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_audit_integrity_check(args: &[String]) -> i32 {
    let since_days = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--since="))
        .and_then(|val| val.trim_end_matches('d').parse::<u32>().ok());
    let fail_on_contradicted = args.iter().any(|arg| arg == "--fail-on-contradicted");
    match audit_integrity_check::run_audit_integrity_check(since_days, fail_on_contradicted) {
        Ok(gaps) => {
            let suspicious = gaps
                .iter()
                .filter(|g| {
                    matches!(
                        g.severity,
                        audit_integrity_check::GapSeverity::Suspicious(_)
                    )
                })
                .count();
            let contradicted = gaps
                .iter()
                .filter(|g| {
                    matches!(
                        g.severity,
                        audit_integrity_check::GapSeverity::Contradicted(_)
                    )
                })
                .count();
            println!(
                "✅ audit-integrity-check passed: {} suspicious, {} contradicted gaps found",
                suspicious, contradicted
            );
        }
        Err(e) => {
            eprintln!("❌ audit-integrity-check failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_hotspot_report(args: &[String]) -> i32 {
    let since_days = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--since="))
        .and_then(|val| val.trim_end_matches('d').parse::<u32>().ok())
        .unwrap_or(30);
    let top_n = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--top="))
        .and_then(|val| val.parse::<usize>().ok())
        .unwrap_or(10);
    let fail_on_critical = args.iter().any(|arg| arg == "--fail-on-critical");
    match hotspot_report::run_hotspot_report(since_days, top_n, fail_on_critical) {
        Ok(hotspots) => {
            println!(
                "{:<50} | {:<10} | {:<16} | Risk",
                "File", "Changes", "Distinct Authors"
            );
            println!("{:-<50}-|-{:-<10}-|-{:-<16}-|-------", "", "", "");
            for h in &hotspots {
                let risk_str = match h.risk_level {
                    hotspot_report::HotspotRisk::Normal => "Normal",
                    hotspot_report::HotspotRisk::Elevated => "Elevated 🟡",
                    hotspot_report::HotspotRisk::Critical => "Critical 🔴",
                };
                println!(
                    "{:<50} | {:<10} | {:<16} | {}",
                    h.file, h.changes_in_window, h.distinct_authors, risk_str
                );
            }
        }
        Err(e) => {
            eprintln!("❌ hotspot-report failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_tag_health(args: &[String]) -> i32 {
    let threshold_days = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--threshold-days="))
        .and_then(|val| val.parse::<i64>().ok())
        .unwrap_or(30);
    match tag_health::run_tag_health(threshold_days) {
        Ok(findings) => {
            println!("✅ tag-health check complete: {} findings", findings.len());
            for f in &findings {
                println!("  {}:{} (tag: {:?})", f.file, f.line, f.tag_id);
            }
        }
        Err(e) => {
            eprintln!("❌ tag-health failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_commit_health(args: &[String]) -> i32 {
    let since_days = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--since="))
        .and_then(|val| val.trim_end_matches('d').parse::<u32>().ok())
        .unwrap_or(30);
    let output_path = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--output="))
        .map(PathBuf::from);
    match commit_health::run_commit_health(since_days, output_path.as_deref()) {
        Ok(report_md) => {
            println!("{}", report_md);
        }
        Err(e) => {
            eprintln!("❌ commit-health failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_panic_inventory(args: &[String]) -> i32 {
    let crate_filter = args.iter().find_map(|arg| arg.strip_prefix("--crate="));
    let exclude_tests = args.iter().any(|arg| arg == "--exclude-tests");
    let strict = args.iter().any(|arg| arg == "--strict");
    match panic_inventory::run_panic_inventory(crate_filter, exclude_tests, strict) {
        Ok(panics) => {
            println!(
                "✅ panic-inventory completed: {} panic entries found",
                panics.len()
            );
        }
        Err(e) => {
            eprintln!("❌ panic-inventory failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_security_scan(args: &[String]) -> i32 {
    let crate_filter = args.iter().find_map(|arg| arg.strip_prefix("--crate="));
    let pattern_str = args.iter().find_map(|arg| arg.strip_prefix("--pattern="));
    let pattern_filter = match pattern_str {
        Some("shell-interpolation" | "ShellInterpolation") => {
            Some(security_scan::SecurityPattern::ShellInterpolation)
        }
        Some("std-fs-in-async" | "StdFsInAsync") => {
            Some(security_scan::SecurityPattern::StdFsInAsync)
        }
        Some("hardcoded-secret" | "HardcodedSecret") => {
            Some(security_scan::SecurityPattern::HardcodedSecret)
        }
        Some(p) => {
            eprintln!("❌ Unknown --pattern option: {}", p);
            return 1;
        }
        None => None,
    };
    match security_scan::run_security_scan(crate_filter, pattern_filter) {
        Ok(findings) => {
            if !findings.is_empty() {
                eprintln!(
                    "❌ security-scan failed: {} security findings found",
                    findings.len()
                );
                for f in &findings {
                    eprintln!(
                        "  {}:{} [{}] {}",
                        f.file,
                        f.line,
                        f.pattern.as_str(),
                        f.context
                    );
                }
                return 1;
            }
            println!("✅ security-scan passed: zero security findings");
        }
        Err(e) => {
            eprintln!("❌ security-scan failed: {}", e);
            return 1;
        }
    }
    0
}
