#![cfg(feature = "dibud")]

use contextra_rank::dibud::{
    fuse_exact_prefix, DiBudFusionState, FusionBudget,
};
use contextra_rank::fusion::{
    weighted_reciprocal_rank_fusion, SearchResult,
};
use contextra_types::DocId;

struct SimpleLcg {
    state: u64,
}

impl SimpleLcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_bounded(&mut self, bound: u32) -> u32 {
        if bound == 0 {
            0
        } else {
            self.next_u32() % bound
        }
    }
}

fn make_doc_id(key_num: u64) -> DocId {
    DocId::from_key(&format!("doc_{key_num:05}")).expect("valid doc id")
}

fn make_doc_key_string(key_num: u64) -> String {
    format!("doc_{key_num:05}")
}

#[test]
fn test_dibud_unlimited_budget_equivalence_200_fixtures() {
    let mut lcg = SimpleLcg::new(0xDEAD_BEEF_1234_5678);

    for fixture_idx in 0..200 {
        let pool_size = 30 + lcg.next_bounded(30); // 30..60
        let len_vec = 5 + lcg.next_bounded(25) as usize;
        let len_text = 5 + lcg.next_bounded(25) as usize;
        let len_graph = 5 + lcg.next_bounded(25) as usize;

        let mut gen_list = |len: usize| -> (Vec<DocId>, Vec<SearchResult>) {
            let mut ids = Vec::with_capacity(len);
            let mut results = Vec::with_capacity(len);
            let mut used = std::collections::HashSet::new();

            while ids.len() < len {
                let key_num = (1 + lcg.next_bounded(pool_size)) as u64;
                if used.insert(key_num) {
                    let doc_id = make_doc_id(key_num);
                    let key_str = make_doc_key_string(key_num);
                    ids.push(doc_id);
                    results.push(SearchResult {
                        id: key_str,
                        score: 1.0,
                        metadata: None,
                        matched_signals: vec![],
                        provenance: None,
                    });
                }
            }
            (ids, results)
        };

        let (vec_ids, vec_res) = gen_list(len_vec);
        let (text_ids, text_res) = gen_list(len_text);
        let (graph_ids, graph_res) = gen_list(len_graph);

        let w_vec = 1.0 + (lcg.next_bounded(10) as f32) * 0.1;
        let w_text = 1.0 + (lcg.next_bounded(10) as f32) * 0.1;
        let w_graph = 1.0 + (lcg.next_bounded(10) as f32) * 0.1;

        let budget = FusionBudget {
            max_total_accesses: 10_000,
            min_certified_results: 10_000,
            edge_reinforcement_weight: 0.0,
            channel_weights: [w_vec, w_text, w_graph],
            edge_reinforcement_upper_bound: 0.0,
        };

        let state = DiBudFusionState::new(128);
        let mut v_iter = vec_ids.clone().into_iter();
        let mut t_iter = text_ids.clone().into_iter();
        let mut g_iter = graph_ids.clone().into_iter();

        let outcome = fuse_exact_prefix(
            state,
            &mut v_iter,
            &mut t_iter,
            &mut g_iter,
            |_| 0.0,
            &budget,
        )
        .expect("fuse_exact_prefix succeeds");

        assert!(
            !outcome.budget_exhausted,
            "Fixture {fixture_idx}: budget should not be exhausted with 10k accesses"
        );
        assert_eq!(
            outcome.certified_len,
            outcome.ranked.len(),
            "Fixture {fixture_idx}: with exhausted channels, all items must be certified"
        );

        let result_sets = vec![
            ("vector".to_string(), vec_res, w_vec),
            ("text".to_string(), text_res, w_text),
            ("graph".to_string(), graph_res, w_graph),
        ];

        let rrf_full = weighted_reciprocal_rank_fusion(result_sets, 10_000);
        let rrf_doc_ids: Vec<DocId> = rrf_full
            .iter()
            .map(|r| DocId::from_key(&r.id).unwrap())
            .collect();

        // Check if there are exact score ties in the outcome
        let mut has_ties = false;
        let mut rrf_scores = std::collections::HashMap::new();
        for item in &rrf_full {
            let doc_id = DocId::from_key(&item.id).unwrap();
            rrf_scores.insert(doc_id, item.score);
        }

        for i in 0..outcome.ranked.len().saturating_sub(1) {
            let s1 = rrf_scores[&outcome.ranked[i]];
            let s2 = rrf_scores[&outcome.ranked[i + 1]];
            if s1 == s2 {
                has_ties = true;
                // Spec tie-breaker check: tied documents must be ordered ascending by DocId
                assert!(
                    outcome.ranked[i] < outcome.ranked[i + 1],
                    "Fixture {fixture_idx}: tied documents at position {i} must be ordered ascending by DocId"
                );
            }
        }

        if !has_ties {
            assert_eq!(
                outcome.ranked, rrf_doc_ids,
                "Fixture {fixture_idx}: DiBud unlimited budget outcome without ties must match weighted_reciprocal_rank_fusion"
            );
        } else {
            // Equal sets check
            let mut set_outcome = outcome.ranked.clone();
            let mut set_rrf = rrf_doc_ids;
            set_outcome.sort();
            set_rrf.sort();
            assert_eq!(
                set_outcome, set_rrf,
                "Fixture {fixture_idx}: document set must match weighted_reciprocal_rank_fusion"
            );
        }
    }
}

#[test]
fn test_dibud_limited_budget_prefix_property() {
    let mut lcg = SimpleLcg::new(0xCAFE_BABE_9876_5432);

    for fixture_idx in 0..100 {
        let pool_size = 40;
        let mut gen_list = |len: usize| -> Vec<DocId> {
            let mut ids = Vec::with_capacity(len);
            let mut used = std::collections::HashSet::new();
            while ids.len() < len {
                let key_num = (1 + lcg.next_bounded(pool_size)) as u64;
                if used.insert(key_num) {
                    ids.push(make_doc_id(key_num));
                }
            }
            ids
        };

        let vec_ids = gen_list(20);
        let text_ids = gen_list(20);
        let graph_ids = gen_list(20);

        let full_budget = FusionBudget {
            max_total_accesses: 1000,
            min_certified_results: 1000,
            edge_reinforcement_weight: 0.0,
            channel_weights: [1.0, 1.0, 1.0],
            edge_reinforcement_upper_bound: 0.0,
        };

        let full_outcome = fuse_exact_prefix(
            DiBudFusionState::new(64),
            &mut vec_ids.clone().into_iter(),
            &mut text_ids.clone().into_iter(),
            &mut graph_ids.clone().into_iter(),
            |_| 0.0,
            &full_budget,
        )
        .expect("full fusion succeeds");

        let limit_budget = FusionBudget {
            max_total_accesses: 10 + (lcg.next_bounded(20) as usize),
            min_certified_results: 3 + (lcg.next_bounded(5) as usize),
            edge_reinforcement_weight: 0.0,
            channel_weights: [1.0, 1.0, 1.0],
            edge_reinforcement_upper_bound: 0.0,
        };

        let lim_outcome = fuse_exact_prefix(
            DiBudFusionState::new(64),
            &mut vec_ids.into_iter(),
            &mut text_ids.into_iter(),
            &mut graph_ids.into_iter(),
            |_| 0.0,
            &limit_budget,
        )
        .expect("limited fusion succeeds");

        assert!(
            lim_outcome.certified_len <= lim_outcome.ranked.len(),
            "Fixture {fixture_idx}: certified_len cannot exceed ranked length"
        );

        let cert_len = lim_outcome.certified_len;
        if cert_len > 0 {
            let certified_prefix = &lim_outcome.ranked[..cert_len];
            let full_prefix = &full_outcome.ranked[..cert_len];
            assert_eq!(
                certified_prefix, full_prefix,
                "Fixture {fixture_idx}: certified prefix under limited budget must exactly match full fusion prefix"
            );
        }
    }
}
