//! Integration test verifying J15 closure symbols in `contextra-graph`.
//!
//! Target symbols verified:
//! - `apprh_diffusion`
//! - `apprh_shadow_compare`
//! - `with_default_monitor`
//! - `as_arc`
//! - `recommended_multiplier`
//! - `log_tl_hfd_vs_baseline_discrepancy`
//! - `with_exclude_seeds`
//! - `append_step`
//! - `children_of`
//! - `set_active_head`
//! - `thresholded_local_hfd`

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use ahash::AHashMap;
use contextra_adapt::{ShadowDiscrepancy, ShadowSink};
#[cfg(feature = "apprh-diffusion")]
use contextra_graph::apprh::{ApprhParams, ApprhSelector, ApprhSelectorConfig};
use contextra_graph::arc_slice::ArcSlice;
use contextra_graph::csr::CsrGraph;
#[cfg(feature = "apprh-diffusion")]
use contextra_graph::path_rag::PprParams;
use contextra_graph::ppr::cost::{PprCostCalibrator, PprCostSample};
use contextra_graph::ppr::shadow_hook::log_tl_hfd_vs_baseline_discrepancy;
use contextra_graph::ppr_stream::PprCandidateStream;
use contextra_graph::session_dag::SessionBranchTree;
use contextra_graph::tl_hfd::TlHfdParams;
use contextra_ports::{Edge, GraphIndex};
use contextra_types::{EntityId, PprAlgorithm, PprConfig, TxId};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct TestShadowSink {
    recorded: Arc<Mutex<Vec<ShadowDiscrepancy<f64>>>>,
}

impl ShadowSink<f64> for TestShadowSink {
    fn record(&self, discrepancy: ShadowDiscrepancy<f64>) {
        if let Ok(mut lock) = self.recorded.lock() {
            lock.push(discrepancy);
        }
    }
}

#[tokio::test]
async fn test_j15_closure_all_symbols() {
    let graph = CsrGraph::new();
    let tx = TxId::new(1);
    let e1 = EntityId(10);
    let e2 = EntityId(20);

    graph
        .add_edge(tx, Edge::new(e1, e2, "rel").with_weight(1.0))
        .await
        .unwrap();

    let seeds = vec![e1];

    // 1 & 2. apprh_diffusion & apprh_shadow_compare
    #[cfg(feature = "apprh-diffusion")]
    {
        let apprh_params = ApprhParams::default();
        let ppr_params = PprParams::default();

        let diff_result = graph.apprh_diffusion(&seeds, &apprh_params).expect("apprh_diffusion failed");
        assert!(!diff_result.is_empty(), "apprh_diffusion result must not be empty");

        let shadow_report = graph
            .apprh_shadow_compare(&seeds, &ppr_params, &apprh_params, 10)
            .expect("apprh_shadow_compare failed");
        assert!(shadow_report.top_k_overlap >= 0.0);

        // 3. with_default_monitor
        let selector_config = ApprhSelectorConfig::default();
        let selector = ApprhSelector::with_default_monitor(selector_config)
            .expect("with_default_monitor failed");
        assert_eq!(selector.mode(), contextra_graph::apprh::ApprhMode::Shadow);
    }

    // 4. as_arc
    let vec_data = vec![10u64, 20u64, 30u64];
    let arc_slice = ArcSlice::from_vec(vec_data);
    let backing_arc: &Arc<[u64]> = arc_slice.as_arc();
    assert_eq!(backing_arc.len(), 3);

    // 5. recommended_multiplier
    let calibrator = PprCostCalibrator::new();
    assert_eq!(calibrator.recommended_multiplier(), None);
    calibrator.record(PprCostSample {
        algorithm: PprAlgorithm::ForwardPush,
        estimated_cost: 1000.0,
        measured_edge_accesses: 1000,
    });
    let mult = calibrator.recommended_multiplier().expect("multiplier expected");
    assert!((mult - 1.0).abs() < 1e-6);

    // 6. log_tl_hfd_vs_baseline_discrepancy
    let mut baseline = AHashMap::new();
    baseline.insert(e1, 0.8f32);
    let mut candidate = AHashMap::new();
    candidate.insert(e1, 0.6f32);

    let sink = TestShadowSink::default();
    log_tl_hfd_vs_baseline_discrepancy(&baseline, &candidate, 1001, &sink);
    let rec = sink.recorded.lock().unwrap();
    assert_eq!(rec.len(), 1);
    assert_eq!(rec[0].context_id, 1001);

    // 7. with_exclude_seeds
    let ppr_config = PprConfig::default();
    let mut stream = PprCandidateStream::new(&graph, vec![e1], ppr_config).with_exclude_seeds(true);
    let batch = stream.next_batch().await.expect("stream next_batch failed");
    assert!(batch.iter().all(|(eid, _)| *eid != e1));

    // 8, 9, 10. append_step, children_of, set_active_head
    let tree = SessionBranchTree::new("Root".into(), "Root response".into());
    let step1 = tree
        .append_step("Step 1".into(), "Resp 1".into(), None, vec![], "main")
        .expect("append_step failed");
    let step2 = tree
        .append_step("Step 2".into(), "Resp 2".into(), None, vec![], "main")
        .expect("append_step failed");

    let children = tree.children_of(step1);
    assert_eq!(children, vec![step2]);

    tree.set_active_head(step1).expect("set_active_head failed");
    assert_eq!(tree.active_head(), step1);

    // 11. thresholded_local_hfd
    let tl_hfd_params = TlHfdParams::default();
    let tl_result = graph
        .thresholded_local_hfd(&seeds, &tl_hfd_params)
        .expect("thresholded_local_hfd failed");
    assert!(!tl_result.is_empty(), "thresholded_local_hfd result must not be empty");
}
