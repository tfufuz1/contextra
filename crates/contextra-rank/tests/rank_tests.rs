//! Unit-Tests für contextra-rank.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_rank::{rrf_fuse, unseen_bound, Candidate, DocHandle, RankError, RrfScore, K_RRF};

/// SPEC: T-RANK-1
#[test]
fn t_rank_1_empty_and_partial_lists() {
    let empty: [Candidate; 0] = [];
    let h1 = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [10; 16],
    };
    let single = [Candidate { handle: h1 }];

    // Völlig leere Eingabe
    let res = rrf_fuse(&[], 10).unwrap();
    assert!(res.is_empty());

    // Leere Teillisten
    let res = rrf_fuse(&[&empty, &empty], 10).unwrap();
    assert!(res.is_empty());

    // Teilweise leere Listen
    let res = rrf_fuse(&[&empty, &single, &empty], 10).unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].handle, h1);
    let expected_score = (1u64 << 40) / (K_RRF as u64 + 1);
    assert_eq!(res[0].score, RrfScore(expected_score));
}

/// SPEC: T-RANK-1
#[test]
fn t_rank_1_golden_ranking_deterministic() {
    let doc_a = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [10; 16],
    };
    let doc_b = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [20; 16],
    };
    let doc_c = DocHandle {
        cluster_tag: [2; 16],
        doc_tag: [20; 16],
    };

    let list_vector = [Candidate { handle: doc_a }, Candidate { handle: doc_b }];
    let list_text = [Candidate { handle: doc_b }, Candidate { handle: doc_c }];

    let fused = rrf_fuse(&[&list_vector, &list_text], 3).unwrap();
    assert_eq!(fused.len(), 3);

    // doc_b ist Rang 2 in Liste 1 und Rang 1 in Liste 2:
    // Score(doc_b) = floor(2^40 / 62) + floor(2^40 / 61)
    let score_rank_1 = (1u64 << 40) / 61;
    let score_rank_2 = (1u64 << 40) / 62;
    let expected_doc_b = score_rank_1 + score_rank_2;
    assert_eq!(fused[0].handle, doc_b);
    assert_eq!(fused[0].score, RrfScore(expected_doc_b));

    // doc_a: Rang 1 in Liste 1 -> score_rank_1
    // doc_c: Rang 2 in Liste 2 -> score_rank_2
    assert_eq!(fused[1].handle, doc_a);
    assert_eq!(fused[1].score, RrfScore(score_rank_1));

    assert_eq!(fused[2].handle, doc_c);
    assert_eq!(fused[2].score, RrfScore(score_rank_2));
}

/// SPEC: T-RANK-1
#[test]
fn t_rank_1_tie_break_order_and_multi_list_sum() {
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

/// SPEC: T-LIMIT-1
#[test]
fn t_limit_1_input_validation_boundaries() {
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

/// SPEC: T-HYB-1
#[test]
fn t_hyb_1_unseen_bound_calculation() {
    let depths = [10, 20];
    let bound = unseen_bound(&depths).unwrap();
    let term1 = (1u64 << 40) / 71;
    let term2 = (1u64 << 40) / 81;
    assert_eq!(bound, RrfScore(term1 + term2));

    // Leere Tiefen => Score 0
    assert_eq!(unseen_bound(&[]).unwrap(), RrfScore(0));
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

/// SPEC: T-RANK-1
/// Golden-Test mit 4 festen Teillisten (z. B. Vektor, BM25, Filter-Teilliste 1, Filter-Teilliste 2).
/// Überprüft bitgenaue Berechnung und deterministische Ordnung über 4 Listen.
#[test]
fn t_rank_1_golden_ranking_four_lists_exact() {
    let d1 = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [1; 16],
    };
    let d2 = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [2; 16],
    };
    let d3 = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [3; 16],
    };
    let d4 = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [4; 16],
    };

    // Liste 1: [d1, d2]
    // Liste 2: [d2, d3]
    // Liste 3: [d3, d4]
    // Liste 4: [d1, d3]
    let l1 = [Candidate { handle: d1 }, Candidate { handle: d2 }];
    let l2 = [Candidate { handle: d2 }, Candidate { handle: d3 }];
    let l3 = [Candidate { handle: d3 }, Candidate { handle: d4 }];
    let l4 = [Candidate { handle: d1 }, Candidate { handle: d3 }];

    let scale = 1u64 << 40;
    let r1 = scale / 61; // Rang 1
    let r2 = scale / 62; // Rang 2

    // Erwartete kumulierte Scores:
    // d1: Rang 1 (L1) + Rang 1 (L4) = r1 + r1
    let exp_d1 = r1 + r1;
    // d2: Rang 2 (L1) + Rang 1 (L2) = r2 + r1
    let exp_d2 = r2 + r1;
    // d3: Rang 2 (L2) + Rang 1 (L3) + Rang 2 (L4) = r2 + r1 + r2
    let exp_d3 = r2 + r1 + r2;
    // d4: Rang 2 (L3) = r2
    let exp_d4 = r2;

    // Da exp_d3 = r1 + 2*r2 (3 Vorkommen), exp_d1 = 2*r1 (2 Vorkommen),
    // exp_d2 = r1 + r2 (2 Vorkommen), exp_d4 = r2 (1 Vorkommen)
    // gilt: exp_d3 > exp_d1 > exp_d2 > exp_d4
    assert!(exp_d3 > exp_d1);
    assert!(exp_d1 > exp_d2);
    assert!(exp_d2 > exp_d4);

    let res = rrf_fuse(&[&l1, &l2, &l3, &l4], 10).unwrap();
    assert_eq!(res.len(), 4);

    assert_eq!(
        res.first().map(|f| (f.handle, f.score)),
        Some((d3, RrfScore(exp_d3)))
    );
    assert_eq!(
        res.get(1).map(|f| (f.handle, f.score)),
        Some((d1, RrfScore(exp_d1)))
    );
    assert_eq!(
        res.get(2).map(|f| (f.handle, f.score)),
        Some((d2, RrfScore(exp_d2)))
    );
    assert_eq!(
        res.get(3).map(|f| (f.handle, f.score)),
        Some((d4, RrfScore(exp_d4)))
    );

    // Begrenzung auf top = 2
    let res_top2 = rrf_fuse(&[&l1, &l2, &l3, &l4], 2).unwrap();
    assert_eq!(res_top2.len(), 2);
    assert_eq!(res_top2.first().map(|f| f.handle), Some(d3));
    assert_eq!(res_top2.get(1).map(|f| f.handle), Some(d1));
}
