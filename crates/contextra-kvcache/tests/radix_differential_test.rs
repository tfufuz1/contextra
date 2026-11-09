//! Differential test for PrefixRadixTree vs. Naive Reference List Model (MISSION KV-1)

use contextra_kvcache::radix::{KvReusePolicy, PrefixRadixTree};
use contextra_types::TenantId;
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Operation {
    Insert(Vec<u32>, u64),
    Lookup(Vec<u32>),
    Remove(Vec<u32>),
}

#[derive(Default)]
struct NaiveReferenceModel {
    entries: Vec<(Vec<u32>, u64)>,
}

impl NaiveReferenceModel {
    fn insert(&mut self, tokens: Vec<u32>, block_id: u64) {
        if let Some(pos) = self.entries.iter().position(|(t, _)| t == &tokens) {
            self.entries[pos].1 = block_id;
        } else {
            self.entries.push((tokens, block_id));
        }
    }

    fn find_longest_prefix(&self, query: &[u32]) -> Option<(usize, u64)> {
        let mut best: Option<(usize, u64)> = None;
        for (tokens, block_id) in &self.entries {
            if query.starts_with(tokens) {
                let len = tokens.len();
                match best {
                    None => best = Some((len, *block_id)),
                    Some((best_len, _)) if len > best_len => {
                        best = Some((len, *block_id));
                    }
                    _ => {}
                }
            }
        }
        best
    }

    fn remove(&mut self, tokens: &[u32]) -> Option<u64> {
        if let Some(pos) = self.entries.iter().position(|(t, _)| t == tokens) {
            Some(self.entries.remove(pos).1)
        } else {
            None
        }
    }
}

prop_compose! {
    fn arb_tokens()(len in 1..=8usize, val in prop::collection::vec(1..=10u32, 1..=8)) -> Vec<u32> {
        val[..len.min(val.len())].to_vec()
    }
}

prop_compose! {
    fn arb_op()(
        op_type in 0..3,
        tokens in arb_tokens(),
        block_id in 1..=1000u64,
    ) -> Operation {
        match op_type {
            0 => Operation::Insert(tokens, block_id),
            1 => Operation::Lookup(tokens),
            _ => Operation::Remove(tokens),
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_radix_differential_oracle(ops in prop::collection::vec(arb_op(), 1..=50)) {
        let tenant = TenantId::try_new(1).unwrap();
        let mut radix = PrefixRadixTree::new(tenant);
        let mut model = NaiveReferenceModel::default();

        for op in ops {
            match op {
                Operation::Insert(tokens, block_id) => {
                    let res_tree = radix.insert(&tokens, block_id);
                    assert!(res_tree.is_ok());
                    model.insert(tokens, block_id);
                }
                Operation::Lookup(query) => {
                    let tree_match = radix.find_longest_prefix(&query, KvReusePolicy::Always);
                    let model_match = model.find_longest_prefix(&query);

                    match (tree_match, model_match) {
                        (Some(tm), Some((m_len, m_id))) => {
                            prop_assert_eq!(tm.matched_len, m_len);
                            prop_assert_eq!(tm.block_id, m_id);
                        }
                        (None, None) => {}
                        (t, m) => {
                            panic!("Mismatch: tree found {:?}, but model found {:?}", t, m);
                        }
                    }
                }
                Operation::Remove(tokens) => {
                    let tree_rem = radix.remove(&tokens);
                    let model_rem = model.remove(&tokens);
                    prop_assert_eq!(tree_rem, model_rem);
                }
            }
        }
    }
}
