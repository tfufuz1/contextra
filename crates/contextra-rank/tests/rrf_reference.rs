//! Referenztests für contextra-rank.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use contextra_rank::{rrf_fuse, Candidate, DocHandle, Fused, RankError, RrfScore, K_RRF};

/// SPEC: T-RANK-1
/// Prüft die naive Referenzimplementierung in Ganzzahlarithmetik gegen `rrf_fuse`.
fn naive_rrf_reference(lists: &[&[Candidate]], top: usize) -> Result<Vec<Fused>, RankError> {
    if top == 0 {
        return Ok(Vec::new());
    }
    for list in lists {
        if list.len() > 1000 {
            return Err(RankError::TooLarge);
        }
        for i in 0..list.len() {
            for j in (i + 1)..list.len() {
                if list.get(i).map(|c| (c.handle, c.chunk))
                    == list.get(j).map(|c| (c.handle, c.chunk))
                {
                    return Err(RankError::Duplicate);
                }
            }
        }
    }

    let mut map: Vec<(DocHandle, u32, u64)> = Vec::new();
    for list in lists {
        for (idx, cand) in list.iter().enumerate() {
            let rank = (idx + 1) as u64;
            let contrib = (1u64 << 40) / (K_RRF as u64 + rank);
            if let Some(pos) = map
                .iter()
                .position(|(h, chunk, _)| *h == cand.handle && *chunk == cand.chunk)
            {
                if let Some(entry) = map.get_mut(pos) {
                    entry.2 = entry.2.checked_add(contrib).ok_or(RankError::Overflow)?;
                }
            } else {
                map.push((cand.handle, cand.chunk, contrib));
            }
        }
    }

    // Totale Ordnung: Score absteigend, dann doc_tag aufsteigend, dann chunk aufsteigend,
    // dann cluster_tag aufsteigend.
    map.sort_by(|a, b| {
        b.2.cmp(&a.2)
            .then_with(|| a.0.doc_tag.cmp(&b.0.doc_tag))
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.0.cluster_tag.cmp(&b.0.cluster_tag))
    });

    let res = map
        .into_iter()
        .take(top)
        .map(|(handle, chunk, score)| Fused {
            handle,
            chunk,
            score: RrfScore(score),
        })
        .collect();

    Ok(res)
}

/// SPEC: T-RANK-1
#[test]
fn t_rank_1_reference_parity_basic() {
    let doc1 = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [10; 16],
    };
    let doc2 = DocHandle {
        cluster_tag: [2; 16],
        doc_tag: [20; 16],
    };
    let doc3 = DocHandle {
        cluster_tag: [1; 16],
        doc_tag: [30; 16],
    };

    let list_a = vec![
        Candidate {
            handle: doc1,
            chunk: 0,
        },
        Candidate {
            handle: doc2,
            chunk: 0,
        },
    ];
    let list_b = vec![
        Candidate {
            handle: doc2,
            chunk: 0,
        },
        Candidate {
            handle: doc3,
            chunk: 0,
        },
    ];

    let lists: [&[Candidate]; 2] = [&list_a, &list_b];
    let actual = rrf_fuse(&lists, 10).expect("rrf_fuse should succeed");
    let expected = naive_rrf_reference(&lists, 10).expect("naive should succeed");

    assert_eq!(actual, expected);
}

/// SPEC: T-RANK-1
#[test]
fn t_rank_1_reference_parity_disjoint() {
    let doc1 = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [1; 16],
    };
    let doc2 = DocHandle {
        cluster_tag: [0; 16],
        doc_tag: [2; 16],
    };
    let list_a = vec![Candidate {
        handle: doc1,
        chunk: 0,
    }];
    let list_b = vec![Candidate {
        handle: doc2,
        chunk: 0,
    }];

    let lists: [&[Candidate]; 2] = [&list_a, &list_b];
    let actual = rrf_fuse(&lists, 2).expect("success");
    let expected = naive_rrf_reference(&lists, 2).expect("success");

    assert_eq!(actual, expected);
}
