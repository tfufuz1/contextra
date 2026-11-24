#![allow(
    clippy::all,
    clippy::type_complexity,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Property-basierte Tests für `PrefixRadixTree` und Mandantenisolation (INV-KV-1).

use std::collections::BTreeMap;
use contextra_kvcache::radix::{KvReusePolicy, PrefixRadixTree};
use contextra_kvcache::types::TenantId;
use proptest::prelude::*;

fn naive_longest_prefix_match(
    map: &BTreeMap<Vec<u32>, u64>,
    query: &[u32],
) -> Option<(usize, u64)> {
    let mut best: Option<(usize, u64)> = None;
    for (k, &block_id) in map {
        if query.len() >= k.len() && &query[..k.len()] == k.as_slice() {
            match best {
                None => best = Some((k.len(), block_id)),
                Some((len, _)) if k.len() > len => best = Some((k.len(), block_id)),
                _ => {}
            }
        }
    }
    best
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_radix_matches_naive_oracle(
        prefixes in prop::collection::vec(
            (prop::collection::vec(1u32..20u32, 1..8), 100u64..1000u64),
            1..15
        ),
        query in prop::collection::vec(1u32..20u32, 1..10)
    ) {
        let tenant = TenantId::try_new(100).unwrap();
        let mut tree = PrefixRadixTree::new(tenant);
        let mut naive_map: BTreeMap<Vec<u32>, u64> = BTreeMap::new();

        for (seq, id) in &prefixes {
            tree.insert(seq, *id).unwrap();
            naive_map.insert(seq.clone(), *id);
        }

        let naive_match = naive_longest_prefix_match(&naive_map, &query);
        let tree_match = tree.find_longest_prefix(&query, KvReusePolicy::Always);

        match (naive_match, tree_match) {
            (None, None) => {},
            (Some((len, _)), Some(pm)) => {
                prop_assert_eq!(len, pm.matched_len, "Matched prefix length must match oracle");
            },
            (Some(n), None) => {
                prop_assert!(false, "Oracle found match {:?} but tree found None for query {:?}", n, query);
            },
            (None, Some(t)) => {
                prop_assert!(false, "Oracle found None but tree found match {:?} for query {:?}", t, query);
            }
        }
    }

    #[test]
    fn prop_cross_tenant_isolation_never_leaks(
        seqs in prop::collection::vec(prop::collection::vec(1u32..30u32, 2..10), 1..10),
        tenant_a_id in 1u64..5000u64,
        tenant_b_id in 5001u64..10000u64
    ) {
        let tenant_a = TenantId::try_new(tenant_a_id).unwrap();
        let tenant_b = TenantId::try_new(tenant_b_id).unwrap();

        let mut tree_a = PrefixRadixTree::new(tenant_a);
        let tree_b = PrefixRadixTree::new(tenant_b);

        for (idx, seq) in seqs.iter().enumerate() {
            tree_a.insert(seq, idx as u64 + 1).unwrap();
        }

        // Tree B is empty for tenant_b; searching seqs in tree_b must never return a hit from tree_a
        for seq in &seqs {
            let res = tree_b.find_longest_prefix(seq, KvReusePolicy::Always);
            prop_assert!(res.is_none(), "Tenant B tree must never match prefixes inserted into Tenant A tree");
        }
    }
}
