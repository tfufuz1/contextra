//! Integration tests for Engine features (Idem, Cursor, Page, Cache, Packgroups, Extension, Feed).

use contextra_storage::engine::cache::ReadCache;
use contextra_storage::engine::cursor::{Cursor, ResumeToken};
use contextra_storage::engine::extension::{filter_visible, ExtensionBlock};
use contextra_storage::engine::feed::ConsumerFeed;
use contextra_storage::engine::idem::{IdemAction, IdemRegistry};
use contextra_storage::engine::packgroups::{split_pack_expired, PackGroup, PackKey, PackPolicy};
use contextra_storage::engine::page::{PageBudget, PageCollector, PageEntry};
use contextra_storage::erase::gate::GateIndex;
use contextra_storage_ports::{BranchId, ClusterId, Error, IdemToken, InstanceId, Protection};

#[test]
fn test_idem_registry_lifecycle_and_conflict() {
    let mut reg = IdemRegistry::new(10);
    let token = IdemToken::from_bytes([1u8; 16]);
    let digest_a = [0xaa; 32];
    let digest_b = [0xbb; 32];

    // 1. Initial reservation
    let action = reg.check_or_reserve(token, digest_a, 100, 1000).unwrap();
    assert_eq!(action, IdemAction::New);

    // 2. Conflict during pending
    let err_conflict = reg.check_or_reserve(token, digest_b, 105, 1000);
    assert!(matches!(err_conflict, Err(Error::IdemConflict)));

    // 3. Commit token
    reg.commit(token, 42, true).unwrap();

    // 4. Repeated query with same digest returns cached result
    let cached = reg.check_or_reserve(token, digest_a, 110, 1000).unwrap();
    assert_eq!(cached, IdemAction::Cached { stamp: 42, durable: true });

    // 5. Query with different digest returns IdemConflict
    let err_diff = reg.check_or_reserve(token, digest_b, 115, 1000);
    assert!(matches!(err_diff, Err(Error::IdemConflict)));

    // 6. Trim past expiration
    reg.trim(50, 2000);
    assert_eq!(reg.len(), 0);
}

#[test]
fn test_cursor_lease_and_resume_token() {
    let c = ClusterId::from_wire([2u8; 16]);
    let b = BranchId(0);
    let mut cursor = Cursor::new(c, b, 100, 500);

    let gate_index = GateIndex::new();
    assert!(cursor.check_validity(400, &gate_index).is_ok());

    // Past lease deadline -> BelowFloor
    assert!(matches!(cursor.check_validity(600, &gate_index), Err(Error::BelowFloor)));

    // Step and ResumeToken
    cursor.step(b"key_alpha".to_vec());
    let token = cursor.resume_token().unwrap();
    let bytes = token.encode();
    let decoded = ResumeToken::decode(&bytes).unwrap();
    assert_eq!(decoded.cluster, c);
    assert_eq!(decoded.branch, b);
    assert_eq!(decoded.last_key, b"key_alpha");
    assert_eq!(decoded.view_stamp, 100);
}

#[test]
fn test_page_collector_budgeting() {
    let budget = PageBudget {
        max_items: 2,
        max_weight: 100,
        max_bytes: 1000,
    };
    let mut collector = PageCollector::new(budget);

    let e1 = PageEntry {
        key: b"k1".to_vec(),
        header: b"h1".to_vec(),
        body: Some(b"body1".to_vec()),
        weight: 40,
        stamp: 10,
    };
    let e2 = PageEntry {
        key: b"k2".to_vec(),
        header: b"h2".to_vec(),
        body: Some(b"body2".to_vec()),
        weight: 50,
        stamp: 11,
    };
    let e3 = PageEntry {
        key: b"k3".to_vec(),
        header: b"h3".to_vec(),
        body: None,
        weight: 20,
        stamp: 12,
    };

    assert!(collector.try_push(e1).unwrap());
    assert!(collector.try_push(e2).unwrap());
    // Exceeds max_items (2)
    assert!(!collector.try_push(e3).unwrap());

    let (entries, next) = collector.finish(Some(b"k3".to_vec()));
    assert_eq!(entries.len(), 2);
    assert_eq!(next, Some(b"k3".to_vec()));
}

#[test]
fn test_read_cache_cluster_invalidation() {
    let mut cache = ReadCache::new(4096);
    let c1 = ClusterId::from_wire([3u8; 16]);
    let c2 = ClusterId::from_wire([4u8; 16]);
    let b = BranchId(0);

    cache.put(c1, b, b"user_1".to_vec(), b"val_1".to_vec(), 10);
    cache.put(c2, b, b"user_2".to_vec(), b"val_2".to_vec(), 20);

    assert_eq!(cache.get(c1, b, b"user_1", 15), Some(b"val_1".to_vec()));
    // View stamp lower than record stamp
    assert_eq!(cache.get(c1, b, b"user_1", 5), None);

    // Invalidate c1
    cache.invalidate_cluster(c1);
    assert_eq!(cache.get(c1, b, b"user_1", 15), None);
    assert_eq!(cache.get(c2, b, b"user_2", 25), Some(b"val_2".to_vec()));
}

#[test]
fn test_packgroups_and_feed_visibility() {
    let pk = PackKey([7u8; 16]);
    let mut pg = PackGroup::new(pk, PackPolicy::Packed, Protection::Plain);
    let clusters: Vec<ClusterId> = (0..10).map(|i| ClusterId::from_wire([i; 16])).collect();
    for c in &clusters {
        pg.add_member(*c).unwrap();
    }
    assert_eq!(pg.len(), 10);

    let batches = split_pack_expired(&clusters, 4);
    assert_eq!(batches.len(), 3);
    assert_eq!(batches[0].len(), 4);
    assert_eq!(batches[1].len(), 4);
    assert_eq!(batches[2].len(), 2);

    let ext = ExtensionBlock {
        owner_tag: 0x1234,
        block_index: 0,
        payload: b"extension_data".to_vec(),
    };
    let encoded = ext.encode();
    let decoded = ExtensionBlock::decode(&encoded).unwrap();
    assert_eq!(decoded.owner_tag, 0x1234);
    assert_eq!(decoded.payload, b"extension_data");

    let gate_index = GateIndex::new();
    assert!(filter_visible(clusters[0], 100, &gate_index).is_ok());

    let inst = InstanceId::from_wire([9u8; 16]);
    let mut feed = ConsumerFeed::new(inst, 50);
    assert!(feed.advance_watermark(80, 100).is_ok());
    // Exceeds durable stamp
    assert!(feed.advance_watermark(120, 100).is_err());
}
