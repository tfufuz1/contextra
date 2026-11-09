//! Property Test for VETO-F10: Partition isolation across distinct namespaces

use bytes::Bytes;
use contextra_kvcache::prefix_store::NamespacePrefixStore;
use contextra_kvcache::radix::KvReusePolicy;
use contextra_ports::kv::KvBlock;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_veto_f10_partition_isolation_property(
        tokens in prop::collection::vec(1..=100u32, 4..=16),
        ns1 in 1..=5000u64,
        ns2 in 5001..=10000u64,
    ) {
        let store = NamespacePrefixStore::new(10 * 1024 * 1024, 1000).with_reuse_policy(KvReusePolicy::Always);

        let data1 = vec![0x11u8; 64];
        let blk1 = KvBlock { block_id: 101, data: Bytes::from(data1) };

        store.insert(ns1, &tokens, vec![blk1]).unwrap();

        let hit1 = store.lookup(ns1, &tokens);
        prop_assert!(hit1.is_some(), "Lookup in owning namespace must succeed");
        prop_assert_eq!(hit1.unwrap().blocks[0].block_id, 101);

        let hit2 = store.lookup(ns2, &tokens);
        prop_assert!(hit2.is_none(), "Cross-namespace lookup MUST be completely isolated (VETO-F10)");

        prop_assert_eq!(store.entry_count(ns1), 1);
        prop_assert_eq!(store.entry_count(ns2), 0);
        prop_assert_eq!(store.total_bytes(ns1), 64);
        prop_assert_eq!(store.total_bytes(ns2), 0);
    }
}
