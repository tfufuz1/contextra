# AUDIT REPORT: Pre-IP-20 Benchmark Baseline & AK-8/AK-4 Regression Protection

**Document Class:** S (Critical Subsystem & Baseline Verification)
**Status:** AUDIT COMPLETED & COMPILATION BLOCKER IDENTIFIED
**Date:** 2026-09-17
**Auditor:** Google-Jules (Principal Rust Systems Engineer — Reviewer / Audit Role)
**Scope:** Workspace-wide Benchmark Baseline Tooling (`just bench-set-baseline`, `bench-delta`)
**Base Commit (HEAD):** `1a73baaeef1d4f3578c6a2e3664681a55453333e`
**Timestamp:** 2026-09-17T13:10:00Z
**Claim Scope:** `cargo xtask claim --crate workspace --scope pre-ip20-bench-baseline`

---

## 1. MISSION & AUDIT CONTEXT

This audit establishes the **Vor-IP-20 (Pre-Hyperedges Welle 8)** benchmark baseline required for validating **AK-8 / AK-4 regression freedom** as specified in:
* `CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v11_2_HYPEREDGES_KRITISCH.md` §3, AK-8:
  *"Kein Regressions-Impact auf bestehende binäre `relate()`/`Edge`-Benchmarks (unverändert aus v11.1)."*
* `v11_1_HYPEREDGES.md` §6, Item 4.

### Pre-Baseline Historical Verification
Prior to this execution, no `benchmarks/results/criterion-baseline.json` existed in the repository.
* `ls -la benchmarks/results/` contained `ann_results.json`, `results.json`, and `summary.md`.
* Git log history (`git log -- benchmarks/results/`) confirmed that commit `ed24481` was unrelated to Criterion baselines and that `just bench-set-baseline` had never previously been committed to git.

---

## 2. WORKSPACE BENCHMARK INVENTORY

A total of 23 benchmark source files across 8 workspace crates and top-level benchmarks were identified via `find . -path "*/benches/*" -iname "*.rs" | sort`:

1. `./benches/competitive_bench.rs` (contextra-db / workspace)
2. `./benches/migration_benchmarks.rs` (contextra-db / workspace)
3. `./benches/rrf_scale_bench.rs` (contextra-db / workspace)
4. `./benches/scale_bench.rs` (contextra-db / workspace)
5. `./crates/contextra-candle/benches/kv_bridge_bench.rs`
6. `./crates/contextra-checkpoint/benches/checkpoint_bench.rs`
7. `./crates/contextra-crypto/benches/crypto_benchmarks.rs`
8. `./crates/contextra-embed/benches/embed_bench.rs`
9. `./crates/contextra-graph/benches/csr_traversal_bench.rs` *(Key binary `relate()` / `Edge` traversal benchmark for AK-8)*
10. `./crates/contextra-index/benches/audit_benchmarks.rs`
11. `./crates/contextra-index/benches/distance_bench.rs`
12. `./crates/contextra-index/benches/distance_nan_impact_bench.rs`
13. `./crates/contextra-index/benches/flush_threshold_amplification.rs`
14. `./crates/contextra-index/benches/hnsw_bench.rs`
15. `./crates/contextra-index/benches/hnsw_delete_search_bench.rs`
16. `./crates/contextra-index/benches/partial_rebuild_recall_bench.rs`
17. `./crates/contextra-index/benches/sq8_bench.rs`
18. `./crates/contextra-router/benches/router_bench.rs`
19. `./crates/contextra-store/benches/block_cache_bench.rs`
20. `./crates/contextra-store/benches/lsm_concurrency_bench.rs`
21. `./crates/contextra-store/benches/memtable_bench.rs`
22. `./crates/contextra-store/benches/sstable_bench.rs`
23. `./crates/contextra-store/benches/wal_bench.rs`

---

## 3. TOOLING EXECUTION & COMPILATION FINDINGS

### Execution of `just bench-set-baseline`
When executing `just bench-set-baseline` (`cargo bench --workspace --bench '*' -- --save-baseline current`), two workspace-level compilation blockers were detected:

#### 1. `contextra-candle` Feature Requirement Blocker
`crates/contextra-candle/benches/kv_bridge_bench.rs` specifies `required-features = ["kv-bridge"]`. Running `--workspace --bench '*'` without explicit `--features kv-bridge` causes `cargo bench` to stop immediately:
```
error: target `kv_bridge_bench` in package `contextra-candle` requires the features: `kv-bridge`
Consider enabling them by passing, e.g., `--features="kv-bridge"`
```

#### 2. `contextra-graph` Source Code Compilation Blocker (Critical)
When running `cargo check` or `cargo bench` on `contextra-graph` (or dependent crates `contextra-db`, `contextra-router`, `contextra-mcp`), 15 rustc compilation errors are emitted in `crates/contextra-graph/src/csr.rs`:
* **E0592:** Duplicate definitions for `get_hyperedge` (lines 1050 and 1139) and `hyperedges_for_entity` (lines 413 and 1070).
* **E0308:** Type mismatches between local `pub mod hyperedge` defined inside `csr.rs` (line 30) and module `crate::hyperedge` defined in `crates/contextra-graph/src/hyperedge.rs`.
* **E0599:** No method `push` on `&mut HashSet<HyperEdgeId>`.
* **E0609:** Unknown field `bindings` and `is_tombstoned` on type `hyperedge::HyperEdge`.

### Root Cause Analysis
An unmerged / incomplete Hyperedge draft change in `crates/contextra-graph/src/csr.rs` introduced an inline `pub mod hyperedge` block that conflicts with `crate::hyperedge` in `hyperedge.rs`. Because `contextra-graph` is a core workspace dependency for `contextra-db`, `contextra-router`, `contextra-mcp`, and top-level benchmarks, workspace-wide `cargo bench --workspace` cannot complete until `csr.rs` is repaired in an upcoming Welle 8 IP-20 implementation prompt.

### Partial Baseline Generation
Under strict **AUDIT role-lock**, no `.rs` files were modified. To maximize regression protection, benchmarks for independent Tier-1 crates (`contextra-store`, `contextra-crypto`, `contextra-checkpoint`) were executed, and 29 benchmark metrics were collected and aggregated into `benchmarks/results/criterion-baseline.json`.

---

## 4. AK-8 / AK-4 REGRESSION FREEDOM MANDATE & NEXT STEPS

1. **Vor-IP-20 Reference Point:** The file `benchmarks/results/criterion-baseline.json` created by this session serves as the canonical pre-IP-20 baseline anchor.
2. **Post-Merge Verification Requirement:** Once the first IP-20 Hyperedge prompt from Welle 8 (which repairs `crates/contextra-graph/src/csr.rs`) is merged:
   - `just bench-set-baseline` MUST be executed across the full workspace (including `csr_traversal_bench.rs`).
   - `just bench-delta` MUST be executed to verify that binary `relate()` and `Edge` traversal latencies remain within the **±5% tolerance threshold**.

---

## 5. VERDICT & ACTION ITEMS

* **STATUS:** AUDIT COMPLETE — Pre-baseline artifact created & build blocker documented.
* **Action Item 1:** IP-20 Prompt 1 (Hyperedge Graph Integration) must clean up duplicate `pub mod hyperedge` and method definitions in `crates/contextra-graph/src/csr.rs`.
* **Action Item 2:** Post-IP-20 merge, execute `just bench-delta` to formally assert AK-8 / AK-4 regression freedom.
