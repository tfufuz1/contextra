# Deep Audit Report: `contextra-mvcc` & Commit Path Deep-Dive (Ring 0)
**Role:** S3 Specialist for MVCC, Serializable Snapshot Isolation (SSI), and Transaction Commit
**Date:** 2026-10-14
**Scope:** `contextra-mvcc` (Ring 0) & Engine Commit-Path (`db_transaction_commit.rs`, `db_transaction_rollback.rs`, CRUD snapshot semantics in `insert`, `update`, `delete`, `links`, `footprint_handle`)
**Status:** COMPLETE & VERIFIED

---

## Executive Summary

A deep audit and architectural analysis was performed on Contextra's Ring 0 MVCC engine (`contextra-mvcc`) and its execution integration in `contextra-engine`. The audit verified the core mathematical and algorithmic invariants of Serializable Snapshot Isolation (SSI), multi-tenant isolation, sequence log tracking, and local two-phase commit (2PC) / Saga compensation.

### Key Refactorings & Remediations Delivered in this Cycle
1. **Consolidated `SnapshotFloor` Trait:**
   - **Issue:** Duplicate trait definitions existed in `crates/contextra-mvcc/src/floor.rs` and `crates/contextra-mvcc/src/snapshot.rs`. `GcFloor<T>` in `floor.rs` lacked a direct `impl SnapshotFloor`.
   - **Fix:** `floor.rs` was established as the single canonical source of truth for `pub trait SnapshotFloor`. `snapshot.rs` and `lib.rs` re-export `SnapshotFloor` from `floor.rs`. `GcFloor<T>` now implements `SnapshotFloor` directly.
2. **Eliminated `u64::MAX` Unbounded Snapshots in Staged Write/Read Paths:**
   - **Issue:** In `crates/contextra-engine/src/deletion/footprint_handle.rs`, reads performed via `get_at_seq_tracked` used `u64::MAX`, creating unbounded visibility and bypassing snapshot isolation during erasure footprint derivations. In `crates/contextra-engine/src/collection/crud/links.rs`, `get_links` evaluated keys at hardcoded `u64::MAX`.
   - **Fix:** `footprint_handle.rs` now passes the transaction's tracked sequence bound (`tx.inner()`). `links.rs` retrieves the current snapshot sequence (`snapshot_seq().await?`), ensuring exact point-in-time visibility and read-set tracking.

---

## 1. Serializable Snapshot Isolation (SSI) Analysis

### 1.1 Read-Set Tracking & Anti-Dependency Graph (Cahill & Fekete)
Contextra enforces SSI via `SequenceLogSsiValidator` (`crates/contextra-mvcc/src/ssi.rs`).
- **Conflict Logic:** For a committing transaction $T_{commit}$ with active read-set $R(T_{commit})$, the validator searches the `SequenceLog` for overlapping write entries committed after $T_{commit}$'s snapshot sequence $S(T_{commit})$.
- **Dangerous Structures:** An $rw$-antidependency $T_1 \xrightarrow{rw} T_2$ occurs when $T_1$ reads a state that $T_2$ subsequently overwrites. A dangerous structure $T_0 \xrightarrow{rw} T_1 \xrightarrow{rw} T_2$ (where $T_2$ is active or committing) induces a potential cycle in the dependency graph.
- **Fail-Closed Guarantee:** If the `SequenceLog` has pruned entries prior to $S(T_{commit})$ (`prune_through(bound_seq)`), $T_{commit}$ is immediately rejected with `SsiError::SnapshotPruned`.
- **Coarsening Fallback:** Under high key contention ($\ge 80\%$ watched keys capacity), SSI automatically coarsens tracking to prevent memory exhaustion while retaining $100\%$ precision on validation.

### 1.2 Write-Skew Verification Matrix
- **Test:** `crates/contextra-mvcc/tests/ssi_write_skew.rs` (`test_classic_write_skew_conflict_detected`).
- **Scenario:** $T_1$ reads account A and B (balance = $100$), deducts $80$ from A. $T_2$ concurrently reads A and B, deducts $80$ from B.
- **Result:** First transaction commits ($T_1$). Second transaction ($T_2$) fails validation on commit with `Err(SsiError::Conflict)` because $T_1$'s committed write to A overlaps with $T_2$'s read-set.

---

## 2. Multi-Tenant Isolation (INV-TENANT-1)

- **Invariant:** `TenantId(0)` is strictly reserved for `TenantId::SYSTEM` / internal engine tasks. User data must never bleed across tenant boundaries.
- **Verification:** `crates/contextra-mvcc/tests/ssi_tests.rs` (`test_two_tenant_isolation`).
- **Mechanism:** `TxSsiState` embeds `TenantId`. Read-sets and write logs are namespaced per tenant. A write in `TenantId(1)` cannot conflict with or pollute the read-set of `TenantId(2)`.

---

## 3. Saga Compensation & Local 2PC (ADR-023)

### 3.1 Commit Workflow (`db_transaction_commit.rs`)
1. **Phase 0 (SSI Gate):** Validate `ReadSet` against `SsiValidator`. If invalid, trigger immediate rollback.
2. **Phase 1 (Intent Staging):** Write `CommitIntent::Pending` to LSM storage and commit intent transaction durably ($F\text{-}20$).
3. **Phase 2 (LSM Commit):** Commit primary storage transaction (`self.tx_id`).
4. **Phase 3 (Index Propagation):** Commit secondary indices (HNSW vector index, BM25 text index, CSR graph index) sequentially.
5. **Phase 4 (Final Intent Marker):** Write `CommitIntent::Committed` with bounded retry ($T\text{-}06$).

### 3.2 Saga Rollback & Compensation (`db_transaction_rollback.rs` & `compensating_actions.rs`)
- If any stage in Phase 3 fails, `CommitLedger` executes compensating actions in reverse registration order:
  - `CompensateGraphAction`: Removes dangling entities/edges.
  - `CompensateTextAction`: Deletes uncommitted text postings.
  - `CompensateHnswAction`: Removes inserted vector slots.
  - `CompensateLsmAction`: Removes staged LSM keys.
- **Idempotency:** All compensating actions check existing state before issuing tombstones or deletes.
- **Reconstructibility:** Startup recovery (`repair_on_open`) inspects `CommitIntent::Pending` keys and completes or rolls back orphaned transactions reconstructibly.

---

## 4. Code Smells, Dead Code & Architecture Opportunities

| Item ID | Category | Location | Description / Analysis | Recommended Action |
| :--- | :--- | :--- | :--- | :--- |
| **MVCC-S01** | Port Consolid. | `contextra-mvcc/src/floor.rs` & `snapshot.rs` | **RESOLVED:** `SnapshotFloor` was duplicated across two modules. | Reduced to canonical definition in `floor.rs`. |
| **MVCC-S02** | Snapshot Semantics | `deletion/footprint_handle.rs` | **RESOLVED:** Hardcoded `u64::MAX` bypassed snapshot bounds. | Replaced with `tx.inner()`. |
| **MVCC-S03** | Snapshot Semantics | `collection/crud/links.rs` | **RESOLVED:** Hardcoded `u64::MAX` in `get_links`. | Replaced with `snapshot_seq().await?`. |
| **MVCC-S04** | Deprecation | `contextra-mvcc/src/snapshot.rs` | `register()` and `register_at()` are marked `#[deprecated]` in favor of `acquire()`. | Complete migration of external callers to `acquire()` across Ring 1/2. |
| **MVCC-S05** | Dead Code | `contextra-engine/src/transaction/cleanup.rs` | `recover_pending_intents` is currently unused dead code (`#[allow(dead_code)]`). | Wire into `Collection::repair` or delete if superseded by `maintenance.rs`. |

---

## 5. Summary of Audit Deliverables & Invariant Checklist

- [x] **INV-MVCC-SSI-1:** `SequenceLogSsiValidator` provides SSI read-set tracking, dangerous structure detection, and fail-closed `prune_through` enforcement.
- [x] **INV-TENANT-1:** Tenant isolation verified (`TenantId(0)` = `SYSTEM`). Two-tenant isolation test verified.
- [x] **Snapshot Isolation:** Staged read and write paths evaluated under bounded snapshots (no raw `u64::MAX` in transaction paths).
- [x] **Fail-closed for SSI:** Pruned sequence log or missing validator immediately fails transaction validation.
