// FILE-CONTEXT
// ZWECK: Cancellation- & RAII-Guard-Tests für den KV-Cache (Task KV-06).
// STAND: TS:2026-09-15T00:00:00Z

use contextra_kvcache::{EvictionWorker, KvSegment, TenantIsolatedKvStore};
use contextra_types::{ContextraError, TenantId};
use std::sync::Arc;
use std::thread;

#[test]
fn test_cancellation_raii_guard_prevents_eviction_and_releases() -> Result<(), ContextraError> {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(10)?;

    store.insert_segment(tenant, KvSegment::new(tenant, 100, vec![0x11; 1024]));

    // Acquire RAII guard for the request
    let guard = store
        .acquire_block_guard(tenant, 100, None)
        .expect("Should acquire guard for segment 100");

    assert_eq!(guard.active_refs(), 1);

    // Attempt eviction while request is running (guard held)
    let freed_while_running = store.evict_lru_fair(1024);
    assert_eq!(
        freed_while_running, 0,
        "Active RAII guard MUST protect segment from eviction during request execution"
    );
    assert_eq!(store.get_tenant_segment_len(tenant), 1);

    // Simulate request cancellation -> drop RAII guard
    drop(guard);

    // Eviction after cancellation MUST succeed
    let freed_after_cancel = store.evict_lru_fair(1024);
    assert!(
        freed_after_cancel >= 1024,
        "Segment must be evictable once RAII guard is dropped on cancellation"
    );
    assert_eq!(store.get_tenant_segment_len(tenant), 0);

    Ok(())
}

#[test]
fn test_cancellation_eviction_worker_notification() -> Result<(), ContextraError> {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(20)?;

    store.insert_segment(tenant, KvSegment::new(tenant, 200, vec![0x22; 512]));

    let worker = Arc::new(EvictionWorker::spawn(Arc::clone(&store)));

    let guard = store
        .acquire_block_guard(tenant, 200, Some(Arc::clone(&worker)))
        .expect("Should acquire guard with worker handle");

    assert_eq!(guard.active_refs(), 1);

    // Drop guard -> triggers worker notification
    drop(guard);

    // Verify worker worker thread is healthy and handles eviction
    worker.trigger_eviction(512);

    let mut freed = false;
    for _ in 0..100 {
        thread::sleep(std::time::Duration::from_millis(10));
        if store.get_tenant_segment_len(tenant) == 0 {
            freed = true;
            break;
        }
    }

    assert!(
        freed,
        "Background worker must evict unguarded segment post-cancellation"
    );

    Ok(())
}

#[test]
fn test_cancellation_transaction_rollback_cleans_up_staged_segments() -> Result<(), ContextraError>
{
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(30)?;

    // Simulate multi-step inference request staging KV segments
    let staged_segment_ids = vec![301u64, 302, 303, 304, 305];
    for &id in &staged_segment_ids {
        store.insert_segment(tenant, KvSegment::new(tenant, id, vec![0x33; 256]));
    }

    assert_eq!(store.get_tenant_segment_len(tenant), 5);

    // Simulate request cancellation mid-inference -> trigger rollback
    store.on_rollback(tenant, &staged_segment_ids);

    // Verify all cancelled segments are purged without memory leaks
    assert_eq!(
        store.get_tenant_segment_len(tenant),
        0,
        "Rollback must purge all cancelled segments"
    );
    assert!(store.get_segments(tenant).is_empty());

    Ok(())
}

#[test]
fn test_cancellation_concurrent_requests_guard_safety() -> Result<(), ContextraError> {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(40)?;

    // Pre-populate 10 KV segments
    for id in 1..=10 {
        store.insert_segment(tenant, KvSegment::new(tenant, id, vec![0x44; 128]));
    }

    let mut handles = Vec::new();

    // Spawn 10 concurrent request threads that acquire guards and randomly cancel/finish
    for i in 1..=10 {
        let store_clone = Arc::clone(&store);
        let handle = thread::spawn(move || {
            if let Some(guard) = store_clone.acquire_block_guard(tenant, i, None) {
                assert!(guard.active_refs() >= 1);
                // Simulate work or early cancellation
                if i % 2 == 0 {
                    // Abort / cancel request scope early
                    drop(guard);
                } else {
                    thread::sleep(std::time::Duration::from_millis(5));
                    drop(guard);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Worker thread panicked");
    }

    // After all concurrent requests complete/cancel, all guards are dropped
    let freed = store.evict_lru_fair(10 * 128);
    assert!(
        freed >= 10 * 128,
        "All segments must be freed after all guards drop"
    );
    assert_eq!(store.get_tenant_segment_len(tenant), 0);

    Ok(())
}
