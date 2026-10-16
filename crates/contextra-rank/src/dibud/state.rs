//! State machine for DiBud (Dynamic Budgeted RRF) fusion.

use ahash::AHashMap;
use contextra_types::{ContextraError, DocId};

use super::types::{BudgetedChannel, DiBudOutcome, FusionBudget};
use crate::fusion::{ProvenanceBuilder, ProvenanceRecord};

const RRF_K: f32 = 60.0;

/// Represents the step action requested by `DiBudFusionState::next_request`.
#[derive(Debug, Clone, PartialEq)]
pub enum DiBudStep {
    /// Request reading the next item from the specified channel.
    Poll(BudgetedChannel),
    /// Fusion is complete; outcome contains certified prefix and ranked documents.
    Done(DiBudOutcome),
}

/// Observed state for a single document during fusion.
#[derive(Debug, Clone, Default)]
struct ObservedDocState {
    /// 1-based ranks in each channel: `[Vector, Text, Graph]`.
    ranks: [Option<u32>; 3],
}

/// Core state machine tracking channel depths, observed document scores, and prefix certification.
#[derive(Debug, Clone)]
pub struct DiBudFusionState {
    channel_depths: [usize; 3],
    exhausted: [bool; 3],
    accesses: usize,
    observed_docs: AHashMap<DocId, ObservedDocState>,
    er_cache: AHashMap<DocId, f32>,
    provenance: AHashMap<DocId, ProvenanceRecord>,
}

impl Default for DiBudFusionState {
    fn default() -> Self {
        Self::new(128)
    }
}

impl DiBudFusionState {
    /// Creates a new `DiBudFusionState` with pre-allocated map capacities.
    pub fn new(capacity: usize) -> Self {
        Self {
            channel_depths: [0; 3],
            exhausted: [false; 3],
            accesses: 0,
            observed_docs: AHashMap::with_capacity(capacity),
            er_cache: AHashMap::with_capacity(capacity),
            provenance: AHashMap::with_capacity(capacity),
        }
    }

    /// Returns current channel depths `[Vector, Text, Graph]`.
    pub fn channel_depths(&self) -> [usize; 3] {
        self.channel_depths
    }

    /// Returns exhausted status for each channel `[Vector, Text, Graph]`.
    pub fn exhausted(&self) -> [bool; 3] {
        self.exhausted
    }

    /// Returns total read accesses performed.
    pub fn accesses(&self) -> usize {
        self.accesses
    }

    /// Returns reference to provenance record for a given document if present.
    pub fn provenance_of(&self, doc_id: &DocId) -> Option<&ProvenanceRecord> {
        self.provenance.get(doc_id)
    }

    /// Feeds the result of reading a channel into the state machine.
    ///
    /// Raw scores are unknown in pure `DocId` streams and are set to 0.0 in `ProvenanceRecord`.
    pub fn feed(
        &mut self,
        ch: BudgetedChannel,
        item: Option<DocId>,
        er: impl Fn(DocId) -> f32,
        budget: &FusionBudget,
    ) -> Result<(), ContextraError> {
        self.accesses += 1;
        let ch_idx = ch.index();
        self.channel_depths[ch_idx] += 1;
        let rank_1based = self.channel_depths[ch_idx] as u32;

        if let Some(doc_id) = item {
            let doc_state = self.observed_docs.entry(doc_id).or_default();
            if doc_state.ranks[ch_idx].is_none() {
                doc_state.ranks[ch_idx] = Some(rank_1based);
            }

            if !self.er_cache.contains_key(&doc_id) {
                let er_val = er(doc_id);
                if !er_val.is_finite() || er_val < 0.0 {
                    return Err(ContextraError::InvalidInput(format!(
                        "Edge reinforcement score for document {doc_id:?} must be a finite non-negative float, got {er_val}"
                    )));
                }
                self.er_cache.insert(doc_id, er_val);
            }

            self.update_provenance(doc_id, budget);
        } else {
            self.exhausted[ch_idx] = true;
        }

        Ok(())
    }

    /// Determines whether fusion is done or returns the next channel to poll.
    pub fn next_request(&self, budget: &FusionBudget) -> DiBudStep {
        let (ranked, certified_len) = self.certify_prefix(budget);

        let all_exhausted = self.exhausted.iter().all(|&e| e);
        let max_accesses_reached = self.accesses >= budget.max_total_accesses;
        let min_certified_reached = certified_len >= budget.min_certified_results;

        if min_certified_reached || max_accesses_reached || all_exhausted {
            return DiBudStep::Done(DiBudOutcome {
                ranked,
                certified_len,
                accesses: self.accesses,
                budget_exhausted: max_accesses_reached,
            });
        }

        // Select unexhausted channel with maximum marginal upper bound contribution w_i / (c + d_i + 1).
        // Tie-breaker: Graph > Text > Vector.
        let mut best_channel = None;
        let mut best_score = -1.0_f32;

        for ch in [
            BudgetedChannel::Graph,
            BudgetedChannel::Text,
            BudgetedChannel::Vector,
        ] {
            let idx = ch.index();
            if !self.exhausted[idx] {
                let score =
                    budget.channel_weights[idx] / (RRF_K + self.channel_depths[idx] as f32 + 1.0);
                if score > best_score {
                    best_score = score;
                    best_channel = Some(ch);
                }
            }
        }

        match best_channel {
            Some(ch) => DiBudStep::Poll(ch),
            None => DiBudStep::Done(DiBudOutcome {
                ranked,
                certified_len,
                accesses: self.accesses,
                budget_exhausted: max_accesses_reached,
            }),
        }
    }

    fn compute_f_minus(&self, doc_id: &DocId, budget: &FusionBudget) -> f32 {
        let mut score = 0.0_f32;
        if let Some(state) = self.observed_docs.get(doc_id) {
            for idx in 0..3 {
                if let Some(rank) = state.ranks[idx] {
                    score += budget.channel_weights[idx] / (RRF_K + rank as f32);
                }
            }
        }
        if let Some(&er_val) = self.er_cache.get(doc_id) {
            score += budget.edge_reinforcement_weight * er_val;
        }
        score
    }

    fn compute_f_plus(&self, doc_id: &DocId, budget: &FusionBudget) -> f32 {
        let mut score = self.compute_f_minus(doc_id, budget);
        if let Some(state) = self.observed_docs.get(doc_id) {
            for idx in 0..3 {
                if state.ranks[idx].is_none() && !self.exhausted[idx] {
                    score += budget.channel_weights[idx]
                        / (RRF_K + self.channel_depths[idx] as f32 + 1.0);
                }
            }
        }
        score
    }

    fn compute_f_plus_unseen(&self, budget: &FusionBudget) -> f32 {
        let mut score = 0.0_f32;
        for idx in 0..3 {
            if !self.exhausted[idx] {
                score +=
                    budget.channel_weights[idx] / (RRF_K + self.channel_depths[idx] as f32 + 1.0);
            }
        }
        score += budget.edge_reinforcement_weight * budget.edge_reinforcement_upper_bound;
        score
    }

    fn has_unobserved_unexhausted_channels(&self, doc_id: &DocId) -> bool {
        if let Some(state) = self.observed_docs.get(doc_id) {
            for idx in 0..3 {
                if state.ranks[idx].is_none() && !self.exhausted[idx] {
                    return true;
                }
            }
        }
        false
    }

    fn can_overtake(
        &self,
        doc_id_p: DocId,
        f_minus_p: f32,
        doc_id_y: DocId,
        budget: &FusionBudget,
    ) -> bool {
        let f_plus_y = self.compute_f_plus(&doc_id_y, budget);
        if f_plus_y > f_minus_p {
            return true;
        }
        if f_plus_y == f_minus_p
            && self.has_unobserved_unexhausted_channels(&doc_id_y)
            && doc_id_y < doc_id_p
        {
            return true;
        }
        false
    }

    fn certify_prefix(&self, budget: &FusionBudget) -> (Vec<DocId>, usize) {
        let mut scored_docs: Vec<(DocId, f32)> = self
            .observed_docs
            .keys()
            .map(|&doc_id| (doc_id, self.compute_f_minus(&doc_id, budget)))
            .collect();

        // Sort descending by lower bound score; tie-breaker ascending DocId
        scored_docs.sort_by(|(id_a, score_a), (id_b, score_b)| {
            score_b
                .partial_cmp(score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| id_a.cmp(id_b))
        });

        let ranked: Vec<DocId> = scored_docs.iter().map(|(id, _)| *id).collect();
        let f_plus_unseen = self.compute_f_plus_unseen(budget);

        let mut certified_len = 0;
        for p in 0..ranked.len() {
            let doc_id_p = ranked[p];
            let f_minus_p = scored_docs[p].1;

            if f_minus_p <= f_plus_unseen {
                break;
            }

            let mut certified = true;
            for y in &ranked[p + 1..] {
                if self.can_overtake(doc_id_p, f_minus_p, *y, budget) {
                    certified = false;
                    break;
                }
            }

            if certified {
                certified_len += 1;
            } else {
                break;
            }
        }

        (ranked, certified_len)
    }

    fn update_provenance(&mut self, doc_id: DocId, budget: &FusionBudget) {
        if let Some(state) = self.observed_docs.get(&doc_id) {
            let mut builder = ProvenanceBuilder::new(RRF_K);
            if let Some(rank) = state.ranks[0] {
                builder = builder.vector(0.0, rank, budget.channel_weights[0]);
            }
            if let Some(rank) = state.ranks[1] {
                builder = builder.bm25(0.0, rank, budget.channel_weights[1]);
            }
            if let Some(rank) = state.ranks[2] {
                builder = builder.graph(0.0, rank, budget.channel_weights[2]);
            }
            self.provenance.insert(doc_id, builder.build());
        }
    }
}
