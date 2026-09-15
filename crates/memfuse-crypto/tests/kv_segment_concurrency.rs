// FILE-CONTEXT
// ZWECK: Concurrency Stress Test für TenantIsolatedKvStore und EvictionWorker unter hoher Parallellast.
// STAND: TS:2026-09-09T16:15:00Z (SESSION: dafac391)

use memfuse_core::TenantId;
use memfuse_crypto::kv_segment::{
    emergency_wipe, EvictionWorker, KvSegment, TenantIsolatedKvStore,
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_concurrent_tenant_store_read_write() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let num_threads = 8;
    let ops_per_thread = 100;

    let handles: Vec<_> = (0..num_threads)
        .map(|t_idx| {
            let store = Arc::clone(&store);
            thread::spawn(move || {
                let tenant = TenantId::try_new(t_idx as u64 + 1).unwrap();
                for op in 0..ops_per_thread {
                    let seg_id = (t_idx * ops_per_thread + op) as u64;
                    let seg = KvSegment::new(tenant, seg_id, vec![t_idx as u8; 128]);
                    store.insert_segment(tenant, seg);

                    let segs = store.get_segments(tenant);
                    assert!(!segs.is_empty());

                    let count = store.get_tenant_segment_len(tenant);
                    assert!(count > 0);

                    let raw_bytes = store.get_segment_bytes(tenant, seg_id);
                    assert!(raw_bytes.is_some());
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    // Verify isolation and total count after concurrent execution
    for t_idx in 0..num_threads {
        let tenant = TenantId::try_new(t_idx as u64 + 1).unwrap();
        assert_eq!(
            store.get_tenant_segment_len(tenant),
            ops_per_thread as usize
        );
        assert_eq!(store.get_segments(tenant).len(), ops_per_thread as usize);
    }
}

#[test]
fn test_concurrent_eviction_worker_triggers() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();

    // Populate store with 50 segments
    for i in 0..50 {
        store.insert_segment(tenant, KvSegment::new(tenant, i, vec![0xFF; 256]));
    }

    let worker = Arc::new(EvictionWorker::spawn(Arc::clone(&store)));
    let num_trigger_threads = 4;

    let handles: Vec<_> = (0..num_trigger_threads)
        .map(|_| {
            let worker = Arc::clone(&worker);
            thread::spawn(move || {
                for _ in 0..10 {
                    worker.trigger_eviction(256);
                    thread::sleep(Duration::from_millis(2));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Trigger thread should not panic");
    }

    // Wait for worker queue to settle
    thread::sleep(Duration::from_millis(100));

    // Segments should have been evicted down
    let remaining = store.get_tenant_segment_len(tenant);
    assert!(
        remaining < 50,
        "Segments should have been evicted by worker thread"
    );
}

#[test]
fn test_concurrent_emergency_wipe_race() {
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();

    for i in 0..100 {
        store.insert_segment(tenant, KvSegment::new(tenant, i, vec![0x11; 64]));
    }

    let store_ref1 = Arc::clone(&store);
    let store_ref2 = Arc::clone(&store);

    let handle1 = thread::spawn(move || {
        emergency_wipe(&store_ref1);
    });

    let handle2 = thread::spawn(move || {
        emergency_wipe(&store_ref2);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    assert_eq!(
        store.get_tenant_segment_len(tenant),
        0,
        "Store must be completely empty after emergency wipe"
    );
}

#[test]
fn test_adr082_lru_order_not_corrupted_by_get_segments() {
    // Setup: 3 Segmente mit zeitlichem Abstand
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(1).unwrap();

    store.insert_segment(tenant, KvSegment::new(tenant, 1, vec![0x11; 128]));
    std::thread::sleep(std::time::Duration::from_millis(1));
    store.insert_segment(tenant, KvSegment::new(tenant, 2, vec![0x22; 128]));
    std::thread::sleep(std::time::Duration::from_millis(1));
    store.insert_segment(tenant, KvSegment::new(tenant, 3, vec![0x33; 128]));

    // Segment 1 gezielt touchen (es ist das "frischeste")
    let _ = store.get_segment_bytes(tenant, 1);

    // get_segments() aufrufen — darf LRU-Reihenfolge NICHT verändern
    let _ids = store.get_segments(tenant);
    let _ids = store.get_segments(tenant);
    let _ids = store.get_segments(tenant);

    // Segment 2 ist jetzt das LRU (least recently used)
    // Eviction MUSS Segment 2 treffen, NICHT Segment 1 (das getouch'd wurde)
    let freed = store.evict_lru_fair(128);
    assert!(freed >= 128, "Muss mindestens 128 Bytes freigeben");
    assert!(
        store.get_segment_bytes(tenant, 2).is_none(),
        "Segment 2 (LRU) muss evictet worden sein — get_segments() darf LRU nicht manipulieren"
    );
    assert!(
        store.get_segment_bytes(tenant, 1).is_some(),
        "Segment 1 (MRU via touch) muss erhalten bleiben"
    );
}

#[test]
fn test_adr082_rollback_om_complexity_not_retain() {
    let store = TenantIsolatedKvStore::new();
    let tenant = TenantId::try_new(42).unwrap();

    // N = 500 Segmente einfügen
    for id in 0u64..500 {
        store.insert_segment(tenant, KvSegment::new(tenant, id, vec![id as u8; 32]));
    }
    assert_eq!(
        store.get_tenant_segment_len(tenant),
        256,
        "LruCache auf Default-Capacity 256 begrenzt"
    );

    // M = 50 IDs rollbacken — Zeitmessung sollte konstant sein
    let rollback_ids: Vec<u64> = (0u64..50).collect();
    let start = std::time::Instant::now();
    store.remove_segments_for_rollback(tenant, &rollback_ids);
    let elapsed = start.elapsed();

    // O(M) mit M=50 sollte deutlich unter 10ms sein (kein O(N×M) Overhead)
    assert!(
        elapsed.as_millis() < 10,
        "remove_segments_for_rollback mit M=50 sollte < 10ms dauern (O(M)), war: {:?}",
        elapsed
    );
}

#[test]
fn test_adr082_shard_isolation_same_shard_different_tenant() {
    let store = TenantIsolatedKvStore::new();
    let tenant_a = TenantId::try_new(1).unwrap();
    let tenant_b = TenantId::try_new(3).unwrap();

    store.insert_segment(tenant_a, KvSegment::new(tenant_a, 100, vec![0xAA; 64]));
    store.insert_segment(tenant_b, KvSegment::new(tenant_b, 200, vec![0xBB; 64]));

    // Trotz gleichen Shards: kein Cross-Tenant-Zugriff
    assert!(
        store.get_segment_bytes(tenant_a, 200).is_none(),
        "Tenant A darf Segment von Tenant B nicht sehen (INV-TENANT)"
    );
    assert!(
        store.get_segment_bytes(tenant_b, 100).is_none(),
        "Tenant B darf Segment von Tenant A nicht sehen (INV-TENANT)"
    );

    // Eigene Segmente sind weiterhin zugänglich
    assert!(store.get_segment_bytes(tenant_a, 100).is_some());
    assert!(store.get_segment_bytes(tenant_b, 200).is_some());
}

#[test]
fn test_adr082_deferred_drop_zeroize_does_not_block_readers() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant = TenantId::try_new(1).unwrap();
    let reader_tenant = TenantId::try_new(2).unwrap();

    // Befülle mit großen Segmenten (simuliert Tensor-Größe)
    for id in 0u64..10 {
        store.insert_segment(tenant, KvSegment::new(tenant, id, vec![0xFF; 64_000]));
    }
    // Reader-Tenant hat mehrere Segmente (999 ist MRU)
    for id in 1u64..10 {
        store.insert_segment(
            reader_tenant,
            KvSegment::new(reader_tenant, id, vec![0xEE; 32]),
        );
    }
    store.insert_segment(
        reader_tenant,
        KvSegment::new(reader_tenant, 999, vec![0xEE; 32]),
    );

    let read_succeeded = Arc::new(AtomicBool::new(false));
    let read_succeeded_clone = Arc::clone(&read_succeeded);
    let store_clone = Arc::clone(&store);

    // Eviction triggern (200KB Target = ~3-4 große Segmente)
    let evict_handle = std::thread::spawn(move || {
        store_clone.evict_lru_fair(200_000);
    });

    // Gleichzeitig lesen — darf nicht blockiert werden (Deferred-Drop außerhalb Lock)
    let read_handle = std::thread::spawn(move || {
        // Kurz warten damit Eviction startet
        std::thread::sleep(std::time::Duration::from_millis(1));
        let result = store.get_segment_bytes(reader_tenant, 999);
        read_succeeded_clone.store(result.is_some(), Ordering::SeqCst);
    });

    evict_handle.join().expect("evict thread panicked");
    read_handle.join().expect("read thread panicked");

    assert!(
        read_succeeded.load(Ordering::SeqCst),
        "Reader MUSS während Eviction/Zeroize erfolgreich lesen können"
    );
}

#[test]
fn test_adr082_concurrent_multitenant_shard_parallelism() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let store = Arc::new(TenantIsolatedKvStore::new());
    let total_ops = Arc::new(AtomicUsize::new(0));
    let num_tenants = 32usize;
    let ops_per_tenant = 50usize;

    let handles: Vec<_> = (1..=num_tenants)
        .map(|t| {
            let store = Arc::clone(&store);
            let ops_counter = Arc::clone(&total_ops);
            std::thread::spawn(move || {
                let tenant = TenantId::try_new(t as u64).unwrap();
                for op in 0..ops_per_tenant {
                    let id = (t * 1000 + op) as u64;
                    store.insert_segment(tenant, KvSegment::new(tenant, id, vec![t as u8; 256]));
                    let _ = store.get_segment_bytes(tenant, id);
                    ops_counter.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().expect("tenant thread panicked");
    }

    let total = total_ops.load(Ordering::Relaxed);
    assert_eq!(
        total,
        num_tenants * ops_per_tenant,
        "Alle {total} Ops müssen ohne Panic/Deadlock abgeschlossen sein"
    );

    // INV-TENANT: Kein Cross-Tenant-Leak
    for t in 1u64..=num_tenants as u64 {
        let tenant = TenantId::try_new(t).unwrap();
        let ids = store.get_segments(tenant);
        for id in &ids {
            // Jede ID muss zum richtigen Tenant gehören
            let expected_tenant = (*id / 1000) as u64;
            // IDs: t*1000..t*1000+50 → expected_tenant == t
            assert_eq!(
                expected_tenant, t,
                "Cross-Tenant-Leak: Tenant {t} hat Segment-ID {id} die zu Tenant {expected_tenant} gehört"
            );
        }
    }
}
