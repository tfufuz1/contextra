#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used)]

use contextra_kvcache::{
    attention_score::AttentionScoreSource, segment::KvSegment, TenantIsolatedKvStore,
};
use contextra_types::TenantId;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

struct MockMapScoreSource {
    scores: HashMap<(TenantId, u64), f32>,
}

impl AttentionScoreSource for MockMapScoreSource {
    fn importance_score(&self, _segment_id: u64) -> Option<f32> {
        None
    }

    fn importance_score_for_tenant(&self, tenant_id: TenantId, segment_id: u64) -> Option<f32> {
        self.scores.get(&(tenant_id, segment_id)).copied()
    }
}

#[test]
fn test_unconfigured_store_behaves_as_pure_lru() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(1).unwrap();

    store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![0x11; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![0x22; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 30, vec![0x33; 512]));

    // Under pure LRU, segment 10 (oldest) is evicted
    let freed = store.evict_lru_fair(500);
    assert!(freed >= 512);

    let remaining = store.get_segments(tenant);
    assert!(
        !remaining.contains(&10),
        "Unconfigured store must evict segment 10 (oldest LRU)"
    );
    assert!(remaining.contains(&20));
    assert!(remaining.contains(&30));
}

#[test]
fn test_configured_default_attention_source_affects_evict_lru_fair() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(1).unwrap();

    // 10 oldest access, 20 middle, 30 newest
    store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![0x11; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![0x22; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 30, vec![0x33; 512]));

    let mut scores = HashMap::new();
    scores.insert((tenant, 10), 100.0); // High importance for oldest segment 10
    scores.insert((tenant, 20), 0.01); // Low importance for segment 20
    scores.insert((tenant, 30), 50.0); // Mid importance for segment 30

    let score_source = Arc::new(MockMapScoreSource { scores });
    store.set_attention_source(score_source);

    // Calling evict_lru_fair without explicit score parameter should now use configured attention_source
    let freed = store.evict_lru_fair(500);
    assert!(freed >= 512);

    let remaining = store.get_segments(tenant);
    assert!(
        remaining.contains(&10),
        "Segment 10 has high score (100.0) and must NOT be evicted despite LRU age"
    );
    assert!(
        !remaining.contains(&20),
        "Segment 20 has low score (0.01) and must be evicted first"
    );
    assert!(remaining.contains(&30));
}

#[test]
fn test_configured_default_attention_source_affects_auto_eviction_on_insert() {
    // Store capacity = 3
    let store = TenantIsolatedKvStore::with_capacity(3);
    let tenant = TenantId::try_new(1).unwrap();

    store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![0x11; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![0x22; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 30, vec![0x33; 512]));

    let mut scores = HashMap::new();
    scores.insert((tenant, 10), 100.0); // Oldest segment 10 has high importance
    scores.insert((tenant, 20), 0.01); // Middle segment 20 has low importance
    scores.insert((tenant, 30), 50.0); // Newer segment 30 has mid importance

    let score_source = Arc::new(MockMapScoreSource { scores });
    store.set_attention_source(score_source);

    // Inserting a 4th segment into full capacity store forces auto-eviction of 1 segment
    store.insert_segment(tenant, KvSegment::new(tenant, 40, vec![0x44; 512]));

    let remaining = store.get_segments(tenant);
    assert_eq!(remaining.len(), 3);
    assert!(
        remaining.contains(&10),
        "Segment 10 (score 100.0) must be retained during auto-eviction"
    );
    assert!(
        !remaining.contains(&20),
        "Segment 20 (score 0.01) must be auto-evicted on capacity overflow"
    );
    assert!(remaining.contains(&30));
    assert!(remaining.contains(&40));
}
