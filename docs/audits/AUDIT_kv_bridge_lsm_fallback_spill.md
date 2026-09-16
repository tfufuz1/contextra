# AUDIT REPORT: KV-Cache-Bridge LSM-Fallback-Spill Reifegrad-Klärung (`memfuse-candle`)

**Audit Date:** September 2026
**Auditor:** Jules (Principal Rust Systems Engineer)
**Task Claim:** `WELLE5-AUDIT-KVBRIDGE-SPILL`
**Target Crate:** `crates/memfuse-candle` (Layer 3)
**Primary Source Under Audit:** `crates/memfuse-candle/src/kv_bridge.rs`
**Secondary Source Under Audit:** `crates/memfuse-crypto/src/kv_segment/store.rs`

---

## 1. Executive Summary & Audit Baseline

### 1.1 Scope & Purpose
This audit clarifies the maturity status ("Reifegrad") of the **LSM-Fallback-Spill** mechanism for the KV-Cache Bridge in `crates/memfuse-candle`.
As noted in `MEMFUSE_ENDPRODUKT_SPEZIFIKATION_FINAL_9.md` §4.8 / §5.3:
> `[REIFEGRAD UNGEPRÜFT]`: LSM-Fallback-Spill (Paged Encrypted mit Fallback in LSM aus LLM-SLM-Spec §2.2) im Code nicht sichtbar — `kv_bridge.rs` nutzt dedizierten `store`, nicht LSM-Fallback.

The purpose of this audit is to:
1. Conduct an unbiased code analysis of `kv_bridge.rs` and its underlying storage dependencies (`TenantIsolatedKvStore`).
2. Categorize the current status: **(a) Completely Missing**, **(b) Partially Present**, or **(c) Fully Present**.
3. Evaluate safety and security invariants (Zero-Panic doctrine, AES-256-GCM-SIV encryption, HKDF key derivation, tenant isolation, and `model_fingerprint` / `rope_offset` validation).
4. Provide a complete, implementation-ready design specification for a follow-up implementation (FIX) prompt.

### 1.2 Final Determination
* **Status:** **(a) VOLLSTÄNDIG FEHLEND (Completely Missing)**.
* **Explanation:** `KvBridgeAdapter` in `crates/memfuse-candle/src/kv_bridge.rs` interacts exclusively with an in-memory `TenantIsolatedKvStore` (`crates/memfuse-crypto/src/kv_segment/store.rs`). When the in-memory store reaches capacity or undergoes LRU eviction, evicted KV segments are immediately dropped and zeroized (`ZeroizeOnDrop`). There is currently **zero code connection** between `KvBridgeAdapter` and `LsmStorage` (`memfuse-store`).

---

## 2. Code Analysis & Specification Comparison

### 2.1 Soll-Zustand (Specification Baseline)
According to `FINAL_9.md` §4.8/§5.3 and the Air-Gap Sovereign LLM-SLM specification:
* The KV-Cache Bridge should provide a two-tier hierarchy:
  1. **Tier 1 (Fast Path / RAM):** In-memory encrypted LRU cache (`TenantIsolatedKvStore`).
  2. **Tier 2 (Slow Path / LSM Spill):** Encrypted fall-back disk persistence in the `LsmStorage` engine (`memfuse-store`) when RAM capacity or memory pressure triggers eviction.
* Upon cache lookup (`try_get_cached_segment`), if a segment is absent from RAM, the system should check Tier 2 (LSM) before declaring a cache miss and falling back to full prompt prefill.
* Security requirements: KV segments written to disk/LSM must maintain strict AES-256-GCM-SIV encryption with HKDF key derivation per `TenantId`, preserving tenant boundary isolation and verifying `model_fingerprint` and `rope_offset` upon retrieval.

### 2.2 Ist-Zustand (Repository Code State)

#### `crates/memfuse-candle/src/kv_bridge.rs`
* **Adapter Structure:**
  ```rust
  #[derive(Clone)]
  pub struct KvBridgeAdapter {
      pub store: Arc<TenantIsolatedKvStore>,
      pub cipher: Arc<KvSegmentCipher>,
      pub consultations: Arc<std::sync::atomic::AtomicU64>,
  }
  ```
* **Storage Path (`store_segment`, lines 152–181):**
  - Serializes `CachedKvPayload` containing `ModelFingerprint`, `rope_offset`, and plaintext KV data.
  - Calls `self.store.insert_encrypted_segment(&self.cipher, tenant, key.chunk_id, ...)`.
  - Errors are caught and logged (`tracing::warn!`), never propagated.
* **Retrieval Path (`try_get_cached_segment`, lines 82–150):**
  - Queries `self.store.get_decrypted_segment(&self.cipher, tenant, key.chunk_id)`.
  - Deserializes `CachedKvPayload`.
  - Validates `payload.fingerprint == key.fingerprint` and `payload.rope_offset == key.rope_offset`.
  - Returns `Some(payload.data)` on success or `None` on any lookup/decryption/validation error.

#### `crates/memfuse-crypto/src/kv_segment/store.rs`
* **Eviction Behavior (`evict_lru_fair_internal`, lines 393–469 & `TenantState::insert_returning_evicted`, lines 26–36):**
  - Managed per-tenant via `lru::LruCache<u64, KvSegment>` with default `segment_capacity` of 256 segments per tenant.
  - When capacity is exceeded during `insert_segment`, `TenantState::insert_returning_evicted` pops the LRU segment.
  - The returned `KvSegment` is dropped outside the shard lock. Its `ZeroizeOnDrop` implementation wipes the encrypted/plaintext buffer from memory.
  - **No spill trigger or callback to disk/LSM exists.**

---

## 3. Strict Invariants & Security Audit

| Invariant / Audit Item | Audit Result | Evidence / Code References |
| :--- | :--- | :--- |
| **Zero-Panic Doctrine** | 🟢 **100% Compliant** | Zero `.unwrap()` or `.expect()` calls in production code of `kv_bridge.rs` (lines 1..190). All unwraps/expects are strictly isolated to `#[cfg(test)] mod tests` (lines 191..352). |
| **Fail-Safe / Fail-Open** | 🟢 **100% Compliant** | Any error (decryption failure, corrupt bincode, fingerprint mismatch, RoPE offset mismatch) in `try_get_cached_segment` gracefully logs a warning and returns `None` (full prefill fallback). |
| **Cryptographic Protection** | 🟢 **100% Compliant** | In-memory segments use `KvSegmentCipher` (AES-256-GCM-SIV + HKDF key derivation per `TenantId`). Plaintext KV activations are never stored unencrypted. |
| **Metadata Integrity** | 🟢 **100% Compliant** | `CachedKvPayload` strictly checks `model_fingerprint` (prevents cross-quantization state pollution) and `rope_offset` (prevents positional sequence corruption). |
| **LSM Spill Path** | 🔴 **Missing** | No reference or dependency on `memfuse-store` (`LsmStorage`) in `memfuse-candle`. Eviction purges segments permanently. |

---

## 4. Risk Assessment & Behavior under Memory Saturation

### 4.1 What happens under high memory pressure?
1. As inference requests arrive, `store_segment` populates `TenantIsolatedKvStore`.
2. When a tenant exceeds 256 cached KV segments (or when `EvictionWorker` triggers `evict_lru_fair`), the oldest segments are popped and zeroized.
3. Upon a subsequent request for an evicted `chunk_id`, `try_get_cached_segment` returns `None`.
4. Inference falls back to full prompt prefill (`fail-open`).

### 4.2 Impact Analysis
* **Functional Integrity / Accuracy:** **0% Risk.** Full prompt prefill generates identical KV activations. Context and response accuracy are fully preserved.
* **Crash / Panic Risk:** **0% Risk.** The fallback path is completely panic-free and transparent to the LLM client.
* **Performance / Latency Impact:** **High.** Under heavy multi-tenant or long-context prompt workloads, frequent RAM cache evictions cause high Time-To-First-Token (TTFT) latency overhead due to repeated matrix-multiplication prefills that could have been satisfied by disk reads if spilled to LSM.

---

## 5. Architectural Implementation Proposal for Follow-up FIX Task

To bridge this gap in a future implementation task, the following architecture should be added.

### 5.1 System Design: Two-Tier KV-Bridge (`KvBridgeAdapter` + `LsmStorage`)

```
                          +-----------------------------------+
                          |      KvBridgeAdapter::try_get     |
                          +-----------------------------------+
                                            |
                                  1. RAM Store Lookup
                                            |
                     +----------------------+----------------------+
                     |                                             |
              [ RAM Hit ]                                    [ RAM Miss ]
                     |                                             |
             Decrypt & Validate                            2. LSM Disk Lookup
                     |                                  (key: __kv_spill:{tenant}:{chunk_id})
             Return KV Tensor                                      |
                                                    +--------------+--------------+
                                                    |                             |
                                             [ LSM Hit ]                   [ LSM Miss ]
                                                    |                             |
                                            Decrypt & Validate            Return None
                                                    |                   (Full Prefill)
                                           Optional Promotion
                                                to RAM
```

### 5.2 Key Design Details

1. **Storage Key Format:**
   `__kv_spill:{tenant_id}:{chunk_id}` formatted as `Vec<u8>` with system prefix `__kv_spill:`.
2. **Encrypted Payload in LSM:**
   The value stored in `LsmStorage` MUST be the **encrypted bincode bytes** produced by `KvSegmentCipher` (or serialized `CachedKvPayload`), ensuring AES-256-GCM-SIV crypt-at-rest protection is maintained in SSTables and WAL.
3. **Eviction Callback Hook:**
   In `TenantIsolatedKvStore::evict_lru_fair_internal`, when a segment is evicted from RAM, pass the evicted encrypted `KvSegment` to an optional `AsyncSpillHandler` closure or channel that writes `(key, encrypted_value)` to `LsmStorage`.
4. **Validation Integrity:**
   When retrieving a segment from LSM disk, `try_get_cached_segment` MUST execute the identical `model_fingerprint` and `rope_offset` validation checks as the RAM fast path before returning `Some(data)`.

### 5.3 Proposed Target Signature Adjustments (For Folge-Prompt)

```rust
// Proposed extension in memfuse-candle/src/kv_bridge.rs

pub struct KvBridgeAdapter {
    pub store: Arc<TenantIsolatedKvStore>,
    pub lsm_store: Option<Arc<memfuse_store::LsmStorage>>, // Added optional LSM fallback tier
    pub cipher: Arc<KvSegmentCipher>,
    pub consultations: Arc<std::sync::atomic::AtomicU64>,
}

impl KvBridgeAdapter {
    pub fn with_lsm_fallback(
        store: Arc<TenantIsolatedKvStore>,
        lsm_store: Arc<memfuse_store::LsmStorage>,
        cipher: Arc<KvSegmentCipher>,
    ) -> Self {
        Self {
            store,
            lsm_store: Some(lsm_store),
            cipher,
            consultations: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub async fn try_get_cached_segment_async(&self, tenant: TenantId, key: &KvCacheKey) -> Option<Vec<u8>> {
        // 1. Try RAM cache first
        if let Some(bytes) = self.try_get_cached_segment(tenant, key) {
            return Some(bytes);
        }

        // 2. Try LSM disk spill fallback if lsm_store is configured
        let lsm = self.lsm_store.as_ref()?;
        let spill_key = format!("__kv_spill:{}:{:#x}", tenant.inner(), key.chunk_id).into_bytes();

        let doc = match lsm.get_raw(&spill_key).await {
            Ok(Some(bytes)) => bytes,
            _ => return None,
        };

        // 3. Decrypt and validate payload identically
        let decrypted_bytes = self.cipher.decrypt_data(tenant, &doc).ok()?;
        let payload: CachedKvPayload = bincode::deserialize(&decrypted_bytes).ok()?;

        if payload.fingerprint != key.fingerprint || payload.rope_offset != key.rope_offset {
            return None;
        }

        Some(payload.data)
    }
}
```

---

## 6. Audit Conclusion & Recommendations

1. **Reifegrad Statement:** The KV-Cache Bridge currently functions as a **single-tier encrypted in-memory cache**. The LSM-Fallback-Spill is **completely missing**.
2. **Safety Statement:** The missing spill path is **safe** due to transparent fail-open semantics returning `None` to trigger full prefill.
3. **Recommendation for Folge-Prompt:** Create a separate FIX prompt targeting `crates/memfuse-candle` and `crates/memfuse-store` to implement `KvBridgeAdapter::with_lsm_fallback` and eviction spill hooks as outlined in Section 5.
