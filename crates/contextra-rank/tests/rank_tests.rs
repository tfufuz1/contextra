//! Unit-Tests für contextra-rank.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_rank::{
    fuse, rrf_fuse, unseen_bound, Candidate, DocHandle, RankError, Ranked, RankedList, RrfScore,
    K_RRF,
};

/// SPEC: T-RANK-1
/// SPEC: T-RANK-1
#[test]
fn t_rank_1_empty_and_partial_lists() {
    let empty: [Candidate; 0] = [];
    let h1 = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [10; 16],
    };
    let single = [Candidate {
        handle: h1,
        chunk: 0,
    }];

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
    assert_eq!(res[0].chunk, 0);
    let expected_score = (1u64 << 40) / (K_RRF as u64 + 1);
    assert_eq!(res[0].score, RrfScore(expected_score));
}

/// SPEC: T-RANK-1
/// SPEC: T-RANK-1
#[test]
fn t_rank_1_golden_ranking_deterministic() {
    let doc_a = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [1; 16],
    };
    let doc_b = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [2; 16],
    };
    let doc_c = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [3; 16],
    };

    // Liste 1: [A, B]
    let l1 = [
        Candidate {
            handle: doc_a,
            chunk: 0,
        },
        Candidate {
            handle: doc_b,
            chunk: 0,
        },
    ];
    // Liste 2: [B, C]
    let l2 = [
        Candidate {
            handle: doc_b,
            chunk: 0,
        },
        Candidate {
            handle: doc_c,
            chunk: 0,
        },
    ];

    let lists: [&[Candidate]; 2] = [&l1, &l2];
    let res = rrf_fuse(&lists, 3).unwrap();

    assert_eq!(res.len(), 3);

    // B kommt in beiden Listen vor: Rang 2 in l1 (1/62) + Rang 1 in l2 (1/61)
    // A kommt nur in l1 vor: Rang 1 in l1 (1/61)
    // C kommt nur in l2 vor: Rang 2 in l2 (1/62)
    // Erwartete Reihenfolge: B (höchster Score), dann A (Rang 1 vs Rang 2), dann C
    let score_b = (1u64 << 40) / 62 + (1u64 << 40) / 61;
    let score_a = (1u64 << 40) / 61;
    let score_c = (1u64 << 40) / 62;

    assert_eq!(res[0].handle, doc_b);
    assert_eq!(res[0].chunk, 0);
    assert_eq!(res[0].score, RrfScore(score_b));

    assert_eq!(res[1].handle, doc_a);
    assert_eq!(res[1].chunk, 0);
    assert_eq!(res[1].score, RrfScore(score_a));

    assert_eq!(res[2].handle, doc_c);
    assert_eq!(res[2].chunk, 0);
    assert_eq!(res[2].score, RrfScore(score_c));
}

/// SPEC: T-RANK-1
/// SPEC: T-RANK-1
#[test]
fn t_rank_1_tie_break_order_and_multi_list_sum() {
    // Wenn zwei Dokumente exakt denselben Score haben, bricht doc_tag aufsteigend den Tie-Break,
    // danach chunk aufsteigend, danach cluster_tag aufsteigend.
    let handle_a = DocHandle {
        cluster_tag: [2; 16],
        doc_tag: [5; 16],
    };
    let handle_b = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [5; 16],
    };
    let handle_c = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [6; 16],
    };

    // Beide an Rang 1 in je einer separaten Liste
    let l1 = [Candidate {
        handle: handle_a,
        chunk: 0,
    }];
    let l2 = [Candidate {
        handle: handle_c,
        chunk: 0,
    }];
    let l3 = [Candidate {
        handle: handle_b,
        chunk: 0,
    }];

    // Fusion von l1 und l2: handle_a und handle_c haben denselben Score (1/61).
    // doc_tag [5; 16] < doc_tag [6; 16], also handle_a vor handle_c
    let res = rrf_fuse(&[&l1, &l2], 10).unwrap();
    assert_eq!(res[0].handle, handle_a);
    assert_eq!(res[1].handle, handle_c);

    // Fusion von l1 und l3: handle_a und handle_b haben dasselbe doc_tag [5; 16] und denselben chunk 0.
    // cluster_tag [1; 16] < cluster_tag [2; 16], also handle_b vor handle_a
    let res2 = rrf_fuse(&[&l1, &l3], 10).unwrap();
    assert_eq!(res2[0].handle, handle_b);
    assert_eq!(res2[1].handle, handle_a);
}

/// SPEC: T-LIMIT-1
/// SPEC: T-LIMIT-1
#[test]
fn t_limit_1_input_validation_boundaries() {
    // top = 0 -> leere Ausgabe
    let h = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [1; 16],
    };
    let list = [Candidate {
        handle: h,
        chunk: 0,
    }];
    let res = rrf_fuse(&[&list], 0).unwrap();
    assert!(res.is_empty());

    // Liste mit mehr als 1.000 Einträgen -> Err(RankError::TooLarge)
    let mut big_list = Vec::with_capacity(1001);
    for i in 0..1001u32 {
        let mut doc_tag = [0u8; 16];
        doc_tag[..4].copy_from_slice(&i.to_be_bytes());
        big_list.push(Candidate {
            handle: DocHandle {
                cluster_tag: [0; 16],
                doc_tag,
            },
            chunk: 0,
        });
    }
    let err = rrf_fuse(&[&big_list], 10);
    assert_eq!(err, Err(RankError::TooLarge));

    // Exakt 1.000 Einträge -> OK
    let boundary_list = &big_list[..1000];
    let ok = rrf_fuse(&[boundary_list], 10);
    assert!(ok.is_ok());

    // Interne Duplikate innerhalb derselben Liste -> Err(RankError::Duplicate)
    let dup_list = [
        Candidate {
            handle: h,
            chunk: 0,
        },
        Candidate {
            handle: h,
            chunk: 0,
        },
    ];
    let dup_err = rrf_fuse(&[&dup_list], 10);
    assert_eq!(dup_err, Err(RankError::Duplicate));
}

/// SPEC: T-HYB-1
/// SPEC: T-HYB-1
#[test]
fn t_hyb_1_unseen_bound_calculation() {
    // Keine Listen -> Bound 0
    let b0 = unseen_bound(&[]).unwrap();
    assert_eq!(b0, RrfScore(0));

    // Zwei Listen mit Tiefe 10 und Tiefe 20:
    // Formel: floor(2^40 / (60 + 10 + 1)) + floor(2^40 / (60 + 20 + 1))
    // = floor(2^40 / 71) + floor(2^40 / 81)
    let b = unseen_bound(&[10, 20]).unwrap();
    let expected = (1u64 << 40) / 71 + (1u64 << 40) / 81;
    assert_eq!(b, RrfScore(expected));
}

/// SPEC: T-DOCREF-1
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
    let l1 = [Candidate {
        handle: h1,
        chunk: 0,
    }];
    let l2 = [Candidate {
        handle: h2,
        chunk: 0,
    }];

    let res = rrf_fuse(&[&l1, &l2], 10).unwrap();
    assert_eq!(res.len(), 2);
    assert_eq!(res.first().map(|f| f.handle), Some(h1));
    assert_eq!(res.get(1).map(|f| f.handle), Some(h2));
}

/// SPEC: T-RANK-1
/// Golden-Test mit 4 festen Teillisten (z. B. Vektor, BM25, Filter-Teilliste 1, Filter-Teilliste 2).
/// Überprüft bitgenaue Berechnung und deterministische Ordnung über 4 Listen.
/// SPEC: T-RANK-1
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

    // 4 Listen
    let l1 = [
        Candidate {
            handle: d1,
            chunk: 0,
        },
        Candidate {
            handle: d2,
            chunk: 0,
        },
        Candidate {
            handle: d3,
            chunk: 0,
        },
    ];
    let l2 = [
        Candidate {
            handle: d2,
            chunk: 0,
        },
        Candidate {
            handle: d4,
            chunk: 0,
        },
    ];
    let l3 = [
        Candidate {
            handle: d1,
            chunk: 0,
        },
        Candidate {
            handle: d3,
            chunk: 0,
        },
        Candidate {
            handle: d4,
            chunk: 0,
        },
    ];
    let l4 = [
        Candidate {
            handle: d2,
            chunk: 0,
        },
        Candidate {
            handle: d1,
            chunk: 0,
        },
    ];

    // Scores berechnen:
    // d1: l1: r1 (1/61), l3: r1 (1/61), l4: r2 (1/62)
    let s1 = (1u64 << 40) / 61 + (1u64 << 40) / 61 + (1u64 << 40) / 62;
    // d2: l1: r2 (1/62), l2: r1 (1/61), l4: r1 (1/61)
    let s2 = (1u64 << 40) / 62 + (1u64 << 40) / 61 + (1u64 << 40) / 61;
    // s1 == s2! Tie-Break: doc_tag [1; 16] < doc_tag [2; 16] -> d1 vor d2
    assert_eq!(s1, s2);

    // d3: l1: r3 (1/63), l3: r2 (1/62)
    let s3 = (1u64 << 40) / 63 + (1u64 << 40) / 62;
    // d4: l2: r2 (1/62), l3: r3 (1/63)
    let s4 = (1u64 << 40) / 62 + (1u64 << 40) / 63;
    // s3 == s4! Tie-Break: doc_tag [3; 16] < doc_tag [4; 16] -> d3 vor d4
    assert_eq!(s3, s4);

    let res = rrf_fuse(&[&l1, &l2, &l3, &l4], 4).unwrap();
    assert_eq!(res.len(), 4);
    assert_eq!(res[0].handle, d1);
    assert_eq!(res[0].score, RrfScore(s1));

    assert_eq!(res[1].handle, d2);
    assert_eq!(res[1].score, RrfScore(s2));

    assert_eq!(res[2].handle, d3);
    assert_eq!(res[2].score, RrfScore(s3));

    assert_eq!(res[3].handle, d4);
    assert_eq!(res[3].score, RrfScore(s4));
}

/// SPEC: T-HYB-2
/// Treffer-Identität ist `(doc_tag, chunk)` (SPEC D-124).
/// Die Fusion verändert sie nie, trennt unterschiedliche Chunks desselben Dokuments,
/// und summiert identische `(handle, chunk)`-Kombinationen across Listen.
/// SPEC: T-HYB-2
/// SPEC: T-RANK-1
#[test]
fn t_rank_1_fuse_alias_and_types_parity() {
    let cand = Candidate {
        handle: DocHandle {
            cluster_tag: [1; 16],
            doc_tag: [2; 16],
        },
        chunk: 0,
    };
    let list: &[Candidate] = &[cand];
    let ranked_list: RankedList = list;
    let res: Ranked = fuse(&[ranked_list], 10).expect("fuse should succeed");
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].handle.doc_tag, [2; 16]);
}

/// SPEC: T-RANK-1
#[test]
fn t_hyb_2_hit_identity_doc_tag_chunk_preservation() {
    let handle = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [10; 16],
    };
    let handle_other = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [20; 16],
    };

    // 1. Unterschiedliche Chunks desselben Dokuments in derselben Liste sind erlaubt (kein Duplikat)
    let c_chunk0 = Candidate { handle, chunk: 0 };
    let c_chunk1 = Candidate { handle, chunk: 1 };
    let list_same_doc = [c_chunk0, c_chunk1];
    let res_same = rrf_fuse(&[&list_same_doc], 10).unwrap();
    assert_eq!(res_same.len(), 2);
    assert_eq!(res_same[0].handle, handle);
    assert_eq!(res_same[0].chunk, 0);
    assert_eq!(res_same[1].handle, handle);
    assert_eq!(res_same[1].chunk, 1);

    // 2. Unterschiedliche Chunks desselben Dokuments über zwei Listen summieren nicht
    let l1 = [Candidate { handle, chunk: 0 }];
    let l2 = [Candidate { handle, chunk: 1 }];
    let res_diff_chunks = rrf_fuse(&[&l1, &l2], 10).unwrap();
    assert_eq!(res_diff_chunks.len(), 2);
    let s_single = (1u64 << 40) / 61;
    assert_eq!(res_diff_chunks[0].score, RrfScore(s_single));
    assert_eq!(res_diff_chunks[1].score, RrfScore(s_single));
    // Tie-break nach chunk aufsteigend
    assert_eq!(res_diff_chunks[0].chunk, 0);
    assert_eq!(res_diff_chunks[1].chunk, 1);

    // 3. Identische (handle, chunk)-Paare über zwei Listen summieren ihren Score
    let l3 = [Candidate { handle, chunk: 1 }];
    let res_sum_chunk = rrf_fuse(&[&l2, &l3], 10).unwrap();
    assert_eq!(res_sum_chunk.len(), 1);
    assert_eq!(res_sum_chunk[0].handle, handle);
    assert_eq!(res_sum_chunk[0].chunk, 1);
    let s_sum = (1u64 << 40) / 61 + (1u64 << 40) / 61;
    assert_eq!(res_sum_chunk[0].score, RrfScore(s_sum));

    // 4. Identitätserhaltung bei komplexem Ranking
    let l_mix_a = [
        Candidate {
            handle: handle_other,
            chunk: 5,
        },
        Candidate { handle, chunk: 42 },
    ];
    let l_mix_b = [
        Candidate { handle, chunk: 42 },
        Candidate {
            handle: handle_other,
            chunk: 5,
        },
    ];
    let res_mix = rrf_fuse(&[&l_mix_a, &l_mix_b], 10).unwrap();
    assert_eq!(res_mix.len(), 2);
    // Beide haben denselben summierten Score: Rang 1 + Rang 2
    // Tie-Break: doc_tag [10; 16] < doc_tag [20; 16], also `handle` vor `handle_other`
    assert_eq!(res_mix[0].handle, handle);
    assert_eq!(res_mix[0].chunk, 42);
    assert_eq!(res_mix[1].handle, handle_other);
    assert_eq!(res_mix[1].chunk, 5);
}
