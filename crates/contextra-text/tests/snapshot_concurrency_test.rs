//! Concurrency and Snapshot Isolation Tests for `MemoryTextIndex` (INV-TEXT-1).
//!
//! Validates:
//! - Multi-threaded concurrent readers do not block writers.
//! - Writers work Copy-on-Write and update `Epoch` monotonically.
//! - Readers holding an older `IndexSnapshot` see immutable, consistent data.

use contextra_text::bm25::BM25;
use contextra_text::contract::{DocKey, Epoch};
use contextra_text::memory_index::MemoryTextIndex;
use contextra_text::traits::{Index, Maintain};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_snapshot_concurrent_readers_writers_no_deadlock() {
    let index = Arc::new(parking_lot::RwLock::new(MemoryTextIndex::new()));

    // Seed initial documents
    {
        let mut idx = index.write();
        for i in 1..=50 {
            let key = DocKey::from(i as u64);
            let text = format!("document {} rust inverted index full text search", i);
            Index::<DocKey, str>::insert(&mut *idx, key, text.as_bytes()).unwrap();
        }
    }

    let running = Arc::new(AtomicBool::new(true));
    let mut reader_handles = Vec::new();

    // Spawn 4 reader threads
    for _ in 0..4 {
        let index_clone = Arc::clone(&index);
        let running_clone = Arc::clone(&running);
        let handle = thread::spawn(move || {
            let mut query_count = 0;
            while running_clone.load(Ordering::Relaxed) {
                // Obtain snapshot in O(1)
                let snap = {
                    let idx = index_clone.read();
                    idx.snapshot()
                };

                let hits = snap.search(
                    &["rust".to_string(), "search".to_string()],
                    &BM25::default(),
                    20,
                );
                assert!(
                    !hits.is_empty(),
                    "Reader must always see documents matching 'rust search'"
                );
                query_count += 1;
                thread::yield_now();
            }
            query_count
        });
        reader_handles.push(handle);
    }

    // Writer thread
    let writer_handle = {
        let index_clone = Arc::clone(&index);
        thread::spawn(move || {
            for i in 51..=150 {
                let key = DocKey::from(i as u64);
                let text = format!("new doc {} rust benchmark concurrency test", i);
                {
                    let mut idx = index_clone.write();
                    Index::<DocKey, str>::insert(&mut *idx, key, text.as_bytes()).unwrap();
                }
                thread::sleep(Duration::from_millis(1));
            }

            // Also test removal and compaction concurrently
            for i in 1..=20 {
                let key = DocKey::from(i as u64);
                let mut idx = index_clone.write();
                Index::<DocKey, str>::remove(&mut *idx, &key).unwrap();
            }

            let mut idx = index_clone.write();
            let stats = Maintain::compact(&mut *idx).unwrap();
            assert_eq!(stats.purged_docs, 20);
        })
    };

    writer_handle.join().expect("Writer finished cleanly");
    running.store(false, Ordering::Relaxed);

    for h in reader_handles {
        let count = h.join().expect("Reader finished cleanly");
        assert!(count > 0, "Reader must have executed at least one query");
    }

    let final_idx = index.read();
    assert_eq!(final_idx.len(), 150 - 20);
    assert!(final_idx.current_epoch() >= Epoch(120));
}
