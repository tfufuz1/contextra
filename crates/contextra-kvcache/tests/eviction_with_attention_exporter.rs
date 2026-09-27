#![forbid(unsafe_code)]

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
fn test_null_attention_exporter_baseline_lru() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();

    // Insert 3 segments sequentially (10 oldest, 20 middle, 30 newest)
    store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![0x11; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![0x22; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 30, vec![0x33; 512]));

    // Pure LRU baseline: segment 10 (oldest access) evicted first
    let freed = store.evict_lru_fair(500);
    assert!(freed >= 512);

    let remaining = store.get_segments(tenant);
    assert!(
        !remaining.contains(&10),
        "Segment 10 must be evicted under pure LRU"
    );
    assert!(remaining.contains(&20));
    assert!(remaining.contains(&30));
}

#[test]
fn test_eviction_with_mock_attention_exporter_retains_important_segment() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();

    // Insert 3 segments sequentially (10 oldest, 20 middle, 30 newest)
    store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![0x11; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![0x22; 512]));
    std::thread::sleep(Duration::from_millis(2));
    store.insert_segment(tenant, KvSegment::new(tenant, 30, vec![0x33; 512]));

    let mock_exporter = Arc::new(MockAttentionExporter::default());
    // Give Segment 10 (oldest access) VERY HIGH attention score (100.0)
    mock_exporter.set_weights(RequestId(10), vec![100.0, 100.0]);
    // Give Segment 20 (middle access) VERY LOW attention score (0.01)
    mock_exporter.set_weights(RequestId(20), vec![0.01, 0.01]);
    // Give Segment 30 (newest access) LOW attention score (0.01)
    mock_exporter.set_weights(RequestId(30), vec![0.01, 0.01]);

    let worker = EvictionWorker::spawn(Arc::clone(&store)).with_attention_exporter(mock_exporter);

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
        "Segment 10 has high attention score and must NOT be evicted despite being oldest LRU access"
    );
    assert!(
        !remaining.contains(&20),
        "Segment 20 has low attention score and must be evicted before Segment 10"
    );
    assert!(remaining.contains(&30));
}
