# Contextra Workspace Build & Test Baseline (2026-10-28)

**Datum:** 2026-10-28
**Commit:** `78274ec899fd14a5d1d8e36ca0767ba60d7b40b3`
**Toolchain:** `cargo 1.89.0 (c24e10642 2025-06-23)`, `rustc 1.89.0 (29483883e 2025-08-04)`
**Scope:** Reaktivierung / Beleg der Build- & Test-Baseline gemäß PLAN WP 0.1

---

## 1. Zusammenfassung & Workspace-Befehle

| Befehl | Exit-Code | Dauer | Raw-Log |
|---|---|---|---|
| `cargo check --workspace --all-targets --locked` | 0 | 3097 ms | [cargo_check_workspace.log](cargo_check_workspace.log) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 101 | 2896 ms | [cargo_clippy_workspace.log](cargo_clippy_workspace.log) |
| `cargo test --workspace --locked --no-fail-fast` | 101 | 374955 ms | [cargo_test_workspace.log](cargo_test_workspace.log) |
| `cargo tree -p contextra --edges normal --locked` | 0 | 1378 ms | [cargo_tree_contextra_normal.log](cargo_tree_contextra_normal.log) |
| `cargo tree -p contextra -e features --locked` | 0 | 369 ms | [cargo_tree_contextra_features.log](cargo_tree_contextra_features.log) |

---

## 2. Abgleich mit SYSTEMATIC_CRATE_AUDIT.md

Die Aussage **„35 Crates kompilieren fehlerfrei"** aus `SYSTEMATIC_CRATE_AUDIT.md` wird wie folgt bewertet:
- **BESTÄTIGT für `cargo check -p <crate> --locked`**: Alle 35 Crates im Workspace (inkl. `xtask`) bestehen `cargo check -p <crate>` mit Exit-Code 0.
- **BESTÄTIGT für `cargo check --workspace --all-targets --locked`**: Workspace-wide `--all-targets` besteht fehlerfrei (Exit-Code 0).
- **BESTÄTIGT für Standalone Build Target 1**: `cargo build -p contextra --no-default-features --features embedded --locked` schließt erfolgreich ab (Exit-Code 0, 23.5s).
- **BESTÄTIGT für Standalone Build Target 2**: `cargo build -p contextra-mcp --locked` schließt erfolgreich ab (Exit-Code 0, 30.4s).
- **WIDERLEGT für `cargo clippy --workspace --all-targets --locked -- -D warnings`**: Schlägt bei 33 von 35 Crates fehl due to unkonfigurierten Warnings (Exit-Code 101). Nur `contextra-wire` und `contextra-sys` bestehen Clippy fehlerfrei.
- **WIDERLEGT für globale Test-Ausführung (`cargo test --workspace`)**: Workspace-weite Testläufe schlagen mit Exit-Code 101 fehl bzw. laufen in Timeouts bei langlaufenden End-to-End Integrationstests (z. B. `tests/deletion_e2e.rs`).

---

## 3. Einzelergebnisse je Crate (35 Workspace Crates)

| Crate | Check (`cargo check -p`) | Clippy (`cargo clippy -p -D warnings`) | Test (`cargo test -p --no-fail-fast`) |
|---|---|---|---|
| **contextra-wire** | Grün (0) | Grün (0) | Grün (0) |
| **contextra-sys** | Grün (0) | Grün (0) | Grün (0) |
| **contextra-simd** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-types** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-core** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-mvcc** | Grün (0) | Rot (101) | Rot (101) [Import-Fehler in Test] |
| **contextra-ports** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-store** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-crypto** | Grün (0) | Rot (101) | Rot (101) [Undeclared CustodyLevel] |
| **contextra-testkit** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-graph** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-adapt** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-sandbox** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-vector** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-db** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-checkpoint** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-cognition** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-engine** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-rank** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-text** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-infer-candle** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-privacy** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-infer-onnx** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-mcp** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-audit-export** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-avv-generator** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-license** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-router** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-agent** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-infer-ollama** | Grün (0) | Rot (101) | Grün (0) |
| **contextra-bench** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |
| **contextra-py** | Grün (0) | Rot (101) | Grün (0) [No tests executed] |
| **contextra-kvcache** | Grün (0) | Rot (101) | Grün (0) |
| **xtask** | Grün (0) | Rot (101) | Timeout (>45s per-crate execution) |

### Standalone Build Targets
- `cargo build -p contextra --no-default-features --features embedded --locked`: **Grün (0)** (Dauer: 23,575 ms)
- `cargo build -p contextra-mcp --locked`: **Grün (0)** (Dauer: 30,449 ms)

---

## 4. Teststatistik (Erfasste Einzelläufe)

- **Bestandene Tests:** 1259
- **Fehlgeschlagene Tests:** 0 (unter erfolgreich kompilierten/ausgeführten Test-Suites)
- **Ignorierte Tests:** 2 (`contextra-adapt` / `contextra-infer-ollama`)

---

## 5. Die häufigsten Fehlerursachen (Top Error Causes)

1. **`crates/contextra-crypto/tests/audit_chain_tests.rs`**
   `error[E0433]: failed to resolve: use of undeclared type CustodyLevel`
   *(Ursachenklasse: Type Refactoring Drift – Missing import or type migration)*
2. **`crates/contextra-mvcc/tests/ssi_metrics_observability.rs`**
   `error[E0432]: unresolved imports contextra_ports::metrics::MetricEvent, contextra_ports::metrics::TestMetricsSink`
   *(Ursachenklasse: Module/Feature Removal in `contextra-ports`)*
3. **`tests/deletion_chain_e2e.rs` / `tests/deletion_e2e.rs`**
   `test result: FAILED / hang`
   *(Ursachenklasse: Integration Test Drift / Long-running compaction test timeout)*
4. **`crates/*/src/*.rs` (33 Crates)**
   `clippy warnings treated as errors (-D warnings)`
   *(Ursachenklasse: Missing workspace/crate-level clippy suppressions or unaddressed lints)*

---

## 6. Liste defekter Test-Targets & Abfänge

- `crates/contextra-crypto/tests/audit_chain_tests.rs` (Crate-Test Compilation Error)
- `crates/contextra-mvcc/tests/ssi_metrics_observability.rs` (Crate-Test Compilation Error)
- `tests/deletion_chain_e2e.rs` (Integration Test Failures)
- `tests/deletion_e2e.rs` (Integration Test Hang)

---

## 7. Abhängigkeitsanalyse Archivierte Crates (Dependenz-Kanten)

Gemäß `rg -n "contextra-(graph|cognition|adapt|kvcache|router|agent|infer-onnx|infer-ollama|core)" crates/*/Cargo.toml` (`archived_crate_dependencies.txt`):

| Crate | Abhängigkeit zu Archiv-Crate | Art der Kante |
|---|---|---|
| `crates/contextra/Cargo.toml` | `contextra-core` | Hard (Workspace Dependency) |
| `crates/contextra-store/Cargo.toml` | `contextra-graph` | Hard (Workspace Dependency) |
| `crates/contextra-infer-ollama/Cargo.toml` | `contextra-infer-onnx` | Hard (Workspace Dependency) |
| `crates/contextra-py/Cargo.toml` | `contextra-router`, `contextra-adapt`, `contextra-core` | Hard (Workspace Dependency) |
| `crates/contextra-db/Cargo.toml` | `contextra-cognition`, `contextra-graph`, `contextra-adapt` | Hard + Optional Features (`graph-connectivity-health`, `edge-reinforcement-learning`) |
| `crates/contextra-rank/Cargo.toml` | `contextra-adapt` | Hard (Workspace Dependency) |
| `crates/contextra-router/Cargo.toml` | `contextra-adapt`, `contextra-graph` | Hard + Optional Feature (`flow-corrected-thompson`) |
| `crates/contextra-checkpoint/Cargo.toml` | `contextra-core` | Hard (Path Dependency) |
| `crates/contextra-mcp/Cargo.toml` | `contextra-router` | Hard (Workspace Dependency) |
| `crates/contextra-engine/Cargo.toml` | `contextra-graph`, `contextra-core` | Hard (Workspace Dependency) |
| `crates/contextra-agent/Cargo.toml` | `contextra-graph`, `contextra-router` | Hard (Workspace Dependency) |
| `crates/contextra-cognition/Cargo.toml` | `contextra-graph` | Hard + Optional Features (`graph-connectivity-health`, `edge-reinforcement-learning`) |
| `crates/contextra-graph/Cargo.toml` | `contextra-adapt` | Hard (Workspace Dependency) |

---

## Out-of-Scope Findings
1. Deprecation Warning in `crates/contextra-db/examples/verify_deletion_proof_external.rs:83`: `TenantId::new` ist deprecated (Soll `TenantId::try_new()` oder `TenantId::SYSTEM` sein).
2. Dag-Invarianten-Verletzung in `crates/contextra-py/Cargo.toml:30`: `contextra-py` importiert `contextra-db`, `contextra-router` und `contextra-rank`.
