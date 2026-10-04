pub mod audit;
pub mod bench;
pub mod checks;
pub mod docs;
pub mod jules;
pub mod misc;

pub type CmdFn = fn(&[String]) -> i32;

pub static COMMAND_DISPATCH_TABLE: &[(&str, CmdFn)] = &[
    (
        "reproducible-build",
        crate::cli::misc::run_reproducible_build,
    ),
    (
        "reproducible-build-check",
        crate::cli::misc::run_reproducible_build,
    ),
    (
        "check-commit-diff-integrity",
        crate::cli::checks::run_check_commit_diff_integrity,
    ),
    ("debt-audit", crate::cli::audit::run_debt_audit),
    (
        "check-branch-overlap",
        crate::cli::misc::run_check_branch_overlap,
    ),
    ("prune-branches", crate::cli::misc::run_prune_branches),
    ("forensic-test", crate::cli::misc::run_forensic_test),
    ("bench-trend", crate::cli::bench::run_bench_trend),
    ("gen-sbom", crate::cli::misc::run_gen_sbom),
    (
        "check-toc-integrity",
        crate::cli::checks::run_check_toc_integrity,
    ),
    (
        "check-agents-freshness",
        crate::cli::checks::run_check_agents_freshness,
    ),
    (
        "check-manifest-completeness",
        crate::cli::checks::run_check_manifest_completeness,
    ),
    (
        "generate-diagnostics",
        crate::cli::jules::run_generate_diagnostics,
    ),
    ("sync-docs", crate::cli::docs::run_sync_docs),
    (
        "check-crate-references",
        crate::cli::checks::run_check_crate_references,
    ),
    ("gen-arch-docs", crate::cli::docs::run_gen_arch_docs),
    (
        "gen-feature-catalog",
        crate::cli::docs::run_gen_feature_catalog,
    ),
    (
        "check-unsafe-islands",
        crate::cli::checks::run_check_unsafe_islands,
    ),
    (
        "check-ring-layering",
        crate::cli::checks::run_check_ring_layering,
    ),
    (
        "check-ring-layering-full",
        crate::cli::checks::run_check_ring_layering_full,
    ),
    (
        "check-duplicate-core-primitives",
        crate::cli::checks::run_check_duplicate_core_primitives,
    ),
    (
        "check-ring0-async-purity",
        crate::cli::checks::run_check_ring0_async_purity,
    ),
    (
        "check-module-reachability",
        crate::cli::checks::run_check_module_reachability,
    ),
    (
        "check-orphan-modules",
        crate::cli::checks::run_check_module_reachability,
    ),
    (
        "check-audit-tool-evidence",
        crate::cli::checks::run_check_audit_tool_evidence,
    ),
    (
        "check-audit-verdict-independence",
        crate::cli::checks::run_check_audit_verdict_independence,
    ),
    (
        "check-ffi-panic-boundary",
        crate::cli::checks::run_check_ffi_panic_boundary,
    ),
    (
        "lint-unsafe-slices",
        crate::cli::checks::run_lint_unsafe_slices,
    ),
    (
        "check-mutation-score-gate",
        crate::cli::checks::run_check_mutation_score_gate,
    ),
    (
        "mutation-score-record",
        crate::cli::checks::run_mutation_score_record,
    ),
    ("check-compile", crate::cli::checks::run_check_compile),
    (
        "check-phantom-files",
        crate::cli::checks::run_check_phantom_files,
    ),
    (
        "check-audit-duplication",
        crate::cli::checks::run_check_audit_duplication,
    ),
    (
        "check-recall-stability",
        crate::cli::checks::run_check_recall_stability,
    ),
    (
        "check-doc-references",
        crate::cli::checks::run_check_doc_references,
    ),
    (
        "check-duplicate-intent",
        crate::cli::checks::run_check_duplicate_intent,
    ),
    (
        "check-commit-messages",
        crate::cli::checks::run_check_commit_messages,
    ),
    (
        "check-duplicate-symbols-cross-file",
        crate::cli::checks::run_check_duplicate_symbols_cross_file,
    ),
    (
        "check-duplicate-symbols",
        crate::cli::checks::run_check_duplicate_symbols,
    ),
    (
        "check-workflow-commands",
        crate::cli::checks::run_check_workflow_commands,
    ),
    (
        "check-placeholder-refs",
        crate::cli::checks::run_check_placeholder_refs,
    ),
    (
        "check-bandit-latency-budget",
        crate::cli::checks::run_check_bandit_latency_budget,
    ),
    (
        "check-action-pinning",
        crate::cli::checks::run_check_action_pinning,
    ),
    (
        "check-flatbuffers-drift",
        crate::cli::checks::run_check_flatbuffers_drift,
    ),
    (
        "check-fbs-drift",
        crate::cli::checks::run_check_flatbuffers_drift,
    ),
    (
        "regenerate-flatbuffers",
        crate::cli::checks::run_regenerate_flatbuffers,
    ),
    (
        "check-adr-deadlines",
        crate::cli::checks::run_check_adr_deadlines,
    ),
    ("check-vetoes", crate::cli::checks::run_check_vetoes),
    ("validate-tags", crate::cli::checks::run_validate_tags),
    (
        "check-jules-context-freshness",
        crate::cli::checks::run_check_jules_context_freshness,
    ),
    (
        "check-review-coverage",
        crate::cli::checks::run_check_review_coverage,
    ),
    (
        "check-agents-integrity",
        crate::cli::checks::run_check_agents_integrity,
    ),
    (
        "check-consistency",
        crate::cli::checks::run_check_consistency,
    ),
    ("check-dag", crate::cli::checks::run_check_dag),
    (
        "run-community-detection",
        crate::cli::misc::run_run_community_detection,
    ),
    ("gen-prompter-data", crate::cli::misc::run_gen_prompter_data),
    ("context-tags", crate::cli::misc::run_context_tags),
    ("post-merge-report", crate::cli::misc::run_post_merge_report),
    ("jules-preflight", crate::cli::jules::run_jules_preflight),
    (
        "check-type-registry",
        crate::cli::checks::run_check_type_registry,
    ),
    ("generate-adr", crate::cli::docs::run_generate_adr),
    ("consolidate-adrs", crate::cli::docs::run_consolidate_adrs),
    ("init-audit-fix", crate::cli::audit::run_init_audit_fix),
    (
        "validate-pr-checklist",
        crate::cli::jules::run_validate_pr_checklist,
    ),
    ("bench-gate", crate::cli::bench::run_bench_gate),
    ("claim", crate::cli::jules::run_claim),
    ("pre-push", crate::cli::jules::run_pre_push),
    (
        "jules-submit-gate",
        crate::cli::jules::run_jules_submit_gate,
    ),
    ("check-stale-tags", crate::cli::checks::run_check_stale_tags),
    (
        "check-max-results-unbound",
        crate::cli::checks::run_check_max_results_unbound,
    ),
    (
        "check-toctou-defaults",
        crate::cli::checks::run_check_toctou_defaults,
    ),
    (
        "check-nan-hot-loop",
        crate::cli::checks::run_check_nan_hot_loop,
    ),
    ("gate-check", crate::cli::jules::run_gate_check),
    (
        "check-result-dropped-io",
        crate::cli::checks::run_check_result_dropped_io,
    ),
    (
        "check-coverage-gate",
        crate::cli::checks::run_check_coverage_gate,
    ),
    ("migrate-docid-128", crate::cli::misc::run_migrate_docid_128),
    (
        "check-marker-drift",
        crate::cli::checks::run_check_marker_drift,
    ),
    ("generate-markers", crate::cli::jules::run_generate_markers),
    (
        "shell-commit-audit",
        crate::cli::audit::run_shell_commit_audit,
    ),
    (
        "audit-integrity-check",
        crate::cli::audit::run_audit_integrity_check,
    ),
    ("hotspot-report", crate::cli::audit::run_hotspot_report),
    ("tag-health", crate::cli::audit::run_tag_health),
    ("commit-health", crate::cli::audit::run_commit_health),
    ("panic-inventory", crate::cli::audit::run_panic_inventory),
    ("security-scan", crate::cli::audit::run_security_scan),
    ("feature-matrix", crate::cli::misc::run_feature_matrix),
    ("workspace-verify", crate::cli::misc::run_workspace_verify),
    (
        "check-ring-capabilities-consistency",
        crate::cli::checks::run_check_ring_capabilities_consistency,
    ),
    ("loom-run", crate::cli::misc::run_loom_run),
    ("bench-compile", crate::cli::bench::run_bench_compile),
    ("bench-download", crate::cli::bench::run_bench_download),
    ("py-test", crate::cli::misc::run_py_test),
    ("context-pack", crate::cli::jules::run_context_pack),
    ("session-init", crate::cli::jules::run_session_init),
    ("env-validate", crate::cli::misc::run_env_validate),
    ("crate-context", crate::cli::jules::run_crate_context),
    (
        "check-veto-deadlines",
        crate::cli::checks::run_check_veto_deadlines,
    ),
];
