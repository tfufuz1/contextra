# Isolated Jules Prompts for Unambiguous Fixes (Schritt 5)

> **Hinweis für die Ausführung**: Die folgenden Prompts sind strikt modular, betreffen disjunkte Dateimengen und können unabhängig voneinander ausgeführt werden.

---

### Prompt 1: Entfernung unreferenzierter No-Op Cargo-Features (Schritt 1a #6a / FIND-08)

```markdown
Task: Clean up unreferenced No-Op Cargo Features across Workspace Cargo.tomls.

Root Cause:
Cargo features like `sieve-cache` in `contextra-store`, `ppr-forward-push` in `contextra-graph`, `kvcache-attention-eviction` in `contextra-kvcache`, `egress-sherman-morrison` in `contextra-router`, `bm25f` in `contextra-text`, `graph` in `contextra-vector`, and `edge-reinforcement-learning`, `coherence-bonus-fusion` in `contextra-engine` are declared in `Cargo.toml` files with empty dependency targets `[]` and have 0 corresponding `#[cfg(feature = "...")]` attributes in Rust source files.

Affected Files:
- `crates/contextra-store/Cargo.toml`
- `crates/contextra-graph/Cargo.toml`
- `crates/contextra-kvcache/Cargo.toml`
- `crates/contextra-router/Cargo.toml`
- `crates/contextra-text/Cargo.toml`
- `crates/contextra-vector/Cargo.toml`
- `crates/contextra-engine/Cargo.toml`

Constraints:
- DO NOT touch any `.rs` files or other `Cargo.toml` files.
- Remove ONLY the specified no-op feature keys from the `[features]` section of each respective `Cargo.toml`.

Verification Command:
`cargo check --workspace`
```

---

### Prompt 2: Bereinigung verwaister Ports in Ring 0 (`contextra-ports`) (FIND-07)

```markdown
Task: Remove zero-implementation orphaned traits `Snapshot` and `TextGenerator` from `contextra-ports`.

Root Cause:
`pub trait Snapshot` and `pub trait TextGenerator` in `crates/contextra-ports` have 0 implementations workspace-wide and create API confusion with active traits like `LlmTextGenerator`.

Affected Files:
- `crates/contextra-ports/src/snapshot.rs`
- `crates/contextra-ports/src/lib.rs`

Constraints:
- DO NOT modify any other trait definitions in `contextra-ports`.
- Remove `snapshot.rs` and its module re-export from `lib.rs`.

Verification Command:
`cargo check -p contextra-ports`
```

---

### Prompt 3: Konsolidierung von EgressGuard in `contextra-mcp` (FIND-04 / ADR-N15 Option 1)

```markdown
Task: Consolidate `EgressGuard` in `contextra-mcp` to use `contextra-privacy::EgressGuard`.

Root Cause:
`contextra-mcp` duplicates `EgressGuard` in `crates/contextra-mcp/src/egress_guard.rs` instead of reusing the canonical Ring 2 implementation `contextra_privacy::EgressGuard`.

Affected Files:
- `crates/contextra-mcp/Cargo.toml`
- `crates/contextra-mcp/src/egress_guard.rs`

Constraints:
- DO NOT touch `crates/contextra-privacy/src/egress_guard.rs`.
- Add `contextra-privacy` to `contextra-mcp/Cargo.toml` dependencies.
- Update `crates/contextra-mcp/src/egress_guard.rs` to re-export `pub use contextra_privacy::EgressGuard;`.

Verification Command:
`cargo test -p contextra-mcp`
```

---

### Prompt 4: Konsolidierung von AdaptiveDecayController in `contextra-engine` (FIND-05 / ADR-N15 Option 1)

```markdown
Task: Consolidate `AdaptiveDecayController` in `contextra-engine` to use `contextra_adapt::decay_controller::AdaptiveDecayController`.

Root Cause:
`contextra-engine` duplicates `AdaptiveDecayController` in `crates/contextra-engine/src/decay_controller.rs` instead of reusing Ring 0 `contextra_adapt::decay_controller::AdaptiveDecayController`.

Affected Files:
- `crates/contextra-engine/src/decay_controller.rs`

Constraints:
- DO NOT touch `crates/contextra-adapt/src/decay_controller.rs`.
- Replace duplicate struct implementation in `crates/contextra-engine/src/decay_controller.rs` with re-exporting `pub use contextra_adapt::decay_controller::AdaptiveDecayController;`.

Verification Command:
`cargo test -p contextra-engine`
```
