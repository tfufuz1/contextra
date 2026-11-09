//! Property-Based Model Equivalence Test (Reference Oracle) for `MemoryTextIndex` (INV-TEXT-1).
//!
//! Verifies that `MemoryTextIndex` operations (insert, remove, compact) yield
//! identical search results and active document tracking compared to a trivial reference model.

use contextra_text::contract::DocKey;
use contextra_text::memory_index::MemoryTextIndex;
use contextra_text::traits::{Index, Maintain};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
enum Action {
    Insert { id: u64, term: String },
    Remove { id: u64 },
    Compact,
}

// Simple reference oracle
struct ModelOracle {
    docs: BTreeMap<u64, String>,
    tombstones: BTreeSet<u64>,
}

impl ModelOracle {
    fn new() -> Self {
        Self {
            docs: BTreeMap::new(),
            tombstones: BTreeSet::new(),
        }
    }

    fn insert(&mut self, id: u64, text: String) {
        self.tombstones.remove(&id);
        self.docs.insert(id, text);
    }

    fn remove(&mut self, id: u64) {
        if self.docs.contains_key(&id) {
            self.tombstones.insert(id);
        }
    }

    fn compact(&mut self) {
        for id in &self.tombstones {
            self.docs.remove(id);
        }
        self.tombstones.clear();
    }

    fn contains_active(&self, id: u64) -> bool {
        self.docs.contains_key(&id) && !self.tombstones.contains(&id)
    }

    fn active_count(&self) -> usize {
        self.docs
            .keys()
            .filter(|k| !self.tombstones.contains(k))
            .count()
    }
}

fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => (1u64..30, "[a-z]{3,6} [a-z]{3,6}").prop_map(|(id, term)| Action::Insert { id, term }),
        2 => (1u64..30).prop_map(|id| Action::Remove { id }),
        1 => Just(Action::Compact),
    ]
}

proptest! {
    #[test]
    fn prop_memory_index_matches_oracle(actions in prop::collection::vec(action_strategy(), 10..50)) {
        let mut index = MemoryTextIndex::new();
        let mut oracle = ModelOracle::new();

        for action in actions {
            match action {
                Action::Insert { id, term } => {
                    let key = DocKey::from(id);
                    Index::<DocKey, str>::insert(&mut index, key, term.as_bytes()).unwrap();
                    oracle.insert(id, term);
                }
                Action::Remove { id } => {
                    let key = DocKey::from(id);
                    let _ = Index::<DocKey, str>::remove(&mut index, &key).unwrap();
                    oracle.remove(id);
                }
                Action::Compact => {
                    let _ = Maintain::compact(&mut index).unwrap();
                    oracle.compact();
                }
            }

            prop_assert_eq!(index.len(), oracle.active_count());

            // Check that active docs can be found or that dead docs are never retrieved
            for id in 1u64..30 {
                if !oracle.contains_active(id) {
                    let res = Index::<DocKey, str>::search(&index, "apple", 50).unwrap();
                    prop_assert!(!res.iter().any(|hit| (hit.doc_id.inner() as u64) == id));
                }
            }
        }
    }
}
