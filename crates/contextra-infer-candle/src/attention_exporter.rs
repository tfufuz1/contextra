#![forbid(unsafe_code)]

//! Attention exporter implementation for Candle inference backend.

use parking_lot::Mutex;
use std::collections::{BTreeMap, VecDeque};

pub const MAX_TRACKED_REQUESTS: usize = 256;

#[derive(Debug)]
pub struct CandleAttentionExporter {
    state: Mutex<ExporterState>,
}

#[derive(Debug, Default)]
struct ExporterState {
    weights: BTreeMap<u64, Vec<f32>>,
    order: VecDeque<u64>,
}

impl Default for CandleAttentionExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl CandleAttentionExporter {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ExporterState::default()),
        }
    }

    pub fn record_attention_weights_local(&self, request_id: u64, weights: Vec<f32>) {
        let mut state = self.state.lock();
        if let std::collections::btree_map::Entry::Occupied(mut entry) =
            state.weights.entry(request_id)
        {
            entry.insert(weights);
            return;
        }

        while state.order.len() >= MAX_TRACKED_REQUESTS {
            if let Some(evicted_id) = state.order.pop_front() {
                state.weights.remove(&evicted_id);
            }
        }

        state.weights.insert(request_id, weights);
        state.order.push_back(request_id);
    }

    pub fn record_layer_head_scores_local(&self, request_id: u64, scores: &[Vec<f32>]) {
        if scores.is_empty() {
            return;
        }

        let seq_len = match scores.first() {
            Some(s) => s.len(),
            None => return,
        };
        let mut combined = vec![0.0f32; seq_len];

        for head_layer_vec in scores {
            for (i, &score) in head_layer_vec.iter().enumerate().take(seq_len) {
                if let Some(c) = combined.get_mut(i) {
                    *c += score;
                }
            }
        }

        self.record_attention_weights_local(request_id, combined);
    }

    pub fn export_weights(&self, request_id: u64) -> Option<Vec<f32>> {
        self.state.lock().weights.get(&request_id).cloned()
    }

    pub fn tracked_request_count(&self) -> usize {
        self.state.lock().weights.len()
    }

    pub fn clear(&self) {
        let mut state = self.state.lock();
        state.weights.clear();
        state.order.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_attention_weights_internal() {
        let exporter = CandleAttentionExporter::new();
        let req_id = 42;
        exporter.record_layer_head_scores_local(req_id, &[vec![1.0, 0.5]]);
        let weights = exporter.export_weights(req_id);
        assert!(weights.is_some());
        assert_eq!(weights.expect("weights present"), vec![1.0, 0.5]);
    }
}
