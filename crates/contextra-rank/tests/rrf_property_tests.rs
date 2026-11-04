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
    prop::collection::vec((arb_doc_handle(), 0u32..5), 0..=max_len).prop_map(|entries| {
        let mut seen = BTreeSet::new();
        let mut list = Vec::new();
        for (h, chunk) in entries {
            if seen.insert((h, chunk)) {
                list.push(Candidate { handle: h, chunk });
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
        dup_chunk in 0u32..5,
    ) {
        // Liste mit mindestens einem Duplikat (gleicher handle und gleicher chunk) erstellen
        let mut dup_list = list;
        dup_list.push(Candidate { handle: dup_handle, chunk: dup_chunk });
        dup_list.push(Candidate { handle: dup_handle, chunk: dup_chunk });

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

    /// SPEC: T-RANK-1 (Asymmetrische / einseitige Kandidatenlisten)
    /// Eine Liste enthält Kandidaten, andere Teillisten sind leer oder disjunkt;
    /// keine Panics, deterministische Ausgabegröße und korrekte Trefferanzahl.
    #[test]
    fn prop_rrf_asymmetric_and_empty_lists(
        full_list in arb_candidate_list(40),
        disjoint_entries in prop::collection::vec((arb_doc_handle(), 0u32..5), 1..=20),
        top in 1usize..100
    ) {
        let empty_list: [Candidate; 0] = [];
        // Erzeuge disjunkte zweite Liste
        let full_set: BTreeSet<(DocHandle, u32)> = full_list.iter().map(|c| (c.handle, c.chunk)).collect();
        let disjoint_list: Vec<Candidate> = disjoint_entries
            .into_iter()
            .filter(|entry| !full_set.contains(entry))
            .map(|(handle, chunk)| Candidate { handle, chunk })
            .collect();

        // Fall 1: Eine volle Liste, eine leere Liste
        let res_single = rrf_fuse(&[&full_list, &empty_list], top).unwrap();
        prop_assert!(res_single.len() <= full_list.len().min(top));

        // Fall 2: Disjunkte Mengen über mehrere Teillisten
        let res_disjoint = rrf_fuse(&[&full_list, &disjoint_list, &empty_list], top).unwrap();
        let total_unique = full_set.len() + disjoint_list.len();
        prop_assert_eq!(res_disjoint.len(), total_unique.min(top));
    }

    /// SPEC: T-RANK-1 (Cross-List Duplikate / Schnittmengen)
    /// Ein Dokument mit Chunk, das in mehreren Teillisten vorkommt, erhält die Summe der
    /// Einzel-RRF-Beiträge und erscheint genau einmal im fusionierten Ranking.
    #[test]
    fn prop_rrf_cross_list_intersection_score_sum(
        shared_handle in arb_doc_handle(),
        shared_chunk in 0u32..5,
        l1_extra in arb_candidate_list(15),
        l2_extra in arb_candidate_list(15),
    ) {
        // Baue Liste 1 mit shared_handle/chunk an Kopf
        let mut l1 = vec![Candidate { handle: shared_handle, chunk: shared_chunk }];
        for c in l1_extra {
            if (c.handle, c.chunk) != (shared_handle, shared_chunk) {
                l1.push(c);
            }
        }

        // Baue Liste 2 mit shared_handle/chunk an Position 2 (nach einem anderen Kandidaten, falls vorhanden)
        let mut l2 = Vec::new();
        let other_cand = l2_extra.iter().find(|c| (c.handle, c.chunk) != (shared_handle, shared_chunk)).copied();
        if let Some(other) = other_cand {
            l2.push(other);
        }
        l2.push(Candidate { handle: shared_handle, chunk: shared_chunk });
        for c in l2_extra {
            if (c.handle, c.chunk) != (shared_handle, shared_chunk) && Some(c) != other_cand {
                l2.push(c);
            }
        }

        let res = rrf_fuse(&[&l1, &l2], 50).unwrap();
        let matches: Vec<_> = res.iter().filter(|f| f.handle == shared_handle && f.chunk == shared_chunk).collect();
        prop_assert_eq!(matches.len(), 1, "Shared candidate must appear exactly once");

        // Berechne erwarteten kumulierten Score
        let rank_in_l1 = 1u64;
        let rank_in_l2 = if other_cand.is_some() { 2u64 } else { 1u64 };
        let contrib1 = (1u64 << 40) / (60 + rank_in_l1);
        let contrib2 = (1u64 << 40) / (60 + rank_in_l2);
        prop_assert_eq!(matches.first().map(|f| f.score.0), Some(contrib1 + contrib2));
    }

    /// SPEC: T-HYB-1 (unseen_bound Property)
    /// Für jeden Treffer (handle, chunk), der in allen Listen erst nach depths[i] auftritt,
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
        let mut set1: BTreeSet<(DocHandle, u32)> = prefix1.iter().map(|c| (c.handle, c.chunk)).collect();
        for c in suffix1 {
            if set1.insert((c.handle, c.chunk)) {
                full1.push(c);
            }
        }

        let mut full2 = prefix2.clone();
        let mut set2: BTreeSet<(DocHandle, u32)> = prefix2.iter().map(|c| (c.handle, c.chunk)).collect();
        for c in suffix2 {
            if set2.insert((c.handle, c.chunk)) {
                full2.push(c);
            }
        }

        let depths = [prefix1.len(), prefix2.len()];
        let bound = unseen_bound(&depths).unwrap();

        let full_res = rrf_fuse(&[&full1, &full2], 1000).unwrap();

        // Any candidate that was NOT in prefix1 and NOT in prefix2 must have score <= bound
        for item in full_res {
            let in_p1 = prefix1.iter().any(|c| c.handle == item.handle && c.chunk == item.chunk);
            let in_p2 = prefix2.iter().any(|c| c.handle == item.handle && c.chunk == item.chunk);
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
