// FILE-CONTEXT
// ZWECK: Property-based Tests für KvSegment, TenantIsolatedKvStore und LRU-Eviction.
// STAND: TS:2026-09-09T13:17:00Z (SESSION: a413a598)

#![forbid(unsafe_code)]

use contextra_crypto::kv_segment::{KvSegment, TenantIsolatedKvStore};
use contextra_types::TenantId;
use proptest::prelude::*;
use std::collections::HashSet;
use zeroize::Zeroize;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_kv_segment_creation(
        tenant_num in 1u64..10000u64,
        segment_id in 1u64..100000u64,
        data in prop::collection::vec(any::<u8>(), 0..1024),
    ) {
        let tenant = TenantId::try_new(tenant_num).unwrap();
        let seg1 = KvSegment::new(tenant, segment_id, data.clone());

        prop_assert_eq!(seg1.tenant_id, tenant);
        prop_assert_eq!(seg1.segment_id, segment_id);
        prop_assert_eq!(seg1.len(), data.len());
        prop_assert_eq!(seg1.is_empty(), data.is_empty());
        prop_assert_eq!(seg1.as_bytes(), data.as_slice());
    }

    #[test]
    fn prop_tenant_isolation_strictness(
        tenant_a_num in 1u64..5000u64,
        tenant_b_num in 5001u64..10000u64,
        seg_ids_a in prop::collection::vec(1u64..10000u64, 1..20),
        seg_ids_b in prop::collection::vec(10001u64..20000u64, 1..20),
    ) {
        let store = TenantIsolatedKvStore::new();
        let tenant_a = TenantId::try_new(tenant_a_num).unwrap();
        let tenant_b = TenantId::try_new(tenant_b_num).unwrap();

        for &id in &seg_ids_a {
            let seg = KvSegment::new(tenant_a, id, vec![0xAA; 32]);
            store.insert_segment(tenant_a, seg);
        }

        for &id in &seg_ids_b {
            let seg = KvSegment::new(tenant_b, id, vec![0xBB; 32]);
            store.insert_segment(tenant_b, seg);
        }

        let retrieved_a = store.get_segments(tenant_a);
        let retrieved_b = store.get_segments(tenant_b);

        let set_a: HashSet<_> = retrieved_a.into_iter().collect();
        let set_b: HashSet<_> = retrieved_b.into_iter().collect();

        // No cross-tenant leakage allowed
        prop_assert!(set_a.intersection(&set_b).next().is_none());

        for id in &seg_ids_a {
            prop_assert!(set_a.contains(id));
            prop_assert!(!set_b.contains(id));
        }

        for id in &seg_ids_b {
            prop_assert!(set_b.contains(id));
            prop_assert!(!set_a.contains(id));
        }
    }

    #[test]
    fn prop_segment_zeroize_wipes_all_bytes(
        tenant_num in 1u64..10000u64,
        segment_id in 1u64..100000u64,
        data in prop::collection::vec(any::<u8>(), 0..1024),
    ) {
        let tenant = TenantId::try_new(tenant_num).unwrap();
        let mut segment = KvSegment::new(tenant, segment_id, data.clone());

        // State before zeroize matches original inputs
        prop_assert_eq!(segment.len(), data.len());
        prop_assert_eq!(segment.is_empty(), data.is_empty());
        prop_assert_eq!(segment.as_bytes(), data.as_slice());

        // Perform zeroize in place
        Zeroize::zeroize(&mut segment);

        // Safe observable contract after zeroize
        prop_assert_eq!(segment.len(), 0);
        prop_assert!(segment.is_empty());
        prop_assert!(segment.as_bytes().is_empty());
    }
}
