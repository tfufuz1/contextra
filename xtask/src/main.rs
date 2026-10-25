use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command> [args...]");
        std::process::exit(1);
    }

    let cmd = &args[1];
    let cmd_args = &args[2..];

    if cmd == "check" || cmd == "verify" || cmd == "gate" || cmd == "explain" {
        let code = xtask::core::cmd::dispatch(cmd, cmd_args);
        std::process::exit(code);
    }

    let res = xtask::legacy::arch::run_arch_command(cmd, cmd_args)
        .or_else(|_| xtask::legacy::code::run_code_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::gov::run_gov_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::ratchet_supply::run_ratchet_supply_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::session::run_session_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::docs::run_docs_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::tags::run_tags_command(cmd, cmd_args))
        .or_else(|_| xtask::legacy::heavy::run_heavy_command(cmd, cmd_args));

    match res {
        Ok(()) => std::process::exit(0),
        Err(_) => {
            eprintln!("Unknown xtask command: {}", cmd);
            eprintln!("Available commands: audit-integrity-check, bench-compile, bench-download, bench-gate, bench-trend, check-action-pinning, check-adr-deadlines, check-agents-freshness, check-agents-integrity, check-audit-duplication, check-audit-tool-evidence, check-audit-verdict-independence, check-bandit-latency-budget, check-branch-overlap, check-commit-diff-integrity, check-commit-messages, check-compile, check-consistency, check-coverage-gate, check-crate-references, check-dag, check-doc-references, check-duplicate-core-primitives, check-duplicate-intent, check-duplicate-symbols, check-duplicate-symbols-cross-file, check-ffi-panic-boundary, check-flatbuffers-drift, check-jules-context-freshness, check-manifest-completeness, check-marker-drift, check-max-results-unbound, check-module-reachability, check-mutation-score-gate, check-nan-hot-loop, check-phantom-files, check-placeholder-refs, check-recall-stability, check-result-dropped-io, check-review-coverage, check-ring-capabilities-consistency, check-ring-layering, check-ring-layering-full, check-ring0-async-purity, check-stale-tags, check-toc-integrity, check-toctou-defaults, check-type-registry, check-unsafe-islands, check-veto-deadlines, check-vetoes, check-workflow-commands, claim, commit-health, consolidate-adrs, context-pack, context-tags, crate-context, debt-audit, env-validate, feature-matrix, forensic-test, gate-check, gen-feature-catalog, gen-prompter-data, gen-sbom, generate-adr, generate-diagnostics, generate-markers, hotspot-report, init-audit-fix, jules-preflight, jules-submit-gate, lint-unsafe-slices, loom-run, migrate-docid-128, mutation-score-record, panic-inventory, post-merge-report, pre-push, prune-branches, py-test, regenerate-flatbuffers, reproducible-build, run-community-detection, security-scan, session-init, shell-commit-audit, sync-docs, tag-health, validate-pr-checklist, validate-tags, workspace-verify");
            std::process::exit(1);
        }
    }
}
