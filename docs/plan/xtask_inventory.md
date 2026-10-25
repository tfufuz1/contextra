# xtask Gate-Inventar (`docs/plan/xtask_inventory.md`)

**Status:** Vollständiges Inventar gemäß REQ-QA-010
**Schutzpfad-Garantie:** `xtask/**` wurde während der Erstellung dieses Inventars **nicht** verändert (`git diff --stat -- xtask/` ist leer).

---

## 1. Übersicht & Datei-Auflistung mit Zeilenzahlen

### 1.1 Quellcode-Module (`xtask/src/` - Total 152 Dateien)

| Nr. | Pfad unter `xtask/src/` | Zeilenzahl | Test vorhanden? | CI-Verdrahtung |
|---|---|---|---|---|
| 1 | `xtask/src/agent_lifecycle/branch_overlap.rs` | 349 | Ja | prune-branches.yml |
| 2 | `xtask/src/agent_lifecycle/mod.rs` | 2 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 3 | `xtask/src/agent_lifecycle/prune_branches.rs` | 24 | Nein | Keine |
| 4 | `xtask/src/artifact_header.rs` | 99 | Ja | Keine |
| 5 | `xtask/src/audit_integrity_check.rs` | 254 | Ja | justfile, audit-integrity.yml |
| 6 | `xtask/src/bench_compile.rs` | 120 | Ja | justfile |
| 7 | `xtask/src/bench_download.rs` | 130 | Ja | justfile |
| 8 | `xtask/src/check_action_pinning.rs` | 195 | Ja | Keine |
| 9 | `xtask/src/check_adr_deadlines.rs` | 378 | Ja | justfile, context-gates.yml |
| 10 | `xtask/src/check_agents_freshness.rs` | 153 | Ja | Keine |
| 11 | `xtask/src/check_agents_integrity.rs` | 153 | Ja | justfile, context-gates.yml, jules_preflight.rs |
| 12 | `xtask/src/check_audit_duplication.rs` | 303 | Ja | Keine |
| 13 | `xtask/src/check_audit_tool_evidence.rs` | 209 | Ja | Keine |
| 14 | `xtask/src/check_audit_verdict_independence.rs` | 433 | Ja | context-gates.yml, merge-gate.yml |
| 15 | `xtask/src/check_commit_diff_integrity.rs` | 409 | Ja | merge-gate.yml |
| 16 | `xtask/src/check_commit_messages.rs` | 202 | Ja | context-gates.yml |
| 17 | `xtask/src/check_compile.rs` | 42 | Nein | context-gates.yml, rust-ci.yml |
| 18 | `xtask/src/check_coverage_gate.rs` | 186 | Ja | justfile, coverage.yml, proof-correctness.yml |
| 19 | `xtask/src/check_doc_references.rs` | 297 | Ja | context-gates.yml |
| 20 | `xtask/src/check_duplicate_core_primitives.rs` | 245 | Ja | justfile, gate_check.rs |
| 21 | `xtask/src/check_duplicate_intent.rs` | 1219 | Ja | context-gates.yml |
| 22 | `xtask/src/check_duplicate_symbols.rs` | 793 | Ja | context-gates.yml, rust-ci.yml, merge-gate.yml, jules_preflight.rs |
| 23 | `xtask/src/check_duplicate_symbols_cross_file.rs` | 245 | Ja | merge-gate.yml |
| 24 | `xtask/src/check_ffi_panic_boundary.rs` | 104 | Nein | contextra-py-ci.yml, merge-gate.yml |
| 25 | `xtask/src/check_flatbuffers_drift.rs` | 203 | Nein | context-gates.yml |
| 26 | `xtask/src/check_jules_context_freshness.rs` | 248 | Ja | jules_preflight.rs |
| 27 | `xtask/src/check_manifest_completeness.rs` | 285 | Ja | Keine |
| 28 | `xtask/src/check_max_results_unbound.rs` | 195 | Ja | justfile, proof-correctness.yml |
| 29 | `xtask/src/check_module_reachability.rs` | 576 | Ja | context-gates.yml, jules_preflight.rs |
| 30 | `xtask/src/check_mutation_score_gate.rs` | 206 | Ja | mutation-testing.yml |
| 31 | `xtask/src/check_nan_validation_in_hot_loop.rs` | 152 | Ja | Keine |
| 32 | `xtask/src/check_orphan_modules.rs` | 13 | Nein | Keine |
| 33 | `xtask/src/check_phantom_files.rs` | 141 | Ja | context-gates.yml |
| 34 | `xtask/src/check_placeholder_refs.rs` | 246 | Ja | context-gates.yml |
| 35 | `xtask/src/check_recall_stability.rs` | 222 | Ja | nucleation-recall-history.yml |
| 36 | `xtask/src/check_result_dropped_on_io.rs` | 342 | Ja | Keine |
| 37 | `xtask/src/check_ring0_async_purity.rs` | 264 | Ja | Keine |
| 38 | `xtask/src/check_ring_capabilities_consistency.rs` | 97 | Ja | justfile |
| 39 | `xtask/src/check_ring_layering.rs` | 672 | Ja | justfile, scheduled-audit.yml, context-gates.yml, merge-gate.yml, gate_check.rs, jules_preflight.rs |
| 40 | `xtask/src/check_stale_tags.rs` | 432 | Ja | Keine |
| 41 | `xtask/src/check_toc_integrity.rs` | 242 | Ja | Keine |
| 42 | `xtask/src/check_toctou_trait_defaults.rs` | 178 | Ja | Keine |
| 43 | `xtask/src/check_type_registry.rs` | 233 | Ja | Keine |
| 44 | `xtask/src/check_unsafe_islands.rs` | 437 | Ja | context-gates.yml, merge-gate.yml, jules_preflight.rs |
| 45 | `xtask/src/check_vetoes.rs` | 644 | Ja | justfile |
| 46 | `xtask/src/check_workflow_commands.rs` | 296 | Ja | context-gates.yml |
| 47 | `xtask/src/claim.rs` | 842 | Ja | justfile, verdict.yml, commit-integrity-guard.yml, jules_preflight.rs |
| 48 | `xtask/src/commit_health.rs` | 175 | Ja | justfile |
| 49 | `xtask/src/context_pack.rs` | 201 | Ja | justfile |
| 50 | `xtask/src/core/catalog.rs` | 75 | Ja | Keine |
| 51 | `xtask/src/core/cmd.rs` | 216 | Nein | gate_check.rs |
| 52 | `xtask/src/core/constitution.rs` | 80 | Nein | Keine |
| 53 | `xtask/src/core/ctx.rs` | 43 | Nein | Keine |
| 54 | `xtask/src/core/evidence.rs` | 104 | Ja | scheduled-audit.yml |
| 55 | `xtask/src/core/exec.rs` | 53 | Nein | justfile, bench.yml, chaos.yml, gate_check.rs |
| 56 | `xtask/src/core/exit.rs` | 7 | Nein | justfile, scheduled-audit.yml, context-gates.yml, bench.yml, deny.yml, mutation-testing.yml, rust-ci.yml, unwrap-gate.yml, nucleation-recall-history.yml, commit-integrity-guard.yml, merge-gate.yml, unwrap-baseline-check.yml, gate_check.rs |
| 57 | `xtask/src/core/gate.rs` | 42 | Ja | justfile, context-gates.yml, bench.yml, mutation-testing.yml, rust-ci.yml, deep-gates.yml, verdict.yml, unwrap-gate.yml, nucleation-recall-history.yml, coverage.yml, contextra-py-ci.yml, proof-correctness.yml, merge-gate.yml, bench-regression.yml, post-merge-verification.yml, gate_check.rs, jules_preflight.rs |
| 58 | `xtask/src/core/mod.rs` | 10 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 59 | `xtask/src/core/ratchet.rs` | 26 | Ja | verdict.yml, unwrap-baseline-check.yml |
| 60 | `xtask/src/core/report.rs` | 84 | Ja | justfile, prune-branches.yml, rust-ci.yml, flake-report.yml, coverage.yml, post-merge-verification.yml, gate_check.rs |
| 61 | `xtask/src/crate_context.rs` | 270 | Ja | justfile |
| 62 | `xtask/src/detect_nested_locks.yml` | 11 | Nein | justfile |
| 63 | `xtask/src/env_validate.rs` | 118 | Ja | justfile |
| 64 | `xtask/src/feature_matrix.rs` | 243 | Ja | justfile, rust-ci.yml |
| 65 | `xtask/src/gate_check.rs` | 366 | Ja | gate_check.rs |
| 66 | `xtask/src/gates/arch/mod.rs` | 1 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 67 | `xtask/src/gates/check_crate_references.rs` | 309 | Ja | jules_preflight.rs |
| 68 | `xtask/src/gates/code/mod.rs` | 1 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 69 | `xtask/src/gates/debt_audit.rs` | 674 | Ja | justfile |
| 70 | `xtask/src/gates/docs/mod.rs` | 1 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 71 | `xtask/src/gates/gov/meta_gates.rs` | 107 | Nein | Keine |
| 72 | `xtask/src/gates/gov/mod.rs` | 1 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 73 | `xtask/src/gates/mod.rs` | 14 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 74 | `xtask/src/gates/spec/mod.rs` | 1 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 75 | `xtask/src/gen_arch_docs.rs` | 295 | Ja | Keine |
| 76 | `xtask/src/gen_feature_catalog.rs` | 104 | Nein | Keine |
| 77 | `xtask/src/gen_prompter_data.rs` | 447 | Ja | justfile, context-gates.yml, update-prompter-data.yml |
| 78 | `xtask/src/generate_adr.rs` | 304 | Ja | Keine |
| 79 | `xtask/src/generate_diagnostics.rs` | 129 | Nein | Keine |
| 80 | `xtask/src/generate_markers.rs` | 228 | Ja | Keine |
| 81 | `xtask/src/generators/mod.rs` | 1 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 82 | `xtask/src/generators/sbom.rs` | 22 | Nein | Keine |
| 83 | `xtask/src/harness/blast_radius.rs` | 334 | Ja | Keine |
| 84 | `xtask/src/harness/can_merge.rs` | 239 | Ja | verdict.yml |
| 85 | `xtask/src/harness/claims_in_pr.rs` | 461 | Ja | verdict.yml |
| 86 | `xtask/src/harness/codeowners_coverage.rs` | 349 | Ja | Keine |
| 87 | `xtask/src/harness/dependency_gate.rs` | 155 | Ja | Keine |
| 88 | `xtask/src/harness/determinism_check.rs` | 434 | Ja | deep-gates.yml |
| 89 | `xtask/src/harness/diff_budget.rs` | 367 | Ja | verdict.yml |
| 90 | `xtask/src/harness/doc_truth.rs` | 724 | Ja | verdict.yml |
| 91 | `xtask/src/harness/env_attest.rs` | 330 | Ja | Keine |
| 92 | `xtask/src/harness/explain.rs` | 279 | Ja | Keine |
| 93 | `xtask/src/harness/flake_report.rs` | 178 | Ja | flake-report.yml |
| 94 | `xtask/src/harness/fuzz_smoke.rs` | 279 | Ja | deep-gates.yml |
| 95 | `xtask/src/harness/gate_weakening.rs` | 679 | Ja | verdict.yml |
| 96 | `xtask/src/harness/gates_catalog_sync.rs` | 419 | Ja | Keine |
| 97 | `xtask/src/harness/jules.rs` | 321 | Ja | justfile, scheduled-audit.yml, context-gates.yml, flake-report.yml, post-merge-verification.yml, jules_preflight.rs |
| 98 | `xtask/src/harness/jules_dispatch.rs` | 290 | Ja | Keine |
| 99 | `xtask/src/harness/ledger.rs` | 460 | Ja | verdict.yml |
| 100 | `xtask/src/harness/lock_audit.rs` | 322 | Ja | deep-gates.yml |
| 101 | `xtask/src/harness/loop_guard.rs` | 501 | Ja | Keine |
| 102 | `xtask/src/harness/mod.rs` | 64 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 103 | `xtask/src/harness/mutants_diff.rs` | 334 | Ja | deep-gates.yml |
| 104 | `xtask/src/harness/plan_lint.rs` | 463 | Ja | Keine |
| 105 | `xtask/src/harness/postmortem.rs` | 276 | Ja | Keine |
| 106 | `xtask/src/harness/protected_paths.rs` | 743 | Ja | verdict.yml |
| 107 | `xtask/src/harness/ratchet.rs` | 750 | Ja | verdict.yml, unwrap-baseline-check.yml |
| 108 | `xtask/src/harness/registry_check.rs` | 429 | Ja | justfile, verdict.yml |
| 109 | `xtask/src/harness/risk.rs` | 213 | Ja | scheduled-audit.yml |
| 110 | `xtask/src/harness/scope_guard.rs` | 407 | Ja | verdict.yml |
| 111 | `xtask/src/harness/session_report.rs` | 312 | Ja | Keine |
| 112 | `xtask/src/harness/symbol_exists.rs` | 662 | Ja | Keine |
| 113 | `xtask/src/harness/task_card.rs` | 610 | Ja | verdict.yml |
| 114 | `xtask/src/harness/test_integrity.rs` | 510 | Ja | verdict.yml |
| 115 | `xtask/src/harness/unsafe_audit.rs` | 391 | Ja | deep-gates.yml |
| 116 | `xtask/src/harness/verdict.rs` | 376 | Ja | context-gates.yml, verdict.yml, merge-gate.yml |
| 117 | `xtask/src/harness/wal_replay_verify.rs` | 386 | Ja | deep-gates.yml |
| 118 | `xtask/src/hotspot_report.rs` | 139 | Ja | justfile |
| 119 | `xtask/src/init_audit_fix.rs` | 204 | Ja | Keine |
| 120 | `xtask/src/jules_preflight.rs` | 771 | Ja | justfile, context-gates.yml, post-merge-verification.yml, jules_preflight.rs |
| 121 | `xtask/src/jules_submit_gate.rs` | 486 | Ja | Keine |
| 122 | `xtask/src/legacy/arch.rs` | 71 | Nein | justfile, release-mcp-binary.yml, context-gates.yml, build-wheels.yml, publish-pypi.yml, proof-correctness.yml, bench-regression.yml, simd-cross-validation.yml |
| 123 | `xtask/src/legacy/code.rs` | 41 | Ja | justfile, scheduled-audit.yml, bench.yml, coverage.yml, gate_check.rs |
| 124 | `xtask/src/legacy/common.rs` | 63 | Nein | Keine |
| 125 | `xtask/src/legacy/docs.rs` | 81 | Nein | justfile, mutation-testing.yml, post-merge-verification.yml, unwrap-baseline-check.yml, gate_check.rs, jules_preflight.rs |
| 126 | `xtask/src/legacy/gov.rs` | 105 | Nein | verdict.yml |
| 127 | `xtask/src/legacy/heavy.rs` | 73 | Nein | Keine |
| 128 | `xtask/src/legacy/mod.rs` | 9 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 129 | `xtask/src/legacy/ratchet_supply.rs` | 53 | Nein | Keine |
| 130 | `xtask/src/legacy/session.rs` | 69 | Ja | justfile, bench.yml, jules_preflight.rs |
| 131 | `xtask/src/legacy/tags.rs` | 25 | Ja | justfile, release-mcp-binary.yml, context-gates.yml, build-wheels.yml, jules_preflight.rs |
| 132 | `xtask/src/lib.rs` | 61 | Nein | entrypoint |
| 133 | `xtask/src/lint_unsafe_slice_bounds.rs` | 341 | Ja | Keine |
| 134 | `xtask/src/loom_run.rs` | 146 | Ja | justfile |
| 135 | `xtask/src/main.rs` | 53 | Ja | entrypoint |
| 136 | `xtask/src/migrate_docid_128.rs` | 267 | Ja | Keine |
| 137 | `xtask/src/panic_inventory.rs` | 269 | Ja | justfile, scheduled-audit.yml, unwrap-baseline-check.yml |
| 138 | `xtask/src/post_merge_report.rs` | 124 | Ja | post-merge-verification.yml |
| 139 | `xtask/src/proof/bench_trend.rs` | 23 | Nein | Keine |
| 140 | `xtask/src/proof/forensic_test.rs` | 25 | Nein | Keine |
| 141 | `xtask/src/proof/mod.rs` | 2 | Nein | justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs |
| 142 | `xtask/src/py_test.rs` | 109 | Ja | justfile |
| 143 | `xtask/src/record_mutation_score.rs` | 254 | Ja | Keine |
| 144 | `xtask/src/reproducible_build.rs` | 289 | Ja | Keine |
| 145 | `xtask/src/security_scan.rs` | 240 | Ja | justfile |
| 146 | `xtask/src/session_init.rs` | 217 | Ja | justfile |
| 147 | `xtask/src/shell_commit_audit.rs` | 184 | Ja | justfile, shell-commit-gate.yml |
| 148 | `xtask/src/tag_health.rs` | 169 | Ja | justfile |
| 149 | `xtask/src/validate_pr_checklist.rs` | 473 | Ja | Keine |
| 150 | `xtask/src/verify_state_transitions.yml` | 12 | Nein | Keine |
| 151 | `xtask/src/veto_deadline_gate.rs` | 278 | Ja | Keine |
| 152 | `xtask/src/workspace_verify.rs` | 497 | Ja | justfile |

### 1.2 Test-Dateien (`xtask/tests/` - Total 75 Dateien)

| Nr. | Pfad unter `xtask/tests/` | Zeilenzahl |
|---|---|---|
| 1 | `xtask/tests/artifact_header_test.rs` | 75 |
| 2 | `xtask/tests/audit_integrity_check_test.rs` | 108 |
| 3 | `xtask/tests/bench_compile_test.rs` | 64 |
| 4 | `xtask/tests/bench_download_test.rs` | 59 |
| 5 | `xtask/tests/branch_overlap_test.rs` | 245 |
| 6 | `xtask/tests/canaries/main.rs` | 712 |
| 7 | `xtask/tests/characterization_test.rs` | 29 |
| 8 | `xtask/tests/check_adr_deadlines_decisions_dir_test.rs` | 107 |
| 9 | `xtask/tests/check_commit_diff_integrity_test.rs` | 130 |
| 10 | `xtask/tests/check_duplicate_symbols_cross_file_test.rs` | 112 |
| 11 | `xtask/tests/check_mutation_score_gate_test.rs` | 104 |
| 12 | `xtask/tests/check_result_dropped_io.rs` | 32 |
| 13 | `xtask/tests/check_ring_capabilities_consistency_test.rs` | 68 |
| 14 | `xtask/tests/check_stale_tags_until_test.rs` | 157 |
| 15 | `xtask/tests/check_veto_deadlines_scoped_test.rs` | 168 |
| 16 | `xtask/tests/check_vetoes_scoped_block_test.rs` | 140 |
| 17 | `xtask/tests/commit_health_test.rs` | 96 |
| 18 | `xtask/tests/context_pack_test.rs` | 109 |
| 19 | `xtask/tests/crate_context_test.rs` | 125 |
| 20 | `xtask/tests/env_validate_test.rs` | 67 |
| 21 | `xtask/tests/feature_matrix_test.rs` | 46 |
| 22 | `xtask/tests/gate_check_level_zero_test.rs` | 105 |
| 23 | `xtask/tests/generate_markers_test.rs` | 148 |
| 24 | `xtask/tests/harness_blast_radius_test.rs` | 33 |
| 25 | `xtask/tests/harness_can_merge_test.rs` | 71 |
| 26 | `xtask/tests/harness_claims_in_pr_test.rs` | 205 |
| 27 | `xtask/tests/harness_codeowners_coverage_test.rs` | 85 |
| 28 | `xtask/tests/harness_dependency_gate_test.rs` | 33 |
| 29 | `xtask/tests/harness_determinism_check_test.rs` | 75 |
| 30 | `xtask/tests/harness_diff_budget_test.rs` | 161 |
| 31 | `xtask/tests/harness_doc_truth_test.rs` | 379 |
| 32 | `xtask/tests/harness_env_attest_test.rs` | 170 |
| 33 | `xtask/tests/harness_explain_test.rs` | 64 |
| 34 | `xtask/tests/harness_flake_report_test.rs` | 19 |
| 35 | `xtask/tests/harness_foundation_test.rs` | 76 |
| 36 | `xtask/tests/harness_fuzz_smoke_test.rs` | 32 |
| 37 | `xtask/tests/harness_gate_weakening_test.rs` | 234 |
| 38 | `xtask/tests/harness_gates_catalog_sync_test.rs` | 108 |
| 39 | `xtask/tests/harness_jules_dispatch_test.rs` | 162 |
| 40 | `xtask/tests/harness_jules_test.rs` | 144 |
| 41 | `xtask/tests/harness_ledger_test.rs` | 210 |
| 42 | `xtask/tests/harness_lock_audit_test.rs` | 55 |
| 43 | `xtask/tests/harness_loop_guard_test.rs` | 150 |
| 44 | `xtask/tests/harness_loop_guard_wiring_test.rs` | 141 |
| 45 | `xtask/tests/harness_mutants_diff_test.rs` | 41 |
| 46 | `xtask/tests/harness_plan_lint_test.rs` | 177 |
| 47 | `xtask/tests/harness_postmortem_test.rs` | 62 |
| 48 | `xtask/tests/harness_protected_paths_test.rs` | 428 |
| 49 | `xtask/tests/harness_ratchet_test.rs` | 159 |
| 50 | `xtask/tests/harness_registry_check_test.rs` | 167 |
| 51 | `xtask/tests/harness_risk_test.rs` | 33 |
| 52 | `xtask/tests/harness_scope_guard_test.rs` | 170 |
| 53 | `xtask/tests/harness_session_report_test.rs` | 125 |
| 54 | `xtask/tests/harness_symbol_exists_test.rs` | 115 |
| 55 | `xtask/tests/harness_task_card_test.rs` | 135 |
| 56 | `xtask/tests/harness_test_integrity_test.rs` | 178 |
| 57 | `xtask/tests/harness_unsafe_audit_test.rs` | 68 |
| 58 | `xtask/tests/harness_verdict_test.rs` | 283 |
| 59 | `xtask/tests/harness_wal_replay_verify_test.rs` | 66 |
| 60 | `xtask/tests/hotspot_report_test.rs` | 90 |
| 61 | `xtask/tests/layering.rs` | 32 |
| 62 | `xtask/tests/loom_run_test.rs` | 52 |
| 63 | `xtask/tests/panic_inventory_test.rs` | 104 |
| 64 | `xtask/tests/py_test_test.rs` | 51 |
| 65 | `xtask/tests/reproducible_build_path_independent.rs` | 52 |
| 66 | `xtask/tests/ring0_async_purity_adr_placeholder_test.rs` | 83 |
| 67 | `xtask/tests/router_dev_only_dependency.rs` | 59 |
| 68 | `xtask/tests/security_scan_test.rs` | 64 |
| 69 | `xtask/tests/session_init_test.rs` | 83 |
| 70 | `xtask/tests/shell_commit_audit_test.rs` | 78 |
| 71 | `xtask/tests/tag_health_test.rs` | 104 |
| 72 | `xtask/tests/unsafe_islands.rs` | 32 |
| 73 | `xtask/tests/veto_deadline_gate_test.rs` | 107 |
| 74 | `xtask/tests/working_state_stability_test.rs` | 75 |
| 75 | `xtask/tests/workspace_verify_test.rs` | 92 |

---

## 2. Detaillierte Modul-Analyse & Gate-Inventar (152 Module)

### 2.1 `xtask/src/agent_lifecycle/branch_overlap.rs`
- **Zweck:** Prüft Überschneidungen und Konflikte zwischen parallelen Feature-Branches.
- **Betroffene Crates / Pfade:** `Git Repository / Agent Lifecycle`
- **Zeilenzahl:** 349
- **CI-Verdrahtung:** prune-branches.yml
- **Tests vorhanden:** Ja (branch_overlap_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.2 `xtask/src/agent_lifecycle/mod.rs`
- **Zweck:** Modul-Deklarationsdatei für Agent-Lifecycle-Operationen.
- **Betroffene Crates / Pfade:** `xtask/src/agent_lifecycle/`
- **Zeilenzahl:** 2
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.3 `xtask/src/agent_lifecycle/prune_branches.rs`
- **Zweck:** Bereinigt veraltete oder gemergte Remote-Branches.
- **Betroffene Crates / Pfade:** `Git Remote Branches`
- **Zeilenzahl:** 24
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.4 `xtask/src/artifact_header.rs`
- **Zweck:** Erzeugt und formatiert standardisierte Metadaten-Header für Test- und Audit-Artefakte.
- **Betroffene Crates / Pfade:** `xtask / Diagnostic Reports`
- **Zeilenzahl:** 99
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (artifact_header_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.5 `xtask/src/audit_integrity_check.rs`
- **Zweck:** Validiert die Format- und Hash-Integrität von Audit-Protokollen und Nachweisen.
- **Betroffene Crates / Pfade:** `docs/audits / Evidence-Files`
- **Zeilenzahl:** 254
- **CI-Verdrahtung:** justfile, audit-integrity.yml
- **Tests vorhanden:** Ja (audit_integrity_check_test.rs, commit_health_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.6 `xtask/src/bench_compile.rs`
- **Zweck:** Misst und evaluiert die Kompilierungszeiten kritischer Crate-Konfigurationen.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 120
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (bench_compile_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.7 `xtask/src/bench_download.rs`
- **Zweck:** Lädt Benchmark-Datensätze herunter und validiert Checksummen.
- **Betroffene Crates / Pfade:** `target/benchmarks / Data Assets`
- **Zeilenzahl:** 130
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (bench_download_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.8 `xtask/src/check_action_pinning.rs`
- **Zweck:** Überprüft, ob GitHub Actions im Workflow exakt per Commit-SHA gepinnt sind.
- **Betroffene Crates / Pfade:** `.github/workflows/*.yml`
- **Zeilenzahl:** 195
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.9 `xtask/src/check_adr_deadlines.rs`
- **Zweck:** Überprüft Fristen und Umsetzungsstatus von Architectural Decision Records (ADRs).
- **Betroffene Crates / Pfade:** `docs/adr/*.md`
- **Zeilenzahl:** 378
- **CI-Verdrahtung:** justfile, context-gates.yml
- **Tests vorhanden:** Ja (check_adr_deadlines_decisions_dir_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.10 `xtask/src/check_agents_freshness.rs`
- **Zweck:** Überprüft die Aktualität von AGENTS.md bezüglich geänderter Crate-Strukturen.
- **Betroffene Crates / Pfade:** `AGENTS.md / Workspace Crates`
- **Zeilenzahl:** 153
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.11 `xtask/src/check_agents_integrity.rs`
- **Zweck:** Validiert die Syntax- und Strukturkonformität von AGENTS.md-Dateien.
- **Betroffene Crates / Pfade:** `AGENTS.md`
- **Zeilenzahl:** 153
- **CI-Verdrahtung:** justfile, context-gates.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.12 `xtask/src/check_audit_duplication.rs`
- **Zweck:** Erkennt redundante oder doppelt vergebene Audit-IDs und Testnachweise.
- **Betroffene Crates / Pfade:** `docs/audits/*.md`
- **Zeilenzahl:** 303
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.13 `xtask/src/check_audit_tool_evidence.rs`
- **Zweck:** Überprüft das Vorhandensein und die Formate verlangter Tool-Nachweise in Audits.
- **Betroffene Crates / Pfade:** `docs/audits/ / Tool Evidence`
- **Zeilenzahl:** 209
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.14 `xtask/src/check_audit_verdict_independence.rs`
- **Zweck:** Stellt sicher, dass Audit-Urteile unabhängig ohne Eigenreferenzierung gefällt wurden.
- **Betroffene Crates / Pfade:** `docs/audits/*.md`
- **Zeilenzahl:** 433
- **CI-Verdrahtung:** context-gates.yml, merge-gate.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.15 `xtask/src/check_commit_diff_integrity.rs`
- **Zweck:** Validiert Commit-Diffs auf Einhaltung von Größenbudgets und verbotenen Dateien.
- **Betroffene Crates / Pfade:** `Git Commits / Diffs`
- **Zeilenzahl:** 409
- **CI-Verdrahtung:** merge-gate.yml
- **Tests vorhanden:** Ja (check_commit_diff_integrity_test.rs, harness_postmortem_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.16 `xtask/src/check_commit_messages.rs`
- **Zweck:** Erzwingt Konventionsregeln für Commit-Titel und Nachrichtenbeschreibungen.
- **Betroffene Crates / Pfade:** `Git Commit History`
- **Zeilenzahl:** 202
- **CI-Verdrahtung:** context-gates.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.17 `xtask/src/check_compile.rs`
- **Zweck:** Führt Standard-Compile-Checks für spezifische Workspace-Ziele durch.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 42
- **CI-Verdrahtung:** context-gates.yml, rust-ci.yml
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.18 `xtask/src/check_coverage_gate.rs`
- **Zweck:** Validiert Code-Coverage-Schwellenwerte gegen hinterlegte Minimalanforderungen.
- **Betroffene Crates / Pfade:** `Coverage Artifacts / lcov.info`
- **Zeilenzahl:** 186
- **CI-Verdrahtung:** justfile, coverage.yml, proof-correctness.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.19 `xtask/src/check_doc_references.rs`
- **Zweck:** Überprüft die Bounding und Korrektheit von internen Dokumentationslinks.
- **Betroffene Crates / Pfade:** `docs/**/*.md`
- **Zeilenzahl:** 297
- **CI-Verdrahtung:** context-gates.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.20 `xtask/src/check_duplicate_core_primitives.rs`
- **Zweck:** Verhindert die Duplizierung von Core-Datentypen außerhalb von contextra-types.
- **Betroffene Crates / Pfade:** `crates/* / Core Primitives`
- **Zeilenzahl:** 245
- **CI-Verdrahtung:** justfile, gate_check.rs
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.21 `xtask/src/check_duplicate_intent.rs`
- **Zweck:** Erkennt duplizierten Intent-Code und redundante Logikmuster.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 1219
- **CI-Verdrahtung:** context-gates.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.22 `xtask/src/check_duplicate_symbols.rs`
- **Zweck:** Durchsucht die Codebasis nach doppelten Funktions- und Symbol-Deklarationen.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 793
- **CI-Verdrahtung:** context-gates.yml, rust-ci.yml, merge-gate.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (check_duplicate_symbols_cross_file_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.23 `xtask/src/check_duplicate_symbols_cross_file.rs`
- **Zweck:** Sucht nach dateiübergreifenden Symbol- und Bezeichner-Duplikaten.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 245
- **CI-Verdrahtung:** merge-gate.yml
- **Tests vorhanden:** Ja (check_duplicate_symbols_cross_file_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.24 `xtask/src/check_ffi_panic_boundary.rs`
- **Zweck:** Stellt sicher, meidend vor Panics, dass FFI-Grenzen durch Catch-Unwind abgesichert sind.
- **Betroffene Crates / Pfade:** `contextra-sys / contextra-c-api`
- **Zeilenzahl:** 104
- **CI-Verdrahtung:** contextra-py-ci.yml, merge-gate.yml
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.25 `xtask/src/check_flatbuffers_drift.rs`
- **Zweck:** Überprüft Schema-Drift zwischen FlatBuffers IDL-Definitionen und generiertem Rust-Code.
- **Betroffene Crates / Pfade:** `contextra-wire / flatbuffers`
- **Zeilenzahl:** 203
- **CI-Verdrahtung:** context-gates.yml
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.26 `xtask/src/check_jules_context_freshness.rs`
- **Zweck:** Überprüft, ob Jules Context-Packs aktuell und mit den Hauptdateien synchron sind.
- **Betroffene Crates / Pfade:** `.jules/ / Context-Packs`
- **Zeilenzahl:** 248
- **CI-Verdrahtung:** jules_preflight.rs
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.27 `xtask/src/check_manifest_completeness.rs`
- **Zweck:** Validiert Cargo.toml Manifest-Felder auf Vollständigkeit und Lizenzangaben.
- **Betroffene Crates / Pfade:** `Cargo.toml files`
- **Zeilenzahl:** 285
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.28 `xtask/src/check_max_results_unbound.rs`
- **Zweck:** Validiert, dass Such- und Query-APIs result-limits erzwingen und unbegrenzte Results verhindern.
- **Betroffene Crates / Pfade:** `contextra-engine / contextra-store`
- **Zeilenzahl:** 195
- **CI-Verdrahtung:** justfile, proof-correctness.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.29 `xtask/src/check_module_reachability.rs`
- **Zweck:** Analysiert Modulbäume auf unerreichbare oder verwaiste Rust-Module.
- **Betroffene Crates / Pfade:** `Workspace Crates / Modul-Graph`
- **Zeilenzahl:** 576
- **CI-Verdrahtung:** context-gates.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.30 `xtask/src/check_mutation_score_gate.rs`
- **Zweck:** Evaluiert Mutation-Testing Scores gegen definierte Qualitäts-Schwellenwerte.
- **Betroffene Crates / Pfade:** `cargo-mutants outputs`
- **Zeilenzahl:** 206
- **CI-Verdrahtung:** mutation-testing.yml
- **Tests vorhanden:** Ja (check_mutation_score_gate_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.31 `xtask/src/check_nan_validation_in_hot_loop.rs`
- **Zweck:** Validiert, dass Fließkomma-Hot-Loops NaN-Handling explizit ausführen.
- **Betroffene Crates / Pfade:** `contextra-rank / contextra-simd`
- **Zeilenzahl:** 152
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.32 `xtask/src/check_orphan_modules.rs`
- **Zweck:** Legacy-Alias für check_module_reachability zur Erkennung verwaister Module.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 13
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/check_module_reachability.rs`)
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Reiner Legacy-Alias für check_module_reachability; sollte langfristig direkt im Hauptmodul check_module_reachability konsolidiert werden.

### 2.33 `xtask/src/check_phantom_files.rs`
- **Zweck:** Identifiziert Phantomeinträge und unreferenzierte Dateien im Dateisystem.
- **Betroffene Crates / Pfade:** `Workspace Tree`
- **Zeilenzahl:** 141
- **CI-Verdrahtung:** context-gates.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.34 `xtask/src/check_placeholder_refs.rs`
- **Zweck:** Erkennt verbleibende TODO-, FIX- oder Placeholdermarker in Produktivcode.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 246
- **CI-Verdrahtung:** context-gates.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.35 `xtask/src/check_recall_stability.rs`
- **Zweck:** Überprüft die Stabilität von Recall-Metriken im Vector- und Retrieval-Index.
- **Betroffene Crates / Pfade:** `contextra-vector / contextra-text`
- **Zeilenzahl:** 222
- **CI-Verdrahtung:** nucleation-recall-history.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.36 `xtask/src/check_result_dropped_on_io.rs`
- **Zweck:** Stellt sicher, dass I/O-Operationen Result-Rückgabewerte nicht ignorieren.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 342
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.37 `xtask/src/check_ring0_async_purity.rs`
- **Zweck:** Erzwingt synchrone Ausführung ohne Async-Runtime-Kopplung in Ring 0 Crates.
- **Betroffene Crates / Pfade:** `Ring 0 Crates (contextra-simd, etc.)`
- **Zeilenzahl:** 264
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (ring0_async_purity_adr_placeholder_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.38 `xtask/src/check_ring_capabilities_consistency.rs`
- **Zweck:** Prüft Konsistenz von Ring-Capabilities in capabilities.toml.
- **Betroffene Crates / Pfade:** `capabilities.toml / Workspace Crates`
- **Zeilenzahl:** 97
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (check_ring_capabilities_consistency_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.39 `xtask/src/check_ring_layering.rs`
- **Zweck:** Validiert strikte Einhaltung der Ring-Architekturabhängigkeiten (Ring 0..4).
- **Betroffene Crates / Pfade:** `Cargo.toml / Dependency Graph`
- **Zeilenzahl:** 672
- **CI-Verdrahtung:** justfile, scheduled-audit.yml, context-gates.yml, merge-gate.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Ja (main.rs, check_ring_capabilities_consistency_test.rs, ring0_async_purity_adr_placeholder_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.40 `xtask/src/check_stale_tags.rs`
- **Zweck:** Spürt veraltete Tag-Annotationen und abgelaufene Markers in der Codebasis auf.
- **Betroffene Crates / Pfade:** `Source Code Tags`
- **Zeilenzahl:** 432
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (check_stale_tags_until_test.rs, tag_health_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.41 `xtask/src/check_toc_integrity.rs`
- **Zweck:** Überprüft die Korrektheit und Links in Inhaltsverzeichnissen von Dokumenten.
- **Betroffene Crates / Pfade:** `docs/**/*.md`
- **Zeilenzahl:** 242
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.42 `xtask/src/check_toctou_trait_defaults.rs`
- **Zweck:** Prüft Trait-Default-Methoden auf Time-of-Check to Time-of-Use Schwachstellen.
- **Betroffene Crates / Pfade:** `Workspace Traits`
- **Zeilenzahl:** 178
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.43 `xtask/src/check_type_registry.rs`
- **Zweck:** Validiert Typ-Registrierungen und Vermeidung von Typ-Kollisionen.
- **Betroffene Crates / Pfade:** `contextra-types / Registry`
- **Zeilenzahl:** 233
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.44 `xtask/src/check_unsafe_islands.rs`
- **Zweck:** Überprüft die Einhaltung genehmigter Unsafe Islands gemäß capabilities.toml.
- **Betroffene Crates / Pfade:** `Workspace Crates / Unsafe Blocks`
- **Zeilenzahl:** 437
- **CI-Verdrahtung:** context-gates.yml, merge-gate.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (main.rs, harness_doc_truth_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.45 `xtask/src/check_vetoes.rs`
- **Zweck:** Setzt Architektur- und Qualitäts-Vetos gegen spefische Anti-Patterns durch.
- **Betroffene Crates / Pfade:** `Workspace Tree`
- **Zeilenzahl:** 644
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (check_veto_deadlines_scoped_test.rs, check_vetoes_scoped_block_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.46 `xtask/src/check_workflow_commands.rs`
- **Zweck:** Validiert GitHub Actions Workflow-Befehle auf Syntax und Pfad-Existenz.
- **Betroffene Crates / Pfade:** `.github/workflows/*.yml`
- **Zeilenzahl:** 296
- **CI-Verdrahtung:** context-gates.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.47 `xtask/src/claim.rs`
- **Zweck:** Verwaltet Entwickler- und Agenten-Claims zur Sperrung von Bearbeitungspfaden.
- **Betroffene Crates / Pfade:** `.jules/claims.json`
- **Zeilenzahl:** 842
- **CI-Verdrahtung:** justfile, verdict.yml, commit-integrity-guard.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (branch_overlap_test.rs, check_commit_diff_integrity_test.rs, context_pack_test.rs, harness_claims_in_pr_test.rs, session_init_test.rs, working_state_stability_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.48 `xtask/src/commit_health.rs`
- **Zweck:** Bewertet die Qualität, Größe und Struktur von Git-Commits.
- **Betroffene Crates / Pfade:** `Git History`
- **Zeilenzahl:** 175
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (commit_health_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.49 `xtask/src/context_pack.rs`
- **Zweck:** Erstellt und validiert komprimierte Kontext-Packs für Agenten-Kontext.
- **Betroffene Crates / Pfade:** `.jules/context_packs/`
- **Zeilenzahl:** 201
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (context_pack_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.50 `xtask/src/core/catalog.rs`
- **Zweck:** Katalogisiert verfügbare Gates und deren Metadaten im System.
- **Betroffene Crates / Pfade:** `xtask/src/gates/`
- **Zeilenzahl:** 75
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_gates_catalog_sync_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.51 `xtask/src/core/cmd.rs`
- **Zweck:** Stellt Abstraktionen zur sicheren Ausführung von CLI-Befehlen bereit.
- **Betroffene Crates / Pfade:** `System Process Execution`
- **Zeilenzahl:** 216
- **CI-Verdrahtung:** gate_check.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.52 `xtask/src/core/constitution.rs`
- **Zweck:** Lädt und validiert globale Projekt-Regeln und Metadaten-Konfigurationen.
- **Betroffene Crates / Pfade:** `project_config.toml / Rules`
- **Zeilenzahl:** 80
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.53 `xtask/src/core/ctx.rs`
- **Zweck:** Stellt Ausführungskontext und Verzeichnis-Discovery für xtask bereit.
- **Betroffene Crates / Pfade:** `xtask Context`
- **Zeilenzahl:** 43
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.54 `xtask/src/core/evidence.rs`
- **Zweck:** Strukturiert und speichert Nachweisdateien von Gate-Ausführungen.
- **Betroffene Crates / Pfade:** `target/evidence/`
- **Zeilenzahl:** 104
- **CI-Verdrahtung:** scheduled-audit.yml
- **Tests vorhanden:** Ja (harness_jules_dispatch_test.rs, harness_plan_lint_test.rs, harness_task_card_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.55 `xtask/src/core/exec.rs`
- **Zweck:** Führt Befehlszeilenketten mit Logging und Fehlerbehandlung aus.
- **Betroffene Crates / Pfade:** `Process Execution`
- **Zeilenzahl:** 53
- **CI-Verdrahtung:** justfile, bench.yml, chaos.yml, gate_check.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.56 `xtask/src/core/exit.rs`
- **Zweck:** Definiert einheitliche Exit-Codes für xtask Gates.
- **Betroffene Crates / Pfade:** `Exit Statuses`
- **Zeilenzahl:** 7
- **CI-Verdrahtung:** justfile, scheduled-audit.yml, context-gates.yml, bench.yml, deny.yml, mutation-testing.yml, rust-ci.yml, unwrap-gate.yml, nucleation-recall-history.yml, commit-integrity-guard.yml, merge-gate.yml, unwrap-baseline-check.yml, gate_check.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.57 `xtask/src/core/gate.rs`
- **Zweck:** Definiert Kern-Traits (Gate, GateSpec) für modulare Gates.
- **Betroffene Crates / Pfade:** `xtask Framework`
- **Zeilenzahl:** 42
- **CI-Verdrahtung:** justfile, context-gates.yml, bench.yml, mutation-testing.yml, rust-ci.yml, deep-gates.yml, verdict.yml, unwrap-gate.yml, nucleation-recall-history.yml, coverage.yml, contextra-py-ci.yml, proof-correctness.yml, merge-gate.yml, bench-regression.yml, post-merge-verification.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Ja (check_mutation_score_gate_test.rs, gate_check_level_zero_test.rs, harness_dependency_gate_test.rs, harness_gate_weakening_test.rs, harness_gates_catalog_sync_test.rs, veto_deadline_gate_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.58 `xtask/src/core/mod.rs`
- **Zweck:** Modul-Deklarationsdatei für xtask-Core.
- **Betroffene Crates / Pfade:** `xtask/src/core/`
- **Zeilenzahl:** 10
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.59 `xtask/src/core/ratchet.rs`
- **Zweck:** Implementiert Ratchet-Mechanismen zur schrittweisen Verschärfung von Qualitätsgrenzen.
- **Betroffene Crates / Pfade:** `Ratchet Metrics`
- **Zeilenzahl:** 26
- **CI-Verdrahtung:** verdict.yml, unwrap-baseline-check.yml
- **Tests vorhanden:** Ja (main.rs, harness_ratchet_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.60 `xtask/src/core/report.rs`
- **Zweck:** Generiert Ausführungsberichte für Gate-Suites.
- **Betroffene Crates / Pfade:** `Target Reports`
- **Zeilenzahl:** 84
- **CI-Verdrahtung:** justfile, prune-branches.yml, rust-ci.yml, flake-report.yml, coverage.yml, post-merge-verification.yml, gate_check.rs
- **Tests vorhanden:** Ja (check_commit_diff_integrity_test.rs, commit_health_test.rs, gate_check_level_zero_test.rs, harness_env_attest_test.rs, harness_flake_report_test.rs, harness_loop_guard_test.rs, harness_session_report_test.rs, hotspot_report_test.rs, shell_commit_audit_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.61 `xtask/src/crate_context.rs`
- **Zweck:** Analysiert Crate-Metadaten und Abhängigkeitsstrukturen im Workspace.
- **Betroffene Crates / Pfade:** `Cargo.toml / Crates`
- **Zeilenzahl:** 270
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (crate_context_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.62 `xtask/src/detect_nested_locks.yml`
- **Zweck:** Ast-Grep Regel zur Erkennung verschachtelter Lock-Acquisitions zur Deadlock-Vermeidung.
- **Betroffene Crates / Pfade:** `Source Files`
- **Zeilenzahl:** 11
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.63 `xtask/src/env_validate.rs`
- **Zweck:** Validiert Umgebungsvariablen und vorausgesetzte System-Werkzeuge.
- **Betroffene Crates / Pfade:** `Environment Variables / System Tools`
- **Zeilenzahl:** 118
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (env_validate_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.64 `xtask/src/feature_matrix.rs`
- **Zweck:** Prüft Kombinationsmöglichkeiten und Gültigkeit von Cargo Feature-Flags.
- **Betroffene Crates / Pfade:** `Cargo.toml [features]`
- **Zeilenzahl:** 243
- **CI-Verdrahtung:** justfile, rust-ci.yml
- **Tests vorhanden:** Ja (feature_matrix_test.rs, workspace_verify_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.65 `xtask/src/gate_check.rs`
- **Zweck:** Orchestriert und führt eine Reihe lokaler Qualiäts-Gates aus.
- **Betroffene Crates / Pfade:** `xtask Gate Suite`
- **Zeilenzahl:** 366
- **CI-Verdrahtung:** gate_check.rs
- **Tests vorhanden:** Ja (gate_check_level_zero_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.66 `xtask/src/gates/arch/mod.rs`
- **Zweck:** Architektur-Gates Modul-Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/gates/arch/`
- **Zeilenzahl:** 1
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.67 `xtask/src/gates/check_crate_references.rs`
- **Zweck:** Validiert Crate-Referenzen auf Gültigkeit und Konsistenz.
- **Betroffene Crates / Pfade:** `Cargo.toml / Source Imports`
- **Zeilenzahl:** 309
- **CI-Verdrahtung:** jules_preflight.rs
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.68 `xtask/src/gates/code/mod.rs`
- **Zweck:** Code-Qualitäts-Gates Modul-Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/gates/code/`
- **Zeilenzahl:** 1
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.69 `xtask/src/gates/debt_audit.rs`
- **Zweck:** Auditierer für Code-Schulden und technische Altlasten.
- **Betroffene Crates / Pfade:** `Workspace Tree`
- **Zeilenzahl:** 674
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.70 `xtask/src/gates/docs/mod.rs`
- **Zweck:** Dokumentations-Gates Modul-Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/gates/docs/`
- **Zeilenzahl:** 1
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.71 `xtask/src/gates/gov/meta_gates.rs`
- **Zweck:** Validiert Meta-Regeln und Selbsttests der Gates.
- **Betroffene Crates / Pfade:** `xtask Gates Framework`
- **Zeilenzahl:** 107
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.72 `xtask/src/gates/gov/mod.rs`
- **Zweck:** Governance-Gates Modul-Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/gates/gov/`
- **Zeilenzahl:** 1
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.73 `xtask/src/gates/mod.rs`
- **Zweck:** Gates-Hauptmodul Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/gates/`
- **Zeilenzahl:** 14
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.74 `xtask/src/gates/spec/mod.rs`
- **Zweck:** Spezifikations-Gates Modul-Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/gates/spec/`
- **Zeilenzahl:** 1
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.75 `xtask/src/gen_arch_docs.rs`
- **Zweck:** Generiert Architektur-Dokumente aus In-Code-Spezifikationen.
- **Betroffene Crates / Pfade:** `docs/architecture/`
- **Zeilenzahl:** 295
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.76 `xtask/src/gen_feature_catalog.rs`
- **Zweck:** Erstellt Markdown-Katalog aller Cargo Feature-Flags.
- **Betroffene Crates / Pfade:** `docs/generated/feature-catalog.md`
- **Zeilenzahl:** 104
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.77 `xtask/src/gen_prompter_data.rs`
- **Zweck:** Generiert Trainings- und Promptdaten für LLM-Assistenten.
- **Betroffene Crates / Pfade:** `docs/prompter/`
- **Zeilenzahl:** 447
- **CI-Verdrahtung:** justfile, context-gates.yml, update-prompter-data.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.78 `xtask/src/generate_adr.rs`
- **Zweck:** Erstellt Template-Dateien für neue Architectural Decision Records.
- **Betroffene Crates / Pfade:** `docs/adr/`
- **Zeilenzahl:** 304
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.79 `xtask/src/generate_diagnostics.rs`
- **Zweck:** Sammelt Diagnosedaten und schreibt strukturierten Gate-Bericht.
- **Betroffene Crates / Pfade:** `target/diagnostics/`
- **Zeilenzahl:** 129
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.80 `xtask/src/generate_markers.rs`
- **Zweck:** Erzeugt In-Code Markierungen und Traceability-Identifier.
- **Betroffene Crates / Pfade:** `Source Files`
- **Zeilenzahl:** 228
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (generate_markers_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.81 `xtask/src/generators/mod.rs`
- **Zweck:** Generatoren Submodul-Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/generators/`
- **Zeilenzahl:** 1
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.82 `xtask/src/generators/sbom.rs`
- **Zweck:** Generiert Software Bill of Materials (SBOM) im CycloneDX Format.
- **Betroffene Crates / Pfade:** `Workspace SBOM`
- **Zeilenzahl:** 22
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.83 `xtask/src/harness/blast_radius.rs`
- **Zweck:** Berechnet den Impact-Radius von Änderungen im Workspace.
- **Betroffene Crates / Pfade:** `Workspace Dependency Graph`
- **Zeilenzahl:** 334
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_blast_radius_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.84 `xtask/src/harness/can_merge.rs`
- **Zweck:** Bewertet die Merge-Bereitschaft eines Pull Requests.
- **Betroffene Crates / Pfade:** `PR Metadata / Checks`
- **Zeilenzahl:** 239
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (harness_can_merge_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.85 `xtask/src/harness/claims_in_pr.rs`
- **Zweck:** Validiert vorhandene Claims bei PR-Erstellung und Review.
- **Betroffene Crates / Pfade:** `PR Claims`
- **Zeilenzahl:** 461
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (harness_claims_in_pr_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.86 `xtask/src/harness/codeowners_coverage.rs`
- **Zweck:** Prüft Abdeckung aller Pfade durch CODEOWNERS-Einträge.
- **Betroffene Crates / Pfade:** `CODEOWNERS / Workspace Tree`
- **Zeilenzahl:** 349
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_codeowners_coverage_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.87 `xtask/src/harness/dependency_gate.rs`
- **Zweck:** Kontrolliert das Hinzufügen externer Crate-Abhängigkeiten.
- **Betroffene Crates / Pfade:** `Cargo.toml Dependencies`
- **Zeilenzahl:** 155
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_dependency_gate_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.88 `xtask/src/harness/determinism_check.rs`
- **Zweck:** Überprüft Deterministisches Verhalten in Tests und Builds.
- **Betroffene Crates / Pfade:** `Test Suite / Builds`
- **Zeilenzahl:** 434
- **CI-Verdrahtung:** deep-gates.yml
- **Tests vorhanden:** Ja (main.rs, harness_determinism_check_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.89 `xtask/src/harness/diff_budget.rs`
- **Zweck:** Erzwingt Höchstgrenzen für geänderte Zeilen pro PR.
- **Betroffene Crates / Pfade:** `Git Diffs`
- **Zeilenzahl:** 367
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (harness_diff_budget_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.90 `xtask/src/harness/doc_truth.rs`
- **Zweck:** Gleicht Code-Implementierungen mit Dokumentations-Behauptungen ab.
- **Betroffene Crates / Pfade:** `docs/ / Source Code`
- **Zeilenzahl:** 724
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (harness_doc_truth_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.91 `xtask/src/harness/env_attest.rs`
- **Zweck:** Attestiert die Integrität der Build- und CI-Umgebung.
- **Betroffene Crates / Pfade:** `CI Runtime / Environment`
- **Zeilenzahl:** 330
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_env_attest_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.92 `xtask/src/harness/explain.rs`
- **Zweck:** Liefert detaillierte Erklärungen zu Gate-Fehlschlägen.
- **Betroffene Crates / Pfade:** `Gate Diagnostics`
- **Zeilenzahl:** 279
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_explain_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.93 `xtask/src/harness/flake_report.rs`
- **Zweck:** Analysiert und berichtet flaky Testläufe.
- **Betroffene Crates / Pfade:** `Test Logs`
- **Zeilenzahl:** 178
- **CI-Verdrahtung:** flake-report.yml
- **Tests vorhanden:** Ja (harness_flake_report_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.94 `xtask/src/harness/fuzz_smoke.rs`
- **Zweck:** Führt Fuzzing-Smoke-Tests für kritische Parser aus.
- **Betroffene Crates / Pfade:** `fuzz/ targets`
- **Zeilenzahl:** 279
- **CI-Verdrahtung:** deep-gates.yml
- **Tests vorhanden:** Ja (harness_fuzz_smoke_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.95 `xtask/src/harness/gate_weakening.rs`
- **Zweck:** Verhindert das Abschwächen oder Entfernen bestehender Gates (REQ-QA-010).
- **Betroffene Crates / Pfade:** `xtask Gates`
- **Zeilenzahl:** 679
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (main.rs, harness_gate_weakening_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.96 `xtask/src/harness/gates_catalog_sync.rs`
- **Zweck:** Stellt Synchronität zwischen Gate-Katalog und Code sicher.
- **Betroffene Crates / Pfade:** `docs/gates.md / xtask`
- **Zeilenzahl:** 419
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_gates_catalog_sync_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.97 `xtask/src/harness/jules.rs`
- **Zweck:** Haupt-Harness für Jules-Agent-Interaktionen und Validierungen.
- **Betroffene Crates / Pfade:** `Jules Workflows`
- **Zeilenzahl:** 321
- **CI-Verdrahtung:** justfile, scheduled-audit.yml, context-gates.yml, flake-report.yml, post-merge-verification.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (branch_overlap_test.rs, main.rs, context_pack_test.rs, harness_diff_budget_test.rs, harness_doc_truth_test.rs, harness_env_attest_test.rs, harness_gates_catalog_sync_test.rs, harness_jules_dispatch_test.rs, harness_jules_test.rs, harness_ledger_test.rs, harness_loop_guard_test.rs, harness_loop_guard_wiring_test.rs, harness_mutants_diff_test.rs, harness_plan_lint_test.rs, harness_scope_guard_test.rs, harness_session_report_test.rs, harness_task_card_test.rs, session_init_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.98 `xtask/src/harness/jules_dispatch.rs`
- **Zweck:** Steuert und verteilt Aufgaben an den Jules Agenten.
- **Betroffene Crates / Pfade:** `Agent Commands`
- **Zeilenzahl:** 290
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_jules_dispatch_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.99 `xtask/src/harness/ledger.rs`
- **Zweck:** Protokolliert Test- und Gate-Ergebnisse fälschungssicher.
- **Betroffene Crates / Pfade:** `target/ledger/`
- **Zeilenzahl:** 460
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (harness_ledger_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.100 `xtask/src/harness/lock_audit.rs`
- **Zweck:** Überprüft Lock-Nutzung und verhindert Lock-Contention.
- **Betroffene Crates / Pfade:** `Source Code Locks`
- **Zeilenzahl:** 322
- **CI-Verdrahtung:** deep-gates.yml
- **Tests vorhanden:** Ja (harness_lock_audit_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.101 `xtask/src/harness/loop_guard.rs`
- **Zweck:** Erkennt unendliche Schleifen oder blockierende Rekursionen.
- **Betroffene Crates / Pfade:** `Source Code Loops`
- **Zeilenzahl:** 501
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_loop_guard_test.rs, harness_loop_guard_wiring_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.102 `xtask/src/harness/mod.rs`
- **Zweck:** Harness-Hauptmodul Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/harness/`
- **Zeilenzahl:** 64
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.103 `xtask/src/harness/mutants_diff.rs`
- **Zweck:** Evaluiert Mutation-Testing Diffs auf geänderten Dateien.
- **Betroffene Crates / Pfade:** `cargo-mutants Diff`
- **Zeilenzahl:** 334
- **CI-Verdrahtung:** deep-gates.yml
- **Tests vorhanden:** Ja (harness_mutants_diff_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.104 `xtask/src/harness/plan_lint.rs`
- **Zweck:** Validiert die Syntax und Vollständigkeit von Ausführungsplänen.
- **Betroffene Crates / Pfade:** `Plan Files / Markdown`
- **Zeilenzahl:** 463
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_plan_lint_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.105 `xtask/src/harness/postmortem.rs`
- **Zweck:** Generiert Incident- und Postmortem-Analysen nach Testfehlern.
- **Betroffene Crates / Pfade:** `Test Artifacts`
- **Zeilenzahl:** 276
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_postmortem_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.106 `xtask/src/harness/protected_paths.rs`
- **Zweck:** Schützt kritische Pfade vor unbefugter oder unbeabsichtigter Änderung.
- **Betroffene Crates / Pfade:** `Protected Files (xtask, etc.)`
- **Zeilenzahl:** 743
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (main.rs, harness_protected_paths_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.107 `xtask/src/harness/ratchet.rs`
- **Zweck:** Setzt Ratchet-Anforderungen durch, damit Metriken nicht regredieren.
- **Betroffene Crates / Pfade:** `Quality Metrics`
- **Zeilenzahl:** 750
- **CI-Verdrahtung:** verdict.yml, unwrap-baseline-check.yml
- **Tests vorhanden:** Ja (main.rs, harness_ratchet_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.108 `xtask/src/harness/registry_check.rs`
- **Zweck:** Prüft Erreichbarkeit und Sicherheit konfigurierter Crate-Registries.
- **Betroffene Crates / Pfade:** `Cargo Registries`
- **Zeilenzahl:** 429
- **CI-Verdrahtung:** justfile, verdict.yml
- **Tests vorhanden:** Ja (harness_registry_check_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.109 `xtask/src/harness/risk.rs`
- **Zweck:** Berechnet Risiko-Scores für geplante PR-Änderungen.
- **Betroffene Crates / Pfade:** `PR Diffs / Risk Metrics`
- **Zeilenzahl:** 213
- **CI-Verdrahtung:** scheduled-audit.yml
- **Tests vorhanden:** Ja (harness_risk_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.110 `xtask/src/harness/scope_guard.rs`
- **Zweck:** Validiert den Arbeitsbereich und verhindert Scope-Exzesse.
- **Betroffene Crates / Pfade:** `Workspace Scope`
- **Zeilenzahl:** 407
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (main.rs, harness_scope_guard_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.111 `xtask/src/harness/session_report.rs`
- **Zweck:** Erstellt Zusammenfassungen von Entwicklungs- und Test-Sitzungen.
- **Betroffene Crates / Pfade:** `Session Logs`
- **Zeilenzahl:** 312
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (harness_session_report_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.112 `xtask/src/harness/symbol_exists.rs`
- **Zweck:** Stellt sicher, dass deklarierte Symbole in Exporte existieren.
- **Betroffene Crates / Pfade:** `Exported API Symbols`
- **Zeilenzahl:** 662
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (main.rs, harness_symbol_exists_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.113 `xtask/src/harness/task_card.rs`
- **Zweck:** Validiert Verknüpfung von Task-Cards mit PRs.
- **Betroffene Crates / Pfade:** `Task Cards / PRs`
- **Zeilenzahl:** 610
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (harness_task_card_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.114 `xtask/src/harness/test_integrity.rs`
- **Zweck:** Validiert, dass Tests nicht manipuliert oder auskommentiert wurden.
- **Betroffene Crates / Pfade:** `Test Suite`
- **Zeilenzahl:** 510
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Ja (main.rs, harness_test_integrity_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.115 `xtask/src/harness/unsafe_audit.rs`
- **Zweck:** Auditiert Unsafe-Nutzung gegen das Unsafe-Islands-Regelwerk.
- **Betroffene Crates / Pfade:** `Unsafe Blocks`
- **Zeilenzahl:** 391
- **CI-Verdrahtung:** deep-gates.yml
- **Tests vorhanden:** Ja (harness_unsafe_audit_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.116 `xtask/src/harness/verdict.rs`
- **Zweck:** Berechnet das finale Gesamturteil über alle ausgeführten Gates.
- **Betroffene Crates / Pfade:** `Gate Results Summary`
- **Zeilenzahl:** 376
- **CI-Verdrahtung:** context-gates.yml, verdict.yml, merge-gate.yml
- **Tests vorhanden:** Ja (audit_integrity_check_test.rs, harness_gates_catalog_sync_test.rs, harness_verdict_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.117 `xtask/src/harness/wal_replay_verify.rs`
- **Zweck:** Überprüft die Verifizierbarkeit der WAL-Replay-Integrität.
- **Betroffene Crates / Pfade:** `contextra-store / WAL`
- **Zeilenzahl:** 386
- **CI-Verdrahtung:** deep-gates.yml
- **Tests vorhanden:** Ja (harness_wal_replay_verify_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.118 `xtask/src/hotspot_report.rs`
- **Zweck:** Identifiziert Code-Hotspots mit hoher Komplexität und Änderungsfrequenz.
- **Betroffene Crates / Pfade:** `Git Commit History / Code`
- **Zeilenzahl:** 139
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (commit_health_test.rs, hotspot_report_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.119 `xtask/src/init_audit_fix.rs`
- **Zweck:** Initialisiert automatische Reparaturabläufe für Audit-Befunde.
- **Betroffene Crates / Pfade:** `Audit Issues`
- **Zeilenzahl:** 204
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.120 `xtask/src/jules_preflight.rs`
- **Zweck:** Führt Preflight-Checks vor der Einreichung von Agenten-Commits durch.
- **Betroffene Crates / Pfade:** `Agent Submissions`
- **Zeilenzahl:** 771
- **CI-Verdrahtung:** justfile, context-gates.yml, post-merge-verification.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.121 `xtask/src/jules_submit_gate.rs`
- **Zweck:** Finales Gate zur Absicherung von Agenten-Submissions.
- **Betroffene Crates / Pfade:** `PR Submissions`
- **Zeilenzahl:** 486
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.122 `xtask/src/legacy/arch.rs`
- **Zweck:** Legacy-CLI-Adapter für Architektur-Checks.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 71
- **CI-Verdrahtung:** justfile, release-mcp-binary.yml, context-gates.yml, build-wheels.yml, publish-pypi.yml, proof-correctness.yml, bench-regression.yml, simd-cross-validation.yml
- **Tests vorhanden:** Nein
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/arch.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/arch.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.123 `xtask/src/legacy/code.rs`
- **Zweck:** Legacy-CLI-Adapter für Code-Qualitäts-Checks.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 41
- **CI-Verdrahtung:** justfile, scheduled-audit.yml, bench.yml, coverage.yml, gate_check.rs
- **Tests vorhanden:** Ja (harness_codeowners_coverage_test.rs)
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/code.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/code.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.124 `xtask/src/legacy/common.rs`
- **Zweck:** Gemeinsame Hilfsfunktionen für Legacy-Module.
- **Betroffene Crates / Pfade:** `xtask/src/legacy/`
- **Zeilenzahl:** 63
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/core/ / xtask/src/utils`)
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Gemeinsame Hilfsfunktionen für legacy Module; sollte bei Konsolidierung der legacy/ Module nach xtask/src/core/ überführt werden.

### 2.125 `xtask/src/legacy/docs.rs`
- **Zweck:** Legacy-CLI-Adapter für Dokumentations-Checks.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 81
- **CI-Verdrahtung:** justfile, mutation-testing.yml, post-merge-verification.yml, unwrap-baseline-check.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/docs.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/docs.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.126 `xtask/src/legacy/gov.rs`
- **Zweck:** Legacy-CLI-Adapter für Governance-Checks.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 105
- **CI-Verdrahtung:** verdict.yml
- **Tests vorhanden:** Nein
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/gov.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/gov.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.127 `xtask/src/legacy/heavy.rs`
- **Zweck:** Legacy-CLI-Adapter für schwere Benchmark- und Compile-Tests.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 73
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/heavy.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/heavy.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.128 `xtask/src/legacy/mod.rs`
- **Zweck:** Legacy Submodul Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/legacy/`
- **Zeilenzahl:** 9
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Modul-Deklarationsdatei für das legacy Subsystem.

### 2.129 `xtask/src/legacy/ratchet_supply.rs`
- **Zweck:** Legacy-CLI-Adapter für Ratchet- und Supply-Checks.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 53
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/ratchet_supply.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/ratchet_supply.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.130 `xtask/src/legacy/session.rs`
- **Zweck:** Legacy-CLI-Adapter für Session-Management.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 69
- **CI-Verdrahtung:** justfile, bench.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (context_pack_test.rs, harness_jules_dispatch_test.rs, harness_jules_test.rs, harness_plan_lint_test.rs, harness_session_report_test.rs, session_init_test.rs, working_state_stability_test.rs)
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/session.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/session.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.131 `xtask/src/legacy/tags.rs`
- **Zweck:** Legacy-CLI-Adapter für Tag-Health-Checks.
- **Betroffene Crates / Pfade:** `Legacy CLI`
- **Zeilenzahl:** 25
- **CI-Verdrahtung:** justfile, release-mcp-binary.yml, context-gates.yml, build-wheels.yml, jules_preflight.rs
- **Tests vorhanden:** Ja (check_stale_tags_until_test.rs)
- **Klassifikation:** `zusammenführen` (Zusammenführung mit `xtask/src/tags.rs bzw. xtask/src/gates/`)
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Legacy CLI-Adapter in xtask/src/legacy/legacy/tags.rs; sollte mit den jeweiligen Haupt-Gates konsolidiert werden.

### 2.132 `xtask/src/lib.rs`
- **Zweck:** Bibliotheks-Einstiegspunkt für das xtask Crate.
- **Betroffene Crates / Pfade:** `xtask Crate`
- **Zeilenzahl:** 61
- **CI-Verdrahtung:** entrypoint
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.133 `xtask/src/lint_unsafe_slice_bounds.rs`
- **Zweck:** Erkennt ungesicherte Indexzugriffe und Slice-Bounds-Zugriffe.
- **Betroffene Crates / Pfade:** `Source Code Slices`
- **Zeilenzahl:** 341
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.134 `xtask/src/loom_run.rs`
- **Zweck:** Führt Loom Concurrency-Analysen für Lock-Free Datenstrukturen aus.
- **Betroffene Crates / Pfade:** `Loom Tests / Concurrency`
- **Zeilenzahl:** 146
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (loom_run_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.135 `xtask/src/main.rs`
- **Zweck:** Hauptausführungseinstiegspunkt der CLI-Anwendung xtask.
- **Betroffene Crates / Pfade:** `xtask Executable`
- **Zeilenzahl:** 53
- **CI-Verdrahtung:** entrypoint
- **Tests vorhanden:** Ja (main.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.136 `xtask/src/migrate_docid_128.rs`
- **Zweck:** Hilfstool zur Migration von DocId auf 128-Bit Darstellungen.
- **Betroffene Crates / Pfade:** `contextra-types / DocId`
- **Zeilenzahl:** 267
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.137 `xtask/src/panic_inventory.rs`
- **Zweck:** Erstellt ein Inventar aller Panic-Aufrufe und unwraps in der Codebasis.
- **Betroffene Crates / Pfade:** `Workspace Crates`
- **Zeilenzahl:** 269
- **CI-Verdrahtung:** justfile, scheduled-audit.yml, unwrap-baseline-check.yml
- **Tests vorhanden:** Ja (main.rs, panic_inventory_test.rs, workspace_verify_test.rs, inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.138 `xtask/src/post_merge_report.rs`
- **Zweck:** Erzeugt Berichte nach erfolgreichem PR-Merge.
- **Betroffene Crates / Pfade:** `Post-Merge Pipeline`
- **Zeilenzahl:** 124
- **CI-Verdrahtung:** post-merge-verification.yml
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.139 `xtask/src/proof/bench_trend.rs`
- **Zweck:** Analysiert Trendverläufe von Criterion-Benchmarks.
- **Betroffene Crates / Pfade:** `Criterion Outputs`
- **Zeilenzahl:** 23
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.140 `xtask/src/proof/forensic_test.rs`
- **Zweck:** Führt beweisbare Testläufe mit forensischen Nachweisen aus.
- **Betroffene Crates / Pfade:** `Forensic Test Runs`
- **Zeilenzahl:** 25
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.141 `xtask/src/proof/mod.rs`
- **Zweck:** Proof-Submodul Einstiegspunkt.
- **Betroffene Crates / Pfade:** `xtask/src/proof/`
- **Zeilenzahl:** 2
- **CI-Verdrahtung:** justfile, context-gates.yml, rust-ci.yml, gate_check.rs, jules_preflight.rs
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.142 `xtask/src/py_test.rs`
- **Zweck:** Führt Testsuites für Python-Bindings (contextra-py) aus.
- **Betroffene Crates / Pfade:** `contextra-py / PyO3`
- **Zeilenzahl:** 109
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (py_test_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.143 `xtask/src/record_mutation_score.rs`
- **Zweck:** Zeichnet Mutation-Score-Ergebnisse in Metrik-Stores auf.
- **Betroffene Crates / Pfade:** `Mutation Metrics`
- **Zeilenzahl:** 254
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.144 `xtask/src/reproducible_build.rs`
- **Zweck:** Überprüft die Pfad-Unabhängigkeit und Reproduzierbarkeit von Builds.
- **Betroffene Crates / Pfade:** `Cargo Build Artifacts`
- **Zeilenzahl:** 289
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (reproducible_build_path_independent.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.145 `xtask/src/security_scan.rs`
- **Zweck:** Durchsucht den Quellcode nach bekannten Sicherheitslücken und secrets.
- **Betroffene Crates / Pfade:** `Workspace Tree`
- **Zeilenzahl:** 240
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (security_scan_test.rs, workspace_verify_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.146 `xtask/src/session_init.rs`
- **Zweck:** Initialisiert Entwicklungssitzungen und richtet Hooks/Umgebungen ein.
- **Betroffene Crates / Pfade:** `Developer Environment`
- **Zeilenzahl:** 217
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (session_init_test.rs, working_state_stability_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.147 `xtask/src/shell_commit_audit.rs`
- **Zweck:** Auditiert Shell-Skripte und Commit-Hooks auf Sicherheit.
- **Betroffene Crates / Pfade:** `Shell Scripts / Git Hooks`
- **Zeilenzahl:** 184
- **CI-Verdrahtung:** justfile, shell-commit-gate.yml
- **Tests vorhanden:** Ja (commit_health_test.rs, shell_commit_audit_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.148 `xtask/src/tag_health.rs`
- **Zweck:** Überprüft den Zustand, Syntax und Gültigkeit von Source-Code-Tags.
- **Betroffene Crates / Pfade:** `Source Code Tags`
- **Zeilenzahl:** 169
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (tag_health_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.149 `xtask/src/validate_pr_checklist.rs`
- **Zweck:** Validiert die PR-Checkliste vor dem Merge.
- **Betroffene Crates / Pfade:** `PR Checklists / Markdown`
- **Zeilenzahl:** 473
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (inline)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.150 `xtask/src/verify_state_transitions.yml`
- **Zweck:** Ast-Grep Regel zur Validierung von MVCC-Statusübergängen in Async-Funktionen.
- **Betroffene Crates / Pfade:** `contextra-mvcc / Transactions`
- **Zeilenzahl:** 12
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Nein
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Ja
- **Begründung:** Aktives Modul/Gate im Workspace, hat aktuell jedoch weder eine CI-Verdrahtung noch direkte Unit-Tests (Kandidat für "unverdrahtet").

### 2.151 `xtask/src/veto_deadline_gate.rs`
- **Zweck:** Überprüft den Ablauf von eingeräumten Veto-Fristen.
- **Betroffene Crates / Pfade:** `docs/adr/ / Veto Deadlines`
- **Zeilenzahl:** 278
- **CI-Verdrahtung:** Nein
- **Tests vorhanden:** Ja (check_veto_deadlines_scoped_test.rs, veto_deadline_gate_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

### 2.152 `xtask/src/workspace_verify.rs`
- **Zweck:** Führt umfassende Workspace-Integritäts- und Konsistenzprüfungen durch.
- **Betroffene Crates / Pfade:** `Cargo Workspace Root`
- **Zeilenzahl:** 497
- **CI-Verdrahtung:** justfile
- **Tests vorhanden:** Ja (workspace_verify_test.rs)
- **Klassifikation:** `behalten`
- **Unverdrahtet-Kandidat:** Nein
- **Begründung:** Aktives Gate oder Kerninfrastruktur im Workspace mit CI-Anbindung und/oder Testabdeckung.

---

## 3. Zusammenfassender Bericht & Klassifikations-Statistik

### 3.1 Anzahl der Module je Klassifikation

- **behalten (keep):** 142 Module
- **zusammenführen (merge):** 10 Module
- **entfernen (remove):** 0 Module (kein Modul prüft ausschließlich archivierte Crates)
- **Gesamtzahl analysierter Module:** 152 Module

### 3.2 Liste der unverdrahteten Module ("unverdrahtet")

Die folgenden 14 Module weisen weder CI-Anbindung (.github/workflows/, justfile, gate_check.rs, jules_preflight.rs) noch dedizierte Unit-/Integrationstests auf. Gemäß Vorgabe werden sie als "unverdrahtet" markiert, jedoch **nicht** entfernt:

1. `xtask/src/agent_lifecycle/prune_branches.rs` - Bereinigt veraltete oder gemergte Remote-Branches.
1. `xtask/src/check_orphan_modules.rs` - Legacy-Alias für check_module_reachability zur Erkennung verwaister Module.
1. `xtask/src/core/constitution.rs` - Lädt und validiert globale Projekt-Regeln und Metadaten-Konfigurationen.
1. `xtask/src/core/ctx.rs` - Stellt Ausführungskontext und Verzeichnis-Discovery für xtask bereit.
1. `xtask/src/gates/gov/meta_gates.rs` - Validiert Meta-Regeln und Selbsttests der Gates.
1. `xtask/src/gen_feature_catalog.rs` - Erstellt Markdown-Katalog aller Cargo Feature-Flags.
1. `xtask/src/generate_diagnostics.rs` - Sammelt Diagnosedaten und schreibt strukturierten Gate-Bericht.
1. `xtask/src/generators/sbom.rs` - Generiert Software Bill of Materials (SBOM) im CycloneDX Format.
1. `xtask/src/legacy/common.rs` - Gemeinsame Hilfsfunktionen für Legacy-Module.
1. `xtask/src/legacy/heavy.rs` - Legacy-CLI-Adapter für schwere Benchmark- und Compile-Tests.
1. `xtask/src/legacy/ratchet_supply.rs` - Legacy-CLI-Adapter für Ratchet- und Supply-Checks.
1. `xtask/src/proof/bench_trend.rs` - Analysiert Trendverläufe von Criterion-Benchmarks.
1. `xtask/src/proof/forensic_test.rs` - Führt beweisbare Testläufe mit forensischen Nachweisen aus.
1. `xtask/src/verify_state_transitions.yml` - Ast-Grep Regel zur Validierung von MVCC-Statusübergängen in Async-Funktionen.

---

## 4. Abnahme & Prüfkriterien Verification

1. **Schutzpfad-Konsistenz (`xtask/**`):** `git diff --stat -- xtask/` ist vollkommen leer.
2. **Vollständigkeit:** Die Anzahl der im Inventar erfassten Module (152) stimmt exakt mit der Anzahl der Dateien unter `xtask/src/` (152) überein.
3. **REQ-QA-010 Konformität:** Es wurden keine Gates geschwächt oder entfernt. Jede Klassifikation besitzt eine explizite Begründung.