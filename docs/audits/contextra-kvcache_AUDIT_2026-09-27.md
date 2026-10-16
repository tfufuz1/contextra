# Architecture & Security Audit: `crates/contextra-kvcache`

**Datum:** 2026-09-27
**Crate:** `contextra-kvcache` (Ring 1 - State & Storage Layer)
**Status:** PASS
**Auditor:** Principal Senior Rust Architect (Contextra Core Team)
**Safety Directives:** `#![forbid(unsafe_code)]` ENFORCED

---

## Executive Summary

An in-depth architecture, security, and correctness audit of `crates/contextra-kvcache/src/` (`radix.rs`, `prefix_store.rs`, `segment.rs`, `store.rs`, `quantize_kivi.rs`, `eviction_worker.rs`, `attention_score.rs`) was conducted.

The crate provides in-memory KV-cache management, token prefix radix matching, asymmetric 2-bit KIVI quantization, SnapKV/H2O attention-aware eviction, AEAD encryption, crypto-shredding, and multi-tenant tiering. All 37 unit tests and 37 integration tests pass under `--all-features`. Unsafe code is strictly forbidden (`#![forbid(unsafe_code)]`).

---

## 1. Tenant-Isolation-Nachweis (P1 — VETO-F10 Compliance)

### Audit Findings
**Verdict:** **STRUCTURALLY ENFORCED (STRUKTURELL ABSOLUT)**

Tenant isolation in `contextra-kvcache` is not merely convention or documentation; it is enforced at the type level and data-structure level across every lookup and insertion path:

1. **`TenantIsolatedKvStore` (`store.rs`):**
   - Segments and Radix trees are sharded by `tenant.inner()`. Within each shard, state is maintained inside `AHashMap<TenantId, TenantState>`.
   - Every read (`get_segment_bytes`, `get_decrypted_segment`, `find_prefix_match`, `get_segments`) takes `TenantId` as a mandatory parameter and indexes strictly via `shard.get(&tenant)` or `shard.get_mut(&tenant)`.
   - It is structurally impossible to query data of `TenantId B` when providing `TenantId A`.

2. **`TenantPrefixKvStore` (`prefix_store.rs`):**
   - The underlying partition map is defined as `RwLock<AHashMap<(TenantId, PrefixKey), Partition>>`.
   - Lookups evaluate `map.get_mut(&(tenant, key.clone()))`. A lookup with `TenantId A` can never match a key bound to `TenantId B`.

3. **`ContentAddressedKvStore` (`radix.rs`):**
   - **Position Index:** Uses `tenant_position_trees: AHashMap<TenantId, PrefixRadixTree>`.
   - **Content-Hash Index:** Uses `content_index: AHashMap<(TenantId, blake3::Hash), KvSegmentRef>`.
   - **Semantic Similarity Index:** Uses `semantic_entries: Vec<(TenantId, Vec<f32>, KvSegmentRef)>` with explicit filtering `if *entry_tenant == tenant`.
   - **Defense-in-Depth:** Even after hash matching, `lookup()` verifies `if seg_ref.tenant_id == tenant`.

### Cross-Tenant Aggregation Audit
Zero cross-tenant aggregation exists in eviction or caching logic. During eviction (`evict_fair_internal`), tenants within a shard are iterated independently, preserving tenant fairness without leaking segment identifiers or tensor contents across tenant boundaries.

---

## 2. Radix-Tree-Test-Coverage & Correctness (P2)

### Audit Findings
`PrefixRadixTree` in `radix.rs` implements a compressed Radix Trie over u32 token sequences (`&[u32]`).

- **Node Splitting Logic:** Correctly computes Longest Common Prefix (LCP) via `RadixNode::lcp_len`. When inserting a sequence that shares a partial prefix with an existing node, the node splits cleanly into a parent LCP node and child branches without loss of existing `block_id` bindings.
- **Prefix Matching:** Supports `KvReusePolicy` (`Always`, `CostBased { min_prefix_len }`, `Never`). Matches return `PrefixMatch { matched_tokens, matched_len, block_id }`.
- **Deletion Integrity:** `remove()` walks child branches and clears `block_id` without corrupting neighbor nodes or child subtrees.
- **Unit Test Verification:**
  - `test_radix_tree_basic_insert_and_match`: Verifies LCP insertion and `CostBased` threshold filtering.
  - `test_radix_tree_node_splitting`: Verifies tree splitting with shared prefix `[1, 2, 3, ...]`.
  - `test_radix_tree_remove_and_clear`: Verifies deletion and total entries counter decay.
  - `test_content_addressed_kv_store_lookup_cascade`: Verifies Position -> Content-Hash -> Semantic cascade lookup.

---

## 3. Tiering-Race-Analyse (P6)

### Audit Findings
**Analysis:** **NON-BLOCKING SPILL DESIGN WITH CACHE MISS FALLBACK**

When a KV block is evicted from `TenantState` (due to LRU capacity or byte budget limits) and a `spill_handler` is configured:

1. **Eviction Execution (`store.rs`):**
   - The segment is removed from `TenantState.cache` under the shard write lock (`shard.lock.write()`).
   - The evicted segment is collected in `deferred_drop`.
   - The shard write lock is released.
   - The `spill_handler` closure is invoked asynchronously/outside the lock: `handler(ev.tenant_id, ev.segment_id, ev.to_spill_bytes())`.

2. **Race Window Identification:**
   - There is a transient window between the removal of the segment from hot memory and the completion of the cold-tier write (LSM storage/disk persistence).
   - If a concurrent read arrives during this window, `get_segment_bytes` or `find_prefix_match` on hot-tier memory will return `None` (Cache Miss).
   - **Architectural Safety Evaluation:** This is the expected and correct behavior for tiering. Upper layers (e.g. `contextra-candle` or `contextra-engine`) handle a hot-tier cache miss by either falling back to Tier-2 storage read or triggering prompt re-prefill.
   - **Protection against Data Loss / Corruption:** `KvBlockGuard` prevents eviction of active blocks (`active_refs > 0`). Any segment currently involved in active inference cannot be evicted or spilled, preventing in-flight read corruption.

---

## 4. Quantisierung, Eviction & AEAD Security Analysis (P3, P4, P5)

### P3: KIVI 2-Bit Quantization (`quantize_kivi.rs`)
- **Asymmetric Quantization Scheme:** Implements per-channel key quantization and per-token value quantization per arXiv:2402.02750.
- **Numeric Bounds & Overflow Safety:** Float normalization handles `f32::INFINITY`, `f32::NEG_INFINITY`, and `NaN` values gracefully by clamping and scale fallbacks. Zero division is prevented by checking `(max_v - min_v).abs() < f32::EPSILON`.
- **Invariants:** Enforces `INV-KIVI-AEAD-ORDER`: Quantization -> Lossless Compression (`compress_bytes`) -> AEAD Encryption (`KvCipher::seal`).

### P4: Eviction Worker Correctness (`eviction_worker.rs` & `attention_score.rs`)
- **Thread Boundaries:** Non-blocking hot-path triggers are dispatched to a dedicated OS thread (`kv-eviction-worker`) via `std::sync::mpsc::mpsc`, ensuring synchronous `Zeroize` operations never block the Tokio async executor.
- **Eviction Strategies:** Combines LRU access age with LLM attention importance scores (`AttentionScoreSource`). Attention weights are normalized and combined via `(1 - w) * norm_lru + w * norm_importance`.
- **TOCTOU & Lock Safety:** Eviction checks `active_refs == 0` and directive pin status (`CacheDirective::Pin`) atomically under the shard write lock. If a segment becomes guarded, `pop_eviction_candidate` skips it.

### P5: AEAD Encryption & Crypto-Shredding (`segment.rs`)
- **Zeroize-on-Drop:** `KvSegment` implements `Zeroize` and `ZeroizeOnDrop` for tensor bytes.
- **Crypto-Shredding:** `ShreddableSegmentKey` wraps `CryptoKey` with `emergency_wipe()`. Calling `.shred()` zeroizes key bytes in memory, rendering all associated Tier-2 disk segments mathematically unrecoverable (GDPR Art. 17 compliance).
- **Integration:** Integrated with `contextra-crypto` (`KvSegmentCipher`, `EncryptedKvLayer`, `ModelFingerprint`).

---

## Verification Results

### Unit & Integration Test Execution
```
cargo test -p contextra-kvcache --all-features --locked -- --nocapture
```
- **Unittests (src/lib.rs):** 37 passed, 0 failed
- **Integration Test Suites:**
  - `cache_directive_pin_survives_eviction`: 5 passed
  - `cancellation`: 4 passed
  - `content_addressed_cross_tenant_isolation`: 3 passed
  - `content_hash_hit_rate_improvement`: 1 passed
  - `eviction_with_attention_exporter`: 2 passed
  - `golden_greedy_decode`: 3 passed
  - `kivi_aead_order_enforced`: 2 passed
  - `kivi_quantize_dequantize_roundtrip`: 3 passed
  - `prefix_store_port`: 9 passed
  - `tenant_isolation`: 5 passed
- **Total:** 74 tests passed, 0 failed.

### Static Code Analysis
- `#![forbid(unsafe_code)]` enforced across all files.
- Zero unsafe blocks present in crate.

---

## Verdict

```
VERDICT: PASS
VERIFIED-BY-SESSION: f6a0f12c-audit-kvcache (TS: 2026-09-27T20:25:00Z)
```
