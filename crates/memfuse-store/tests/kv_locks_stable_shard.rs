// Testpflicht AK-14 (docs/specs/MEMFUSE_SPEC_v2.md)

use memfuse_store::KvKeyLocks;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

#[test]
fn test_kv_key_locks_hash_stability() {
    let locks1 = KvKeyLocks::new(4);

    let key_a = "entity:tenant_123:doc_456";
    let key_b = "entity:tenant_123:doc_789";
    let key_c = 42u64;

    // 1. Same instance stability (sequential calls yield identical hashes)
    let hash_a1 = locks1.key_hash(&key_a);
    let hash_a2 = locks1.key_hash(&key_a);
    assert_eq!(
        hash_a1, hash_a2,
        "Repeated key_hash calls on same instance must be identical"
    );

    let hash_b1 = locks1.key_hash(&key_b);
    let hash_b2 = locks1.key_hash(&key_b);
    assert_eq!(
        hash_b1, hash_b2,
        "Repeated key_hash calls on same instance must be identical"
    );

    let hash_c1 = locks1.key_hash(&key_c);
    let hash_c2 = locks1.key_hash(&key_c);
    assert_eq!(
        hash_c1, hash_c2,
        "Repeated key_hash calls on same instance must be identical"
    );

    // 2. Cross-instance reproducibility (fixed seeds guarantee identical hashes across distinct instances)
    let locks2 = KvKeyLocks::new(4);
    assert_eq!(
        locks1.key_hash(&key_a),
        locks2.key_hash(&key_a),
        "Cross-instance key_hash must be identical for identical construction parameters"
    );
    assert_eq!(
        locks1.key_hash(&key_b),
        locks2.key_hash(&key_b),
        "Cross-instance key_hash must be identical for identical construction parameters"
    );
    assert_eq!(
        locks1.key_hash(&key_c),
        locks2.key_hash(&key_c),
        "Cross-instance key_hash must be identical for identical construction parameters"
    );
}

#[test]
fn test_kv_key_locks_mutual_exclusion() {
    let locks = Arc::new(KvKeyLocks::new(4));
    let key = "shared_entity_entity_id_101";
    let key_hash = locks.key_hash(&key);

    let active_count = Arc::new(AtomicU32::new(0));
    let max_concurrent = Arc::new(AtomicU32::new(0));

    let num_threads = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for _ in 0..num_threads {
        let locks_clone = Arc::clone(&locks);
        let active_clone = Arc::clone(&active_count);
        let max_clone = Arc::clone(&max_concurrent);
        let barrier_clone = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            // Synchronize starting time so all threads contend simultaneously
            barrier_clone.wait();

            let _guard = locks_clone.acquire(key_hash);

            let curr = active_clone.fetch_add(1, Ordering::SeqCst) + 1;
            // Update peak concurrent execution count
            max_clone.fetch_max(curr, Ordering::SeqCst);

            // Hold lock briefly to force contenders to block
            thread::sleep(Duration::from_millis(15));

            active_clone.fetch_sub(1, Ordering::SeqCst);
        }));
    }

    for handle in handles {
        handle.join().expect("Thread join failed");
    }

    assert_eq!(
        max_concurrent.load(Ordering::SeqCst),
        1,
        "Mutual exclusion violation: multiple concurrent threads acquired the lock for the same key hash"
    );
}

#[test]
fn test_kv_key_locks_different_keys_concurrency() {
    let locks = Arc::new(KvKeyLocks::new(4));

    // Find two keys that hash to different shard indices under 16 shards (pow2=4, mask=0xF)
    let mut key1 = "key_alpha_1";
    let mut key2 = "key_beta_1";
    let mut idx = 0usize;

    while (locks.key_hash(&key1) & 0xF) == (locks.key_hash(&key2) & 0xF) {
        idx += 1;
        key1 = Box::leak(format!("key_alpha_{}", idx).into_boxed_str());
        key2 = Box::leak(format!("key_beta_{}", idx).into_boxed_str());
    }

    let hash1 = locks.key_hash(&key1);
    let hash2 = locks.key_hash(&key2);

    let active_count = Arc::new(AtomicU32::new(0));
    let max_concurrent = Arc::new(AtomicU32::new(0));

    let barrier = Arc::new(Barrier::new(2));

    let t1_locks = Arc::clone(&locks);
    let t1_active = Arc::clone(&active_count);
    let t1_max = Arc::clone(&max_concurrent);
    let t1_barrier = Arc::clone(&barrier);

    let handle1 = thread::spawn(move || {
        let _guard = t1_locks.acquire(hash1);
        let curr = t1_active.fetch_add(1, Ordering::SeqCst) + 1;
        t1_max.fetch_max(curr, Ordering::SeqCst);

        t1_barrier.wait();
        thread::sleep(Duration::from_millis(15));

        t1_active.fetch_sub(1, Ordering::SeqCst);
    });

    let t2_locks = Arc::clone(&locks);
    let t2_active = Arc::clone(&active_count);
    let t2_max = Arc::clone(&max_concurrent);
    let t2_barrier = Arc::clone(&barrier);

    let handle2 = thread::spawn(move || {
        let _guard = t2_locks.acquire(hash2);
        let curr = t2_active.fetch_add(1, Ordering::SeqCst) + 1;
        t2_max.fetch_max(curr, Ordering::SeqCst);

        t2_barrier.wait();
        thread::sleep(Duration::from_millis(15));

        t2_active.fetch_sub(1, Ordering::SeqCst);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();

    assert_eq!(
        max_concurrent.load(Ordering::SeqCst),
        2,
        "Distinct shards should allow concurrent lock acquisition"
    );
}
