#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use crate::error::{ContextraError, Result};

use crate::types::domain::*;

use proptest::prop_assert;

#[test]
fn test_tenant_id_defaults_and_constants() {
    let default_tenant = TenantId::default();
    assert_eq!(default_tenant, TenantId::SYSTEM);
    assert_eq!(default_tenant.inner(), 0);
    assert_eq!(default_tenant.as_u64(), 0);
    assert_eq!(TenantId::try_new(42).unwrap().inner(), 42);
    assert_eq!(TenantId::try_new(42).unwrap().as_u64(), 42);
    assert_eq!(
        TenantId::try_from(100u64).unwrap(),
        TenantId::try_new(100).unwrap()
    );
    assert_eq!(format!("{default_tenant}"), "TenantId(0)");
}

#[test]
fn test_tenant_id_collections_hashmap_btreemap() {
    use std::collections::{BTreeMap, HashMap};

    let t0 = TenantId::SYSTEM;
    let t1 = TenantId::try_new(1).unwrap();
    let t2 = TenantId::try_new(2).unwrap();

    // HashMap key test
    let mut map = HashMap::new();
    map.insert(t0, "default_tenant");
    map.insert(t1, "tenant_one");
    assert_eq!(map.get(&TenantId::default()), Some(&"default_tenant"));
    assert_eq!(map.get(&t1), Some(&"tenant_one"));
    assert_eq!(map.get(&t2), None);

    // BTreeMap key test (testing Ord / PartialOrd)
    let mut bmap = BTreeMap::new();
    bmap.insert(t2, "tenant_two");
    bmap.insert(t0, "tenant_zero");
    bmap.insert(t1, "tenant_one");

    let keys: Vec<TenantId> = bmap.keys().copied().collect();
    assert_eq!(keys, vec![t0, t1, t2]);
}

#[test]
fn test_tenant_id_serde_roundtrip() {
    let tenant = TenantId::try_new(987654321).unwrap();
    let serialized = serde_json::to_string(&tenant).expect("TenantId serialization failed");
    assert_eq!(serialized, "987654321");

    let deserialized: TenantId =
        serde_json::from_str(&serialized).expect("TenantId deserialization failed");
    assert_eq!(tenant, deserialized);
}

#[test]
fn test_doc_id_from_key_no_panic() {
    assert!(DocId::from_key("").is_err());
    let keys = vec![
        "a",
        "short",
        "very_long_key_that_exceeds_blake3_block_size_maybe_not_really_but_long",
    ];
    for key in keys {
        let res = DocId::from_key(key);
        assert!(res.is_ok(), "DocId::from_key failed for key: {}", key);
    }
}

#[test]
fn test_doc_id_determinism() {
    let key = "consistent_key";
    let id1 = DocId::from_key(key).unwrap(); // unwrap
    let id2 = DocId::from_key(key).unwrap(); // unwrap
    assert_eq!(id1, id2);
}

#[test]
fn test_tenant_and_collection_id() {
    let tenant = TenantId::try_new(1).unwrap();
    assert_eq!(tenant.inner(), 1);
    assert_eq!(tenant.to_string(), "TenantId(1)");
    assert!(TenantId::try_new(0).is_err());

    let collection = CollectionId::try_new(42).unwrap();
    assert_eq!(collection.inner(), 42);
    assert_eq!(collection.to_string(), "CollectionId(42)");
    assert!(CollectionId::try_new(0).is_err());
}

#[test]
fn test_doc_id_multibyte_unicode_keys() {
    let unicode_keys = vec![
        "🦀_crab_key",
        "äöü_german_key",
        "日本語_japanese_key",
        "🚀✨🔥",
    ];
    for key in unicode_keys {
        let doc_id = DocId::from_key(key).expect("multibyte unicode key should derive doc_id"); // expect #[cfg(test)]
        assert!(doc_id.inner() > 0);
        let entity_id =
            EntityId::from_key(key).expect("multibyte unicode key should derive entity_id"); // expect #[cfg(test)]
        assert_eq!(entity_id, EntityId::from_doc_id(doc_id));
    }
}

#[test]
fn test_entity_id_methods() {
    let entity_id = EntityId::new(12345);
    assert_eq!(entity_id.inner(), 12345);
    assert_eq!(entity_id.as_bytes(), b"12345");

    let doc_id = DocId::new(998877);
    let derived_entity = EntityId::from_doc_id(doc_id);
    assert_eq!(derived_entity.inner(), 998877);

    // String / &str conversions
    let parsed_num: EntityId = "12345".into();
    assert_eq!(parsed_num.inner(), 12345);

    let hashed_str: EntityId = "not_a_number".into();
    assert!(hashed_str.inner() > 0);

    let from_string: EntityId = String::from("9999").into();
    assert_eq!(from_string.inner(), 9999);
}

#[test]
fn doc_id_valid_key_is_ok() {
    let id = DocId::from_key("valid-key-123").unwrap(); // unwrap
    assert!(id.inner() > 0);
}

#[test]
fn doc_id_from_empty_returns_err() {
    assert!(DocId::from_key("").is_err());
}

#[test]
fn test_entity_id_from_key_empty_err() {
    assert!(EntityId::from_key("").is_err());
    assert!(EntityId::from_key("node_1").is_ok());
}

#[test]
fn tx_id_ordering_is_consistent() {
    let t1 = TxId::new(1);
    let t2 = TxId::new(2);
    assert!(t1 < t2);
    assert!(t2 > t1);
    assert_eq!(t1, TxId::new(1));
}

#[test]
fn test_tx_id_internal() {
    let tx = TxId::internal();
    assert_eq!(tx.inner(), TxId::INTERNAL_BASE);
    assert!(tx.to_string().contains("TxId"));

    let invalid = TxId::INVALID;
    assert_eq!(invalid.inner(), 0);
    assert!(invalid < tx);
    assert_eq!(invalid, TxId::new(0));
}

#[test]
fn test_tx_id_is_valid_origin() {
    // Collection-sequenced range [0, 10^12]
    assert!(TxId::new(0).is_valid_origin());
    assert!(TxId::new(1).is_valid_origin());
    assert!(TxId::new(1_000_000).is_valid_origin());
    assert!(TxId::new(TxId::MAX_COLLECTION_SEQUENCE).is_valid_origin());

    // Internal system range [INTERNAL_BASE, u64::MAX]
    assert!(TxId::new(TxId::INTERNAL_BASE).is_valid_origin());
    assert!(TxId::new(TxId::INTERNAL_BASE + 500).is_valid_origin());
    assert!(TxId::new(u64::MAX).is_valid_origin());

    // Wall-clock-derived or unmanaged gap range (10^12 < tx < INTERNAL_BASE)
    assert!(!TxId::new(TxId::MAX_COLLECTION_SEQUENCE + 1).is_valid_origin());
    assert!(!TxId::new(1_700_000_000_000_000_000).is_valid_origin());
    assert!(!TxId::new(TxId::INTERNAL_BASE - 1).is_valid_origin());
}

#[test]
fn test_tx_id_range_boundary_exhaustion_simulation() {
    use std::sync::atomic::{AtomicU64, Ordering};

    // 1. Invariant assertions: strict gap between collection sequence and internal system range
    const {
        assert!(
            TxId::INTERNAL_BASE > TxId::MAX_COLLECTION_SEQUENCE,
            "INTERNAL_BASE must strictly exceed MAX_COLLECTION_SEQUENCE"
        );
    }
    let gap_size = TxId::INTERNAL_BASE - TxId::MAX_COLLECTION_SEQUENCE;
    assert!(
            gap_size > 1_000_000_000_000_000,
            "Gap between collection range and internal range must be large enough to catch unmanaged TxIds"
        );

    // 2. Exact boundary origin checks
    let boundary_collection_max = TxId::new(TxId::MAX_COLLECTION_SEQUENCE);
    let boundary_gap_start = TxId::new(TxId::MAX_COLLECTION_SEQUENCE + 1);
    let boundary_gap_end = TxId::new(TxId::INTERNAL_BASE - 1);
    let boundary_internal_base = TxId::new(TxId::INTERNAL_BASE);
    let boundary_u64_max = TxId::new(u64::MAX);

    assert!(
        boundary_collection_max.is_valid_origin(),
        "MAX_COLLECTION_SEQUENCE must be a valid origin"
    );
    assert!(
        !boundary_gap_start.is_valid_origin(),
        "MAX_COLLECTION_SEQUENCE + 1 must fall in invalid gap"
    );
    assert!(
        !boundary_gap_end.is_valid_origin(),
        "INTERNAL_BASE - 1 must fall in invalid gap"
    );
    assert!(
        boundary_internal_base.is_valid_origin(),
        "INTERNAL_BASE must be a valid origin"
    );
    assert!(
        boundary_u64_max.is_valid_origin(),
        "u64::MAX must be a valid origin"
    );

    // 3. Exhaustion simulation via test hook (AtomicU64 counter positioned near MAX_COLLECTION_SEQUENCE boundary)
    let simulated_next_tx = AtomicU64::new(TxId::MAX_COLLECTION_SEQUENCE - 2);

    // Helper allocation function matching Collection::allocate_tx logic
    let allocate_simulated = |counter: &AtomicU64| -> Result<TxId> {
        let id = counter.fetch_add(1, Ordering::SeqCst);
        if id > TxId::MAX_COLLECTION_SEQUENCE {
            return Err(ContextraError::Transaction(
                    "TxId counter exhausted: MAX_COLLECTION_SEQUENCE range exceeded. Collection must be recreated.".into(),
                ));
        }
        Ok(TxId::new(id))
    };

    // Tx #1: MAX_COLLECTION_SEQUENCE - 2 (Valid)
    let tx1 = allocate_simulated(&simulated_next_tx).expect("Allocation at MAX - 2 should succeed"); // expect
    assert_eq!(tx1.inner(), TxId::MAX_COLLECTION_SEQUENCE - 2);
    assert!(tx1.is_valid_origin());

    // Tx #2: MAX_COLLECTION_SEQUENCE - 1 (Valid)
    let tx2 = allocate_simulated(&simulated_next_tx).expect("Allocation at MAX - 1 should succeed"); // expect
    assert_eq!(tx2.inner(), TxId::MAX_COLLECTION_SEQUENCE - 1);
    assert!(tx2.is_valid_origin());

    // Tx #3: MAX_COLLECTION_SEQUENCE (Exact upper boundary - Valid)
    let tx3 = allocate_simulated(&simulated_next_tx).expect("Allocation at MAX should succeed"); // expect
    assert_eq!(tx3.inner(), TxId::MAX_COLLECTION_SEQUENCE);
    assert!(tx3.is_valid_origin());

    // Tx #4: Attempt allocation at MAX_COLLECTION_SEQUENCE + 1 (Boundary breach -> Controlled Error)
    let err = allocate_simulated(&simulated_next_tx)
        .expect_err("Allocation beyond MAX_COLLECTION_SEQUENCE must return error");
    assert!(
        matches!(err, ContextraError::Transaction(ref msg) if msg.contains("MAX_COLLECTION_SEQUENCE range exceeded")),
        "Expected controlled ContextraError::Transaction on counter exhaustion, got: {:?}",
        err
    );

    // Verify counter position did not cause collision with INTERNAL_BASE
    let current_counter = simulated_next_tx.load(Ordering::SeqCst);
    assert!(
        current_counter < TxId::INTERNAL_BASE,
        "Counter increment must not silently collide with TxId::INTERNAL_BASE"
    );
}

#[test]
fn test_doc_id_from_key_collisions_and_distribution() {
    use std::collections::HashSet;

    const KEY_COUNT: usize = 100_000;
    let mut seen = HashSet::with_capacity(KEY_COUNT);

    for i in 0..KEY_COUNT {
        let key = format!("doc_key_test_sample_{i}");
        let doc_id = DocId::from_key(&key).expect("DocId::from_key failed"); // expect
        assert!(
            seen.insert(doc_id.inner()),
            "Collision detected for DocId at key {key} (index {i})"
        );
    }

    assert_eq!(seen.len(), KEY_COUNT);
}

#[test]
fn test_doc_id_and_entity_id_unicode_keys() {
    let unicode_keys = vec![
        "Gedächtnis_01",
        "記憶_メモリ_99",
        "🧠_cognitive_memory_node",
        "Crème_brûlée_recipe",
    ];

    for key in unicode_keys {
        let doc_id1 = DocId::from_key(key).expect("DocId from unicode key"); // expect
        let doc_id2 = DocId::from_key(key).expect("DocId from unicode key"); // expect
        assert_eq!(doc_id1, doc_id2);
        assert_ne!(doc_id1.inner(), 0);

        let ent_id1 = EntityId::from_key(key).expect("EntityId from unicode key"); // expect
        let ent_id2 = EntityId::from_key(key).expect("EntityId from unicode key"); // expect
        assert_eq!(ent_id1, ent_id2);
        assert_ne!(ent_id1.inner(), 0);
    }
}

#[test]
fn test_tx_id_invalid_sentinel_and_conversion_boundary() {
    assert_eq!(TxId::INVALID.inner(), 0);
    assert!(TxId::INVALID.is_valid_origin());
    assert_eq!(format!("{}", TxId::INVALID), "TxId(0)");

    let doc_id = DocId::new(42);
    let entity_id = EntityId::from_doc_id(doc_id);
    assert_eq!(entity_id.inner(), 42);
    assert_eq!(entity_id.as_bytes(), b"42");
}

#[test]
fn test_tx_id_ranges_and_internal_boundary_checks() {
    let valid_col_tx = TxId::new(500_000);
    assert!(valid_col_tx.is_valid_origin());

    let valid_internal_tx = TxId::internal();
    assert!(valid_internal_tx.is_valid_origin());
    assert_eq!(valid_internal_tx.inner(), TxId::INTERNAL_BASE);

    // Wall-clock derived TxId in gap should fail is_valid_origin()
    let wall_clock_gap_tx = TxId::new(1_700_000_000_000_000_000);
    assert!(!wall_clock_gap_tx.is_valid_origin());
}

#[test]
fn test_tx_id_system_range_wraparound_safety() {
    // Valid offsets within [0, 1_000_000]
    let tx0 = TxId::try_from_internal_offset(0).expect("Offset 0 should succeed"); // expect
    assert_eq!(tx0.inner(), TxId::INTERNAL_BASE);
    assert!(tx0.is_valid_origin());

    let tx_mid = TxId::try_from_internal_offset(500_000).expect("Offset 500k should succeed"); // expect
    assert_eq!(tx_mid.inner(), TxId::INTERNAL_BASE + 500_000);
    assert!(tx_mid.is_valid_origin());

    let max_offset = u64::MAX - TxId::INTERNAL_BASE;
    assert_eq!(max_offset, 1_000_000);
    let tx_max =
        TxId::try_from_internal_offset(max_offset).expect("Max valid offset should succeed"); // expect
    assert_eq!(tx_max.inner(), u64::MAX);
    assert!(tx_max.is_valid_origin());

    // Overflow attempt (offset > 1_000_000)
    let err_overflow = TxId::try_from_internal_offset(max_offset + 1);
    assert!(
        matches!(err_overflow, Err(ContextraError::Transaction(ref msg)) if msg.contains("overflows u64::MAX")),
        "Expected controlled error on offset overflow, got: {:?}",
        err_overflow
    );

    let err_huge = TxId::try_from_internal_offset(u64::MAX);
    assert!(
        matches!(err_huge, Err(ContextraError::Transaction(ref msg)) if msg.contains("overflows u64::MAX")),
        "Expected controlled error on u64::MAX offset, got: {:?}",
        err_huge
    );

    // Prove system allocations NEVER land in collection sequence range [1, MAX_COLLECTION_SEQUENCE]
    // regardless of offset choice
    for offset in [0, 1, 100, 500_000, 1_000_000] {
        let tx = TxId::try_from_internal_offset(offset).unwrap(); // unwrap
        assert!(
            tx.inner() > TxId::MAX_COLLECTION_SEQUENCE,
            "Internal allocation must be strictly above MAX_COLLECTION_SEQUENCE"
        );
        assert!(
            tx.inner() >= TxId::INTERNAL_BASE,
            "Internal allocation must be >= INTERNAL_BASE"
        );
    }
}

proptest::proptest! {
    #[test]
    fn prop_tx_id_range_isolation(offset in 0u64..=1_000_000u64) {
        let tx = TxId::try_from_internal_offset(offset).unwrap(); // unwrap
        prop_assert!(tx.is_valid_origin());
        prop_assert!(tx.inner() >= TxId::INTERNAL_BASE);
        prop_assert!(tx.inner() > TxId::MAX_COLLECTION_SEQUENCE);
    }

    #[test]
    fn prop_tx_id_overflow_isolation(offset in 1_000_001u64..=u64::MAX) {
        let res = TxId::try_from_internal_offset(offset);
        prop_assert!(res.is_err());
    }
}

#[test]
fn test_tenant_id_system_reserved() {
    assert!(TenantId::try_new(0).is_err());
}

#[test]
fn test_tenant_id_valid() {
    let t = TenantId::try_new(42).expect("valid tenant_id");
    assert_eq!(t.inner(), 42);
    assert_eq!(t.as_u64(), 42);
    assert!(!t.is_system());
}

#[test]
fn test_tenant_id_system_constant() {
    assert_eq!(TenantId::SYSTEM.inner(), 0);
    assert!(TenantId::SYSTEM.is_system());
}

#[test]
fn test_tenant_id_hardening() {
    assert!(TenantId::try_new(0).is_err());
    assert_eq!(TenantId::SYSTEM.inner(), 0);
}

#[test]
fn test_tenant_id_try_from_boundary_and_display() {
    assert!(TenantId::try_from(0u64).is_err());
    let t = match TenantId::try_from(u64::MAX) {
        Ok(val) => val,
        Err(e) => panic!("u64::MAX is valid tenant id: {e:?}"),
    };
    assert_eq!(t.inner(), u64::MAX);
    assert_eq!(t.as_u64(), u64::MAX);
    assert_eq!(format!("{t}"), format!("TenantId({})", u64::MAX));
}
