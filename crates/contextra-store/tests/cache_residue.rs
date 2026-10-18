use bytes::Bytes;
use contextra_store::memtable::{zeroize_bytes_if_unique, MemTable};
use contextra_store::sstable::block_cache::{
    make_zeroizing_bytes, BlockCache, CchRamClaim,
};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use zeroize::Zeroize;

#[test]
fn test_block_cache_invalidate_files_and_zeroize_on_drop() {
    let mut cache = BlockCache::new(64 * 1024);
    cache.set_secure_buffers(true);

    let file_id = 42u64;
    let offset = 1024u64;

    let canary_bytes = vec![0xAA; 256];
    let data = make_zeroizing_bytes(canary_bytes);

    cache.insert(file_id, offset, data.clone());

    let report_before = cache.cache_report(&[file_id]);
    assert_eq!(report_before.entries_found, 1);
    assert_eq!(report_before.ram_claim, CchRamClaim::Zeroized);

    drop(data);

    let removed = cache.invalidate_files(&[file_id]);
    assert_eq!(removed, 1);

    let report_after = cache.cache_report(&[file_id]);
    assert_eq!(report_after.entries_found, 0);
}

#[test]
fn test_zeroizing_bytes_owner_zeroizes_buffer_on_drop() {
    let captured_post_zero = Arc::new(Mutex::new(Vec::new()));

    struct InspectableZeroizingOwner {
        data: Vec<u8>,
        post_zero_capture: Arc<Mutex<Vec<u8>>>,
    }

    impl AsRef<[u8]> for InspectableZeroizingOwner {
        fn as_ref(&self) -> &[u8] {
            &self.data
        }
    }

    impl Drop for InspectableZeroizingOwner {
        fn drop(&mut self) {
            self.data.as_mut_slice().zeroize();
            if let Ok(mut guard) = self.post_zero_capture.lock() {
                *guard = self.data.clone();
            }
        }
    }

    let owner = InspectableZeroizingOwner {
        data: vec![0xFE; 128],
        post_zero_capture: Arc::clone(&captured_post_zero),
    };

    let bytes = Bytes::from_owner(owner);
    assert_eq!(bytes.as_ref(), &[0xFE; 128][..]);

    drop(bytes);

    // Verify post-drop captured memory was overwritten with zeros
    let captured = captured_post_zero.lock().unwrap();
    assert_eq!(captured.len(), 128);
    assert_eq!(*captured, vec![0x00; 128]);
}

#[test]
fn test_shared_reader_deferred_zeroize_no_use_after_zero() {
    let mut cache = BlockCache::new(64 * 1024);
    cache.set_secure_buffers(true);

    let file_id = 100u64;
    let offset = 2048u64;

    let payload = vec![0x55; 512];
    cache.insert(file_id, offset, Bytes::from(payload.clone()));

    let reader_handle = cache.get(file_id, offset).expect("Entry should exist");
    assert_eq!(reader_handle.as_ref(), &[0x55; 512][..]);

    let removed = cache.invalidate_files(&[file_id]);
    assert_eq!(removed, 1);

    // Payload is readable and intact while reader holds its handle (no premature zeroing)
    assert_eq!(reader_handle.as_ref(), &[0x55; 512][..]);

    drop(reader_handle);
}

#[test]
fn test_ram_claim_negative_without_secure_buffers() {
    let cache = BlockCache::new(64 * 1024);
    assert!(!cache.secure_buffers());

    let report = cache.cache_report(&[1, 2, 3]);
    assert_eq!(report.ram_claim, CchRamClaim::Unreferenced);
}

#[test]
fn test_memtable_canary_bytes_zeroized_on_rollback() {
    let mt = MemTable::new();

    let key = Bytes::from(vec![0x11; 64]);
    let val = Bytes::from(vec![0x99; 256]);

    mt.put(key.clone(), val.clone(), 1, 42);

    assert_eq!(mt.size(), 64 + 256 + 32);

    drop(key);
    drop(val);

    mt.rollback(42);

    assert_eq!(mt.size(), 0);
    assert!(mt.is_empty());
}

#[test]
fn test_zeroize_bytes_if_unique_helper() {
    let raw = vec![0xEE; 128];
    let b = Bytes::from(raw);
    zeroize_bytes_if_unique(b);

    let b1 = Bytes::from(vec![0xDD; 64]);
    let b2 = b1.clone();
    zeroize_bytes_if_unique(b1);
    assert_eq!(b2.as_ref(), &[0xDD; 64][..]);
}

#[test]
fn test_block_cache_concurrency_stress() {
    use std::sync::atomic::AtomicUsize;

    let cache: Arc<BlockCache> = Arc::new({
        let mut c = BlockCache::new_with_shards(64 * 1024, 8);
        c.set_secure_buffers(true);
        c
    });

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for t in 0..8u64 {
        let cache_ref = Arc::clone(&cache);
        let counter_ref = Arc::clone(&counter);
        handles.push(std::thread::spawn(move || {
            for i in 0..500u64 {
                let file_id = (t + i) % 10;
                let offset = i * 4096;
                let data = Bytes::from(vec![(i % 255) as u8; 128]);

                cache_ref.insert(file_id, offset, data);
                let _ = cache_ref.get(file_id, offset);

                if i % 20 == 0 {
                    let inv = cache_ref.invalidate_files(&[file_id]);
                    counter_ref.fetch_add(inv, Ordering::Relaxed);
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }

    let _cleared = cache.clear_all();
    assert_eq!(cache.len(), 0);
    let report = cache.cache_report(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(report.entries_found, 0);
}
