# Gate-Zuordnung (`docs/status/GATE-ZUORDNUNG.md`)

*Soll-Architektur & Qualitätssicherung gemäß Contextra Spezifikation 1.2 (§27, §28, §33)*

> **Hinweis zur Repo-Struktur**: Das vormals getrennte Crate `xtask-heavy` wurde im Rahmen der Konsolidierung aufgelöst; sämtliche Gates (einschließlich aller heavy/long-running Gates) sind nun in `xtask/src/` (inkl. `xtask/src/harness/`, `xtask/src/gates/`, `xtask/src/proof/`) angesiedelt und inventarisiert.

Die folgende Tabelle führt alle in `xtask/src/` vorhandenen Gates und Verifikationskommandos auf und ordnet diese den entsprechenden Anforderungen aus §33 (Anforderungskatalog) der Spezifikation 1.2 zu.

| Gate-Datei | Befehl | Zugeordnete REQ-IDs (§33) | Mechanismus | Status |
|---|---|---|:---:|:---:|
| `xtask/src/agent_lifecycle/branch_overlap.rs` | `cargo xtask check-branch-overlap` | REQ-GOV-001 | G | belegt |
| `xtask/src/agent_lifecycle/prune_branches.rs` | `cargo xtask prune-branches` | REQ-GOV-001 | G | belegt |
| `xtask/src/audit_integrity_check.rs` | `cargo xtask audit-integrity-check` | REQ-QA-001 | G | belegt |
| `xtask/src/bench_compile.rs` | `cargo xtask bench-compile` | REQ-QA-007 | G | belegt |
| `xtask/src/bench_download.rs` | `cargo xtask bench-download` | REQ-QA-007 | G | belegt |
| `xtask/src/bench_gate.rs` | `cargo xtask bench-gate` | REQ-QA-007 | G | belegt |
| `xtask/src/check_action_pinning.rs` | `cargo xtask check-action-pinning` | REQ-GOV-006 | G | belegt |
| `xtask/src/check_adr_deadlines.rs` | `cargo xtask check-adr-deadlines` | REQ-GOV-003 | G | belegt |
| `xtask/src/check_agents_freshness.rs` | `cargo xtask check-agents-freshness` | REQ-ARCH-011, REQ-GOV-004 | G | belegt |
| `xtask/src/check_agents_integrity.rs` | `cargo xtask check-agents-integrity` | REQ-ARCH-011, REQ-GOV-004 | G | belegt |
| `xtask/src/check_audit_duplication.rs` | `cargo xtask check-audit-duplication` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_audit_tool_evidence.rs` | `cargo xtask check-audit-tool-evidence` | REQ-QA-001 | G | belegt |
| `xtask/src/check_audit_verdict_independence.rs` | `cargo xtask check-audit-verdict-independence` | REQ-QA-001 | G | belegt |
| `xtask/src/check_bandit_latency_budget.rs` | `cargo xtask check-bandit-latency-budget` | REQ-EXP-001 | G | belegt |
| `xtask/src/check_commit_diff_integrity.rs` | `cargo xtask check-commit-diff-integrity` | REQ-GOV-001 | G | belegt |
| `xtask/src/check_commit_messages.rs` | `cargo xtask check-commit-messages` | REQ-GOV-001 | G | belegt |
| `xtask/src/check_compile.rs` | `cargo xtask check-compile` | REQ-ARCH-006 | G | belegt |
| `xtask/src/check_coverage_gate.rs` | `cargo xtask check-coverage-gate` | REQ-QA-001 | G | belegt |
| `xtask/src/check_doc_references.rs` | `cargo xtask check-doc-references` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_duplicate_core_primitives.rs` | `cargo xtask check-duplicate-core-primitives` | REQ-ARCH-013, REQ-DEL-027 | G | belegt |
| `xtask/src/check_duplicate_intent.rs` | `cargo xtask check-duplicate-intent` | REQ-GOV-001 | G | belegt |
| `xtask/src/check_duplicate_symbols.rs` | `cargo xtask check-duplicate-symbols` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_duplicate_symbols_cross_file.rs` | `cargo xtask check-duplicate-symbols-cross-file` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_ffi_panic_boundary.rs` | `cargo xtask check-ffi-panic-boundary` | REQ-ARCH-006 | G | belegt |
| `xtask/src/check_flatbuffers_drift.rs` | `cargo xtask check-flatbuffers-drift` | REQ-DUR-005 | G | belegt |
| `xtask/src/check_jules_context_freshness.rs` | `cargo xtask check-jules-context-freshness` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_manifest_completeness.rs` | `cargo xtask check-manifest-completeness` | REQ-ARCH-020 | G | belegt |
| `xtask/src/check_max_results_unbound.rs` | `cargo xtask check-max-results-unbound` | REQ-ARCH-006 | G | belegt |
| `xtask/src/check_module_reachability.rs` | `cargo xtask check-module-reachability` | REQ-ARCH-021 | G | belegt |
| `xtask/src/check_mutation_score_gate.rs` | `cargo xtask check-mutation-score-gate` | REQ-QA-005 | G | belegt |
| `xtask/src/check_nan_validation_in_hot_loop.rs` | `cargo xtask check-nan-hot-loop` | REQ-ARCH-006, REQ-ARCH-015 | G | belegt |
| `xtask/src/check_orphan_modules.rs` | `cargo xtask check-orphan-modules` | REQ-ARCH-021 | G | belegt |
| `xtask/src/check_phantom_files.rs` | `cargo xtask check-phantom-files` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_placeholder_refs.rs` | `cargo xtask check-placeholder-refs` | REQ-IDX-011, REQ-FAC-003 | G | belegt |
| `xtask/src/check_recall_stability.rs` | `cargo xtask check-recall-stability` | REQ-QA-007 | G | belegt |
| `xtask/src/check_result_dropped_on_io.rs` | `cargo xtask check-result-dropped-io` | REQ-DUR-004 | G | belegt |
| `xtask/src/check_ring0_async_purity.rs` | `cargo xtask check-ring0-async-purity` | REQ-ARCH-008 | G | belegt |
| `xtask/src/check_ring_capabilities_consistency.rs` | `cargo xtask check-ring-capabilities-consistency` | REQ-ARCH-018 | G | belegt |
| `xtask/src/check_ring_layering.rs` | `cargo xtask check-ring-layering` | REQ-ARCH-001, REQ-ARCH-002, REQ-ARCH-018 | G | belegt |
| `xtask/src/check_stale_tags.rs` | `cargo xtask check-stale-tags` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_toc_integrity.rs` | `cargo xtask check-toc-integrity` | REQ-GOV-004 | G | belegt |
| `xtask/src/check_toctou_trait_defaults.rs` | `cargo xtask check-toctou-defaults` | REQ-MVCC-004 | G | belegt |
| `xtask/src/check_type_registry.rs` | `cargo xtask check-type-registry` | REQ-DEL-027 | G | belegt |
| `xtask/src/check_unsafe_islands.rs` | `cargo xtask check-unsafe-islands` | REQ-ARCH-004, REQ-ARCH-005 | G | belegt |
| `xtask/src/check_vetoes.rs` | `cargo xtask check-vetoes` | REQ-IDX-003 | G | belegt |
| `xtask/src/check_workflow_commands.rs` | `cargo xtask check-workflow-commands` | REQ-GOV-001 | G | belegt |
| `xtask/src/claim.rs` | `cargo xtask claim` | REQ-GOV-001 | G | belegt |
| `xtask/src/commit_health.rs` | `cargo xtask commit-health` | REQ-GOV-001 | G | belegt |
| `xtask/src/context_pack.rs` | `cargo xtask context-pack` | REQ-GOV-004 | G | belegt |
| `xtask/src/crate_context.rs` | `cargo xtask crate-context` | REQ-GOV-004 | G | belegt |
| `xtask/src/env_validate.rs` | `cargo xtask env-validate` | REQ-QA-009 | G | belegt |
| `xtask/src/feature_matrix.rs` | `cargo xtask feature-matrix` | REQ-ARCH-012, REQ-ARCH-020 | G | belegt |
| `xtask/src/gate_check.rs` | `cargo xtask gate-check` | REQ-QA-001 | G | belegt |
| `xtask/src/gates/check_crate_references.rs` | `cargo xtask check-crate-references` | REQ-GOV-004 | G | belegt |
| `xtask/src/gates/debt_audit.rs` | `cargo xtask debt-audit` | REQ-ARCH-006, REQ-GOV-004 | G | belegt |
| `xtask/src/gen_arch_docs.rs` | `cargo xtask gen-arch-docs` | REQ-ARCH-011 | G | belegt |
| `xtask/src/gen_feature_catalog.rs` | `cargo xtask gen-feature-catalog` | REQ-ARCH-012, REQ-ARCH-020 | G | belegt |
| `xtask/src/gen_prompter_data.rs` | `cargo xtask gen-prompter-data` | REQ-GOV-004 | G | belegt |
| `xtask/src/generate_adr.rs` | `cargo xtask generate-adr` | REQ-GOV-003 | G | belegt |
| `xtask/src/generate_diagnostics.rs` | `cargo xtask generate-diagnostics` | REQ-QA-001 | G | belegt |
| `xtask/src/generate_markers.rs` | `cargo xtask generate-markers` | REQ-ARCH-011 | G | belegt |
| `xtask/src/generators/sbom.rs` | `cargo xtask gen-sbom` | REQ-GOV-006 | G | belegt |
| `xtask/src/harness/blast_radius.rs` | `cargo xtask blast-radius` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/can_merge.rs` | `cargo xtask can-merge` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/claims_in_pr.rs` | `cargo xtask claims-in-pr` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/codeowners_coverage.rs` | `cargo xtask codeowners-coverage` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/dependency_gate.rs` | `cargo xtask dependency-gate` | REQ-GOV-006 | G | belegt |
| `xtask/src/harness/determinism_check.rs` | `cargo xtask gate-determinism-check` | REQ-ARCH-009, REQ-ARCH-015 | G | belegt |
| `xtask/src/harness/diff_budget.rs` | `cargo xtask diff-budget` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/doc_truth.rs` | `cargo xtask doc-truth` | REQ-ARCH-011, REQ-ARCH-023, REQ-GOV-004 | G | belegt |
| `xtask/src/harness/env_attest.rs` | `cargo xtask env-attest` | REQ-QA-009 | G | belegt |
| `xtask/src/harness/explain.rs` | `cargo xtask explain` | REQ-GOV-004 | G | belegt |
| `xtask/src/harness/flake_report.rs` | `cargo xtask flake-report` | REQ-QA-001 | G | belegt |
| `xtask/src/harness/fuzz_smoke.rs` | `cargo xtask fuzz-smoke` | REQ-QA-003, REQ-QA-004 | G | belegt |
| `xtask/src/harness/gate_weakening.rs` | `cargo xtask gate-weakening` | REQ-QA-010 | G | belegt |
| `xtask/src/harness/gates_catalog_sync.rs` | `cargo xtask gates-catalog-sync` | REQ-QA-008, REQ-GOV-004 | G | belegt |
| `xtask/src/harness/jules.rs` | `cargo xtask jules` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/jules_dispatch.rs` | `cargo xtask jules-dispatch` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/ledger.rs` | `cargo xtask ledger` | REQ-DEL-015 | G | belegt |
| `xtask/src/harness/lock_audit.rs` | `cargo xtask gate-lock-audit` | REQ-MVCC-004 | G | belegt |
| `xtask/src/harness/loop_guard.rs` | `cargo xtask loop-guard` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/mutants_diff.rs` | `cargo xtask mutants-diff` | REQ-QA-005 | G | belegt |
| `xtask/src/harness/plan_lint.rs` | `cargo xtask plan-lint` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/postmortem.rs` | `cargo xtask postmortem` | REQ-QA-001 | G | belegt |
| `xtask/src/harness/protected_paths.rs` | `cargo xtask protected-paths` | REQ-GOV-002 | G | belegt |
| `xtask/src/harness/ratchet.rs` | `cargo xtask ratchet` | REQ-ARCH-006, REQ-QA-005 | G | belegt |
| `xtask/src/harness/registry_check.rs` | `cargo xtask registry-check` | REQ-DEL-027, REQ-KEY-011 | G | belegt |
| `xtask/src/harness/risk.rs` | `cargo xtask risk` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/scope_guard.rs` | `cargo xtask scope-guard` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/session_report.rs` | `cargo xtask session-report` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/symbol_exists.rs` | `cargo xtask symbol-exists` | REQ-GOV-004 | G | belegt |
| `xtask/src/harness/task_card.rs` | `cargo xtask task-card` | REQ-GOV-001 | G | belegt |
| `xtask/src/harness/test_integrity.rs` | `cargo xtask test-integrity` | REQ-QA-010 | G | belegt |
| `xtask/src/harness/unsafe_audit.rs` | `cargo xtask gate-unsafe-audit` | REQ-ARCH-004, REQ-ARCH-005 | G | belegt |
| `xtask/src/harness/verdict.rs` | `cargo xtask verdict` | REQ-QA-001 | G | belegt |
| `xtask/src/harness/wal_replay_verify.rs` | `cargo xtask gate-wal-replay-verify` | REQ-DUR-016 | G | belegt |
| `xtask/src/hotspot_report.rs` | `cargo xtask hotspot-report` | REQ-GOV-001 | G | belegt |
| `xtask/src/init_audit_fix.rs` | `cargo xtask init-audit-fix` | REQ-GOV-001 | G | belegt |
| `xtask/src/jules_preflight.rs` | `cargo xtask jules-preflight` | REQ-QA-001 | G | belegt |
| `xtask/src/jules_submit_gate.rs` | `cargo xtask jules-submit-gate` | REQ-GOV-001 | G | belegt |
| `xtask/src/lint_unsafe_slice_bounds.rs` | `cargo xtask lint-unsafe-slices` | REQ-ARCH-004, REQ-ARCH-005 | G | belegt |
| `xtask/src/loom_run.rs` | `cargo xtask loom-run` | REQ-MVCC-005 | G | belegt |
| `xtask/src/migrate_docid_128.rs` | `cargo xtask migrate-docid-128` | ohne Anforderung | G | ohne Anforderung |
| `xtask/src/panic_inventory.rs` | `cargo xtask panic-inventory` | REQ-ARCH-006 | G | belegt |
| `xtask/src/post_merge_report.rs` | `cargo xtask post-merge-report` | REQ-GOV-001 | G | belegt |
| `xtask/src/proof/bench_trend.rs` | `cargo xtask bench-trend` | REQ-QA-007 | G | belegt |
| `xtask/src/proof/forensic_test.rs` | `cargo xtask forensic-test` | REQ-QA-001 | G | belegt |
| `xtask/src/py_test.rs` | `cargo xtask py-test` | REQ-QA-001 | G | belegt |
| `xtask/src/record_mutation_score.rs` | `cargo xtask mutation-score-record` | REQ-QA-005 | G | belegt |
| `xtask/src/reproducible_build.rs` | `cargo xtask reproducible-build` | REQ-QA-009 | G | belegt |
| `xtask/src/security_scan.rs` | `cargo xtask security-scan` | REQ-KEY-010, REQ-AUD-002 | G | belegt |
| `xtask/src/session_history.rs` | `cargo xtask session-history` | REQ-GOV-001 | G | belegt |
| `xtask/src/session_init.rs` | `cargo xtask session-init` | REQ-GOV-001 | G | belegt |
| `xtask/src/shell_commit_audit.rs` | `cargo xtask shell-commit-audit` | REQ-GOV-001 | G | belegt |
| `xtask/src/tag_health.rs` | `cargo xtask tag-health` | REQ-GOV-004 | G | belegt |
| `xtask/src/validate_pr_checklist.rs` | `cargo xtask validate-pr-checklist` | REQ-GOV-001 | G | belegt |
| `xtask/src/veto_deadline_gate.rs` | `cargo xtask check-veto-deadlines` | REQ-IDX-003 | G | belegt |
| `xtask/src/workspace_verify.rs` | `cargo xtask workspace-verify` | REQ-QA-001 | G | belegt |
