//! # Konformitätsprüfsuiten für KV-Ports (`conformance/kv_ports`)
//!
//! Dieses Modul definiert standardisierte Konformitätsprüfsuiten für die
//! KV-Cache-bezogenen Ports aus `contextra-ports`:
//! - [`contextra_ports::KvPrefixStore`]
//! - [`contextra_ports::KvBridgeStorage`]
//! - [`contextra_ports::AttentionExporter`]
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Regel 10 (Fail-Closed):** Suiten liefern ein `ConformanceResult`, um Panics
//!   in Test-Harnesses zu vermeiden.

// FILE-CONTEXT
// STAND: 2026-11-11
// ZWECK: Conformance suites for KvPrefixStore, KvBridgeStorage, and AttentionExporter.
// INVARIANTEN: Zero-panic test execution returning ConformanceResult; fail-closed diagnostics; deterministic execution.

use super::failure::{ConformanceFailure, ConformanceResult};
use bytes::Bytes;
use contextra_ports::{
    AttentionExporter, KvBlock, KvBridgeStorage, KvLayout, KvPrefixStore, ModelFingerprint,
    PrefixKey, RequestId, RopeConfig, TenantId, TxId,
};
use std::future::Future;

fn create_sample_prefix_key(model_name: &str) -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0x42; 32], model_name, "Q4_K_M"),
        tokenizer_hash: [0x11; 32],
        layout: KvLayout {
            n_layer: 32,
            n_kv_head: 8,
            head_dim: 128,
            dtype: "f16".to_string(),
        },
        rope: RopeConfig {
            base: 10000.0,
            scaling: None,
        },
    }
}

fn create_sample_blocks(block_ids: &[u64]) -> Vec<KvBlock> {
    block_ids
        .iter()
        .map(|&id| KvBlock {
            block_id: id,
            data: Bytes::from(vec![(id & 0xFF) as u8; 64]),
        })
        .collect()
}

// ANCHOR[contextra-testkit:CONF-KV-PREFIX-001]
/// Prüft die Konformität einer [`KvPrefixStore`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfte Verträge:**
/// 1. `roundtrip_and_exact_match`: Gespeicherte Blöcke werden für passende Präfixe vollständig zurückgegeben.
/// 2. `isolation_tenant_and_key`: Kein Datenübertrag zwischen unterschiedlichen Mandanten oder Modellen.
/// 3. `overwrite_updates_data`: Erneutes Einfügen unter demselben Key überschreibt alte Blöcke.
/// 4. `evict_clears_data`: `evict` entfernt vorhandene Daten idempotent und liefert die Anzahl entfernter Blöcke.
/// 5. `deterministic_repeated_lookup`: Wiederholte Abfragen liefern identische Ergebnisse.
pub fn run_kv_prefix_store_suite<S: KvPrefixStore>(store: &S) -> ConformanceResult {
    let suite = "KvPrefixStore";

    let tenant1 = TenantId::try_new(101).map_err(|e| {
        ConformanceFailure::new(suite, "setup", format!("Invalid tenant1: {e}"))
    })?;
    let tenant2 = TenantId::try_new(102).map_err(|e| {
        ConformanceFailure::new(suite, "setup", format!("Invalid tenant2: {e}"))
    })?;

    let key_a = create_sample_prefix_key("model-a");
    let key_b = create_sample_prefix_key("model-b");

    let tokens = vec![101, 102, 103, 104];
    let blocks = create_sample_blocks(&[1, 2]);

    // Test 1: Roundtrip and exact match
    if let Err(e) = store.insert(tenant1, &key_a, &tokens, blocks.clone()) {
        return Err(ConformanceFailure::new(
            suite,
            "roundtrip_and_exact_match",
            format!("insert failed: {e}"),
        ));
    }

    let hit = store.lookup(tenant1, &key_a, &tokens);
    match hit {
        Some(h) => {
            if h.matched_tokens != tokens.len() {
                return Err(ConformanceFailure::new(
                    suite,
                    "roundtrip_and_exact_match",
                    format!(
                        "matched_tokens mismatch: expected {}, got {}",
                        tokens.len(),
                        h.matched_tokens
                    ),
                ));
            }
            if h.blocks != blocks {
                return Err(ConformanceFailure::new(
                    suite,
                    "roundtrip_and_exact_match",
                    format!("blocks mismatch: expected {blocks:?}, got {:?}", h.blocks),
                ));
            }
        }
        None => {
            return Err(ConformanceFailure::new(
                suite,
                "roundtrip_and_exact_match",
                "lookup returned None for inserted prefix",
            ));
        }
    }

    // Test 2: Tenant and Model isolation
    let cross_tenant_hit = store.lookup(tenant2, &key_a, &tokens);
    if cross_tenant_hit.is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "isolation_tenant_and_key",
            "cross-tenant lookup returned a hit (isolation violation)",
        ));
    }

    let cross_key_hit = store.lookup(tenant1, &key_b, &tokens);
    if cross_key_hit.is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "isolation_tenant_and_key",
            "cross-model lookup returned a hit (isolation violation)",
        ));
    }

    // Test 3: Overwrite updates data
    let new_blocks = create_sample_blocks(&[10, 20, 30]);
    if let Err(e) = store.insert(tenant1, &key_a, &tokens, new_blocks.clone()) {
        return Err(ConformanceFailure::new(
            suite,
            "overwrite_updates_data",
            format!("insert for overwrite failed: {e}"),
        ));
    }

    let updated_hit = store.lookup(tenant1, &key_a, &tokens);
    match updated_hit {
        Some(h) => {
            if h.blocks != new_blocks {
                return Err(ConformanceFailure::new(
                    suite,
                    "overwrite_updates_data",
                    format!(
                        "updated blocks mismatch: expected {new_blocks:?}, got {:?}",
                        h.blocks
                    ),
                ));
            }
        }
        None => {
            return Err(ConformanceFailure::new(
                suite,
                "overwrite_updates_data",
                "lookup returned None after overwrite",
            ));
        }
    }

    // Test 4: Evict clears data
    let evicted_count = store.evict(tenant1, &key_a).map_err(|e| {
        ConformanceFailure::new(suite, "evict_clears_data", format!("evict failed: {e}"))
    })?;
    if evicted_count == 0 {
        return Err(ConformanceFailure::new(
            suite,
            "evict_clears_data",
            "evict returned 0 evicted blocks for populated entry",
        ));
    }

    let hit_after_evict = store.lookup(tenant1, &key_a, &tokens);
    if hit_after_evict.is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "evict_clears_data",
            "lookup returned a hit after entry was evicted",
        ));
    }

    // Test 5: Deterministic repeated lookup
    let fresh_blocks = create_sample_blocks(&[99]);
    let single_token = vec![777];
    store
        .insert(tenant1, &key_b, &single_token, fresh_blocks.clone())
        .map_err(|e| {
            ConformanceFailure::new(
                suite,
                "deterministic_repeated_lookup",
                format!("insert failed: {e}"),
            )
        })?;

    let res1 = store.lookup(tenant1, &key_b, &single_token);
    let res2 = store.lookup(tenant1, &key_b, &single_token);
    if res1 != res2 {
        return Err(ConformanceFailure::new(
            suite,
            "deterministic_repeated_lookup",
            "repeated lookup returned inconsistent results",
        ));
    }

    Ok(())
}

// ANCHOR[contextra-testkit:CONF-KV-BRIDGE-001]
/// Prüft die Konformität einer [`KvBridgeStorage`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfte Verträge:**
/// 1. `get_empty_key`: Unbekannte Schlüssel liefern `Ok(None)`.
/// 2. `uncommitted_put_invisible`: Ungespeicherte / uncommittete Schreibvorgänge sind vor dem Commit nicht sichtbar.
/// 3. `commit_makes_visible`: Nach erfolgreichem `commit` ist der Wert lesbar.
/// 4. `multi_tx_isolation`: Mehrere Transaktionen schreiben isoliert voneinander.
/// 5. `overwrite_key`: Überschreiben eines Schlüssels aktualisiert den Wert nach dem Commit.
pub async fn run_kv_bridge_storage_suite<F, Fut, S>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = S>,
    S: KvBridgeStorage,
{
    let suite = "KvBridgeStorage";
    let storage = factory().await;

    // Test 1: get_empty_key
    let key = b"nonexistent_key";
    let res = storage.get(key).await.map_err(|e| {
        ConformanceFailure::new(suite, "get_empty_key", format!("get failed: {e}"))
    })?;
    if res.is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "get_empty_key",
            "expected None for nonexistent key",
        ));
    }

    // Test 2: uncommitted_put_invisible
    let tx1 = TxId::new(1001);
    let key1 = b"cache_block_01";
    let val1 = b"tensor_bytes_01";

    storage.put(tx1, key1, val1).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "uncommitted_put_invisible",
            format!("put failed: {e}"),
        )
    })?;

    let val_before_commit = storage.get(key1).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "uncommitted_put_invisible",
            format!("get before commit failed: {e}"),
        )
    })?;
    if val_before_commit.is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "uncommitted_put_invisible",
            "uncommitted put is visible before commit (isolation violation)",
        ));
    }

    // Test 3: commit_makes_visible
    storage.commit(tx1).await.map_err(|e| {
        ConformanceFailure::new(suite, "commit_makes_visible", format!("commit failed: {e}"))
    })?;

    let val_after_commit = storage.get(key1).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "commit_makes_visible",
            format!("get after commit failed: {e}"),
        )
    })?;
    match val_after_commit {
        Some(bytes) => {
            if bytes.as_ref() != val1 {
                return Err(ConformanceFailure::new(
                    suite,
                    "commit_makes_visible",
                    format!(
                        "value mismatch: expected {:?}, got {:?}",
                        val1,
                        bytes.as_ref()
                    ),
                ));
            }
        }
        None => {
            return Err(ConformanceFailure::new(
                suite,
                "commit_makes_visible",
                "key not found after commit",
            ));
        }
    }

    // Test 4: multi_tx_isolation
    let tx2 = TxId::new(1002);
    let tx3 = TxId::new(1003);
    let key2 = b"cache_block_02";
    let key3 = b"cache_block_03";
    let val2 = b"tensor_bytes_02";
    let val3 = b"tensor_bytes_03";

    storage.put(tx2, key2, val2).await.map_err(|e| {
        ConformanceFailure::new(suite, "multi_tx_isolation", format!("put tx2 failed: {e}"))
    })?;
    storage.put(tx3, key3, val3).await.map_err(|e| {
        ConformanceFailure::new(suite, "multi_tx_isolation", format!("put tx3 failed: {e}"))
    })?;

    // Commit only tx2
    storage.commit(tx2).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "multi_tx_isolation",
            format!("commit tx2 failed: {e}"),
        )
    })?;

    let read_key2 = storage.get(key2).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "multi_tx_isolation",
            format!("get key2 failed: {e}"),
        )
    })?;
    let read_key3 = storage.get(key3).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "multi_tx_isolation",
            format!("get key3 failed: {e}"),
        )
    })?;

    if read_key2.as_deref() != Some(val2.as_ref()) {
        return Err(ConformanceFailure::new(
            suite,
            "multi_tx_isolation",
            "committed tx2 key2 not visible or corrupted",
        ));
    }
    if read_key3.is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "multi_tx_isolation",
            "uncommitted tx3 key3 leaked into storage",
        ));
    }

    // Test 5: overwrite_key
    let tx4 = TxId::new(1004);
    let val2_updated = b"tensor_bytes_02_updated";
    storage.put(tx4, key2, val2_updated).await.map_err(|e| {
        ConformanceFailure::new(suite, "overwrite_key", format!("put overwrite failed: {e}"))
    })?;
    storage.commit(tx4).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "overwrite_key",
            format!("commit overwrite failed: {e}"),
        )
    })?;

    let read_updated = storage.get(key2).await.map_err(|e| {
        ConformanceFailure::new(
            suite,
            "overwrite_key",
            format!("get updated key2 failed: {e}"),
        )
    })?;
    if read_updated.as_deref() != Some(val2_updated.as_ref()) {
        return Err(ConformanceFailure::new(
            suite,
            "overwrite_key",
            "overwritten value not correctly visible after commit",
        ));
    }

    Ok(())
}

// ANCHOR[contextra-testkit:CONF-ATTENTION-001]
/// Prüft die Konformität einer [`AttentionExporter`]-Implementierung gegen ihren dokumentierten Vertrag.
///
/// **Geprüfte Verträge:**
/// 1. `unknown_request_none`: Unbekannte RequestIds liefern `None`.
/// 2. `known_request_weights`: Bekannte RequestIds liefern einen Vektor von Aufmerksamkeitsgewichten.
/// 3. `deterministic_repeated_export`: Wiederholte Aufrufe für dieselbe RequestId liefern identische Vektoren gleicher Länge.
pub fn run_attention_exporter_suite<E: AttentionExporter>(
    exporter: &E,
    known_request: RequestId,
) -> ConformanceResult {
    let suite = "AttentionExporter";

    // Test 1: unknown_request_none
    let unknown = RequestId(u64::MAX - 7);
    if exporter.export_attention_weights(unknown).is_some() {
        return Err(ConformanceFailure::new(
            suite,
            "unknown_request_none",
            "expected None for unknown RequestId",
        ));
    }

    // Test 2: known_request_weights
    let weights1 = match exporter.export_attention_weights(known_request) {
        Some(w) => w,
        None => {
            return Err(ConformanceFailure::new(
                suite,
                "known_request_weights",
                "expected Some(Vec<f32>) for known RequestId",
            ));
        }
    };

    // Test 3: deterministic_repeated_export
    let weights2 = match exporter.export_attention_weights(known_request) {
        Some(w) => w,
        None => {
            return Err(ConformanceFailure::new(
                suite,
                "deterministic_repeated_export",
                "repeated export returned None for known RequestId",
            ));
        }
    };

    if weights1 != weights2 {
        return Err(ConformanceFailure::new(
            suite,
            "deterministic_repeated_export",
            "repeated export returned differing weights",
        ));
    }

    Ok(())
}

// ============================================================================
// CONVENIENCE PANICKING ASSERTIONS (Z3)
// ============================================================================

/// Führt die Konformitätsprüfung für einen [`KvPrefixStore`] durch und löst bei Fehlern
/// eine informative Panic aus.
#[track_caller]
pub fn assert_kv_prefix_store_conformance<S: KvPrefixStore>(store: &S) {
    if let Err(failure) = run_kv_prefix_store_suite(store) {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}

/// Führt die Konformitätsprüfung für einen [`KvBridgeStorage`] durch und löst bei Fehlern
/// eine informative Panic aus.
pub async fn assert_kv_bridge_storage_conformance<F, Fut, S>(factory: F)
where
    F: Fn() -> Fut,
    Fut: Future<Output = S>,
    S: KvBridgeStorage,
{
    if let Err(failure) = run_kv_bridge_storage_suite(factory).await {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}

/// Führt die Konformitätsprüfung für einen [`AttentionExporter`] durch und löst bei Fehlern
/// eine informative Panic aus.
#[track_caller]
pub fn assert_attention_exporter_conformance<E: AttentionExporter>(
    exporter: &E,
    known_request: RequestId,
) {
    if let Err(failure) = run_attention_exporter_suite(exporter, known_request) {
        #[allow(clippy::panic)]
        ::std::panic::panic_any(failure.to_string());
    }
}
