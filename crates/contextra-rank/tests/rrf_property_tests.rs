//! Property-Tests für contextra-rank.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::disallowed_types
)]

use contextra_rank::{rrf_fuse, unseen_bound, Candidate, DocHandle, RankError};
use proptest::prelude::*;
use std::collections::BTreeSet;

fn arb_doc_handle() -> impl Strategy<Value = DocHandle> {
    (any::<[u8; 16]>(), any::<[u8; 16]>()).prop_map(|(cluster_tag, doc_tag)| DocHandle {
        cluster_tag,
        doc_tag,
    })
}

fn arb_candidate_list(max_len: usize) -> impl Strategy<Value = Vec<Candidate>> {
    prop::collection::vec(arb_doc_handle(), 0..=max_len).prop_map(|handles| {
        let mut seen = BTreeSet::new();
        let mut list = Vec::new();
        for h in handles {
            if seen.insert(h) {
                list.push(Candidate { handle: h });
            }
        }
        list
    })
}

proptest! {
    /// SPEC: T-RANK-1 (Permutationsinvarianz)
    /// Beliebige Vertauschung von Listen liefert bitgleiches Ergebnis,
    /// einschließlich leerer und teilweiser Listen.
    #[test]
    fn prop_rrf_permutation_invariance(
        l1 in arb_candidate_list(30),
        l2 in arb_candidate_list(30),
        l3 in arb_candidate_list(30),
        l4 in arb_candidate_list(30),
        top in 0usize..100
    ) {
        let lists_1: [&[Candidate]; 4] = [&l1, &l2, &l3, &l4];
        let lists_2: [&[Candidate]; 4] = [&l4, &l2, &l1, &l3];
        let lists_3: [&[Candidate]; 4] = [&l3, &l1, &l4, &l2];

        let res1 = rrf_fuse(&lists_1, top).unwrap();
        let res2 = rrf_fuse(&lists_2, top).unwrap();
        let res3 = rrf_fuse(&lists_3, top).unwrap();

        prop_assert_eq!(&res1, &res2);
        prop_assert_eq!(&res1, &res3);
    }

    /// SPEC: T-RANK-1 (Duplikatinvarianz und Fehlerprüfung)
    /// Listen mit internen Duplikaten werden immer mit `RankError::Duplicate` abgelehnt.
    #[test]
    fn prop_rrf_duplicate_rejection(
        list in arb_candidate_list(20),
        dup_handle in arb_doc_handle(),
    ) {
        // Liste mit mindestens einem Duplikat erstellen
        let mut dup_list = list;
        dup_list.push(Candidate { handle: dup_handle });
        dup_list.push(Candidate { handle: dup_handle });

        let res = rrf_fuse(&[&dup_list], 10);
        prop_assert_eq!(res, Err(RankError::Duplicate));
    }

    /// SPEC: T-RANK-1 (Monotonie)
    /// Besserer Rang (kleinerer Index) führt zu strikt höherem oder gleichem Score.
    #[test]
    fn prop_rrf_monotonicity(
        l1 in arb_candidate_list(50),
        top in 1usize..100
    ) {
        let lists: [&[Candidate]; 1] = [&l1];
        let res = rrf_fuse(&lists, top).unwrap();
        for i in 1..res.len() {
            let prev = res.get(i - 1).map(|f| f.score);
            let curr = res.get(i).map(|f| f.score);
            prop_assert!(prev >= curr, "Scores must be monotonically decreasing");
        }
    }

    /// SPEC: T-HYB-1 (unseen_bound Property)
    /// Für jedes Dokument, das in allen Listen erst nach depths[i] auftritt,
    /// darf der tatsächliche Score die unseen_bound nicht überschreiten.
    #[test]
    fn prop_rrf_unseen_bound(
        prefix1 in arb_candidate_list(20),
        prefix2 in arb_candidate_list(20),
        suffix1 in arb_candidate_list(20),
        suffix2 in arb_candidate_list(20),
    ) {
        // Build full list = prefix + suffix (ensuring no internal duplicates)
        let mut full1 = prefix1.clone();
        let mut set1: BTreeSet<DocHandle> = prefix1.iter().map(|c| c.handle).collect();
        for c in suffix1 {
            if set1.insert(c.handle) {
                full1.push(c);
            }
        }

        let mut full2 = prefix2.clone();
        let mut set2: BTreeSet<DocHandle> = prefix2.iter().map(|c| c.handle).collect();
        for c in suffix2 {
            if set2.insert(c.handle) {
                full2.push(c);
            }
        }

        let depths = [prefix1.len(), prefix2.len()];
        let bound = unseen_bound(&depths).unwrap();

        let full_res = rrf_fuse(&[&full1, &full2], 1000).unwrap();

        // Any candidate that was NOT in prefix1 and NOT in prefix2 must have score <= bound
        for item in full_res {
            let in_p1 = prefix1.iter().any(|c| c.handle == item.handle);
            let in_p2 = prefix2.iter().any(|c| c.handle == item.handle);
            if !in_p1 && !in_p2 {
                prop_assert!(
                    item.score <= bound,
                    "Score {:?} exceeded unseen_bound {:?}",
                    item.score,
                    bound
                );
            }
        }
    }
}
