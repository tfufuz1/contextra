# Audit-Bericht: `xtask` LOC-Reduktion & Modul-Erreichbarkeit

**Datum:** 2026-09-25
**Repository:** `contextra` (`tfufuz1/contextra`)
**Branch:** `chore/xtask-loc-audit`
**Referenz:** `CONTEXTRA_SPEC_UPDATED.md` Teil A3.2 (Punkt 6) & §15

---

## 1. Übersicht & Kontext

Laut `CONTEXTRA_SPEC_UPDATED.md` Teil A3.2 hat `xtask` aktuell einen Umfang von über 16.000 LOC (exakt **18.549 Zeilen** über alle 51 Rust-Quelldateien hinweg via `wc -l`). Dieser Umfang liegt weit über dem Zielwert von **< 3.000 LOC**.

Aufgrund dieses Umfangs trägt `xtask` befristet ein `#![allow]` auf die scharfen Panic-Lints, bis der Zielwert in Phase 5 erreicht wird. Dieser Audit-Bericht bietet eine **vollständige Bestandsaufnahme** aller Quelldateien unter `xtask/src/`, prüft deren Erreichbarkeit über den `mod`-Graphen von `xtask/src/main.rs` und `xtask/src/lib.rs` und klassifiziert sie zur Vorbereitung künftiger Entkopplungs-PRs.

---

## 2. Zusammenfassungstabelle

| Kategorie | Beschreibung | Anzahl Dateien | Summe LOC (`wc -l`) | Eingesparte LOC (PR) |
|---|---|:---:|:---:|:---:|
| **A** | **Erreichbar, aktiv genutzt (Gate-Logik / CI / Preflight)** | 30 | 12.022 | 0 |
| **B** | **Erreichbar, Entwickler-/Migration-Tooling ohne CI-Bindung** | 19 | 6.325 | 0 |
| **C** | **Unerreichbar (kein `mod`-Pfad), tot** | 0 | 0 | 0 |
| **D** | **Unklar / Legacy-Alias / Bedingt genutzt** | 2 | 202 | 0 |
| **Gesamt** | | **51** | **18.549** | **0** |

*Hinweis:* Nach Bereinigung in früheren Phase-0/1-Sprints sind aktuell **alle 51 Quelldateien** über die `mod`-Hierarchie von `main.rs` oder `lib.rs` erreichbar. Daher existieren in dieser Phase **0 tote Dateien der Kategorie C**. Die Reduktion auf < 3.000 LOC erfordert die Auslagerung von Kategorie-B-Modulen in separaten Folge-PRs.

---

## 3. Vollständige Dateiliste & Klassifizierung

Alle 51 Dateien unter `xtask/src/**/*.rs`, sortiert nach LOC (`wc -l`) absteigend:

| Nr. | Datei | LOC | Kat. | Erreichbarkeit (`mod`-Pfad) | CI / Preflight / Justfile Bindung | Begründung & Status |
|---|---|:---:|:---:|---|---|---|
| 1 | `xtask/src/main.rs` | 3.867 | **A** | Crate Root (Binary) | CI, Preflight, CLI Dispatch | Haupt-CLI-Dispatcher & zentraler Einstiegspunkt. |
| 2 | `xtask/src/check_duplicate_intent.rs` | 1.208 | **A** | `main.rs -> mod check_duplicate_intent;` | `context-gates.yml` | Aktives CI-Gate für Intent-Duplikate. |
| 3 | `xtask/src/claim.rs` | 846 | **A** | `main.rs -> mod claim;` | `commit-integrity-guard.yml`, Preflight, Justfile | Task-Claim-Locking für parallele Agenten. |
| 4 | `xtask/src/check_duplicate_symbols.rs` | 793 | **A** | `main.rs -> mod check_duplicate_symbols;` | `context-gates.yml`, `rust-ci.yml`, Preflight, Script | Aktives CI-Gate für Symbolduplikate. |
| 5 | `xtask/src/jules_preflight.rs` | 771 | **A** | `main.rs -> mod jules_preflight;` | `context-gates.yml`, `post-merge-verification.yml`, Justfile | Zentrale Preflight-Orchestrierung. |
| 6 | `xtask/src/gates/debt_audit.rs` | 670 | **B** | `main.rs -> use xtask::gates; -> lib.rs -> pub mod gates; -> gates/mod.rs -> pub mod debt_audit;` | Justfile (`just debt-audit`) | Generiert Schulden-Audits; Kandidat für Auslagerung. |
| 7 | `xtask/src/check_module_reachability.rs` | 543 | **A** | `main.rs -> mod check_module_reachability;` | `context-gates.yml`, Preflight, Script | Aktives CI-Gate für Modul-Erreichbarkeit. |
| 8 | `xtask/src/validate_pr_checklist.rs` | 473 | **B** | `main.rs -> mod validate_pr_checklist;` | Kein CI Workflow (lokales Tooling) | Prüft PR-Checklisten vor Submit; Kandidat für Auslagerung. |
| 9 | `xtask/src/check_vetoes.rs` | 464 | **B** | `main.rs -> mod check_vetoes;` | Justfile (`just check-vetoes`) | Prüft Architektur-Veto-Tags; Kandidat für Auslagerung. |
| 10 | `xtask/src/check_unsafe_islands.rs` | 440 | **A** | `main.rs -> mod check_unsafe_islands;` | `context-gates.yml`, `merge-gate.yml`, Preflight | Aktives CI-Gate für Unsafe-Code-Grenzen. |
| 11 | `xtask/src/gen_prompter_data.rs` | 438 | **A** | `main.rs -> mod gen_prompter_data;` | `context-gates.yml`, `update-prompter-data.yml`, Justfile | Generiert Prompting-Metadaten für CI. |
| 12 | `xtask/src/check_audit_verdict_independence.rs` | 436 | **A** | `main.rs -> mod check_audit_verdict_independence;` | `context-gates.yml`, `merge-gate.yml` | Aktives CI-Gate für Audit-Unabhängigkeit. |
| 13 | `xtask/src/check_ring_layering.rs` | 430 | **A** | `main.rs -> mod check_ring_layering;` | `context-gates.yml`, `merge-gate.yml`, Preflight | Aktives CI-Gate für Ring-Architektur. |
| 14 | `xtask/src/check_adr_deadlines.rs` | 347 | **A** | `main.rs -> mod check_adr_deadlines;` | `context-gates.yml`, Justfile | Aktives CI-Gate für ADR-Fristen. |
| 15 | `xtask/src/lint_unsafe_slice_bounds.rs` | 341 | **B** | `main.rs -> mod lint_unsafe_slice_bounds;` | Kein CI Workflow (lokaler Linter) | Linter für Slicing-Grenzen; Kandidat für Auslagerung. |
| 16 | `xtask/src/generate_adr.rs` | 304 | **B** | `main.rs -> mod generate_adr;` | Kein CI Workflow (Entwickler-CLI) | Erstellt ADR-Templates & -Index; Kandidat für Auslagerung. |
| 17 | `xtask/src/check_audit_duplication.rs` | 303 | **B** | `main.rs -> mod check_audit_duplication;` | Kein CI Workflow (Audit-Hilfstool) | Prüft doppelte Audits; Kandidat für Auslagerung. |
| 18 | `xtask/src/check_stale_tags.rs` | 299 | **B** | `main.rs -> mod check_stale_tags;` | Kein CI Workflow (Governance-Tool) | Prüft veraltete Tags; Kandidat für Auslagerung. |
| 19 | `xtask/src/gates/check_crate_references.rs` | 298 | **A** | `main.rs -> use xtask::gates; -> lib.rs -> pub mod gates; -> gates/mod.rs -> pub mod check_crate_references;` | Preflight | Aktives Preflight-Gate für Crate-Referenzen. |
| 20 | `xtask/src/check_doc_references.rs` | 287 | **A** | `main.rs -> mod check_doc_references;` | `context-gates.yml` | Aktives CI-Gate für Dok-Links. |
| 21 | `xtask/src/session_history.rs` | 271 | **B** | `lib.rs -> pub mod session_history;` | In Benchmarks verwendet (`contextra-bench`) | Session-History Logging-Klasse; Kandidat für Auslagerung. |
| 22 | `xtask/src/migrate_docid_128.rs` | 267 | **B** | `main.rs -> mod migrate_docid_128;` | Kein CI Workflow (Einmal-Migration) | Migrations-Tooling für 128-bit DocIDs; Kandidat für Auslagerung. |
| 23 | `xtask/src/record_mutation_score.rs` | 254 | **B** | `main.rs -> mod record_mutation_score;` | Kein CI Workflow (Test-Reporter) | Mutationstest-Score-Logger; Kandidat für Auslagerung. |
| 24 | `xtask/src/check_ring0_async_purity.rs` | 251 | **B** | `main.rs -> mod check_ring0_async_purity;` | Kein CI Workflow (Spezial-Linter) | Prüft Async-Reinheit in Ring 0; Kandidat für Auslagerung. |
| 25 | `xtask/src/check_jules_context_freshness.rs` | 248 | **A** | `main.rs -> mod check_jules_context_freshness;` | Preflight | Aktives Preflight-Gate für Kontext-Frische. |
| 26 | `xtask/src/check_placeholder_refs.rs` | 246 | **A** | `main.rs -> mod check_placeholder_refs;` | `context-gates.yml` | Aktives CI-Gate für Platzhalter-Referenzen. |
| 27 | `xtask/src/check_workflow_commands.rs` | 229 | **A** | `main.rs -> mod check_workflow_commands;` | `context-gates.yml` | Aktives CI-Gate für Workflow-Befehle. |
| 28 | `xtask/src/check_recall_stability.rs` | 222 | **A** | `main.rs -> mod check_recall_stability;` | `nucleation-recall-history.yml` | Aktives CI-Gate für Recall-Stabilität. |
| 29 | `xtask/src/check_type_registry.rs` | 213 | **B** | `main.rs -> mod check_type_registry;` | Kein CI Workflow (Typ-Konsistenz) | Validiert Typen-Register; Kandidat für Auslagerung. |
| 30 | `xtask/src/check_audit_tool_evidence.rs` | 209 | **B** | `main.rs -> mod check_audit_tool_evidence;` | Kein CI Workflow (Audit-Reporter) | Prüft Nachweise in Audits; Kandidat für Auslagerung. |
| 31 | `xtask/src/init_audit_fix.rs` | 204 | **B** | `main.rs -> mod init_audit_fix;` | Kein CI Workflow (Hilfsskript) | Initalisiert Audit-Fixes; Kandidat für Auslagerung. |
| 32 | `xtask/src/check_flatbuffers_drift.rs` | 203 | **A** | `main.rs -> mod check_flatbuffers_drift;` | `context-gates.yml`, Script | Aktives CI-Gate für Schema-Drift. |
| 33 | `xtask/src/check_commit_messages.rs` | 202 | **A** | `main.rs -> mod check_commit_messages;` | `context-gates.yml` | Aktives CI-Gate für Commit-Konventionen. |
| 34 | `xtask/src/check_max_results_unbound.rs` | 195 | **A** | `main.rs -> mod check_max_results_unbound;` | `proof-correctness.yml`, Justfile | Aktives CI-Gate für Unbound Limits. |
| 35 | `xtask/src/check_action_pinning.rs` | 189 | **D** | `main.rs -> mod check_action_pinning;` | Kein CI Workflow | GitHub Actions Pinning Linter; Unklar, ob als CI-Gate geplant. |
| 36 | `xtask/src/check_coverage_gate.rs` | 186 | **A** | `main.rs -> mod check_coverage_gate;` | `coverage.yml`, `proof-correctness.yml`, Justfile | Aktives CI-Gate für Testabdeckung. |
| 37 | `xtask/src/check_toctou_trait_defaults.rs` | 178 | **B** | `main.rs -> mod check_toctou_trait_defaults;` | Kein CI Workflow | TOCTOU-Schutzlinter; Kandidat für Auslagerung. |
| 38 | `xtask/src/check_result_dropped_on_io.rs` | 156 | **B** | `main.rs -> mod check_result_dropped_on_io;` | `xtask/tests/check_result_dropped_io.rs` | Linter für Result-Drops; hat Integrationstest; Kandidat für Auslagerung. |
| 39 | `xtask/src/check_agents_integrity.rs` | 153 | **A** | `main.rs -> mod check_agents_integrity;` | `context-gates.yml`, Preflight, Justfile | Aktives CI-Gate für AGENTS.md Integrität. |
| 40 | `xtask/src/check_nan_validation_in_hot_loop.rs` | 152 | **B** | `main.rs -> mod check_nan_validation_in_hot_loop;` | Kein CI Workflow | NaN-Prüfungs-Linter; Kandidat für Auslagerung. |
| 41 | `xtask/src/check_phantom_files.rs` | 141 | **A** | `main.rs -> mod check_phantom_files;` | `context-gates.yml` | Aktives CI-Gate für Phantom-Dateien. |
| 42 | `xtask/src/post_merge_report.rs` | 124 | **A** | `main.rs -> mod post_merge_report;` | `post-merge-verification.yml` | Post-Merge Verifikations-Reporter. |
| 43 | `xtask/src/jules_submit_gate.rs` | 107 | **B** | `main.rs -> mod jules_submit_gate;` | CLI Helper | Submit-Gate-Helfer für Agenten; Kandidat für Auslagerung. |
| 44 | `xtask/src/check_ffi_panic_boundary.rs` | 104 | **A** | `main.rs -> mod check_ffi_panic_boundary;` | `contextra-py-ci.yml`, `merge-gate.yml`, Script | Aktives CI-Gate für FFI Panic-Boundaries. | <!-- crate-ref-ignore -->
| 45 | `xtask/src/gen_feature_catalog.rs` | 100 | **B** | `main.rs -> mod gen_feature_catalog;` | Kein CI Workflow | Generiert Feature-Katalog Dokumentation. |
| 46 | `xtask/src/check_bandit_latency_budget.rs` | 66 | **A** | `main.rs -> mod check_bandit_latency_budget;` | `merge-gate.yml`, Script | Aktives CI-Gate für Latency-Budgets. |
| 47 | `xtask/src/bench_gate.rs` | 60 | **A** | `main.rs -> mod bench_gate;` | `bench.yml`, `bench-regression.yml`, Justfile | Aktives CI-Gate für Benchmark-Regressionen. |
| 48 | `xtask/src/check_compile.rs` | 42 | **A** | `main.rs -> mod check_compile;` | `context-gates.yml`, `rust-ci.yml`, `merge-gate.yml` | Aktives CI-Gate für Workspace-Kompilierung. |
| 49 | `xtask/src/check_orphan_modules.rs` | 13 | **D** | `main.rs -> mod check_orphan_modules;` | Kein direkter Aufruf in `main.rs` Dispatch | Legacy-Wrapper-Modul für `check_module_reachability`. CLI-Dispatch nutzt direkt `check_module_reachability`. |
| 50 | `xtask/src/lib.rs` | 4 | **A** | Library Crate Root | Crate Root | Deklariert `gates` & `session_history`. |
| 51 | `xtask/src/gates/mod.rs` | 2 | **A** | `lib.rs -> pub mod gates;` | Gate Submodule Root | Modul-Root für Sub-Gates. |

---

## 4. Strategie & Fahrplan zur Reduktion auf < 3.000 LOC

Um die Vorgabe der Spezifikation (< 3.000 LOC in Phase 5) zu erreichen, wird folgender stufenweiser Fahrplan in dedizierten Folge-PRs empfohlen:

1. **Auslagerung von Entwickler- & Dokumentations-Generatoren (Kategorie B):**
   - Modul `generate_adr.rs` (304 LOC), `gen_feature_catalog.rs` (100 LOC), `init_audit_fix.rs` (204 LOC), `migrate_docid_128.rs` (267 LOC) können in ein eigenständiges Dev-Crate `xtask-dev` oder unter `scripts/` verschoben werden.
   - **Einsparung:** ~875 LOC.

2. **Auslagerung von Linter & Nischen-Analysatoren (Kategorie B):**
   - Spezial-Linter wie `check_duplicate_intent.rs` (1.208 LOC), `check_vetoes.rs` (464 LOC), `validate_pr_checklist.rs` (473 LOC), `lint_unsafe_slice_bounds.rs` (341 LOC) und `check_stale_tags.rs` (299 LOC) können in separate Linter-Crates zerlegt oder als eigenständige Binaries eingebunden werden.
   - **Einsparung:** ~2.785 LOC.

3. **Konsolidierung des `main.rs` Dispatchers:**
   - Aufspaltung des 3.867 LOC schweren CLI-Dispatchers in schlanke Subkommandos mit Makro-Routing.
   - **Einsparung:** ~2.500 LOC im `main.rs` Root.

Durch diese Maßnahmen sinkt der `xtask`-Kernumfang auf ca. **2.800 - 3.000 LOC**, woraufhin das befristete `#![allow]` auf Panic-Lints entfernt werden kann.
