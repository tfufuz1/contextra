#![allow(unused_imports, dead_code)]
use std::env;
use std::process;
use std::time::Instant;
use xtask::cli::COMMAND_DISPATCH_TABLE;

fn main() {
    let raw_args: Vec<String> = env::args().collect();
    let timing_requested = env::var("XTASK_TIMINGS").map(|v| v == "1").unwrap_or(false)
        || raw_args.iter().any(|a| a == "--timings");
    let args: Vec<String> = raw_args.into_iter().filter(|a| a != "--timings").collect();
    let subcommand = args.get(1).map(|s| s.as_str()).unwrap_or("sync-docs");
    let extra_args = if args.len() >= 2 { &args[2..] } else { &[] };
    let start_time = Instant::now();

    if let Some(code) = xtask::harness::dispatch_with_builtin(subcommand, extra_args) {
        if timing_requested {
            eprintln!(
                "[timings] Command '{}' took {:?}",
                subcommand,
                start_time.elapsed()
            );
        }
        process::exit(code);
    }

    for (cmd_name, handler) in COMMAND_DISPATCH_TABLE {
        if *cmd_name == subcommand {
            let code = handler(&args);
            if timing_requested {
                eprintln!(
                    "[timings] Command '{}' took {:?}",
                    subcommand,
                    start_time.elapsed()
                );
            }
            process::exit(code);
        }
    }

    eprintln!("Unknown xtask command: {}", subcommand);
    let mut available_cmds: Vec<&str> = COMMAND_DISPATCH_TABLE
        .iter()
        .map(|(name, _)| *name)
        .collect();
    available_cmds.sort_unstable();
    available_cmds.dedup();
    eprintln!("Available commands: {}", available_cmds.join(", "));
    process::exit(1);
}

// "--"=> "all"=> "ann-sift1m"=> "audit"=> "audit-integrity-check"=> "beir"=> "bench-compile"=>
// "bench-download"=> "bench-gate"=> "bench-trend"=> "check-action-pinning"=> "check-adr-deadlines"=>
// "check-agents-freshness"=> "check-agents-integrity"=> "check-audit-duplication"=>
// "check-audit-tool-evidence"=> "check-audit-verdict-independence"=> "check-bandit-latency-budget"=>
// "check-branch-overlap"=> "check-commit-diff-integrity"=> "check-commit-messages"=> "check-compile"=>
// "check-consistency"=> "check-coverage-gate"=> "check-crate-references"=> "check-dag"=>
// "check-doc-references"=> "check-duplicate-core-primitives"=> "check-duplicate-intent"=>
// "check-duplicate-symbols"=> "check-duplicate-symbols-cross-file"=> "check-fbs-drift"=>
// "check-ffi-panic-boundary"=> "check-flatbuffers-drift"=> "check-jules-context-freshness"=>
// "check-manifest-completeness"=> "check-marker-drift"=> "check-max-results-unbound"=>
// "check-module-reachability"=> "check-mutation-score-gate"=> "check-nan-hot-loop"=>
// "check-orphan-modules"=> "check-phantom-files"=> "check-placeholder-refs"=> "check-recall-stability"=>
// "check-result-dropped-io"=> "check-review-coverage"=> "check-ring-capabilities-consistency"=>
// "check-ring-layering"=> "check-ring-layering-full"=> "check-ring0-async-purity"=> "check-stale-tags"=>
// "check-toc-integrity"=> "check-toctou-defaults"=> "check-type-registry"=> "check-unsafe-islands"=>
// "check-veto-deadlines"=> "check-vetoes"=> "check-workflow-commands"=> "claim"=> "commit-health"=>
// "consolidate-adrs"=> "context-pack"=> "context-tags"=> "crate-context"=> "debt-audit"=> "deprecated"=>
// "env-validate"=> "experimental"=> "fast"=> "feature-matrix"=> "forensic-test"=> "gate-check"=>
// "gen-arch-docs"=> "gen-feature-catalog"=> "gen-prompter-data"=> "gen-sbom"=> "generate-adr"=>
// "generate-diagnostics"=> "generate-markers"=> "hardcoded-secret"=> "hotspot-report"=> "init-audit-fix"=>
// "json"=> "jules-preflight"=> "jules-submit-gate"=> "lint-unsafe-slices"=> "loom-run"=> "migrate-docid-128"=>
// "mutation-score-record"=> "onnx-test-model"=> "panic-inventory"=> "post-merge-report"=> "pre-push"=>
// "prune-branches"=> "py-test"=> "regenerate-flatbuffers"=> "reproducible-build"=> "reproducible-build-check"=>
// "run-community-detection"=> "security-scan"=> "session-init"=> "shell-commit-audit"=> "shell-interpolation"=>
// "stable"=> "std-fs-in-async"=> "sync-docs"=> "tag-health"=> "validate-pr-checklist"=> "validate-tags"=> "workspace-verify"=>
