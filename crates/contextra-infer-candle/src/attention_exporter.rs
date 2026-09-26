#![forbid(unsafe_code)]

//! Attention exporter implementation for Candle inference backend.

use contextra_ports::{AttentionExporter, RequestId};
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};

/// Maximum number of tracked requests in the attention history ring buffer.
pub const MAX_TRACKED_REQUESTS: usize = 256;

/// Sums and exports attention weights across heads and layers of prefill steps.
///
/// Maintains a bounded history (`MAX_TRACKED_REQUESTS = 256`) per active request.
#[derive(Debug)]
pub struct CandleAttentionExporter {
    state: Mutex<ExporterState>,
}

#[derive(Debug, Default)]
struct ExporterState {
    weights: HashMap<RequestId, Vec<f32>>,
    order: VecDeque<RequestId>,
}

impl Default for CandleAttentionExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl CandleAttentionExporter {
    /// Creates a new `CandleAttentionExporter`.
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ExporterState::default()),
        }
    }

    /// Records pre-summed attention weights for a request ID.
    ///
    /// Enforces [`MAX_TRACKED_REQUESTS`] ring-buffer capacity limit.
    pub fn record_attention_weights(&self, request_id: RequestId, weights: Vec<f32>) {
        let mut state = self.state.lock();
        if state.weights.contains_key(&request_id) {
            state.weights.insert(request_id, weights);
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

    /// Combines and records attention weights from layer/head score matrices/vectors.
    ///
    /// Sums weights across heads/layers per token position in $O(\text{seq\_len})$ time.
    pub fn record_layer_head_scores(&self, request_id: RequestId, scores: &[Vec<f32>]) {
        if scores.is_empty() {
            return;
        }

        let seq_len = scores[0].len();
        let mut combined = vec![0.0f32; seq_len];

        for head_layer_vec in scores {
            for (i, &score) in head_layer_vec.iter().enumerate().take(seq_len) {
                combined[i] += score;
            }
        }

        self.record_attention_weights(request_id, combined);
    }

    /// Returns the current number of tracked requests in memory.
    pub fn tracked_request_count(&self) -> usize {
        self.state.lock().weights.len()
    }

    /// Clears all tracked request attention histories.
    pub fn clear(&self) {
        let mut state = self.state.lock();
        state.weights.clear();
        state.order.clear();
    }
}

impl AttentionExporter for CandleAttentionExporter {
    fn export_attention_weights(&self, request_id: RequestId) -> Option<Vec<f32>> {
        self.state.lock().weights.get(&request_id).cloned()
    }
}
