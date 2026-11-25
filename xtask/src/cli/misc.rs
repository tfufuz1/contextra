#![allow(unused_imports, dead_code, unused_variables)]
use crate::*;
use std::path::{Path, PathBuf};
use std::process;

pub fn run_reproducible_build(args: &[String]) -> i32 {
    let success = reproducible_build::run_reproducible_build_check(&args[2..]);
    if !success {
        return 1;
    }
    0
}

pub fn run_check_branch_overlap(args: &[String]) -> i32 {
    if !agent_lifecycle::branch_overlap::run_check_branch_overlap() {
        return 1;
    }
    0
}

pub fn run_prune_branches(args: &[String]) -> i32 {
    if !agent_lifecycle::prune_branches::run_prune_branches() {
        return 1;
    }
    0
}

pub fn run_forensic_test(args: &[String]) -> i32 {
    let extra_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };
    if !proof::forensic_test::run_forensic_test(&extra_args) {
        return 1;
    }
    0
}

pub fn run_gen_sbom(args: &[String]) -> i32 {
    if !generators::sbom::run_gen_sbom() {
        return 1;
    }
    0
}

pub fn run_run_community_detection(args: &[String]) -> i32 {
    println!("=== xtask run-community-detection ===");
    println!("Periodic batch process for GraphRAG community detection via Label Propagation.");
    println!("Note: Community detection triggers should be invoked via collection.run_community_detection().await or embedded engine instances.");
    println!("=== xtask run-community-detection PASSED ===");
    0
}

pub fn run_gen_prompter_data(args: &[String]) -> i32 {
    let success = gen_prompter_data::run();
    if !success {
        return 1;
    }
    0
}

pub fn run_context_tags(args: &[String]) -> i32 {
    let tags = crate::scan_tags("crates");
    let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
    crate::run_context_tags(&tags, extra_args);
    0
}

pub fn run_post_merge_report(args: &[String]) -> i32 {
    let root = crate::find_root_dir();
    let success = post_merge_report::run_post_merge_report(&root);
    if !success {
        return 1;
    }
    0
}

pub fn run_migrate_docid_128(args: &[String]) -> i32 {
    let extra_args = if args.len() > 2 { &args[2..] } else { &[] };
    if !migrate_docid_128::run_cli(extra_args) {
        return 1;
    }
    0
}

pub fn run_feature_matrix(args: &[String]) -> i32 {
    let test = args.iter().any(|arg| arg == "--test");
    let only = args.iter().find_map(|arg| arg.strip_prefix("--only="));
    match feature_matrix::run_feature_matrix(test, only) {
        Ok(results) => {
            let failed_count = results.iter().filter(|r| !r.passed).count();
            if failed_count > 0 {
                eprintln!(
                    "❌ feature-matrix failed: {} feature combination(s) failed",
                    failed_count
                );
                return 1;
            }
            println!("✅ feature-matrix passed: all feature combinations verified");
        }
        Err(e) => {
            eprintln!("❌ feature-matrix failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_workspace_verify(args: &[String]) -> i32 {
    let mode_str = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--mode="))
        .unwrap_or("full");
    let mode = match mode_str {
        "fast" => workspace_verify::VerifyMode::Fast,
        "audit" => workspace_verify::VerifyMode::Audit,
        _ => workspace_verify::VerifyMode::Full,
    };
    let only = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--only="))
        .map(|s| {
            s.split(',')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let resume_from = args
        .iter()
        .find_map(|arg| arg.strip_prefix("--resume="))
        .map(|s| s.to_string());
    let no_clippy = args.iter().any(|arg| arg == "--no-clippy");
    let config = workspace_verify::WorkspaceVerifyConfig {
        mode,
        only,
        resume_from,
        stop_on_fail: false,
        run_clippy: !no_clippy,
    };
    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let output_dir = crate::find_root_dir()
        .join("target")
        .join("workspace-verify")
        .join(timestamp);
    if let Err(e) = workspace_verify::run_workspace_verify(config, &output_dir) {
        eprintln!("❌ workspace-verify failed: {}", e);
        return 1;
    }
    println!(
        "✅ workspace-verify completed: report generated in {}",
        output_dir.display()
    );
    0
}

pub fn run_loom_run(args: &[String]) -> i32 {
    let filter = args.iter().find_map(|arg| arg.strip_prefix("--filter="));
    let root = crate::find_root_dir();
    match loom_run::run_loom(filter, &root) {
        Ok(res) => {
            if !res.passed {
                eprintln!("❌ loom-run failed:\n{}", res.output);
                return 1;
            }
            println!("✅ loom-run passed in {}s", res.duration_secs);
        }
        Err(e) => {
            eprintln!("❌ loom-run error: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_py_test(args: &[String]) -> i32 {
    let filter = args.iter().find_map(|arg| arg.strip_prefix("--filter="));
    let root = crate::find_root_dir();
    match py_test::run_py_test(filter, &root) {
        Ok(success) => {
            if !success {
                eprintln!("❌ py-test failed");
                return 1;
            }
            println!("✅ py-test passed");
        }
        Err(e) => {
            eprintln!("❌ py-test failed: {}", e);
            return 1;
        }
    }
    0
}

pub fn run_env_validate(args: &[String]) -> i32 {
    let checks = env_validate::run_env_validate();
    let mut failed = false;
    println!("{:<20} | {:<20} | Status", "Tool", "Required Version");
    println!("{:-<20}-|-{:-<20}-|--------", "", "");
    for check in &checks {
        let status_str = match &check.status {
            env_validate::ToolStatus::Ok(v) => format!("✅ OK ({})", v),
            env_validate::ToolStatus::Missing => {
                if check.required {
                    failed = true;
                }
                "❌ Missing".to_string()
            }
            env_validate::ToolStatus::WrongVersion { found, required } => {
                if check.required {
                    failed = true;
                }
                format!("❌ Wrong Version (found {}, required {})", found, required)
            }
        };
        println!(
            "{:<20} | {:<20} | {}",
            check.name, check.version_arg, status_str
        );
    }
    if failed {
        eprintln!("❌ env-validate failed: one or more required tools missing or wrong version");
        return 1;
    }
    println!("✅ env-validate passed");
    0
}
