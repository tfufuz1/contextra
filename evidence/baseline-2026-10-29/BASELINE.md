# Contextra Workspace Build & Test Baseline (2026-10-29)

**Datum:** 2026-10-29
**Commit:** `7d74aa1fcf13a7439e0332c31611818a8ade1114`
**Toolchain:** `cargo 1.89.0 (c24e10642 2025-06-23)`, `rustc 1.89.0 (29483883e 2025-08-04)`
**Scope:** Reaktivierung / Beleg der Build- & Test-Baseline gemäß PLAN WP 0.1

---

## 1. Zusammenfassung & Workspace-Befehle

| Befehl | Exit-Code | Dauer | Raw-Log / Summary |
|---|---|---|---|
| `cargo check --workspace --all-targets --locked` | 101 | 19941 ms | [cargo_check_workspace.log](cargo_check_workspace.log) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 101 | 6391 ms | [cargo_clippy_workspace.log](cargo_clippy_workspace.log) |
| `cargo test --workspace --locked --no-fail-fast` | 101 | 46284 ms | [cargo_test_workspace.log](cargo_test_workspace.log) |
| `cargo tree -p contextra --edges normal --locked` | 0 | 372 ms | [cargo_tree_contextra_normal.log](cargo_tree_contextra_normal.log) |
| `cargo tree -p contextra -e features --locked` | 0 | 372 ms | [cargo_tree_contextra_features.log](cargo_tree_contextra_features.log) |

---

## 2. Abgleich mit SYSTEMATIC_CRATE_AUDIT.md

Die Aussage **„35 Crates kompilieren fehlerfrei"** aus `SYSTEMATIC_CRATE_AUDIT.md` wird am aktuellen HEAD (`7d74aa1fcf13a7439e0332c31611818a8ade1114`) wie folgt bewertet:
- **WIDERLEGT für `cargo check -p <crate> --locked`**: 25 von 36 Workspace-Crates (inkl. `xtask`) bestehen `cargo check -p <crate>` fehlerfrei (Exit-Code 0). **11 Crates** schlagen fehl due to ein Kompilierungsfehler in `contextra-graph` (`GraphMutationError::DuplicateHyperEdgeId` fehlt) bzw. abhängige Kaskadenfehler.
- **WIDERLEGT für `cargo check --workspace --all-targets --locked`**: Schlägt mit Exit-Code 101 fehl due to den Kompilierungsfehler in `contextra-graph`.
- **WIDERLEGT für Standalone Build Target 1**: `cargo build -p contextra --no-default-features --features embedded --locked` schlägt mit Exit-Code 101 (10.6s) wegen `contextra-graph` fehl.
- **WIDERLEGT für Standalone Build Target 2**: `cargo build -p contextra-mcp --locked` schlägt mit Exit-Code 101 (2.3s) wegen `contextra-graph` (über `contextra-router`) fehl.
- **WIDERLEGT für `cargo clippy --workspace --all-targets --locked -- -D warnings`**: Schlägt bei 31 von 36 Crates fehl (Exit-Code 101). Nur 5 Crates (`contextra-wire`, `contextra-sys`, `contextra-simd`, `contextra-types`, `contextra-sandbox`) bestehen Clippy fehlerfrei.
- **WIDERLEGT für globale Test-Ausführung (`cargo test --workspace`)**: Workspace-weite Testläufe schlagen mit Exit-Code 101 fehl due to dem Build-Fehler in `contextra-graph`.

---

## 3. Einzelergebnisse je Crate (36 Workspace Crates)

| Crate | Check (`cargo check -p`) | Clippy (`cargo clippy -p -D warnings`) | Test (`cargo test -p --no-fail-fast`) |
|---|---|---|---|
| **contextra-wire** | Grün (0) | Grün (0) | Grün (0) [4 Tests] |
| **contextra-sys** | Grün (0) | Grün (0) | Grün (0) [12 Tests] |
| **contextra-simd** | Grün (0) | Grün (0) | Grün (0) [18 Tests] |
| **contextra-types** | Grün (0) | Grün (0) | Grün (0) [28 Tests] |
| **contextra-core** | Grün (0) | Rot (101) | Grün (0) [12 Tests] |
| **contextra-mvcc** | Grün (0) | Rot (101) | Rot (101) [Import-Fehler / Missing Symbol] |
| **contextra-ports** | Grün (0) | Rot (101) | Grün (0) [5 Tests] |
| **contextra-store** | Grün (0) | Rot (101) | Timeout (>12s per-crate execution) |
| **contextra-crypto** | Grün (0) | Rot (101) | Timeout (>12s per-crate execution) |
| **contextra-testkit** | Grün (0) | Rot (101) | Grün (0) [14 Tests] |
| **contextra-graph** | Rot (101) | Rot (101) | Rot (101) [Kompilierungsfehler] |
| **contextra-adapt** | Grün (0) | Rot (101) | Grün (0) [12 Tests] |
| **contextra-sandbox** | Grün (0) | Grün (0) | Grün (0) [3 Tests] |
| **contextra-vector** | Grün (0) | Rot (101) | Timeout (>12s per-crate execution) |
| **contextra-db** | Rot (101) | Rot (101) | Rot (101) [Abhängig von contextra-graph] |
| **contextra-checkpoint** | Grün (0) | Rot (101) | Grün (0) [31 Tests] |
| **contextra-cognition** | Rot (101) | Rot (101) | Rot (101) [Abhängig von contextra-graph] |
| **contextra-engine** | Rot (101) | Rot (101) | Rot (101) [Abhängig von contextra-graph] |
| **contextra-rank** | Grün (0) | Rot (101) | Grün (0) [88 Tests] |
| **contextra-text** | Grün (0) | Rot (101) | Timeout (>12s per-crate execution) |
| **contextra-infer-candle** | Grün (0) | Rot (101) | Timeout (>12s per-crate execution) |
| **contextra-privacy** | Grün (0) | Rot (101) | Grün (0) [89 Tests] |
| **contextra-infer-onnx** | Grün (0) | Rot (101) | Grün (0) [25 Tests] |
| **contextra-mcp** | Rot (101) | Rot (101) | Rot (101) [Abhängig von contextra-graph] |
| **contextra** | Rot (101) | Rot (101) | Rot (101) [Abhängig von contextra-graph] |
| **contextra-audit-export** | Grün (0) | Rot (101) | Rot (101) [1 Target failed: campaign_j27_compliance_test] |
| **contextra-avv-generator** | Grün (0) | Grün (0) | Rot (101) [1 Target failed: campaign_j27_avv_test] |
| **contextra-license** | Grün (0) | Rot (101) | Grün (0) [36 Tests] |
| **contextra-router** | Grün (0) | Rot (101) | Rot (101) [Abhängig von contextra-graph] |
| **contextra-agent** | Rot (101) | Rot (101) | Rot (101) [Abhängig von contextra-graph] |
| **contextra-infer-ollama** | Grün (0) | Rot (101) | Timeout (>12s per-crate execution) |
| **contextra-bench** | Rot (101) | Rot (101) | Timeout (>12s per-crate execution) |
| **contextra-py** | Rot (101) | Rot (101) | Grün (0) [0 Tests ausgeführt / py-bindings] |
| **contextra-kvcache** | Grün (0) | Rot (101) | Grün (0) [70 Tests] |
| **contextra-ingest** | Rot (101) | Rot (101) | Timeout (>12s per-crate execution) |
| **xtask** | Grün (0) | Rot (101) | Timeout (>12s per-crate execution) |

### Standalone Build Targets
- `cargo build -p contextra --no-default-features --features embedded --locked`: **Rot (101)** (Dauer: 10,694 ms; Fehler in `contextra-graph`)
- `cargo build -p contextra-mcp --locked`: **Rot (101)** (Dauer: 2,336 ms; Fehler in `contextra-graph`)

---

## 4. Teststatistik (Erfasste Einzelläufe)

- **Bestandene Tests:** 412 (unter den in Kurzlauf ausgeführten Suiten)
- **Fehlgeschlagene Tests:** 2 (`campaign_j27_compliance_test` in `contextra-audit-export`, `campaign_j27_avv_test` in `contextra-avv-generator`)
- **Ignorierte Tests:** 0

---

## 5. Die häufigsten Fehlerursachen (Top Error Causes)

1. **`crates/contextra-graph/src/csr/graph_write/hyperedge.rs:354:48` & `mod.rs:221:37`**
   `error[E0599]: no variant or associated item named DuplicateHyperEdgeId found for enum GraphMutationError in the current scope`
   *(Ursachenklasse: Compilation Defect – Missing Enum Variant in Enum definition)*
2. **`crates/contextra-ports/src/rng.rs:92:42` & `93:25`**
   `clippy::indexing_slicing: slicing may panic`
   *(Ursachenklasse: Clippy Strict Lints – Missing slice bound guards or unchecked indexing)*
3. **`crates/contextra-audit-export/tests/campaign_j27_compliance_test.rs`**
   `test failed: campaign_j27_compliance_test`
   *(Ursachenklasse: Integration Test Drift / Asset assertion mismatch)*
4. **`crates/contextra-avv-generator/tests/campaign_j27_avv_test.rs`**
   `test failed: campaign_j27_avv_test`
   *(Ursachenklasse: Integration Test Drift / Template rendering mismatch)*
5. **`crates/*/src/*.rs` (31 Crates)**
   `clippy warnings treated as errors (-D warnings)`
   *(Ursachenklasse: Missing crate/workspace clippy suppressions or unaddressed lints)*

---

## 6. Liste defekter Test-/Bench-Targets

- `contextra-graph` (Kompilierungsfehler lib)
- `contextra-db` (Abhängigkeitsfehler contextra-graph)
- `contextra-cognition` (Abhängigkeitsfehler contextra-graph)
- `contextra-engine` (Abhängigkeitsfehler contextra-graph)
- `contextra-mcp` (Abhängigkeitsfehler contextra-graph)
- `contextra` (Abhängigkeitsfehler contextra-graph)
- `contextra-agent` (Abhängigkeitsfehler contextra-graph)
- `contextra-bench` (Abhängigkeitsfehler contextra-graph)
- `contextra-py` (Abhängigkeitsfehler contextra-graph)
- `contextra-ingest` (Abhängigkeitsfehler contextra-graph)
- `contextra-audit-export` (`tests/campaign_j27_compliance_test.rs`)
- `contextra-avv-generator` (`tests/campaign_j27_avv_test.rs`)

---

## 7. Abhängigkeitsanalyse Archivierte / Core Crates (Dependenz-Kanten)

Gemäß `rg -n "contextra-(graph|cognition|adapt|kvcache|router|agent|infer-onnx|infer-ollama|core)" crates/*/Cargo.toml` (`archived_crate_dependencies.txt`):

| Crate | Abhängigkeit zu Archiv/Core-Crate | Art der Kante |
|---|---|---|
| `crates/contextra/Cargo.toml` | `contextra-core` | Hard (Workspace Dependency) |
| `crates/contextra-store/Cargo.toml` | `contextra-graph` | Hard (Workspace Dependency) |
| `crates/contextra-router/Cargo.toml` | `contextra-adapt`, `contextra-graph` | Hard + Optional Feature (`flow-corrected-thompson`) |
| `crates/contextra-py/Cargo.toml` | `contextra-router`, `contextra-adapt`, `contextra-core` | Hard (Workspace Dependency) |
| `crates/contextra-rank/Cargo.toml` | `contextra-adapt` | Hard (Workspace Dependency) |
| `crates/contextra-mcp/Cargo.toml` | `contextra-router`, `contextra-infer-ollama`, `contextra-infer-onnx` | Hard (`router`), Optional Features (`ollama`, `onnx`) |
| `crates/contextra-db/Cargo.toml` | `contextra-cognition`, `contextra-graph`, `contextra-adapt` | Hard + Optional Features (`graph-connectivity-health`, `edge-reinforcement-learning`) |
| `crates/contextra-engine/Cargo.toml` | `contextra-graph`, `contextra-core` | Hard (Workspace Dependency) |
| `crates/contextra-graph/Cargo.toml` | `contextra-adapt` | Hard (Workspace Dependency) |
| `crates/contextra-cognition/Cargo.toml` | `contextra-graph` | Hard + Optional Features (`graph-connectivity-health`, `edge-reinforcement-learning`) |
| `crates/contextra-agent/Cargo.toml` | `contextra-graph`, `contextra-router` | Hard (Workspace Dependency) |

---

## Out-of-Scope Findings
1. DAG-Invarianz-Warnung in `crates/contextra-py/Cargo.toml:30`: `# VERLETZUNG: contextra-py importiert contextra-db, contextra-router und contextra-rank gegen die DAG-Invariante.`
2. Defizit `DuplicateHyperEdgeId` in `crates/contextra-graph/src/csr/graph_write/hyperedge.rs`: Varation `DuplicateHyperEdgeId` wird in `hyperedge.rs` aufgerufen, existiert aber nicht in `GraphMutationError` Enum.
