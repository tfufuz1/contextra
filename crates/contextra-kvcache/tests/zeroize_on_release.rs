#![forbid(unsafe_code)]

use bytes::Bytes;
use contextra_kvcache::prefix_store::TenantPrefixKvStore;
use contextra_kvcache::scratchpad_invalidation::{
    PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator,
};
use contextra_kvcache::segment::KvSegment;
use contextra_kvcache::store::TenantIsolatedKvStore;
use contextra_ports::kv::{KvBlock, KvLayout, PrefixKey, RopeConfig};
use contextra_types::error::{CacheDirective, StepId};
use contextra_types::model_fingerprint::ModelFingerprint;
use contextra_types::TenantId;
use parking_lot::Mutex;
use std::sync::Arc;

fn create_test_prefix_key(name: &str) -> PrefixKey {
    PrefixKey {
        model: ModelFingerprint::new([0xde; 32], name, "F16"),
        tokenizer_hash: [0x22; 32],
        layout: KvLayout {
            n_layer: 16,
            n_kv_head: 4,
            head_dim: 64,
            dtype: "f16".to_string(),
        },
        rope: RopeConfig {
            base: 10000.0,
            scaling: None,
        },
    }
}

#[test]
fn test_zeroize_on_release_step() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(1).unwrap();
    let step = StepId(101);

    let inspector = Arc::new(Mutex::new(Vec::new()));
    let payload = vec![0xABu8; 1024];

    let mut segment = KvSegment::new(tenant, 1, payload);
    segment.test_inspector = Some(Arc::clone(&inspector));

    store
        .insert_segment_with_directive(
            tenant,
            segment,
            CacheDirective::ReleaseAfterStep { step_id: step },
        )
        .unwrap();

    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    // Release step
    store.release_step(tenant, step);

    assert_eq!(store.get_tenant_segment_len(tenant), 0);

    let recorded = inspector.lock();
    assert_eq!(
        recorded.len(),
        1024,
        "Inspector buffer length must match original segment length"
    );
    assert!(
        recorded.iter().all(|&b| b == 0),
        "Segment payload buffer MUST be 0-filled after release_step"
    );
}

#[test]
fn test_zeroize_on_eviction() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(1).unwrap();

    let inspector = Arc::new(Mutex::new(Vec::new()));
    let payload = vec![0xCDu8; 512];

    let mut segment = KvSegment::new(tenant, 10, payload);
    segment.test_inspector = Some(Arc::clone(&inspector));

    store.insert_segment(tenant, segment);

    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    // Evict fair
    let freed = store.evict_lru_fair(512);
    assert!(freed >= 512);
    assert_eq!(store.get_tenant_segment_len(tenant), 0);

    let recorded = inspector.lock();
    assert_eq!(recorded.len(), 512);
    assert!(
        recorded.iter().all(|&b| b == 0),
        "Segment payload buffer MUST be 0-filled after LRU eviction"
    );
}

#[test]
fn test_zeroize_on_prefix_replace() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(1).unwrap();

    let inspector_old = Arc::new(Mutex::new(Vec::new()));
    let old_payload = vec![0x11u8; 256];

    let mut old_segment = KvSegment::new(tenant, 42, old_payload);
    old_segment.test_inspector = Some(Arc::clone(&inspector_old));

    store.insert_segment(tenant, old_segment);

    // Replace segment id 42 with new content
    let new_segment = KvSegment::new(tenant, 42, vec![0x22u8; 256]);
    store.insert_segment(tenant, new_segment);

    let recorded = inspector_old.lock();
    assert_eq!(recorded.len(), 256);
    assert!(
        recorded.iter().all(|&b| b == 0),
        "Replaced segment payload buffer MUST be 0-filled upon replacement"
    );
}

#[test]
fn test_zeroize_on_rollback() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(42).unwrap();

    let inspector = Arc::new(Mutex::new(Vec::new()));
    let mut segment = KvSegment::new(tenant, 77, vec![0x77u8; 128]);
    segment.test_inspector = Some(Arc::clone(&inspector));

    store.insert_segment(tenant, segment);

    store.on_rollback(tenant, &[77]);
    assert_eq!(store.get_tenant_segment_len(tenant), 0);

    let recorded = inspector.lock();
    assert_eq!(recorded.len(), 128);
    assert!(
        recorded.iter().all(|&b| b == 0),
        "Segment payload buffer MUST be 0-filled upon rollback"
    );
}

#[test]
fn test_zeroize_on_clear_all() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(1).unwrap();

    let inspector = Arc::new(Mutex::new(Vec::new()));
    let mut segment = KvSegment::new(tenant, 99, vec![0xFFu8; 2048]);
    segment.test_inspector = Some(Arc::clone(&inspector));

    store.insert_segment(tenant, segment);

    store.clear_all();
    assert_eq!(store.get_tenant_segment_len(tenant), 0);

    let recorded = inspector.lock();
    assert_eq!(recorded.len(), 2048);
    assert!(
        recorded.iter().all(|&b| b == 0),
        "Segment payload buffer MUST be 0-filled upon clear_all"
    );
}

#[test]
fn test_zeroize_on_scratchpad_invalidation() {
    let store = TenantPrefixKvStore::new();
    let tenant = TenantId::try_new(10).unwrap();
    let key = create_test_prefix_key("scratchpad-test");

    let scratchpad_tokens = vec![10, 20, 30, 40];
    let inspector = Arc::new(Mutex::new(Vec::new()));

    let block = KvBlock {
        block_id: 1,
        data: Bytes::from(vec![0xAAu8; 512]),
    };

    store
        .insert(tenant, &key, &scratchpad_tokens, vec![block])
        .unwrap();

    store.set_group_test_inspector(tenant, &key, &scratchpad_tokens, Arc::clone(&inspector));

    let invalidator = PrefixStoreScratchpadInvalidator::new(&store).with_keys(vec![key.clone()]);
    let scope = ScratchpadCacheScope::new(tenant, scratchpad_tokens.clone());

    let count = invalidator.invalidate_scope(&scope).unwrap();
    assert_eq!(count, 1);

    assert!(store.lookup(tenant, &key, &scratchpad_tokens).is_none());

    let recorded = inspector.lock();
    assert_eq!(recorded.len(), 512);
    assert!(
        recorded.iter().all(|&b| b == 0),
        "Block group payload buffer MUST be 0-filled upon scratchpad invalidation"
    );
}

#[test]
fn test_eviction_under_load_no_unzeroed_data() {
    let store = TenantIsolatedKvStore::with_capacity(16);
    let tenant = TenantId::try_new(100).unwrap();

    let inspectors: Vec<Arc<Mutex<Vec<u8>>>> = (0..100)
        .map(|_| Arc::new(Mutex::new(Vec::new())))
        .collect();

    for i in 0..100 {
        let mut seg = KvSegment::new(tenant, i, vec![(i % 255) as u8 + 1; 128]);
        seg.test_inspector = Some(Arc::clone(&inspectors[i as usize]));
        store.insert_segment(tenant, seg);
    }

    // Force evict down to 10 segments
    store.evict_lru_fair(90 * 128);

    // Evicted inspectors must record 0-filled buffers
    let mut zeroed_count = 0;
    for insp in &inspectors {
        let rec = insp.lock();
        if !rec.is_empty() {
            assert_eq!(rec.len(), 128);
            assert!(rec.iter().all(|&b| b == 0));
            zeroed_count += 1;
        }
    }

    assert!(
        zeroed_count >= 80,
        "At least 80 evicted segments must be zeroed (got {zeroed_count})"
    );
}
