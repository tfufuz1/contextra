//! Shadow mode hook for comparing TL-HFD and APPRH PPR scores against baseline PPR scores (§12).

use ahash::AHashMap;
use contextra_adapt::{ShadowDiscrepancy, ShadowSink};
use contextra_types::EntityId;

/// Configuration parameters for APPRH and TL-HFD shadow comparison execution.
#[derive(Debug, Clone)]
pub struct ShadowParams {
    #[cfg(feature = "apprh-diffusion")]
    pub apprh: Option<crate::apprh::ApprhParams>,
    pub tl_hfd: Option<crate::tl_hfd::TlHfdParams>,
    pub context_id: u64,
    pub top_k: usize,
}

impl Default for ShadowParams {
    fn default() -> Self {
        Self {
            #[cfg(feature = "apprh-diffusion")]
            apprh: None,
            tl_hfd: None,
            context_id: 0,
            top_k: 10,
        }
    }
}

/// Runs optional APPRH and/or TL-HFD shadow mode executions alongside productive PPR.
///
/// Calling this function never alters or mutates the productive PPR result set.
/// Errors during shadow comparison are caught and logged via `tracing::warn!`.
///
/// When `sampled` is `false`, this function exits immediately without executing any shadow tasks.
pub fn run_shadow_comparison<G: crate::path_rag::PathGraph>(
    graph: &G,
    seeds: &[EntityId],
    ppr_params: &crate::path_rag::PprParams,
    shadow_params: &ShadowParams,
    sink: Option<&dyn ShadowSink<f64>>,
    sampled: bool,
) {
    if !sampled || seeds.is_empty() {
        return;
    }

    #[cfg(feature = "apprh-diffusion")]
    if let Some(ref apprh_params) = shadow_params.apprh {
        match crate::apprh::shadow_compare_forward_push_vs_apprh(
            graph,
            seeds,
            ppr_params,
            apprh_params,
            shadow_params.top_k,
        ) {
            Ok(_comp) => {
                // Shadow comparison succeeded; results discarded
            }
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "APPRH shadow comparison failed; continuing without affecting main result"
                );
            }
        }
    }

    if let Some(ref tl_hfd_params) = shadow_params.tl_hfd {
        match crate::tl_hfd::tl_hfd_local(graph, seeds, tl_hfd_params) {
            Ok(candidate_map) => {
                if let Some(s) = sink {
                    let fp_map = crate::path_rag::forward_push_ppr(graph, seeds, ppr_params);
                    log_tl_hfd_vs_baseline_discrepancy(
                        &fp_map,
                        &candidate_map,
                        shadow_params.context_id,
                        s,
                    );
                }
            }
            Err(err) => {
                tracing::warn!(
                    error = %err,
                    "TL-HFD shadow execution failed; continuing without affecting main result"
                );
            }
        }
    }
}

/// Protokolliert die mittlere absolute Differenz (MAE) zwischen den Baseline- und
/// TL-HFD-Kandidaten-PPR-Score-Maps über die gegebene Diskrepanz-Senke.
pub fn log_tl_hfd_vs_baseline_discrepancy(
    baseline: &AHashMap<EntityId, f32>,
    candidate: &AHashMap<EntityId, f32>,
    context_id: u64,
    sink: &dyn ShadowSink<f64>,
) {
    let mut all_keys = std::collections::HashSet::new();
    for key in baseline.keys() {
        all_keys.insert(*key);
    }
    for key in candidate.keys() {
        all_keys.insert(*key);
    }

    if all_keys.is_empty() {
        sink.record(ShadowDiscrepancy {
            baseline: 0.0,
            candidate: 0.0,
            context_id,
        });
        return;
    }

    let mut total_diff = 0.0f64;
    for key in &all_keys {
        let b_val = baseline.get(key).copied().unwrap_or(0.0) as f64;
        let c_val = candidate.get(key).copied().unwrap_or(0.0) as f64;
        total_diff += (b_val - c_val).abs();
    }

    let mae = total_diff / (all_keys.len() as f64);

    sink.record(ShadowDiscrepancy {
        baseline: 0.0,
        candidate: mae,
        context_id,
    });
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MockPprSink {
        recorded: Arc<Mutex<Vec<ShadowDiscrepancy<f64>>>>,
    }

    impl ShadowSink<f64> for MockPprSink {
        fn record(&self, discrepancy: ShadowDiscrepancy<f64>) {
            if let Ok(mut lock) = self.recorded.lock() {
                lock.push(discrepancy);
            }
        }
    }

    /// Test (c): log_tl_hfd_vs_baseline_discrepancy berechnet die mittlere absolute Differenz korrekt
    #[test]
    fn test_log_tl_hfd_vs_baseline_discrepancy_mae_calculation() {
        let mut baseline = AHashMap::new();
        baseline.insert(EntityId(1), 0.8f32);
        baseline.insert(EntityId(2), 0.2f32);

        let mut candidate = AHashMap::new();
        candidate.insert(EntityId(1), 0.6f32);
        candidate.insert(EntityId(2), 0.4f32);

        let sink = MockPprSink::default();
        log_tl_hfd_vs_baseline_discrepancy(&baseline, &candidate, 42, &sink);

        let list = sink.recorded.lock().unwrap();
        assert_eq!(list.len(), 1);
        let disc = &list[0];

        assert_eq!(disc.context_id, 42);
        let expected_mae = 0.2f64; // (|0.8 - 0.6| + |0.2 - 0.4|) / 2 = 0.2
        assert!(
            (disc.candidate - expected_mae).abs() < 1e-6,
            "Expected MAE {}, got {}",
            expected_mae,
            disc.candidate
        );
    }

    #[test]
    fn test_run_shadow_comparison_sampled_false_does_nothing() {
        use crate::csr::CsrGraph;
        let graph = CsrGraph::new();
        let seeds = vec![EntityId(1)];
        let ppr_params = crate::path_rag::PprParams {
            alpha: 0.15,
            epsilon: 1e-6,
            hyperedge_decay: 0.85,
        };
        let mut shadow_params = ShadowParams::default();
        shadow_params.tl_hfd = Some(crate::tl_hfd::TlHfdParams::default());

        let sink = MockPprSink::default();
        run_shadow_comparison(
            &graph,
            &seeds,
            &ppr_params,
            &shadow_params,
            Some(&sink),
            false, // sampled = false
        );

        assert!(sink.recorded.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_run_shadow_comparison_sampled_true_runs_and_records() {
        use crate::csr::CsrGraph;
        use contextra_ports::GraphIndex;
        use contextra_types::{Edge, Entity};

        let graph = CsrGraph::new();
        let tx = contextra_types::TxId::internal();
        graph
            .add_entity(tx, Entity::new(EntityId(1), "N1", "Node"))
            .await
            .unwrap();
        graph
            .add_entity(tx, Entity::new(EntityId(2), "N2", "Node"))
            .await
            .unwrap();
        graph
            .add_edge(tx, Edge::new(EntityId(1), EntityId(2), "rel"))
            .await
            .unwrap();

        let seeds = vec![EntityId(1)];
        let ppr_params = crate::path_rag::PprParams {
            alpha: 0.15,
            epsilon: 1e-6,
            hyperedge_decay: 0.85,
        };
        let mut shadow_params = ShadowParams::default();
        shadow_params.tl_hfd = Some(crate::tl_hfd::TlHfdParams::default());
        shadow_params.context_id = 99;

        let sink = MockPprSink::default();
        run_shadow_comparison(
            &graph,
            &seeds,
            &ppr_params,
            &shadow_params,
            Some(&sink),
            true, // sampled = true
        );

        let rec = sink.recorded.lock().unwrap();
        assert_eq!(rec.len(), 1);
        assert_eq!(rec[0].context_id, 99);
    }
}
