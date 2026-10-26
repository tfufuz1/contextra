//! Integration tests for `ApprhSelector` shadow-to-production lifecycle and hysteresis.

#![cfg(feature = "apprh-diffusion")]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::apprh::{
    ApprhGateMonitor, ApprhGateMonitorConfig, ApprhMode, ApprhParams, ApprhSelector,
    ApprhSelectorConfig,
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

#[test]
fn test_selector_shadow_always_returns_forward_push() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_binary_edge(e2, e3, 1.0);
    g.add_hyperedge(100, vec![e1, e2, e3], 1.0);

    let seeds = vec![e1];
    let ppr = PprParams::default();
    let apprh_params = ApprhParams {
        hyperedge_decay_factor: 0.1,
        ..Default::default()
    };

    let monitor = Arc::new(ApprhGateMonitor::default());
    let selector_config = ApprhSelectorConfig {
        sample_interval: 1,
        max_seeds: 10,
        shadow_top_k: 5,
    };
    let selector = ApprhSelector::new(selector_config, monitor);

    assert_eq!(selector.mode(), ApprhMode::Shadow);

    let fp_expected = contextra_graph::path_rag::forward_push_ppr(&g, &seeds, &ppr);

    let res = selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;

    assert!(!res.is_empty());
    assert_eq!(res.len(), fp_expected.len());

    Ok(())
}

#[test]
fn test_selector_readiness_does_not_auto_promote() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);

    g.add_binary_edge(e1, e2, 1.0);

    let seeds = vec![e1];
    let ppr = PprParams::default();
    let apprh_params = ApprhParams::default();

    let monitor_config = ApprhGateMonitorConfig {
        window_size: 5,
        required_consecutive_passes: 2,
    };
    let monitor = Arc::new(ApprhGateMonitor::new(monitor_config)?);
    let selector = ApprhSelector::new(
        ApprhSelectorConfig {
            sample_interval: 1,
            max_seeds: 10,
            shadow_top_k: 5,
        },
        monitor.clone(),
    );

    for _ in 0..5 {
        selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    }

    assert!(monitor.production_ready());
    assert_eq!(selector.mode(), ApprhMode::Shadow);

    Ok(())
}

#[test]
fn test_selector_explicit_promotion_yields_apprh() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);
    let e3 = EntityId::new(3);

    g.add_binary_edge(e1, e2, 1.0);
    g.add_hyperedge(1, vec![e1, e2, e3], 2.0);

    let seeds = vec![e1];
    let ppr = PprParams::default();
    let apprh_params = ApprhParams::default();

    let monitor = Arc::new(ApprhGateMonitor::new(ApprhGateMonitorConfig {
        window_size: 5,
        required_consecutive_passes: 1,
    })?);
    let selector = ApprhSelector::new(ApprhSelectorConfig::default(), monitor.clone());

    assert!(selector.promote_to_production().is_err());

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert!(monitor.production_ready());

    selector.promote_to_production()?;
    assert_eq!(selector.mode(), ApprhMode::Production);

    let res = selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert!(!res.is_empty());

    Ok(())
}

#[test]
fn test_selector_outlier_causes_hysteresis_demotion() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);

    g.add_binary_edge(e1, e2, 1.0);

    let seeds = vec![e1];
    let ppr = PprParams::default();
    let apprh_params = ApprhParams::default();

    let monitor = Arc::new(ApprhGateMonitor::new(ApprhGateMonitorConfig {
        window_size: 5,
        required_consecutive_passes: 1,
    })?);
    let selector = ApprhSelector::new(ApprhSelectorConfig::default(), monitor.clone());

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert!(monitor.production_ready());

    selector.promote_to_production()?;
    assert_eq!(selector.mode(), ApprhMode::Production);

    let outlier_report = contextra_graph::apprh::ApprhShadowComparison {
        max_abs_diff: 0.99,
        top_k_overlap: 0.10,
        discrepancy: true,
        forward_push_results: vec![],
        apprh_results: vec![],
    };
    monitor.observe(&outlier_report);

    assert!(!monitor.production_ready());

    let res = selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert!(!res.is_empty());
    assert_eq!(selector.mode(), ApprhMode::Shadow);

    Ok(())
}

#[test]
fn test_selector_deterministic_sampling() -> Result<(), Box<dyn std::error::Error>> {
    let mut g = MockGraph::new();
    let e1 = EntityId::new(1);
    let e2 = EntityId::new(2);

    g.add_binary_edge(e1, e2, 1.0);

    let seeds = vec![e1];
    let ppr = PprParams::default();
    let apprh_params = ApprhParams::default();

    let monitor = Arc::new(ApprhGateMonitor::default());
    let selector = ApprhSelector::new(
        ApprhSelectorConfig {
            sample_interval: 3,
            max_seeds: 10,
            shadow_top_k: 5,
        },
        monitor.clone(),
    );

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert_eq!(monitor.snapshot().total_observations, 0);

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert_eq!(monitor.snapshot().total_observations, 0);

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert_eq!(monitor.snapshot().total_observations, 1);

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert_eq!(monitor.snapshot().total_observations, 1);

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert_eq!(monitor.snapshot().total_observations, 1);

    selector.evaluate_and_run(&g, &seeds, &ppr, &apprh_params)?;
    assert_eq!(monitor.snapshot().total_observations, 2);

    Ok(())
}
