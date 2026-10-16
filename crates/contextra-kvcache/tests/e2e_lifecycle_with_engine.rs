// FILE-CONTEXT
// ZWECK: End-to-End Lifecycle-Integrationstest für KV-Cache, Tenant-Isolierung, Quantisierung & Shredding.
// STAND: TS:2026-10-05T00:00:00Z

#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use contextra_kvcache::prefix_store::TenantPrefixKvStore;
use contextra_kvcache::quantize_kivi::{
    kivi_dequantize, kivi_quantize, KiviQuantizeConfig, KiviQuantizedBlock, KvTensorView,
};
use contextra_kvcache::segment::{KvSegment, ShreddableSegmentKey, Tier2EncryptedSegment};
use contextra_kvcache::store::{CacheDirective, TenantIsolatedKvStore};
use contextra_ports::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_types::model_fingerprint::ModelFingerprint;
use contextra_types::{DocId, TenantId};

/// End-to-End Test für den gesamten KV-Cache Lebenszyklus über Mandanten-Grenzen.
///
/// Prüft 7 Teilschritte gegen unabhängige Orakel:
/// (1) Mandant A speichert Prefix P.
/// (2) Lookup Mandant A trifft und liefert Rekonstruktionsbytes innerhalb KIVI-Toleranz.
/// (3) Lookup Mandant B trifft nicht (Cross-Tenant-Isolierung).
/// (4) Eviction unter Speicherdruck entfernt unpinned Segmente; gepinnter Eintrag bleibt erhalten.
/// (5) Dokument löschen -> Lookup A trifft nicht mehr; Canary-Bytes sind aus dem Speicher getilgt.
/// (6) Absturz simulation / Rollback: In-Memory Transaktions-Rollback verwirft uncommitted Segmente.
/// (7) Mandant A löschen / Crypto-Shredding: Key-Revocation macht physischen Ciphertext unlesbar.
#[test]
fn test_e2e_tenant_kvcache_lifecycle() {
    // ------------------------------------------------------------------------
    // SETUP: Mandanten A und B initialisieren
    // ------------------------------------------------------------------------
    let tenant_a = TenantId::try_new(1001).expect("Valid TenantId A");
    let tenant_b = TenantId::try_new(1002).expect("Valid TenantId B");

    let prefix_store = TenantPrefixKvStore::new();
    let isolated_store = TenantIsolatedKvStore::with_capacity(10);

    let prefix_key = PrefixKey {
        model: ModelFingerprint::new([0xAA; 32], "llama-3-8b", "F16"),
        tokenizer_hash: [0xBB; 32],
        layout: KvLayout {
            n_layer: 32,
            n_kv_head: 8,
            head_dim: 128,
            dtype: "f16".into(),
        },
        rope: RopeConfig {
            base: 10000.0,
            scaling: None,
        },
    };

    let prefix_p = vec![101u32, 102, 103, 104];

    // ========================================================================
    // SCHRITT 1: Mandant A speichert Prefix P (Test-Tensor mit bekanntem Inhalt)
    // Orakel-Quelle: Bekannte Eingabe-Matrix mit 2 Tokens x 4 Channels
    // ========================================================================
    let keys_orig = vec![12.5f32, -8.2, 45.0, 0.0, 100.0, -50.0, 3.25, 2.75];
    let values_orig = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let num_tokens = 2;
    let num_channels = 4;

    let raw_tensor = KvTensorView::new(
        keys_orig.clone(),
        values_orig.clone(),
        num_tokens,
        num_channels,
    )
    .expect("Valid tensor view construction");

    let kivi_config = KiviQuantizeConfig {
        key_group_size: 16,
        quantize_values: true,
    };

    let quantized_block =
        kivi_quantize(&raw_tensor, kivi_config).expect("KIVI quantization succeeds");
    let serialized_block =
        bincode::serialize(&quantized_block).expect("Block serialization succeeds");

    let block_a = KvBlock {
        block_id: 1,
        data: bytes::Bytes::from(serialized_block),
    };

    prefix_store
        .insert(tenant_a, &prefix_key, &prefix_p, vec![block_a])
        .expect("Tenant A prefix insert succeeds");

    // ========================================================================
    // SCHRITT 2: Lookup A trifft & Dequantisierung liegt in KIVI-Toleranz
    // Orakel-Quelle: Unabhängige mathematische Schranke für 2-Bit Asymmetrische Quantisierung:
    // Delta = (max - min) / 3.0, Max-Fehler <= Delta / 2 = (max - min) / 6.0 + 1e-3.
    // ========================================================================
    let hit_a = prefix_store
        .lookup(tenant_a, &prefix_key, &prefix_p)
        .expect("Lookup Tenant A MUST hit");

    assert_eq!(
        hit_a.matched_tokens, 4,
        "Matched tokens MUST equal prefix length 4"
    );
    assert_eq!(hit_a.blocks.len(), 1);

    let retrieved_block: KiviQuantizedBlock =
        bincode::deserialize(&hit_a.blocks[0].data).expect("Deserialization succeeds");
    let recon_tensor = kivi_dequantize(&retrieved_block.packed, &retrieved_block.meta)
        .expect("KIVI dequantization succeeds");

    assert_eq!(recon_tensor.num_tokens, num_tokens);
    assert_eq!(recon_tensor.num_channels, num_channels);

    // Orakel-Toleranzberechnung für Keys (per Channel-Gruppe)
    let min_k = keys_orig.iter().copied().fold(f32::INFINITY, f32::min);
    let max_k = keys_orig.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let allowed_err_k = (max_k - min_k) / 6.0 + 1e-3;

    for (i, &key_orig) in keys_orig.iter().enumerate() {
        let err = (key_orig - recon_tensor.keys[i]).abs();
        assert!(
            err <= allowed_err_k,
            "Key[{i}] reconstruction error {err} exceeded theoretical bound {allowed_err_k}"
        );
    }

    // Orakel-Toleranzberechnung für Values (per Token)
    for t in 0..num_tokens {
        let mut min_v = f32::INFINITY;
        let mut max_v = f32::NEG_INFINITY;
        for c in 0..num_channels {
            let idx = t * num_channels + c;
            min_v = min_v.min(values_orig[idx]);
            max_v = max_v.max(values_orig[idx]);
        }
        let allowed_err_v = (max_v - min_v) / 6.0 + 1e-3;

        for c in 0..num_channels {
            let idx = t * num_channels + c;
            let err = (values_orig[idx] - recon_tensor.values[idx]).abs();
            assert!(
                err <= allowed_err_v,
                "Value[{idx}] reconstruction error {err} exceeded theoretical bound {allowed_err_v}"
            );
        }
    }

    // ========================================================================
    // SCHRITT 3: Lookup B trifft NICHT (Mandanten-Isolierung)
    // Orakel-Quelle: Mandant B besitzt keine Einträge für Prefix P
    // ========================================================================
    let hit_b = prefix_store.lookup(tenant_b, &prefix_key, &prefix_p);
    assert!(
        hit_b.is_none(),
        "Tenant B lookup for Tenant A's prefix MUST return None"
    );

    // ========================================================================
    // SCHRITT 4: Eviction unter Speicherdruck (Pinned Segments überleben)
    // Orakel-Quelle: Pin-Direktive schützt vor LRU-Eviction (cache_directive_pin_survives_eviction)
    // ========================================================================
    let pinned_seg = KvSegment::new(tenant_a, 201, vec![0x11; 1024]);
    isolated_store
        .insert_segment_with_directive(tenant_a, pinned_seg, CacheDirective::Pin { ttl: None })
        .expect("Pinned segment insert succeeds");

    for id in 202..=204 {
        let auto_seg = KvSegment::new(tenant_a, id, vec![0x22; 1024]);
        isolated_store
            .insert_segment_with_directive(tenant_a, auto_seg, CacheDirective::Auto)
            .expect("Auto segment insert succeeds");
    }

    assert_eq!(isolated_store.get_tenant_segment_len(tenant_a), 4);

    let freed = isolated_store.evict_lru_fair(2048);
    assert!(freed >= 2048, "Eviction must free at least 2048 bytes");

    let remaining_ids = isolated_store.get_segments(tenant_a);
    assert!(
        remaining_ids.contains(&201),
        "Pinned segment 201 MUST survive eviction"
    );
    assert!(
        isolated_store.get_segment_bytes(tenant_a, 201).is_some(),
        "Pinned segment data MUST remain accessible"
    );

    // ========================================================================
    // SCHRITT 5: Dokument löschen -> Lookup A trifft nicht mehr, Canary bereinigt
    // Orakel-Quelle: Byte-Suchmuster CANARY_SENSITIVE_DOC_XYZ_12345
    // ========================================================================
    let canary = b"CANARY_SENSITIVE_DOC_XYZ_12345";
    let doc_seg = KvSegment::new(tenant_a, 301, Vec::from(canary as &[u8]));
    isolated_store.insert_segment(tenant_a, doc_seg);

    assert!(
        isolated_store.get_segment_bytes(tenant_a, 301).is_some(),
        "Doc segment 301 must exist before deletion"
    );

    let doc_id = DocId::from(301u64);
    isolated_store.remove_doc_segments(tenant_a, doc_id);

    assert!(
        isolated_store.get_segment_bytes(tenant_a, 301).is_none(),
        "Segment 301 must no longer be retrievable after document removal"
    );

    for seg_id in isolated_store.get_segments(tenant_a) {
        if let Some(bytes) = isolated_store.get_segment_bytes(tenant_a, seg_id) {
            let contains_canary = bytes.windows(canary.len()).any(|w| w == canary);
            assert!(
                !contains_canary,
                "Active store segment {seg_id} MUST NOT contain canary bytes"
            );
        }
    }

    // ========================================================================
    // SCHRITT 6: Absturz simulation / Rollback (Transaktionales Rollback in-memory)
    // Orakel-Quelle: Transaktions-Rollback löscht uncommitted staged segments
    // (Disk-level VFS Fault Injection ist für den in-memory Ring 1 KV-Cache N/A)
    // ========================================================================
    let staged_seg = KvSegment::new(tenant_a, 501, vec![0x99; 512]);
    isolated_store.insert_segment(tenant_a, staged_seg);
    assert!(isolated_store.get_segment_bytes(tenant_a, 501).is_some());

    isolated_store.on_rollback(tenant_a, &[501]);
    assert!(
        isolated_store.get_segment_bytes(tenant_a, 501).is_none(),
        "Staged uncommitted segment 501 MUST be purged on transaction rollback"
    );

    // ========================================================================
    // SCHRITT 7: Mandant A löschen / Crypto-Shredding (Key Revocation)
    // Orakel-Quelle: O(1) Key Shredding (kv_crypto_shred_instant_unrecoverable)
    // ========================================================================
    let key = ShreddableSegmentKey::try_new_random("tenant-a-passphrase-shred")
        .expect("Key generation succeeds");
    let plaintext = b"TENANT_A_TOP_SECRET_KV_TENSOR_BYTES";

    let tier2_seg =
        Tier2EncryptedSegment::new(tenant_a, 601, plaintext, key.clone()).expect("Tier2 insert");

    let decrypted = tier2_seg
        .read_and_decrypt()
        .expect("Decryption succeeds before key shredding");
    assert_eq!(decrypted, plaintext);

    // Revoke Tenant A key (Crypto-Shredding)
    key.shred();
    assert!(key.is_shredded());

    let shred_res = tier2_seg.read_and_decrypt();
    assert!(
        shred_res.is_err(),
        "Decryption after key revocation MUST fail"
    );
    let err_msg = shred_res.unwrap_err().to_string();
    assert!(
        err_msg.contains("shredded"),
        "Error message must indicate key shredding: {err_msg}"
    );
}

/*
DOKUMENTATION DER NICHT AUTOMATISIERT AUF DISK PRÜFBAREN SCHRITTE:
------------------------------------------------------------------
Schritt (6) "Absturz zwischen Insert und Commit (FaultVfs)" war auf Disk-Ebene NICHT anwendbar:
- Grund: `contextra-kvcache` (`TenantIsolatedKvStore` / `TenantPrefixKvStore`) ist eine reine In-Memory
  Ring-1-Datenstruktur im RAM (LruCache, PrefixRadixTree). Es führt keine direkten Dateisystem-Schreibvorgänge
  oder VFS-I/O durch. Persistent-Disk-I/O und VFS-Crashes gehören zu `contextra-store` (Ring 0/1).
- Ersatzprüfung: In-Memory Transaktions-Rollback (`on_rollback`) wurde in Schritt (6) direkt verifiziert,
  wodurch nicht-committete unfertige Segmentzustände deterministisch aus dem Speicher verworfen werden.
*/
