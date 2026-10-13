// FILE-CONTEXT
// ZWECK: Integration tests for BlockCache invalidation, count scan, and secure buffer claims.
// STAND: TS:2026-10-12T00:00:00Z

#![forbid(unsafe_code)]

use bytes::Bytes;
use contextra_store::sstable::{BlockCache, CchCacheReport, CchRamClaim};

#[test]
fn test_block_cache_file_invalidation_and_count_scan() {
    let cache = BlockCache::new(1024 * 1024);

    // Insert blocks for file 100, 200, and 300
    cache.insert(100, 0, Bytes::from_static(b"file_100_block_0"));
    cache.insert(100, 4096, Bytes::from_static(b"file_100_block_1"));
    cache.insert(200, 0, Bytes::from_static(b"file_200_block_0"));
    cache.insert(300, 0, Bytes::from_static(b"file_300_block_0"));

    // Negative control: verify scanner detects entries prior to invalidation
    let target_files = vec![100, 200];
    let initial_count = cache.entries_for_files(&target_files);
    assert_eq!(
        initial_count, 3,
        "Scanner MUST find 3 entries before invalidation"
    );

    let report_before = cache.cache_report(&target_files);
    assert_eq!(report_before.caches_checked, 64);
    assert_eq!(report_before.entries_found, 3);
    assert_eq!(report_before.ram_claim, CchRamClaim::Unreferenced);

    // Action: invalidate files 100 and 200
    let invalidated = cache.invalidate_files(&target_files);
    assert_eq!(invalidated, 3, "Invalidation must return exact removed count");

    // Postcondition: count scan must yield 0 for invalidated files
    let post_count = cache.entries_for_files(&target_files);
    assert_eq!(
        post_count, 0,
        "Count scan MUST be 0 after file invalidation"
    );

    // Verify file 300 remains untouched
    assert!(cache.contains(300, 0));
    assert_eq!(cache.entries_for_files(&[300]), 1);

    // Clear remaining cache
    let cleared = cache.clear_all();
    assert_eq!(cleared, 1);
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_secure_buffers_ram_claim_enforcement() {
    let mut cache = BlockCache::new(1024 * 1024);

    // Default configuration: secure_buffers = false
    assert!(!cache.secure_buffers());
    assert_eq!(
        cache.ram_claim(),
        CchRamClaim::Unreferenced,
        "Without secure_buffers, ram_claim MUST return Unreferenced"
    );

    // Enable secure_buffers
    cache.set_secure_buffers(true);
    assert!(cache.secure_buffers());
    assert_eq!(
        cache.ram_claim(),
        CchRamClaim::Zeroized,
        "With secure_buffers enabled, ram_claim MUST return Zeroized"
    );

    // Insert block with secure_buffers enabled
    cache.insert(500, 0, Bytes::from_static(b"secure_block_500"));
    assert_eq!(cache.entries_for_files(&[500]), 1);
    let report_sec = cache.cache_report(&[500]);
    assert_eq!(
        report_sec,
        CchCacheReport {
            caches_checked: 64,
            entries_found: 1,
            ram_claim: CchRamClaim::Zeroized,
        }
    );

    // Invalidate secure block
    let invalidated_sec = cache.invalidate_files(&[500]);
    assert_eq!(invalidated_sec, 1);
    assert_eq!(cache.entries_for_files(&[500]), 0);
}
