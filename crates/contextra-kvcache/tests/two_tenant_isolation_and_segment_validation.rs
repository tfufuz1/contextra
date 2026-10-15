// FILE-CONTEXT
// ZWECK: Zwei-Mandanten-Isolationstests (VETO-F10) & Segment-Validierung (Länge, Bounds, Bytecheck).
// STAND: TS:2026-10-13T00:00:00Z

#![forbid(unsafe_code)]

use bytes::Bytes;
use contextra_kvcache::prefix_store::TenantPrefixKvStore;
use contextra_kvcache::radix::KvReusePolicy;
use contextra_kvcache::scratchpad_invalidation::{
    PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator,
};
use contextra_kvcache::segment::KvSegment;
use contextra_kvcache::store::TenantIsolatedKvStore;
use contextra_ports::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_types::model_fingerprint::ModelFingerprint;
use contextra_types::{ContextraError, TenantId};

#[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
use contextra_kvcache::quantize_kivi::KvTensorView;

fn create_prefix_key(name: &str) -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0x33; 32], name, "F16"),
        tokenizer_hash: [0x44; 32],
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

#[test]
fn test_two_tenant_isolation_no_cross_read_or_cross_invalidation() {
    let isolated_store = TenantIsolatedKvStore::new();
    let prefix_store = TenantPrefixKvStore::new().with_reuse_policy(KvReusePolicy::Always);

    let tenant_a = TenantId::try_new(1001).expect("valid tenant_a");
    let tenant_b = TenantId::try_new(2002).expect("valid tenant_b");

    let seg_a = KvSegment::new(tenant_a, 100, vec![0x11; 64]);
    let seg_b = KvSegment::new(tenant_b, 200, vec![0x22; 64]);

    isolated_store.try_insert_segment(tenant_a, seg_a).unwrap();
    isolated_store.try_insert_segment(tenant_b, seg_b).unwrap();

    // 1. Tenant A cannot read Tenant B's segment
    assert_eq!(isolated_store.get_segments(tenant_a), vec![100]);
    assert_eq!(isolated_store.get_segments(tenant_b), vec![200]);
    assert!(isolated_store.get_segment_bytes(tenant_a, 200).is_none());
    assert!(isolated_store.get_segment_bytes(tenant_b, 100).is_none());

    // 2. Tenant A purging/rollback does NOT affect Tenant B
    isolated_store.purge_tenant(tenant_a);
    assert!(isolated_store.get_segments(tenant_a).is_empty());
    assert_eq!(isolated_store.get_segments(tenant_b), vec![200]);

    // 3. PrefixStore Scratchpad Invalidation Two-Tenant Isolation
    let key = create_prefix_key("shared-gpt4-model");
    let tokens = vec![10, 20, 30];

    let block_a = KvBlock {
        block_id: 1,
        data: Bytes::from_static(b"tenant_a_scratch"),
    };
    let block_b = KvBlock {
        block_id: 2,
        data: Bytes::from_static(b"tenant_b_scratch"),
    };

    prefix_store
        .insert(tenant_a, &key, &tokens, vec![block_a])
        .unwrap();
    prefix_store
        .insert(tenant_b, &key, &tokens, vec![block_b])
        .unwrap();

    let invalidator_a =
        PrefixStoreScratchpadInvalidator::new(&prefix_store).with_keys(vec![key.clone()]);
    let scope_a = ScratchpadCacheScope::new(tenant_a, tokens.clone());
    let scope_b = ScratchpadCacheScope::new(tenant_b, tokens.clone());

    // Invalidation by Tenant A
    let removed_a = invalidator_a.invalidate_scope(&scope_a).unwrap();
    assert_eq!(removed_a, 1);

    // Tenant A's entry is gone
    assert!(prefix_store.lookup(tenant_a, &key, &tokens).is_none());

    // Tenant B's entry MUST remain intact and retrievable (VETO-F10)
    let hit_b = prefix_store
        .lookup(tenant_b, &key, &tokens)
        .expect("Tenant B entry untouched");
    assert_eq!(hit_b.matched_tokens, 3);
    assert_eq!(invalidator_a.count_entries_in_scope(&scope_b).unwrap(), 1);
}

#[test]
fn test_segment_validation_length_bounds_and_bytecheck() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(999).expect("valid tenant");

    // 1. Empty segment validation failure
    let empty_seg = KvSegment::new(tenant, 1, vec![]);
    let res_empty = store.try_insert_segment(tenant, empty_seg);
    assert!(res_empty.is_err());
    let err_empty = res_empty.err().unwrap();
    assert!(matches!(err_empty, ContextraError::InvalidInput(_)));
    assert!(err_empty.to_string().contains("cannot be empty"));

    // 2. Valid raw segment succeeds
    let valid_seg = KvSegment::new(tenant, 2, vec![0xAA; 30]);
    let res_valid = store.try_insert_segment(tenant, valid_seg);
    assert!(res_valid.is_ok());
    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    // 3. KvTensorView structural alignment and bytecheck validation (if feature enabled)
    #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
    {
        // Invalid zero tokens/channels
        let invalid_tensor = KvTensorView::new(vec![1.0, 2.0], vec![1.0, 2.0], 0, 2);
        assert!(invalid_tensor.is_err());

        // Length mismatch (keys len 3 vs expected 4 for 2x2)
        let mismatch_tensor =
            KvTensorView::new(vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        assert!(mismatch_tensor.is_err());

        // Non-finite bytecheck failure (NaN value)
        let nan_tensor = KvTensorView::new(vec![f32::NAN, 2.0], vec![1.0, 2.0], 1, 2);
        assert!(nan_tensor.is_err());
    }
}
