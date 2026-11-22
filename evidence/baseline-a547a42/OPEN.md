# Open Issues / Baseline Status (Commit `53c7d51401d5b67ce3358d5729fabd5b1f85fd4f` / `a547a42`)

## Übersicht
Der Build- und Testbaum ist auf dem Basis-Commit **rot**. Dieses Dokument erfasst alle fehlgeschlagenen Befehle, betroffenen Crates, Quelldateien und die ersten 20 Zeilen der jeweiligen Fehlermeldungen gemäß Auftrag WP 0.1.

---

### 1. `cargo check --workspace --all-targets --locked` (Exit: 101)
- **Crate:** `contextra-engine`
- **Datei:** `crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5`
- **Fehlerursache:** Unresolved import / missing dependency on `contextra_graph`.
```text
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_graph`
 --> crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5
  |
6 | use contextra_graph::hyperedge::HyperEdgeId;
  |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_graph`
  |
help: there is a crate or module with a similar name
  |
6 - use contextra_graph::hyperedge::HyperEdgeId;
6 + use contextra_rank::hyperedge::HyperEdgeId;
  |

error[E0432]: unresolved import `contextra_graph`
  --> crates/contextra-engine/src/collection/maintenance.rs:10:5
   |
10 | use contextra_graph::{detect_communities, CommunityAssignment, CommunityDetectionConfig};
   |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_graph`
```

---

### 2. `cargo clippy --workspace --all-targets --locked -- -D warnings` (Exit: 101)
- **Crate:** `contextra-engine`
- **Datei:** `crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5`
- **Fehlerursache:** Unresolved import / missing dependency on `contextra_graph`.
```text
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_graph`
 --> crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5
  |
6 | use contextra_graph::hyperedge::HyperEdgeId;
  |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_graph`
  |
help: there is a crate or module with a similar name
  |
6 - use contextra_graph::hyperedge::HyperEdgeId;
6 + use contextra_rank::hyperedge::HyperEdgeId;
  |

error[E0432]: unresolved import `contextra_graph`
  --> crates/contextra-engine/src/collection/maintenance.rs:10:5
   |
10 | use contextra_graph::{detect_communities, CommunityAssignment, CommunityDetectionConfig};
   |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_graph`
```

---

### 3. `cargo test --workspace --locked --no-run` (Exit: 101)
- **Crate:** `contextra-engine`
- **Datei:** `crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5`
- **Fehlerursache:** Compilation failed due to unresolved imports of `contextra_graph`.
```text
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_graph`
 --> crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5
  |
6 | use contextra_graph::hyperedge::HyperEdgeId;
  |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_graph`
```

---

### 4. `cargo test -p contextra-checkpoint --locked` (Exit: 101)
- **Crate:** `contextra-checkpoint`
- **Datei:** `crates/contextra-checkpoint/src/hardlink_cloner.rs:1:5`
- **Fehlerursache:** Unresolved imports of `contextra_core`.
```text
error[E0432]: unresolved import `contextra_core`
 --> crates/contextra-checkpoint/src/hardlink_cloner.rs:1:5
  |
1 | use contextra_core::{ContextraError, Result};
  |     ^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_core`
```

---

### 5. `cargo test -p contextra-engine --locked` (Exit: 101)
- **Crate:** `contextra-engine`
- **Datei:** `crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5`
- **Fehlerursache:** Unresolved imports of `contextra_graph`.
```text
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `contextra_graph`
 --> crates/contextra-engine/src/background_workers/hyperedge_worker.rs:6:5
  |
6 | use contextra_graph::hyperedge::HyperEdgeId;
  |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_graph`
```

---

### 6. `cargo test -p contextra-db --locked` (Exit: 101)
- **Crate:** `contextra-db`
- **Datei:** `crates/contextra-db/src/cognition_bridge.rs:1:5`
- **Fehlerursache:** Unresolved imports of `contextra_cognition`, `contextra_graph`, `contextra_adapt`.
```text
error[E0432]: unresolved import `contextra_cognition`
 --> crates/contextra-db/src/cognition_bridge.rs:1:5
  |
1 | use contextra_cognition::synthesis_phase::SynthesisPhaseResult;
  |     ^^^^^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_cognition`

error[E0432]: unresolved import `contextra_graph`
 --> crates/contextra-db/src/graph_bridge.rs:1:5
  |
1 | use contextra_graph::percolation::PercolationConfig;
  |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_graph`
```

---

### 7. `cargo test -p contextra-mcp --locked` (Exit: 101)
- **Crate:** `contextra-mcp`
- **Datei:** `crates/contextra-mcp/src/agent_lifecycle/state_machine.rs:8:5`
- **Fehlerursache:** Unresolved imports of `contextra_agent`, `contextra_privacy`, `contextra_rank`, `contextra_router`.
```text
error[E0432]: unresolved import `contextra_agent`
 --> crates/contextra-mcp/src/agent_lifecycle/state_machine.rs:8:5
  |
8 | use contextra_agent::{AgentContext, DeadLetterQueue, OrchestratorEngine};
  |     ^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_agent`

error[E0432]: unresolved import `contextra_privacy`
 --> crates/contextra-mcp/src/egress_vault.rs:3:5
  |
3 | use contextra_privacy::{EgressFilterRule, PrivacyPolicyEngine};
  |     ^^^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_privacy`
```

---

### 8. `cargo test -p contextra --locked` (Exit: 101)
- **Crate:** `contextra`
- **Datei:** `crates/contextra/src/builder.rs:12:5`
- **Fehlerursache:** Unresolved imports of `contextra_core` and `contextra_license`.
```text
error[E0432]: unresolved import `contextra_core`
  --> crates/contextra/src/builder.rs:12:5
   |
12 | use contextra_core::{ContextraError, Result};
   |     ^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_core`

error[E0432]: unresolved import `contextra_license`
  --> crates/contextra/src/license.rs:1:5
   |
1  | use contextra_license::SignedLicenseGate;
   |     ^^^^^^^^^^^^^^^^^ use of unresolved module or unlinked crate `contextra_license`
```

---

### 9. `cargo fmt --check` (Exit: 1)
- **Crate:** `contextra-db`
- **Datei:** `benches/`
- **Fehlerursache:** References to non-existent example/bench relative file paths in manifest.
```text
Error: file `/app/crates/contextra-db/../../benches/competitive_bench.rs` does not exist
Error: file `/app/crates/contextra-db/../../benches/migration_benchmarks.rs` does not exist
Error: file `/app/crates/contextra-db/../../benches/relate_bench.rs` does not exist
Error: file `/app/crates/contextra-db/../../benches/rrf_scale_bench.rs` does not exist
Error: file `/app/crates/contextra-db/../../benches/scale_bench.rs` does not exist
```

---

### 10. `cargo deny check` (Exit: 7)
- **Crate:** `contextra-mcp`
- **Datei:** `crates/contextra-mcp/Cargo.toml`
- **Fehlerursache:** Unlicensed crate and duplicate dependency entries.
```text
error[unlicensed]: contextra-mcp = 0.1.0 is unlicensed
 ├ contextra-mcp v0.1.0 (*)
warning[duplicate]: found 2 duplicate entries for crate 'base64'
```

---

### 11. `cargo audit` (Exit: 1)
- **Crate:** `workspace`
- **Datei:** `Cargo.lock`
- **Fehlerursache:** 33 security vulnerabilities found in dependencies.
```text
Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 1378 security advisories (from /home/jules/.cargo/advisory-db)
    Updating crates.io index
    Scanning Cargo.lock for vulnerabilities (575 crate dependencies)
error: 33 vulnerabilities found!
```

---

### 12. External Cargo Subcommands (`public-api`, `miri`, `mutants`) (Exit: 101 / 1)
- **Fehlerursache:** Subcommands `public-api`, `miri`, and `mutants` are not installed or available in the toolchain environment.

---

### 13. `cargo xtask check`, `cargo xtask audit`, `cargo xtask verify` (Exit: 1)
- **Crate:** `xtask`
- **Fehlerursache:** Failures in underlying gates (e.g., clippy warnings, deny checks, missing licenses).
