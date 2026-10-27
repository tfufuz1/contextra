# Audit-Bericht: Governance-Pfade & Pfadspaltung

**Status:** PR 1 (Audit)
**Normativ:** SPEC §18.7, INV-DOC-SINGLE-SOURCE-1
**Datum:** 2026-10-27

---

## 1. Übersicht & Motivation

Nach der Umstrukturierung 332c26958 (`.jules/` → `.agents/`) entstand eine Pfadspaltung bezüglich Governance- und Harness-Konfigurationsdateien.
Dieser Bericht erfasst systematisch den IST-Zustand aller Dateien in `governance/` und `.agents/governance/` sowie sämtlicher Referenzen in `xtask/`, `justfile`, `.github/`, `AGENTS.md`, `CLAUDE.md`, `docs/` und `capabilities.toml`.

---

### Tabelle A: Dateien unter `governance/` und `.agents/governance/`

| Dateiname | governance/ (Größe, Last Commit) | .agents/governance/ (Größe, Last Commit) | Diff-Befund |
|---|---|---|---|
| `README.md` | 2834 B (`unknown`) | 1754 B (`unknown`) | **Abweichend** |
| `clippy.toml` | 2411 B (`unknown`) | fehlt | Nur in `governance/` |
| `constitution.toml` | fehlt | 28681 B (`unknown`) | Nur in `.agents/governance/` |
| `determinism-allow.toml` | 998 B (`unknown`) | 722 B (`unknown`) | **Abweichend** |
| `determinism-baseline.toml` | fehlt | 14 B (`unknown`) | Nur in `.agents/governance/` |
| `gates.toml` | 12699 B (`unknown`) | 8812 B (`unknown`) | **Abweichend** |
| `key_purposes.toml` | 3446 B (`unknown`) | fehlt | Nur in `governance/` |
| `protected-paths.toml` | 3293 B (`unknown`) | 2841 B (`unknown`) | **Abweichend** |
| `ratchet.toml` | 1909 B (`unknown`) | 223 B (`unknown`) | **Abweichend** |
| `security_ports.toml` | 4145 B (`unknown`) | fehlt | Nur in `governance/` |
| `unsafe-safety-baseline.toml` | fehlt | 361 B (`unknown`) | Nur in `.agents/governance/` |
| `verdict-required.toml` | fehlt | 1448 B (`unknown`) | Nur in `.agents/governance/` |

### Tabelle B: Pfadangaben in Konfigurations- und Dokumentationsdateien

| Fundstelle (Datei:Zeile) | Referenzierter Pfad | Existiert? |
|---|---|---|
| `.github/workflows/context-gates.yml:131` | `.jules/prompter-data.json` | **nein** |
| `.github/workflows/flake-report.yml:39` | `.jules/local` | **nein** |
| `.github/workflows/flake-report.yml:46` | `.jules/local` | **nein** |
| `.github/workflows/flake-report.yml:49` | `.jules/local` | **nein** |
| `.github/workflows/scheduled-audit.yml:29` | `.jules/tasks` | **nein** |
| `.github/workflows/scheduled-audit.yml:40` | `.jules/tasks` | **nein** |
| `.github/workflows/scheduled-audit.yml:60` | `.jules/setup` | **nein** |
| `.github/workflows/scheduled-audit.yml:71` | `.jules/AUDIT_INTAKE_PROTOCOL.md` | **nein** |
| `.github/workflows/scheduled-audit.yml:95` | `.jules/AUDIT_INTAKE_PROTOCOL.md` | **nein** |
| `.github/workflows/verdict.yml:464` | `governance/verdict-required.toml` | **nein** |
| `AGENTS.md:17` | `governance/gates.toml` | ja |
| `AGENTS.md:89` | `governance/` | ja |
| `AGENTS.md:263` | `governance/gates.toml` | ja |
| `AGENTS.md:264` | `governance/ratchet.toml` | ja |
| `AGENTS.md:265` | `governance/security_ports.toml` | ja |
| `AGENTS.md:266` | `governance/key_purposes.toml` | ja |
| `AGENTS.md:267` | `governance/clippy.toml` | ja |
| `AGENTS.md:268` | `governance/determinism-allow.toml` | ja |
| `AGENTS.md:269` | `governance/protected-paths.toml` | ja |
| `CLAUDE.md:7` | `governance/gates.toml` | ja |
| `CLAUDE.md:48` | `governance/gates.toml` | ja |
| `CLAUDE.md:84` | `governance/gates.toml` | ja |
| `CLAUDE.md:85` | `governance/ratchet.toml` | ja |
| `CLAUDE.md:86` | `governance/security_ports.toml` | ja |
| `CLAUDE.md:87` | `governance/key_purposes.toml` | ja |
| `CLAUDE.md:88` | `governance/clippy.toml` | ja |
| `CLAUDE.md:89` | `governance/determinism-allow.toml` | ja |
| `CLAUDE.md:90` | `governance/protected-paths.toml` | ja |
| `CLAUDE.md:91` | `governance/README.md` | ja |
| `docs/PLAN.md:42` | `governance/gates.toml` | ja |
| `docs/PLAN.md:74` | `governance/security_ports.toml` | ja |
| `docs/PLAN.md:74` | `governance/key_purposes.toml` | ja |
| `docs/PLAN.md:152` | `governance` | ja |
| `docs/PLAN.md:153` | `governance` | ja |
| `docs/PLAN.md:163` | `governance` | ja |
| `docs/PLAN.md:272` | `governance/security_ports.toml` | ja |
| `docs/PLAN.md:273` | `governance/key_purposes.toml` | ja |
| `docs/PLAN.md:274` | `governance/determinism-allow.toml` | ja |
| `docs/PLAN.md:275` | `governance/ratchet.toml` | ja |
| `docs/PLAN.md:276` | `governance/gates.toml` | ja |
| `docs/PLAN.md:441` | `governance/security_ports.toml` | ja |
| `docs/PLAN.md:490` | `governance/ratchet.toml` | ja |
| `docs/PLAN.md:586` | `governance/key_purposes.toml` | ja |
| `docs/PLAN.md:595` | `governance/key_purposes.toml` | ja |
| `docs/PLAN.md:980` | `governance/gates.toml` | ja |
| `docs/SOURCE_OF_TRUTH.md:40` | `governance/gates.toml` | ja |
| `docs/SOURCE_OF_TRUTH.md:41` | `governance/ratchet.toml` | ja |
| `docs/SPEC.md:92` | `governance/` | ja |
| `docs/SPEC.md:183` | `governance/ratchet.toml` | ja |
| `docs/SPEC.md:263` | `governance/security_ports.toml` | ja |
| `docs/SPEC.md:462` | `governance/security_ports.toml` | ja |
| `docs/SPEC.md:1064` | `governance/key_purposes.toml` | ja |
| `docs/SPEC.md:1855` | `governance/ratchet.toml` | ja |
| `docs/SPEC.md:1864` | `governance/gates.toml` | ja |
| `docs/SPEC.md:1876` | `governance/security_ports.toml` | ja |
| `docs/SPEC.md:1877` | `governance/key_purposes.toml` | ja |
| `docs/SPEC.md:1980` | `governance/protected-paths.toml` | ja |
| `docs/SPEC.md:2032` | `governance/constitution.toml` | **nein** |
| `docs/SPEC.md:2108` | `governance/security_ports.toml` | ja |
| `docs/SPEC.md:2249` | `governance/ratchet.toml` | ja |
| `docs/SPEC.md:2343` | `governance/security_ports.toml` | ja |
| `docs/SPEC.md:2343` | `governance/key_purposes.toml` | ja |
| `docs/SPEC.md:2379` | `governance/key_purposes.toml` | ja |
| `docs/SPEC.md:2505` | `governance/security_ports.toml` | ja |
| `docs/SPEC.md:2505` | `governance/key_purposes.toml` | ja |
| `docs/git-help/04_create_issues.zsh:131` | `governance/ratchet.toml` | ja |
| `docs/git-help/04_create_issues.zsh:134` | `governance/ratchet.toml` | ja |
| `docs/git-help/04_create_issues.zsh:166` | `governance/` | ja |
| `docs/git-help/04_create_issues.zsh:186` | `governance/security_ports.toml` | ja |
| `docs/git-help/04_create_issues.zsh:187` | `governance/key_purposes.toml` | ja |
| `docs/git-help/04_create_issues.zsh:188` | `governance/clippy.toml` | ja |
| `docs/git-help/04_create_issues.zsh:189` | `governance/gates.toml` | ja |
| `docs/git-help/04_create_issues.zsh:190` | `governance/ratchet.toml` | ja |
| `docs/git-help/04_create_issues.zsh:191` | `governance/protected-paths.toml` | ja |
| `docs/git-help/04_create_issues.zsh:192` | `governance/determinism-allow.toml` | ja |
| `docs/git-help/04_create_issues.zsh:196` | `governance/` | ja |
| `docs/git-help/04_create_issues.zsh:454` | `governance/` | ja |
| `docs/spec/GOVERNANCE-PATHS.md:11` | `.jules/` | **nein** |
| `docs/spec/GOVERNANCE-PATHS.md:12` | `governance/` | ja |
| `docs/spec/GOVERNANCE-PATHS.md:12` | `.agents/governance/` | ja |
| `justfile:36` | `governance` | ja |
| `justfile:40` | `governance/gates.toml` | ja |
| `xtask/src/agent_lifecycle/branch_overlap.rs:99` | `.jules/tasks` | **nein** |
| `xtask/src/agent_lifecycle/branch_overlap.rs:235` | `.jules/SESSION.md` | **nein** |
| `xtask/src/agent_lifecycle/branch_overlap.rs:251` | `.jules/claims.json` | **nein** |
| `xtask/src/check_doc_references.rs:16` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/src/check_doc_references.rs:82` | `.jules/` | **nein** |
| `xtask/src/check_jules_context_freshness.rs:111` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/src/claim.rs:123` | `.jules/claims.json` | **nein** |
| `xtask/src/claim.rs:127` | `.jules/claims.json` | **nein** |
| `xtask/src/claim.rs:157` | `.jules/SESSION.md` | **nein** |
| `xtask/src/claim.rs:172` | `.jules/SESSION.md` | **nein** |
| `xtask/src/claim.rs:218` | `.jules/claims.json` | **nein** |
| `xtask/src/claim.rs:292` | `.jules/claims.json` | **nein** |
| `xtask/src/claim.rs:603` | `.jules/claims.json` | **nein** |
| `xtask/src/claim.rs:658` | `.jules` | **nein** |
| `xtask/src/claim.rs:759` | `.jules/claims.json` | **nein** |
| `xtask/src/claim.rs:833` | `.jules/SESSION.md` | **nein** |
| `xtask/src/context_pack.rs:94` | `.jules/claims.json` | **nein** |
| `xtask/src/core/cmd.rs:24` | `governance/gates.toml` | ja |
| `xtask/src/core/cmd.rs:147` | `governance/gates.toml` | ja |
| `xtask/src/core/cmd.rs:201` | `governance/gates.toml` | ja |
| `xtask/src/gates/gov/meta_gates.rs:73` | `governance/gates.toml` | ja |
| `xtask/src/gen_prompter_data.rs:170` | `.jules/prompter-tiers.toml` | **nein** |
| `xtask/src/gen_prompter_data.rs:171` | `.jules/prompter-tiers.toml` | **nein** |
| `xtask/src/gen_prompter_data.rs:351` | `.jules/prompter-data.json` | **nein** |
| `xtask/src/harness/codeowners_coverage.rs:62` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/codeowners_coverage.rs:63` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/codeowners_coverage.rs:75` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/codeowners_coverage.rs:78` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/codeowners_coverage.rs:87` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/codeowners_coverage.rs:90` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/codeowners_coverage.rs:154` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/determinism_check.rs:273` | `governance/determinism-allow.toml` | ja |
| `xtask/src/harness/determinism_check.rs:324` | `governance/determinism-baseline.toml` | **nein** |
| `xtask/src/harness/determinism_check.rs:346` | `governance` | ja |
| `xtask/src/harness/diff_budget.rs:292` | `.jules/tasks` | **nein** |
| `xtask/src/harness/doc_truth.rs:131` | `governance` | ja |
| `xtask/src/harness/doc_truth.rs:137` | `.jules/SESSION_BOOTSTRAP.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:138` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:139` | `.jules/COMMON_LLM_ERRORS.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:140` | `.jules/SCHEDULED_AUDIT.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:141` | `.jules/AUDIT_INTAKE_PROTOCOL.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:142` | `.jules/ISSUE_AUTOMATION_POLICY.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:660` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:662` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:696` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/src/harness/doc_truth.rs:715` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/src/harness/env_attest.rs:60` | `.jules/setup` | **nein** |
| `xtask/src/harness/env_attest.rs:198` | `.jules/setup` | **nein** |
| `xtask/src/harness/env_attest.rs:210` | `.jules/setup` | **nein** |
| `xtask/src/harness/env_attest.rs:228` | `.jules/setup` | **nein** |
| `xtask/src/harness/env_attest.rs:231` | `.jules/setup` | **nein** |
| `xtask/src/harness/env_attest.rs:256` | `.jules/local` | **nein** |
| `xtask/src/harness/env_attest.rs:257` | `.jules/local` | **nein** |
| `xtask/src/harness/flake_report.rs:106` | `.jules/local` | **nein** |
| `xtask/src/harness/gate_weakening.rs:361` | `governance/ratchet.toml` | ja |
| `xtask/src/harness/gate_weakening.rs:367` | `governance/ratchet.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:1` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:1` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:1` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:82` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:83` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:91` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:93` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:94` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:103` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:105` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:106` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:112` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:113` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:116` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:117` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:131` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:139` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:158` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:165` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:184` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:192` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:206` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:209` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:210` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:220` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:223` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:224` | `governance/gates.toml` | ja |
| `xtask/src/harness/gates_catalog_sync.rs:266` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:269` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:270` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:281` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:284` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:285` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:328` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:332` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:334` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:345` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:349` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/gates_catalog_sync.rs:351` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/harness/jules.rs:88` | `.jules/harness-phases.toml` | **nein** |
| `xtask/src/harness/jules_dispatch.rs:77` | `.jules/jules-api.toml` | **nein** |
| `xtask/src/harness/jules_dispatch.rs:83` | `.jules/jules-api.toml` | **nein** |
| `xtask/src/harness/jules_dispatch.rs:87` | `.jules/PREAMBLE.md` | **nein** |
| `xtask/src/harness/jules_dispatch.rs:193` | `.jules/local` | **nein** |
| `xtask/src/harness/loop_guard.rs:86` | `.jules/local` | **nein** |
| `xtask/src/harness/loop_guard.rs:223` | `.jules/local` | **nein** |
| `xtask/src/harness/loop_guard.rs:408` | `.jules/local` | **nein** |
| `xtask/src/harness/loop_guard.rs:431` | `.jules/local` | **nein** |
| `xtask/src/harness/loop_guard.rs:450` | `.jules/local` | **nein** |
| `xtask/src/harness/loop_guard.rs:481` | `.jules/local` | **nein** |
| `xtask/src/harness/mutants_diff.rs:31` | `.jules/prompter-tiers.toml` | **nein** |
| `xtask/src/harness/plan_lint.rs:257` | `.jules/local` | **nein** |
| `xtask/src/harness/protected_paths.rs:457` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/protected_paths.rs:523` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/protected_paths.rs:537` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/protected_paths.rs:548` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/protected_paths.rs:551` | `governance/protected-paths.toml` | ja |
| `xtask/src/harness/ratchet.rs:259` | `governance/ratchet.toml` | ja |
| `xtask/src/harness/ratchet.rs:721` | `governance/ratchet.toml` | ja |
| `xtask/src/harness/scope_guard.rs:88` | `.jules/tasks` | **nein** |
| `xtask/src/harness/scope_guard.rs:206` | `.jules/tasks` | **nein** |
| `xtask/src/harness/scope_guard.rs:222` | `.jules/tasks` | **nein** |
| `xtask/src/harness/session_report.rs:280` | `.jules/local` | **nein** |
| `xtask/src/harness/session_report.rs:294` | `.jules/local` | **nein** |
| `xtask/src/harness/task_card.rs:77` | `.jules/setup` | **nein** |
| `xtask/src/harness/task_card.rs:82` | `.jules/tasks` | **nein** |
| `xtask/src/harness/task_card.rs:103` | `.jules/tasks` | **nein** |
| `xtask/src/harness/task_card.rs:115` | `.jules/setup` | **nein** |
| `xtask/src/harness/task_card.rs:246` | `.jules/prompter-tiers.toml` | **nein** |
| `xtask/src/harness/task_card.rs:434` | `.jules/tasks` | **nein** |
| `xtask/src/harness/unsafe_audit.rs:253` | `governance/unsafe-safety-baseline.toml` | **nein** |
| `xtask/src/harness/unsafe_audit.rs:266` | `governance` | ja |
| `xtask/src/harness/unsafe_audit.rs:297` | `governance/unsafe-safety-baseline.toml` | **nein** |
| `xtask/src/harness/verdict.rs:58` | `governance/verdict-required.toml` | **nein** |
| `xtask/src/init_audit_fix.rs:123` | `.jules/AUDIT_INTAKE_PROTOCOL.md` | **nein** |
| `xtask/src/init_audit_fix.rs:137` | `.jules/AUDIT_INTAKE_PROTOCOL.md` | **nein** |
| `xtask/src/jules_preflight.rs:151` | `.jules/claims.json` | **nein** |
| `xtask/src/jules_preflight.rs:696` | `.jules` | **nein** |
| `xtask/src/jules_preflight.rs:725` | `.jules` | **nein** |
| `xtask/src/jules_submit_gate.rs:29` | `.jules` | **nein** |
| `xtask/src/jules_submit_gate.rs:130` | `.jules/gate_ledger.json` | **nein** |
| `xtask/src/main.rs:12` | `governance/gates.toml` | ja |
| `xtask/src/main.rs:25` | `governance/gates.toml` | ja |
| `xtask/src/main.rs:42` | `governance/gates.toml` | ja |
| `xtask/src/main.rs:78` | `governance/gates.toml` | ja |
| `xtask/src/session_init.rs:75` | `.jules/prompter-data.json` | **nein** |
| `xtask/src/session_init.rs:126` | `.jules/claims.json` | **nein** |
| `xtask/src/session_init.rs:179` | `.jules/session.env` | **nein** |
| `xtask/tests/branch_overlap_test.rs:119` | `.jules/tasks` | **nein** |
| `xtask/tests/branch_overlap_test.rs:121` | `.jules/tasks` | **nein** |
| `xtask/tests/branch_overlap_test.rs:132` | `.jules/tasks` | **nein** |
| `xtask/tests/branch_overlap_test.rs:183` | `.jules/tasks` | **nein** |
| `xtask/tests/branch_overlap_test.rs:185` | `.jules/tasks` | **nein** |
| `xtask/tests/branch_overlap_test.rs:196` | `.jules/tasks` | **nein** |
| `xtask/tests/canaries/main.rs:97` | `governance` | ja |
| `xtask/tests/canaries/main.rs:103` | `governance/protected-paths.toml` | ja |
| `xtask/tests/canaries/main.rs:106` | `governance/` | ja |
| `xtask/tests/canaries/main.rs:149` | `governance` | ja |
| `xtask/tests/canaries/main.rs:154` | `governance/protected-paths.toml` | ja |
| `xtask/tests/canaries/main.rs:157` | `governance/` | ja |
| `xtask/tests/canaries/main.rs:354` | `.jules/tasks` | **nein** |
| `xtask/tests/canaries/main.rs:358` | `.jules/tasks` | **nein** |
| `xtask/tests/canaries/main.rs:379` | `.jules/tasks` | **nein** |
| `xtask/tests/canaries/main.rs:389` | `.jules/tasks` | **nein** |
| `xtask/tests/canaries/main.rs:393` | `.jules/tasks` | **nein** |
| `xtask/tests/canaries/main.rs:416` | `.jules/tasks` | **nein** |
| `xtask/tests/canaries/main.rs:548` | `governance` | ja |
| `xtask/tests/canaries/main.rs:553` | `governance/ratchet.toml` | ja |
| `xtask/tests/context_pack_test.rs:80` | `.jules` | **nein** |
| `xtask/tests/harness_codeowners_coverage_test.rs:14` | `governance` | ja |
| `xtask/tests/harness_codeowners_coverage_test.rs:14` | `governance` | ja |
| `xtask/tests/harness_codeowners_coverage_test.rs:17` | `governance/protected-paths.toml` | ja |
| `xtask/tests/harness_codeowners_coverage_test.rs:51` | `governance` | ja |
| `xtask/tests/harness_codeowners_coverage_test.rs:51` | `governance` | ja |
| `xtask/tests/harness_codeowners_coverage_test.rs:54` | `governance/protected-paths.toml` | ja |
| `xtask/tests/harness_diff_budget_test.rs:124` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_diff_budget_test.rs:126` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_diff_budget_test.rs:147` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:78` | `.jules` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:79` | `.jules` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:81` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:83` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:94` | `.jules/SESSION_BOOTSTRAP.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:96` | `.jules/SESSION_BOOTSTRAP.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:103` | `.jules/COMMON_LLM_ERRORS.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:105` | `.jules/COMMON_LLM_ERRORS.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:109` | `.jules/SCHEDULED_AUDIT.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:111` | `.jules/SCHEDULED_AUDIT.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:115` | `.jules/AUDIT_INTAKE_PROTOCOL.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:117` | `.jules/AUDIT_INTAKE_PROTOCOL.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:121` | `.jules/ISSUE_AUTOMATION_POLICY.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:123` | `.jules/ISSUE_AUTOMATION_POLICY.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:257` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:304` | `.jules/JULES_CONTEXT.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:331` | `.jules/JULES_LOG.md` | **nein** |
| `xtask/tests/harness_doc_truth_test.rs:333` | `.jules/JULES_LOG.md` | **nein** |
| `xtask/tests/harness_env_attest_test.rs:28` | `.jules/setup` | **nein** |
| `xtask/tests/harness_env_attest_test.rs:94` | `.jules/local` | **nein** |
| `xtask/tests/harness_gates_catalog_sync_test.rs:14` | `governance` | ja |
| `xtask/tests/harness_gates_catalog_sync_test.rs:14` | `governance` | ja |
| `xtask/tests/harness_gates_catalog_sync_test.rs:15` | `.jules` | **nein** |
| `xtask/tests/harness_gates_catalog_sync_test.rs:18` | `governance/gates.toml` | ja |
| `xtask/tests/harness_gates_catalog_sync_test.rs:33` | `.jules/harness-phases.toml` | **nein** |
| `xtask/tests/harness_gates_catalog_sync_test.rs:43` | `governance/verdict-required.toml` | **nein** |
| `xtask/tests/harness_gates_catalog_sync_test.rs:63` | `governance` | ja |
| `xtask/tests/harness_gates_catalog_sync_test.rs:63` | `governance` | ja |
| `xtask/tests/harness_gates_catalog_sync_test.rs:64` | `.jules` | **nein** |
| `xtask/tests/harness_gates_catalog_sync_test.rs:67` | `governance/gates.toml` | ja |
| `xtask/tests/harness_gates_catalog_sync_test.rs:82` | `.jules/harness-phases.toml` | **nein** |
| `xtask/tests/harness_gates_catalog_sync_test.rs:92` | `governance/verdict-required.toml` | **nein** |
| `xtask/tests/harness_jules_dispatch_test.rs:26` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_jules_dispatch_test.rs:34` | `.jules/jules-api.toml` | **nein** |
| `xtask/tests/harness_jules_dispatch_test.rs:42` | `.jules/PREAMBLE.md` | **nein** |
| `xtask/tests/harness_jules_dispatch_test.rs:44` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_jules_dispatch_test.rs:55` | `.jules/setup` | **nein** |
| `xtask/tests/harness_jules_dispatch_test.rs:99` | `.jules/jules-api.toml` | **nein** |
| `xtask/tests/harness_jules_dispatch_test.rs:138` | `.jules/local` | **nein** |
| `xtask/tests/harness_jules_test.rs:10` | `.jules` | **nein** |
| `xtask/tests/harness_ledger_test.rs:26` | `.jules/local` | **nein** |
| `xtask/tests/harness_loop_guard_test.rs:121` | `.jules/local` | **nein** |
| `xtask/tests/harness_loop_guard_wiring_test.rs:13` | `.jules` | **nein** |
| `xtask/tests/harness_loop_guard_wiring_test.rs:59` | `.jules/local` | **nein** |
| `xtask/tests/harness_loop_guard_wiring_test.rs:108` | `.jules/local` | **nein** |
| `xtask/tests/harness_mutants_diff_test.rs:17` | `.jules` | **nein** |
| `xtask/tests/harness_plan_lint_test.rs:26` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_plan_lint_test.rs:27` | `.jules/local` | **nein** |
| `xtask/tests/harness_plan_lint_test.rs:35` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_plan_lint_test.rs:46` | `.jules/setup` | **nein** |
| `xtask/tests/harness_plan_lint_test.rs:150` | `.jules/local` | **nein** |
| `xtask/tests/harness_postmortem_test.rs:44` | `governance` | ja |
| `xtask/tests/harness_protected_paths_test.rs:27` | `governance` | ja |
| `xtask/tests/harness_protected_paths_test.rs:28` | `governance` | ja |
| `xtask/tests/harness_protected_paths_test.rs:34` | `governance/protected-paths.toml` | ja |
| `xtask/tests/harness_protected_paths_test.rs:37` | `governance/` | ja |
| `xtask/tests/harness_protected_paths_test.rs:398` | `governance/protected-paths.toml` | ja |
| `xtask/tests/harness_ratchet_test.rs:27` | `governance` | ja |
| `xtask/tests/harness_scope_guard_test.rs:28` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_scope_guard_test.rs:30` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_scope_guard_test.rs:67` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_scope_guard_test.rs:105` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_session_report_test.rs:119` | `.jules/local` | **nein** |
| `xtask/tests/harness_task_card_test.rs:10` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_task_card_test.rs:18` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_task_card_test.rs:27` | `.jules/setup` | **nein** |
| `xtask/tests/harness_task_card_test.rs:65` | `.jules/tasks` | **nein** |
| `xtask/tests/harness_task_card_test.rs:107` | `.jules/tasks` | **nein** |
| `xtask/tests/session_init_test.rs:69` | `.jules/session.env` | **nein** |
| `xtask/tests/session_init_test.rs:76` | `.jules/SESSION.md` | **nein** |

### Analyse der abweichenden Dateien

#### `gates.toml`

##### Git Log (governance/)
```
f1aa13c00 Mon Oct 26 16:19:04 2026 +0100 rules-gourvernace-git tools, gates, xtask
332c26958 Mon Oct 26 13:56:03 2026 +0100 .agents/ statt .jules/ meta-system umstrukturierung kl
d8bbd8384 Sun Oct 25 21:17:12 2026 +0000 refactor(xtask): xtask-Kern, Gate-Katalog, Legacy-Aufteilung [MP-02]
```

##### Git Log (.agents/governance/)
```
26d8d13ce Mon Oct 26 14:48:31 2026 +0100 xtask: xtask-heavy eingelagert, CLAUDE.md, GitHub-Workflow für Jules
332c26958 Mon Oct 26 13:56:03 2026 +0100 .agents/ statt .jules/ meta-system umstrukturierung kl
```

##### Inhaltlicher Diff (governance/ vs .agents/governance/)
```diff
--- governance/gates.toml	2026-10-27 08:23:15.962086490 +0000
+++ .agents/governance/gates.toml	2026-10-27 08:23:15.130086499 +0000
@@ -1,11 +1,3 @@
-# governance/gates.toml — Single Source of Truth für alle Gates
-# doc-stamp: 2026-10-26 | Normativ: docs/SPEC.md §17.3
-# Dieses File wird von: cargo xtask, justfile, .github/workflows/ konsumiert.
-# Änderungen erfordern ADR (protected path gemäß protected-paths.toml).
-
-# WICHTIG: active_phase steuert, welche Gates ab welcher Phase erzwungen werden.
-# PH0 = sofort aktiv | PH1 = Phase 1 | PH2 = Phase 2 | etc.
-
 [meta]
 active_phase = "PH0"

@@ -31,12 +23,11 @@

 [[gate]]
 name = "ring-dag"
-aliases = ["ring-layering"]    # K-14: SPEC §17.3 führt beide Namen
 tier = "pr"
 from_phase = "PH0"
 kind = "builtin"
 enforces = ["P1", "P5", "P30", "INV-CORE-ISOLATION-1"]
-why = "Verifiziert acyclische Ring-Abhängigkeiten im Crate-Graph (SPEC §17.3: ring-layering / ring-dag)."
+why = "Verifiziert acyclische Ring-Abhängigkeiten im Crate-Graph."
 owner = "MP-03"

 [[gate]]
@@ -416,136 +407,3 @@
 enforces = ["INV-PLATFORM-1"]
 why = "Verifiziert Bau auf allen Zielplattformen."
 owner = "MP-10"
-
-
-# ── Fehlende Gates aus SPEC §17.3 (WP 0.4 Ergänzung) ──────────────────────────
-# Diese Gates sind in der Spezifikation benannt, fehlten aber im Gate-Katalog.
-# Status: definiert, Implementierung noch ausstehend (xtask-Module).
-
-[[gate]]
-name = "no-security-stub"
-tier = "pr"
-from_phase = "PH1"
-kind = "builtin"
-enforces = ["A1", "INV-WIRING-2"]
-why = "Keine Sicherheitsstubs in Produktion: kein cfg(not(feature))-Ersatz für Sicherheitscode (SPEC §17.3, A1). Befund B-02, B-15."
-owner = "MP-04"
-
-[[gate]]
-name = "no-literal-secrets"
-tier = "pr"
-from_phase = "PH1"
-kind = "builtin"
-enforces = ["INV-TENANT-1", "VETO-F10"]
-why = "Keine Literal-Schlüssel, Mandanten-Fallbacks oder Hardcoded-Secrets (SPEC §17.3). Befund B-04."
-owner = "MP-04"
-
-[[gate]]
-name = "panic-surface"
-aliases = ["ratchet-panic"]
-tier = "pr"
-from_phase = "PH0"
-kind = "builtin"
-enforces = ["P7", "REQ-QA-005"]
-why = "Kein unwrap/expect/panic/todo/unreachable/indexing in Produktionscode (SPEC §0.2 R2, §17.3). Ratchet-Baselines governance/ratchet.toml."
-owner = "MP-05"
-
-[[gate]]
-name = "async-io"
-aliases = ["ring0-async-purity"]
-tier = "pr"
-from_phase = "PH0"
-kind = "builtin"
-enforces = ["REQ-ASYNC-2", "P26"]
-why = "Kein Blockieren im async-Kontext; keine Async-Runtime in Ring 0 (SPEC §17.3). 18 Verstöße (IST)."
-owner = "MP-03"
-
-[[gate]]
-name = "doc-stamp"
-tier = "pr"
-from_phase = "PH0"
-kind = "builtin"
-enforces = ["A9", "INV-DOC-SINGLE-SOURCE-1"]
-why = "Dokumentstempel und Single-Source-Prüfung (SPEC §17.3, A9)."
-owner = "MP-07"
-
-[[gate]]
-name = "surface-coverage"
-tier = "pr"
-from_phase = "PH2"
-kind = "builtin"
-enforces = ["INV-SURFACE-1"]
-why = "Jede persistente Dateiklasse einer Löschschicht zugeordnet (SPEC §12.4, §17.3)."
-owner = "MP-08"
-
-[[gate]]
-name = "claim-generator"
-tier = "pr"
-from_phase = "PH7"
-kind = "builtin"
-enforces = ["INV-CLAIM-BIND-1", "AC-24"]
-why = "Außentexte nur aus Satzbibliothek, jeder Satz an Messwert gebunden (SPEC §17.3)."
-owner = "MP-08"
-
-[[gate]]
-name = "inv-coverage"
-tier = "merge"
-from_phase = "PH2"
-kind = "builtin"
-enforces = ["A8"]
-why = "Jede INV-* Invariante hat einen Test, der ihre Verletzung erkennt (SPEC §17.3, A8)."
-owner = "MP-09"
-
-[[gate]]
-name = "error-domain"
-tier = "pr"
-from_phase = "PH1"
-kind = "builtin"
-enforces = ["A6"]
-why = "Alle Fehler münden in ContextraError (SPEC §13.6, §17.3)."
-owner = "MP-05"
-
-[[gate]]
-name = "loc-budget"
-tier = "merge"
-from_phase = "PH2"
-kind = "builtin"
-enforces = []
-why = "Code-Zeilenbudget nach tools/loc.py (SPEC §1.3, §4.2, §17.3)."
-owner = "MP-03"
-
-[[gate]]
-name = "boot-order"
-tier = "pr"
-from_phase = "PH5"
-kind = "builtin"
-enforces = ["INV-BOOT-1"]
-why = "Phasen B0–B9 in korrekter Reihenfolge (SPEC §17.3)."
-owner = "MP-04"
-
-[[gate]]
-name = "unbounded-spawn"
-tier = "pr"
-from_phase = "PH4"
-kind = "builtin"
-enforces = ["REQ-ASYNC-3"]
-why = "Jede spawn-Stelle begrenzt (SPEC §17.3)."
-owner = "MP-03"
-
-[[gate]]
-name = "product-hull"
-tier = "merge"
-from_phase = "PH3"
-kind = "builtin"
-enforces = []
-why = "Produkt baut ohne Rand-Module (SPEC §4.5, §17.3)."
-owner = "MP-03"
-
-[[gate]]
-name = "test-surface"
-tier = "pr"
-from_phase = "PH1"
-kind = "builtin"
-enforces = ["REQ-QA-005"]
-why = "Prüft Testabdeckungs-Mindestanforderungen je Crate (SPEC §17.3). 18 fehlend (IST)."
-owner = "MP-09"
```

#### `ratchet.toml`

##### Git Log (governance/)
```
f8d2612a9 Tue Oct 27 07:55:18 2026 +0000 WP-0.4(governance,xtask): Lint-Satz vervollstaendigt, Ratchet verifiziert
f1aa13c00 Mon Oct 26 16:19:04 2026 +0100 rules-gourvernace-git tools, gates, xtask
332c26958 Mon Oct 26 13:56:03 2026 +0100 .agents/ statt .jules/ meta-system umstrukturierung kl
```

##### Git Log (.agents/governance/)
```
332c26958 Mon Oct 26 13:56:03 2026 +0100 .agents/ statt .jules/ meta-system umstrukturierung kl
```

##### Inhaltlicher Diff (governance/ vs .agents/governance/)
```diff
--- governance/ratchet.toml	2026-10-27 08:23:15.962086490 +0000
+++ .agents/governance/ratchet.toml	2026-10-27 08:23:15.130086499 +0000
@@ -1,15 +1,3 @@
-# governance/ratchet.toml — Ratchet-Baselines (dürfen nur sinken, nie steigen)
-# doc-stamp: 2026-10-26 | Normativ: docs/SPEC.md §17, REQ-QA-005, PLAN.md WP 0.4
-# Baselines aus Commit a547a42. Jede Änderung muss durch ADR gerechtfertigt werden.
-#
-# Prinzip: Baselines sinken monoton. Ziel: 0 (wo nicht anders vermerkt).
-# Ein Ratchet ist kein Beleg für „null" (P7). Nur ein Gate-Lauf ist Beleg.
-#
-# Schema-Version 2 (WP 0.4): erweitert um PLAN.md §6/§17 Baselines.
-# Vorheriges Schema (v1): unwrap_legacy_grep, [unsafe_blocks] — bleibt für
-# Kompatibilität erhalten. Neue Felder kommen unter [baselines].
-
-# ── Legacy-Zähler (xtask-kompatibel, NICHT entfernen) ─────────────────────────
 unwrap_legacy_grep = 1175
 unwrap_exact = 502
 expect_exact = 242
@@ -22,31 +10,3 @@
 contextra-simd = 285
 contextra-sys = 30
 contextra-wire = 27
-
-# ── Neue Baselines (PLAN.md WP 0.4, §6, §17) ─────────────────────────────────
-# Jeder Wert ist die Obergrenze. Neue PRs dürfen sie senken, nie erhöhen.
-[baselines]
-# Panic-Aufrufe in Produktionscode (P7) — Ziel: 0
-panic_calls = 2
-
-# async fn in Ring-0-Crates (D-09: Async bleibt Ratchet) — Ziel: 0
-async_fn_ring0_vector = 52
-async_fn_ring0_text = 43
-async_fn_ring0_rank = 1
-
-# SAFETY-Lücken: unsafe-Blöcke ohne vollständigen SAFETY-Kommentar (P2)
-safety_gaps = 49
-
-# Sensitive #[allow(...)] Attribute in Produktionscode — Ziel: 0
-sensitive_allow = 6
-
-# Zeit-Aufrufe in aktiven Crates (P28) — total: 44, aktiv: 35 (9 in Archiv)
-time_calls = 44
-time_calls_active = 35
-
-# Tote Features (REQ-ARCH-012, WP 2.4) — Ziel: 0
-dead_features = 24
-
-# unwrap/expect in Produktionscode (P7, SPEC §0.2 R2) — Ziel: 0
-unwrap_exact = 502
-expect_exact = 242
```

#### `protected-paths.toml`

##### Git Log (governance/)
```
05a6a9afe Mon Oct 26 16:44:18 2026 +0100 context optimiert
f1aa13c00 Mon Oct 26 16:19:04 2026 +0100 rules-gourvernace-git tools, gates, xtask
332c26958 Mon Oct 26 13:56:03 2026 +0100 .agents/ statt .jules/ meta-system umstrukturierung kl
```

##### Git Log (.agents/governance/)
```
26d8d13ce Mon Oct 26 14:48:31 2026 +0100 xtask: xtask-heavy eingelagert, CLAUDE.md, GitHub-Workflow für Jules
332c26958 Mon Oct 26 13:56:03 2026 +0100 .agents/ statt .jules/ meta-system umstrukturierung kl
```

##### Inhaltlicher Diff (governance/ vs .agents/governance/)
```diff
--- governance/protected-paths.toml	2026-10-27 08:23:15.962086490 +0000
+++ .agents/governance/protected-paths.toml	2026-10-27 08:23:15.130086499 +0000
@@ -1,6 +1,5 @@
-# governance/protected-paths.toml — Geschützte Pfade (REQ-GOV-002)
-# doc-stamp: 2026-10-26 | Normativ: docs/SPEC.md §17, AGENTS.md §3
-# Änderungen an diesen Pfaden erfordern ADR-Trailer und PR-Label "protected-path".
+# Governance Protected Paths Configuration
+# Any change to files matching these patterns requires an approved ADR trailer and PR label.

 [[protected]]
 glob = ".github/**"
@@ -67,10 +66,6 @@
 reason = "Cargo-Konfigurationen und Build-Flags."

 [[protected]]
-glob = "clippy.toml"
-reason = "Workspace-Clippy-Konfiguration mit P28-Verbotsliste (SPEC §17.3, PLAN.md WP 0.4)."
-
-[[protected]]
 glob = "sgconfig.yml"
 reason = "Ast-Grep / Static Analysis Regeln."

@@ -105,11 +100,3 @@
 [[protected]]
 glob = "docs/unwrap_baseline_history.jsonl"
 reason = "Historie der Zero-Panic/Unwrap-Baselines zur Verfolgung von Tech-Debt."
-
-[[protected]]
-glob = "governance/**"
-reason = "Governance Single Source of Truth — schützt Gates, Ratchets und Policies."
-
-[[protected]]
-glob = ".agents/**"
-reason = "Agenten-Konfiguration, Session-Skripte und Governance-Dateien."
```


### Gate-Messungen & Ausfallanalyse

#### Executed Commands & Outcomes

1. **`cargo xtask gate-list`**
   - **Exit-Code:** 0
   - **Ergebnis:** Gibt alle 58 konfigurierten Gates aus `governance/gates.toml` zurück.

2. **`cargo xtask check`**
   - **Exit-Code:** 2 (Summary: 39 total, 1 passed/N/A, 1 failed, 37 errors)
   - **Befund:** Ins Leere laufende / fehlende Builtin-Gate-Implementierungen oder fehlende Dateien:
     - `verdict-required.toml`: `xtask` sucht unter `governance/verdict-required.toml`, Datei existiert jedoch nur unter `.agents/governance/verdict-required.toml`.
     - `unsafe-safety-baseline.toml`: `xtask` sucht unter `governance/unsafe-safety-baseline.toml`, Datei existiert jedoch nur unter `.agents/governance/verdict-required.toml`.
     - `determinism-baseline.toml`: `xtask` sucht unter `governance/determinism-baseline.toml`, Datei existiert jedoch nur unter `.agents/governance/determinism-baseline.toml`.
     - `.jules/harness-phases.toml`: `xtask` sucht unter `.jules/harness-phases.toml`, existiert jedoch unter `.agents/jules/harness-phases.toml`.
