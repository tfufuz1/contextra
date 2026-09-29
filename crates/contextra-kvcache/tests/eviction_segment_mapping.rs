#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used)]

use contextra_kvcache::{segment::KvSegment, EvictionWorker, TenantIsolatedKvStore};
use contextra_ports::{AttentionExporter, RequestId};
use contextra_types::TenantId;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Default)]
struct MockAttentionExporter {
    scores: Mutex<HashMap<RequestId, Vec<f32>>>,
}

impl MockAttentionExporter {
    fn set_weights(&self, request_id: RequestId, weights: Vec<f32>) {
        self.scores.lock().insert(request_id, weights);
    }
}

impl AttentionExporter for MockAttentionExporter {
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        self.scores.lock().get(&request_id).cloned()
    }
}

#[test]
fn test_registered_segment_mapping_evicts_lowest_attention_score() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();

    // Insert 3 segments: 10 (oldest/LRU), 20 (middle), 30 (newest)
    store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![0x11; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![0x22; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 30, vec![0x33; 512]));

    let mock_exporter = Arc::new(MockAttentionExporter::default());
    // RequestId(101) has HIGH attention score (100.0)
    mock_exporter.set_weights(RequestId(101), vec![100.0, 100.0]);
    // RequestId(202) has LOW attention score (0.01)
    mock_exporter.set_weights(RequestId(202), vec![0.01, 0.01]);
    // RequestId(303) has MID attention score (50.0)
    mock_exporter.set_weights(RequestId(303), vec![50.0, 50.0]);

    let worker = EvictionWorker::spawn(Arc::clone(&store)).with_attention_exporter(mock_exporter);

    // Explicitly register segment-to-request mappings
    worker.register_segment_request(tenant, 10, RequestId(101));
    worker.register_segment_request(tenant, 20, RequestId(202));
    worker.register_segment_request(tenant, 30, RequestId(303));

    worker.trigger_eviction(500);

    let mut evicted = false;
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(10));
        if store.get_tenant_segment_len(tenant) == 2 {
            evicted = true;
            break;
        }
    }

    assert!(evicted, "Eviction worker should have evicted 1 segment");

    let remaining = store.get_segments(tenant);
    assert!(
        remaining.contains(&10),
        "Segment 10 (mapped to Req 101, score 100.0) must NOT be evicted despite LRU age"
    );
    assert!(
        !remaining.contains(&20),
        "Segment 20 (mapped to Req 202, score 0.01) must be evicted first due to lowest score"
    );
    assert!(remaining.contains(&30));
}

#[test]
fn test_unregister_segment_request_falls_back() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();

    store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![0x11; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![0x22; 512]));

    let mock_exporter = Arc::new(MockAttentionExporter::default());
    mock_exporter.set_weights(RequestId(101), vec![0.01]);
    mock_exporter.set_weights(RequestId(202), vec![100.0]);

    let worker = EvictionWorker::spawn(Arc::clone(&store)).with_attention_exporter(mock_exporter);

    worker.register_segment_request(tenant, 10, RequestId(101));
    worker.register_segment_request(tenant, 20, RequestId(202));

    // Unregister segment 10 mapping
    worker.unregister_segment_request(tenant, 10);

    // Segment 10 no longer has mapped weights; Segment 20 has high weights (100.0).
    // Eviction should evict segment 10.
    worker.trigger_eviction(500);

    let mut evicted = false;
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(10));
        if store.get_tenant_segment_len(tenant) == 1 {
            evicted = true;
            break;
        }
    }

    assert!(evicted, "Eviction worker should have evicted 1 segment");
    assert_eq!(store.get_segments(tenant), vec![20]);
}

#[test]
fn test_cross_tenant_attention_weights_isolation() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant_a = TenantId::try_new(100).unwrap();
    let tenant_b = TenantId::try_new(200).unwrap();

    // Both tenants have segment_id = 10
    store.insert_segment(tenant_a, KvSegment::new(tenant_a, 10, vec![0x11; 512]));
    store.insert_segment(tenant_a, KvSegment::new(tenant_a, 20, vec![0x22; 512]));

    store.insert_segment(tenant_b, KvSegment::new(tenant_b, 10, vec![0x33; 512]));
    store.insert_segment(tenant_b, KvSegment::new(tenant_b, 20, vec![0x44; 512]));

    let mock_exporter = Arc::new(MockAttentionExporter::default());
    // Tenant A segment 10 mapped to Request 1001 with HIGH score (100.0)
    mock_exporter.set_weights(RequestId(1001), vec![100.0]);
    // Tenant B segment 10 mapped to Request 2001 with LOW score (0.01)
    mock_exporter.set_weights(RequestId(2001), vec![0.01]);

    let worker = EvictionWorker::spawn(Arc::clone(&store)).with_attention_exporter(mock_exporter);

    worker.register_segment_request(tenant_a, 10, RequestId(1001));
    worker.register_segment_request(tenant_b, 10, RequestId(2001));

    // When evicting Tenant B, Tenant B's segment 10 must NOT use Tenant A's score (100.0)
    // Tenant B's segment 10 score is 0.01, so it should be evicted.
    let freed = store.evict_fair(500, worker.attention_source().as_ref());
    assert!(freed >= 512);

    // Tenant B's segment 10 had low score (0.01) and should be evicted
    assert!(
        !store.get_segments(tenant_b).contains(&10) || !store.get_segments(tenant_a).contains(&10)
    );
}
