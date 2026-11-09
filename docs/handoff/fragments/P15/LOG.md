# Protokoll P15

## Start-Check vor Arbeitsbeginn
- `docs/handoff/fragments/P00/BUILD_LADDER.md` [V]: Nicht vorhanden.
- Compiler-Status vor Arbeitsbeginn [V]:
  - `cargo check --workspace --exclude contextra-py`: Schlägt fehl.
  - Fremdfehler `contextra-crypto`: `unexpected cfg condition value: serde` in `crates/contextra-crypto/src/secret.rs:94:7`.
  - Fremdfehler `contextra-graph`: 151 Fehler bezüglich ungelöster Module `contextra_types` und `contextra_ports`.
  - Diese Fehler liegen außerhalb des Besitzes von P15 und wurden vor Beginn dokumentiert.

## Inventar aller 35 CI-Workflows (Aufgabe 1)

Referenzen auf `contextra-erasure`, `-audit-export`, `-avv-generator`, `-license`, `-agent`, `-cognition` gelten als „existieren nach P41 nicht mehr“.

| Workflow | Trigger | referenziert xtask? | referenzierte Crates (heute / nach Archivierung)? | Zielaktion |
|---|---|---|---|---|
| `audit-integrity.yml` | schedule, workflow_dispatch | ja | keine | DEAKTIVIEREN |
| `baukasten-gates.yml` | push, pull_request | nein | contextra-graph (ja / ja), contextra-ports (ja / ja), contextra-simd (ja / ja), contextra-sys (ja / ja), contextra-text (ja / ja), contextra-types (ja / ja), contextra-vector (ja / ja) | DEAKTIVIEREN |
| `bench-regression.yml` | push | ja | contextra-db (ja / ja), contextra-router (ja / ja), contextra-store (ja / ja), contextra-vector (ja / ja) | BEHALTEN |
| `bench.yml` | push, pull_request, workflow_dispatch | ja | contextra-bench (ja / ja), contextra-db (ja / ja) | BEHALTEN |
| `build-wheels.yml` | workflow_dispatch, push | nein | contextra-py (ja / ja) | BEHALTEN |
| `chaos.yml` | schedule, workflow_dispatch | nein | contextra-store (ja / ja), contextra-vector (ja / ja) | DEAKTIVIEREN |
| `commit-integrity-guard.yml` | push, pull_request | nein | keine | DEAKTIVIEREN |
| `context-gates.yml` | push, pull_request | nein | contextra-mcp (ja / ja), contextra-ports (ja / ja), contextra-simd (ja / ja), contextra-sys (ja / ja), contextra-types (ja / ja) | DEAKTIVIEREN |
| `contextra-py-ci.yml` | push, pull_request | ja | contextra-py (ja / ja) | BEHALTEN |
| `coverage.yml` | push, pull_request, schedule | ja | contextra-core-ipc-gen (nein / nein), contextra-py (ja / ja) | DEAKTIVIEREN (redundant zu rust-ci) |
| `deep-gates.yml` | workflow_call, workflow_dispatch | ja | keine | DEAKTIVIEREN |
| `deny.yml` | push, pull_request | nein | keine | BEHALTEN |
| `flake-report.yml` | schedule, workflow_dispatch | ja | keine | DEAKTIVIEREN |
| `fuzz-extended.yml` | schedule, workflow_dispatch | nein | contextra-db (ja / ja), contextra-store (ja / ja), contextra-text (ja / ja), contextra-vector (ja / ja) | BEHALTEN |
| `jules-dispatch.yml` | issue_comment, pull_request_review_comment | ja | keine | DEAKTIVIEREN |
| `loom-gate.yml` | schedule, workflow_dispatch, pull_request | nein | contextra-crypto (ja / ja), contextra-store (ja / ja), contextra-vector (ja / ja) | BEHALTEN |
| `merge-gate.yml` | push, pull_request | ja | contextra-adapt (ja / ja), contextra-agent (ja / nein), contextra-cognition (ja / nein), contextra-graph (ja / ja), contextra-infer-ollama (ja / ja), contextra-infer-onnx (ja / ja), contextra-kvcache (ja / ja), contextra-py (ja / ja), contextra-router (ja / ja) | DEAKTIVIEREN |
| `mutation-testing.yml` | schedule, workflow_dispatch, pull_request | ja | contextra-candle (nein / nein), contextra-checkpoint (ja / ja), contextra-crypto (ja / ja), contextra-db (ja / ja), contextra-graph (ja / ja), contextra-rank (ja / ja), contextra-store (ja / ja), contextra-text (ja / ja), contextra-vector (ja / ja) | BEHALTEN |
| `nucleation-recall-history.yml` | schedule, workflow_dispatch | ja | contextra-vector (ja / ja) | DEAKTIVIEREN |
| `post-merge-verification.yml` | push | ja | keine | DEAKTIVIEREN |
| `proof-correctness.yml` | push, pull_request | ja | contextra-db (ja / ja), contextra-graph (ja / ja), contextra-py (ja / ja), contextra-store (ja / ja), contextra-text (ja / ja), contextra-vector (ja / ja) | DEAKTIVIEREN |
| `prune-branches.yml` | schedule, workflow_dispatch, pull_request | nein | keine | DEAKTIVIEREN |
| `publish-pypi.yml` | workflow_dispatch | nein | contextra-py (ja / ja) | BEHALTEN |
| `refresh-jules-context.yml` | workflow_dispatch, push, pull_request | ja | keine | DEAKTIVIEREN |
| `release-mcp-binary.yml` | push, workflow_dispatch | nein | contextra-mcp (ja / ja) | BEHALTEN |
| `rust-ci.yml` | push, pull_request | ja | contextra-adapt (ja / ja), contextra-candle (nein / nein), contextra-checkpoint (ja / ja), contextra-core (nein / nein), contextra-crypto (ja / ja), contextra-db (ja / ja), contextra-graph (ja / ja), contextra-infer-candle (ja / ja), contextra-infer-onnx (ja / ja), contextra-mcp (ja / ja), contextra-py (ja / ja), contextra-router (ja / ja), contextra-store (ja / ja), contextra-text (ja / ja), contextra-types (ja / ja), contextra-vector (ja / ja) | BEHALTEN |
| `scheduled-audit.yml` | schedule, workflow_dispatch | ja | contextra-agent (ja / nein), contextra-db (ja / ja), contextra-graph (ja / ja), contextra-vector (ja / ja) | DEAKTIVIEREN |
| `shell-commit-gate.yml` | push | ja | keine | DEAKTIVIEREN |
| `simd-cross-validation.yml` | pull_request, workflow_dispatch | nein | contextra-simd (ja / ja) | BEHALTEN |
| `tsan.yml` | schedule, workflow_dispatch, pull_request | nein | contextra-graph (ja / ja), contextra-store (ja / ja), contextra-vector (ja / ja) | BEHALTEN |
| `unwrap-baseline-check.yml` | push | ja | keine | DEAKTIVIEREN |
| `unwrap-gate.yml` | pull_request | nein | keine | BEHALTEN |
| `update-prompter-data.yml` | workflow_dispatch | ja | keine | DEAKTIVIEREN |
| `verdict.yml` | pull_request | ja | keine | DEAKTIVIEREN |
| `veto-deadline-gate.yml` | schedule, workflow_dispatch | ja | keine | DEAKTIVIEREN |

## Durchgeführte Aktionen
1. 20 Governance-/Gate-Workflows und `coverage.yml` per `git mv` nach `.github/_disabled/` verschoben.
2. `rust-ci.yml`: `xtask check-compile` durch `cargo check --workspace --exclude contextra-py` ersetzt. `check-duplicate-symbols` und `lint-unsafe-slices` entfernt. Crate-Referenzen bereinigt.
3. `bench.yml`: Trigger auf `workflow_dispatch` und wöchentlichen Schedule angepasst, Jobs mit `continue-on-error: true` nicht-blockierend gemacht, `xtask bench-gate` entfernt.
4. `bench-regression.yml`: Trigger auf `workflow_dispatch` und wöchentlichen Schedule angepasst, Job mit `continue-on-error: true` nicht-blockierend gemacht, `xtask bench-gate` und `bandit-latency-budget` entfernt.
5. `mutation-testing.yml`: Pull-Request-Trigger entfernt (nur noch schedule und dispatch), `continue-on-error: true` gesetzt, `xtask`-Schritte entfernt.
6. `contextra-py-ci.yml`: `xtask check-ffi-panic-boundary` entfernt.
7. `fuzz-extended.yml`: Fuzz-Targets auf tatsächlich existierende Targets korrigiert (`rrf_fusion`, `fuzz_hnsw_persistence`, `hnsw_insert_search`, `fuzz_bm25_tokenize`, `wal_mutation_chaos`, `fuzz_manifest_load`).
8. `docs/CI.md` erstellt mit vollständiger Übersicht aller 35 Workflows (Zweck, Trigger, Blockierend-Status, Deaktivierungsgrund).
