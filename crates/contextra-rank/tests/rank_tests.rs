//! Unit-Tests für contextra-rank.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_rank::{rrf_fuse, unseen_bound, Candidate, DocHandle, RankError, RrfScore};

/// SPEC: RK-03
#[test]
fn rk_03_tie_break_order_and_multi_list_sum() {
    // Gleicher Score: Tie-Break nach doc_tag aufsteigend, dann cluster_tag aufsteigend
    let handle_a = DocHandle {
        cluster_tag: [5; 16],
        doc_tag: [1; 16],
    };
    let handle_b = DocHandle {
        cluster_tag: [2; 16],
        doc_tag: [2; 16],
    };
    let handle_c = DocHandle {
        cluster_tag: [9; 16],
        doc_tag: [1; 16],
    };

    let list1 = [Candidate { handle: handle_a }];
    let list2 = [Candidate { handle: handle_c }];

    let res = rrf_fuse(&[&list1, &[Candidate { handle: handle_b }]], 5).unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res.first().map(|f| f.handle), Some(handle_a));
    assert_eq!(res.get(1).map(|f| f.handle), Some(handle_b));

    // Now test cluster_tag tie-break when doc_tag is identical
    let res2 = rrf_fuse(&[&list1, &list2], 5).unwrap();
    assert_eq!(res2.len(), 2);
    assert_eq!(res2.first().map(|f| f.handle), Some(handle_a));
    assert_eq!(res2.get(1).map(|f| f.handle), Some(handle_c));

    // Gleiches (cluster_tag, doc_tag) in mehreren Listen => summiert
    let list_multi1 = [Candidate { handle: handle_b }];
    let list_multi2 = [Candidate { handle: handle_b }];
    let res_sum = rrf_fuse(&[&list_multi1, &list_multi2], 5).unwrap();
    assert_eq!(res_sum.len(), 1);
    let expected_single = (1u64 << 40) / 61;
    assert_eq!(
        res_sum.first().map(|f| f.score),
        Some(RrfScore(expected_single * 2))
    );
}

/// SPEC: RK-06
#[test]
fn rk_06_input_validation_boundaries() {
    let dummy = Candidate {
        handle: DocHandle {
            cluster_tag: [0; 16],
            doc_tag: [1; 16],
        },
    };

    // top = 0 => leere Ausgabe
    let list = [dummy];
    let res = rrf_fuse(&[&list], 0).unwrap();
    assert!(res.is_empty());

    // Mehr als 1000 Kandidaten je Liste => RankError::TooLarge
    let mut large_list = Vec::with_capacity(1001);
    for i in 0..1001u32 {
        let mut doc_tag = [0u8; 16];
        doc_tag[..4].copy_from_slice(&i.to_be_bytes());
        large_list.push(Candidate {
            handle: DocHandle {
                cluster_tag: [0; 16],
                doc_tag,
            },
        });
    }
    let err = rrf_fuse(&[&large_list], 10);
    assert_eq!(err, Err(RankError::TooLarge));

    // Duplikate innerhalb einer Liste => RankError::Duplicate
    let dup_list = [dummy, dummy];
    let err_dup = rrf_fuse(&[&dup_list], 10);
    assert_eq!(err_dup, Err(RankError::Duplicate));
}

/// SPEC: RK-07
#[test]
fn rk_07_unseen_bound_calculation() {
    let depths = [10, 20];
    let bound = unseen_bound(&depths);
    let term1 = (1u64 << 40) / 71;
    let term2 = (1u64 << 40) / 81;
    assert_eq!(bound, RrfScore(term1 + term2));
}

/// SPEC: T-DOCREF-1
#[test]
fn t_docref_1_handle_identity() {
    // Gleiche doc_tag mit anderem cluster_tag => verschiedene Einträge
    let h1 = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [42; 16],
    };
    let h2 = DocHandle {
        cluster_tag: [2; 16],
        doc_tag: [42; 16],
    };
    let l1 = [Candidate { handle: h1 }];
    let l2 = [Candidate { handle: h2 }];

    let res = rrf_fuse(&[&l1, &l2], 10).unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res.first().map(|f| f.handle), Some(h1));
    assert_eq!(res.get(1).map(|f| f.handle), Some(h2));
}
