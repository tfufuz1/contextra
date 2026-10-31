//! Integration and unit tests for `ApprhGateMonitor`.

#![cfg(feature = "apprh-diffusion")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::apprh::{
    shadow_compare_forward_push_vs_apprh, ApprhError, ApprhGateMonitor, ApprhGateMonitorConfig,
    ApprhParams, ApprhShadowComparison,
};
use contextra_graph::path_rag::{PathGraph, PprParams};
use contextra_graph::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::EntityId;
use std::collections::HashMap;
use std::sync::Arc;

struct MockGraph {
    binary_edges: HashMap<EntityId, Vec<(EntityId, f32)>>,
    hyperedges: HashMap<HyperEdgeId, Arc<HyperEdge>>,
    node_to_hyperedges: HashMap<EntityId, Vec<HyperEdgeId>>,
}

impl MockGraph {
    fn new() -> Self {
        Self {
            binary_edges: HashMap::new(),
            hyperedges: HashMap::new(),
            node_to_hyperedges: HashMap::new(),
        }
    }

    fn add_binary_edge(&mut self, u: EntityId, v: EntityId, w: f32) {
        self.binary_edges.entry(u).or_default().push((v, w));
        self.binary_edges.entry(v).or_default().push((u, w));
    }

    fn add_hyperedge(&mut self, id: u64, participants: Vec<EntityId>, w: f32) {
        let hid = HyperEdgeId::new(id);
        let roles: Vec<RoleBinding> = participants
            .into_iter()
            .enumerate()
            .map(|(idx, entity)| RoleBinding::new(RoleId::new(idx as u32 + 1), entity))
            .collect();

        let he = Arc::new(HyperEdge::new(
            hid,
            contextra_graph::csr::EdgeType::Default,
            roles.clone(),
            w,
        ));

        self.hyperedges.insert(hid, he);
        for r in roles {
            self.node_to_hyperedges
                .entry(r.entity)
                .or_default()
                .push(hid);
        }
    }
}

impl PathGraph for MockGraph {
    fn neighbors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.binary_edges.get(&node).cloned().unwrap_or_default()
    }

    fn predecessors_with_weights(&self, node: EntityId) -> Vec<(EntityId, f32)> {
        self.neighbors_with_weights(node)
    }

    fn hyperedges_for_entity(&self, node: EntityId) -> Vec<HyperEdgeId> {
        self.node_to_hyperedges
            .get(&node)
            .cloned()
            .unwrap_or_default()
    }

    fn get_hyperedge(&self, id: HyperEdgeId) -> Option<Arc<HyperEdge>> {
        self.hyperedges.get(&id).cloned()
    }
}

fn make_sample_report(
    max_abs_diff: f32,
    top_k_overlap: f32,
    discrepancy: bool,
) -> ApprhShadowComparison {
    ApprhShadowComparison {
        max_abs_diff,
        top_k_overlap,
        discrepancy,
        forward_push_results: vec![(EntityId::new(1), 0.5), (EntityId::new(2), 0.5)],
        apprh_results: vec![(EntityId::new(1), 0.5), (EntityId::new(2), 0.5)],
    }
}

#[test]
fn test_gate_monitor_streak_logic_and_production_readiness() {
    let config = ApprhGateMonitorConfig {
        window_size: 10,
        required_consecutive_passes: 5,
    };
    let monitor = ApprhGateMonitor::new(config).expect("valid config");

    assert!(!monitor.production_ready());

    let valid_report = make_sample_report(0.01, 0.90, false);

    for i in 1..=4 {
        monitor.observe(&valid_report);
        let snap = monitor.snapshot();
        assert_eq!(snap.current_streak, i);
        assert_eq!(snap.total_observations, i as u64);
        assert!(!monitor.production_ready());
    }

    // 5th consecutive valid observation triggers production_ready
    monitor.observe(&valid_report);
    let snap = monitor.snapshot();
    assert_eq!(snap.current_streak, 5);
    assert_eq!(snap.total_observations, 5);
    assert!(monitor.production_ready());
    assert!(snap.production_ready);
    assert_eq!(snap.last_overlap, Some(0.90));
}

#[test]
fn test_gate_monitor_outlier_resets_streak_hysteresis() {
    let config = ApprhGateMonitorConfig {
        window_size: 10,
        required_consecutive_passes: 3,
    };
    let monitor = ApprhGateMonitor::new(config).expect("valid config");

    let valid_report = make_sample_report(0.01, 0.90, false);
    let discrepancy_report = make_sample_report(0.05, 0.90, true);
    let low_overlap_report = make_sample_report(0.01, 0.70, false);

    // Reach production ready
    for _ in 0..3 {
        monitor.observe(&valid_report);
    }
    assert!(monitor.production_ready());
    assert_eq!(monitor.snapshot().current_streak, 3);

    // Outlier 1: discrepancy flag set
    monitor.observe(&discrepancy_report);
    assert!(!monitor.production_ready());
    assert_eq!(monitor.snapshot().current_streak, 0);

    // Build streak back up
    monitor.observe(&valid_report);
    monitor.observe(&valid_report);
    assert_eq!(monitor.snapshot().current_streak, 2);
    assert!(!monitor.production_ready());

    // Outlier 2: overlap below threshold (0.70 < 0.85)
    monitor.observe(&low_overlap_report);
    assert!(!monitor.production_ready());
    assert_eq!(monitor.snapshot().current_streak, 0);

    // 3 clean passes required to recover
    for _ in 0..3 {
        monitor.observe(&valid_report);
    }
    assert!(monitor.production_ready());
    assert_eq!(monitor.snapshot().current_streak, 3);
}

#[test]
fn test_gate_monitor_window_bounds_and_snapshot() {
    let config = ApprhGateMonitorConfig {
        window_size: 3,
        required_consecutive_passes: 2,
    };
    let monitor = ApprhGateMonitor::new(config).expect("valid config");

    // Initially snapshot is clean
    let snap0 = monitor.snapshot();
    assert_eq!(snap0.total_observations, 0);
    assert_eq!(snap0.current_streak, 0);
    assert_eq!(snap0.last_overlap, None);
    assert_eq!(snap0.max_abs_diff_in_window, 0.0);
    assert_eq!(snap0.min_overlap_in_window, 1.0);
    assert_eq!(snap0.discrepancy_count_in_window, 0);

    let r1 = make_sample_report(0.02, 0.95, false);
    let r2 = make_sample_report(0.05, 0.88, false);
    let r3 = make_sample_report(0.12, 0.80, true); // discrepancy
    let r4 = make_sample_report(0.01, 0.99, false);

    monitor.observe(&r1);
    monitor.observe(&r2);
    monitor.observe(&r3);

    let snap = monitor.snapshot();
    assert_eq!(snap.total_observations, 3);
    assert_eq!(snap.max_abs_diff_in_window, 0.12);
    assert_eq!(snap.min_overlap_in_window, 0.80);
    assert_eq!(snap.discrepancy_count_in_window, 1);
    assert_eq!(snap.last_overlap, Some(0.80));

    // 4th observation pushes out r1 (window size = 3)
    monitor.observe(&r4);
    let snap2 = monitor.snapshot();
    assert_eq!(snap2.total_observations, 4);
    // Window now contains [r2, r3, r4]
    assert_eq!(snap2.max_abs_diff_in_window, 0.12);
    assert_eq!(snap2.min_overlap_in_window, 0.80);
    assert_eq!(snap2.discrepancy_count_in_window, 1);
    assert_eq!(snap2.last_overlap, Some(0.99));
}

#[test]
fn test_gate_monitor_invalid_configuration() {
    let err_zero_window = ApprhGateMonitorConfig {
        window_size: 0,
        required_consecutive_passes: 5,
    };
    assert!(matches!(
        ApprhGateMonitor::new(err_zero_window),
        Err(ApprhError::InvalidParameter(_))
    ));

    let err_zero_passes = ApprhGateMonitorConfig {
        window_size: 10,
        required_consecutive_passes: 0,
    };
    assert!(matches!(
        ApprhGateMonitor::new(err_zero_passes),
        Err(ApprhError::InvalidParameter(_))
    ));

    let err_passes_exceed_window = ApprhGateMonitorConfig {
        window_size: 5,
        required_consecutive_passes: 10,
    };
    assert!(matches!(
        ApprhGateMonitor::new(err_passes_exceed_window),
        Err(ApprhError::InvalidParameter(_))
    ));
}

#[test]
fn test_gate_monitor_reset() {
    let config = ApprhGateMonitorConfig {
        window_size: 10,
        required_consecutive_passes: 3,
    };
    let monitor = ApprhGateMonitor::new(config).expect("valid config");

    let report = make_sample_report(0.01, 0.90, false);
    for _ in 0..5 {
        monitor.observe(&report);
    }
    assert!(monitor.production_ready());
    assert_eq!(monitor.snapshot().total_observations, 5);

    monitor.reset();

    let snap = monitor.snapshot();
    assert_eq!(snap.total_observations, 0);
    assert_eq!(snap.current_streak, 0);
    assert_eq!(snap.last_overlap, None);
    assert_eq!(snap.max_abs_diff_in_window, 0.0);
    assert_eq!(snap.min_overlap_in_window, 1.0);
    assert_eq!(snap.discrepancy_count_in_window, 0);
    assert!(!snap.production_ready);
}

#[test]
fn test_gate_monitor_concurrency() {
    let config = ApprhGateMonitorConfig {
        window_size: 50,
        required_consecutive_passes: 20,
    };
    let monitor = Arc::new(ApprhGateMonitor::new(config).expect("valid config"));

    let num_threads = 8;
    let obs_per_thread = 100;
    let mut handles = Vec::new();

    for _ in 0..num_threads {
        let mon = Arc::clone(&monitor);
        let handle = std::thread::spawn(move || {
            let report = make_sample_report(0.01, 0.90, false);
            for _ in 0..obs_per_thread {
                mon.observe(&report);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("thread join succeeded");
    }

    let snap = monitor.snapshot();
    assert_eq!(
        snap.total_observations,
        (num_threads * obs_per_thread) as u64
    );
    assert!(snap.production_ready);
}

#[test]
fn test_gate_monitor_integration_with_real_shadow_comparison(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_binary_edge(e2, e3, 1.0);
    g.add_hyperedge(10, vec![e1, e2, e3], 1.0);

    let seeds = vec![e1];
    let ppr = PprParams::default();
    let apprh_params = ApprhParams::default();

    let comparison = shadow_compare_forward_push_vs_apprh(&g, &seeds, &ppr, &apprh_params, 5)?;

    let monitor = ApprhGateMonitor::new(ApprhGateMonitorConfig {
        window_size: 5,
        required_consecutive_passes: 1,
    })?;

    monitor.observe(&comparison);

    let snap = monitor.snapshot();
    assert_eq!(snap.total_observations, 1);
    assert!(snap.last_overlap.is_some());

    Ok(())
}
