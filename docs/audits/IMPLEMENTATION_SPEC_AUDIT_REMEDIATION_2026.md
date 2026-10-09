# Contextra – Architectural Implementation Specification: Audit Remediation & CI Gate Hardening
**Document-ID:** ARCH-SPEC-AUDIT-REMEDIATION-2026-10
**Author:** Principal Senior Rust Architect, Contextra Ökosystem
**Date:** 09.10.2026
**Target HEAD:** ace2a42 (and subsequent releases)

---

## 1. Executive Summary & Audit Synthesis

An architectural review of the five audit scripts (`scan_stub_implementations.py`, `scan_doctrine_violations.py`, `audit_unwired_ports.py`, `audit_dependency_graph.py`, `scan_orphan_symbols.py`) and their corresponding `cargo xtask` implementations was conducted.

While the preliminary script outputs identified superficial metrics (such as 893 panic violations or 7 unwired ports), an in-depth codebase counterprobe revealed that a significant portion of these metrics were scan artifacts caused by naive regular-expression matching and failure to isolate test fixtures from production code.

This specification details the precise technical remedies required across the codebase to:
1. Eliminate all genuine production flaws (e.g., `unreachable!` in `HnswIndex::commit`, unwired core ports `KmsProvider` and `ModuleVerifier`).
2. Hard-wire the KV deletion and `DeletionProof` pipeline (WP-E) across `contextra-engine` and `contextra-crypto`.
3. Upgrade all `cargo xtask` audit commands to use AST-based (`syn`) parsing via `WorkspaceIndex`, ensuring deterministic CI gates with zero false positives.

---

## 2. Specification 1: Stub & Placeholder Remediation (`stub-impls`)

### 2.1 Problem Analysis
`cargo xtask stub-impls` classifies functions containing macro calls (`unreachable!`, `todo!`, `unimplemented!`) or empty function bodies (`{}`) as **🔴 HOCH**.

1. **Genuine Critical Defect:** `HnswIndex::commit` in `crates/contextra-vector/src/hnsw/vector_index_impl.rs` calls `_ => unreachable!()` inside a `match op` statement during transaction commit. Even if preceded by a validation pass, invoking `unreachable!()` in Ring 0 core code violates the Zero-Panic (P7) doctrine and risks process crashes on unforeseen `IndexOp` variants.
2. **Scan Artifacts (Null Objects / Intentional No-Ops):** Empty method bodies in `NoopScratchpadCacheInvalidator`, `NoopContextEditAuditSink`, `NoopMetricsSink`, and `NoopReplayProgressSink` represent intentional Null Objects used when optional observability or caching is disabled.

### 2.2 Precise Implementation Steps

#### Step 1.1: Eradicate `unreachable!` in `HnswIndex::commit`
Modify `crates/contextra-vector/src/hnsw/vector_index_impl.rs`:
- Replace `_ => unreachable!()` with explicit error propagation or an exhaustive match over `IndexOp::Insert` and `IndexOp::Delete`.
- If an unsupported `IndexOp` variant is encountered in Phase 2, return `Err(ContextraError::Index(...))` rather than invoking a panic macro.

#### Step 1.2: Intentional No-Op Annotation Standard
To prevent `cargo xtask stub-impls` from flagging valid Null Object patterns:
- Annotate intentional empty method bodies in Ring 0-4 crates with `// INTENTIONAL-NOOP: Null Object Pattern`.
- Update `xtask/src/harness/stub_impls.rs` to ignore method bodies containing the `INTENTIONAL-NOOP` annotation or belonging to structs matching `Noop*` / `Null*` naming conventions.

---

## 3. Specification 2: Zero-Panic Doctrine & AST-Based Code Classification (`doctrine-scan`)

### 3.1 Problem Analysis
`scan_doctrine_violations.py` reported 893 violations across crates, causing false `status_contradiction` errors for completed crates. Counterprobe confirmed that 874 of these findings originate in test code (`#[cfg(test)]` blocks, `tests/` directories, `benches/`, and `contextra-testkit`).

Only 19 instances of `unwrap`/`expect`/`panic!` reside in non-test production code.

### 3.2 Precise Implementation Steps

#### Step 2.1: AST-Based Test Scope Isolation in `cargo xtask doctrine-scan`
Update `xtask/src/harness/doctrine_scan.rs`:
- Integrate `syn::File` AST parsing to accurately determine if an AST item is enclosed within an item decorated with `#[cfg(test)]`, `#[test]`, `#[bench]`, or located within an integration test path (`tests/`, `benches/`, `examples/`).
- Exclude all test-scoped items from production panic counts.

#### Step 2.2: Systematic Elimination of Production Panics
For the 19 production panic sites across `contextra-engine`, `contextra-vector`, `contextra-checkpoint`, and `contextra-store`:
- Convert all `unwrap()` and `expect()` calls to proper `?` error propagation returning `ContextraError`.
- For array index bounds or mutex locks, use safe checked operations (`get()`, `map_err(...)`).

#### Step 2.3: `#[allow(...)]` Security Lint Governance
For the 80 `#[allow(...)]` overrides on security-relevant lints (`unwrap_used`, `expect_used`, `panic`, `unsafe_code`):
- Restrict `#[allow(unsafe_code)]` strictly to declared unsafe islands (`contextra-simd`, `contextra-sys`, `contextra-wire`).
- For any remaining `#[allow(clippy::unwrap_used)]` or `#[allow(clippy::panic)]` in production code, require an explicit rationale comment citing the relevant ADR or safety invariant (e.g., `// INTENTIONAL-ALLOW: ADR-023`).

---

## 4. Specification 3: Unwired Ports & Composition-Root Resolution (`unwired-ports`)

### 4.1 Problem Analysis
The initial report listed 7 `UNWIRED` traits. AST analysis shows:
- **Genuine Unwired Ports (2):** `KmsProvider` (`contextra-crypto/src/wal_crypto.rs`) and `ModuleVerifier` (`contextra-sandbox/src/admission.rs`) lacked concrete production implementations in their respective crates.
- **Scan Artifacts (5):** `Checkpoint`, `CheckpointCoordinator`, `CommunityResolver`, `ContextPreparer`, `HybridSearchProvider` are implemented in `contextra-checkpoint` or `contextra-engine`, but were missed due to generic parameters (`impl<S: StorageEngine> CheckpointCoordinator for ...`) or module path re-exports.

### 4.2 Precise Implementation Steps

#### Step 3.1: Concrete Implementation for `KmsProvider`
In `crates/contextra-crypto/src/wal_crypto.rs`:
- Implement `LocalKeyManagerKms` wrapping `crate::crypto::KeyManager` to fulfill `KmsProvider`, providing secure DEK retrieval for Encryption at Rest.

#### Step 3.2: Concrete Implementation for `ModuleVerifier`
In `crates/contextra-sandbox/src/admission.rs`:
- Implement `DefaultModuleVerifier` to fulfill `ModuleVerifier`, validating WASM binary payloads against header magic bytes (`\0asm`) and expected digest hashes before constructing an `AdmittedModule`.

#### Step 3.3: AST Cross-Crate Trait Resolution in `cargo xtask unwired-ports`
Update `xtask/src/harness/unwired_ports.rs`:
- Parse generic `impl<...> Trait for Type` blocks using `syn`.
- Resolve trait aliases and re-exported types to ensure zero false positives for `UNWIRED` or `SOLO` classifications.

---

## 5. Specification 4: Ring 0 Dependency Graph & Async-Kernel Governance (`dependency-graph-audit` / P26)

### 5.1 Problem Analysis
Audit confirms **0 cycles** and **0 upward dependency edges** across all 34 workspace crates.
Regarding Ring 0 async purity (P26):
- 0 Ring 0 crates include `tokio` in their production `[dependencies]`.
- Ring 0 crates (`contextra-vector`, `contextra-graph`, `contextra-ports`, `contextra-text`) expose `async fn` signatures for non-blocking state machines.

### 5.2 Precise Implementation Steps

#### Step 4.1: Ring 0 Async Kernel Standard
Confirm the architectural invariant:
- Ring 0 crates are permitted to expose `async fn` futures representing pure non-blocking state machine state transitions.
- Ring 0 code **MUST NEVER** block the executing thread, perform blocking synchronous disk I/O inside futures, or instantiate an async runtime (Tokio/async-std) within Ring 0 production code.

---

## 6. Specification 5: WP-E Deletion Proof Pipeline Wiring & Orphan Symbols Governance (`orphan-symbols`)

### 6.1 Problem Analysis
Functions `delete_kv_segment` and `generate_kv_deletion_proof` in `contextra-engine/src/collection/mod.rs` delegate deletion operations to `KvSegmentManager` in `contextra-store`.
The report noted these as "orphan symbols" because they were not yet exposed in top-level facade APIs or called outside integration tests.

### 6.2 Precise Implementation Steps

#### Step 5.1: Wire KV Deletion & Proof Methods in `Collection` & `Contextra` Facades
In `crates/contextra-engine/src/collection/mod.rs` and `crates/contextra/src/lib.rs`:
- Expose `purge_kv_segment_and_prove(&self, group_id: u64) -> Result<DeletionProof>` on `Collection` and `Contextra`.
- Connect the invocation chain: `Contextra::purge_tenant_kv` -> `Collection::delete_kv_segment` -> `KvSegmentManager::delete_segment` -> `RevocationLog::append` -> `DeletionProof::create`.

#### Step 5.2: Public API Boundary Classification in `cargo xtask orphan-symbols`
Update `xtask/src/harness/orphan_symbols.rs`:
- Recognize `pub fn` items on public facade structs (`Contextra`, `Collection`, `LsmStorage`, `Builder`) as intentional public API boundary exports when annotated with `#[doc(inline)]` or belonging to Ring 4 entry crates, avoiding false positive `ORPHAN` flags.

---

## 7. Specification 6: Hardened CI/CD Quality Gates

All five audit scripts are fully consolidated as `cargo xtask` commands in `xtask/src/harness/`:
1. `cargo xtask stub-impls`
2. `cargo xtask doctrine-scan`
3. `cargo xtask unwired-ports`
4. `cargo xtask dependency-graph-audit`
5. `cargo xtask orphan-symbols`

These commands return exit code `0` on success and exit code `2` on non-negotiable policy violations, making them fully suitable as blocking CI gates in `.github/workflows/` and `cargo xtask jules check`.
