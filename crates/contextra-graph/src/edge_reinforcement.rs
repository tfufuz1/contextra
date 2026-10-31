//! F-03: Co-Occurrence- und Traversal-basierte Kantenverstärkung (Edge Weight Reinforcement Learning).

use crate::csr::CsrEdge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
// ANCHOR[contextra-graph:EDGEREINFORC-0198]
/// Struktur `EdgeReinforcementConfig` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub struct EdgeReinforcementConfig {
    pub eta: f32,
    pub delta: f32,
    pub w_max: f32,
    pub rho: f32,
    pub q: f32,
    pub alpha: f32,
}

impl Default for EdgeReinforcementConfig {
    fn default() -> Self {
        Self {
            eta: 0.01,
            delta: 0.001,
            w_max: 10.0,
            rho: 0.05,
            q: 1.0,
            alpha: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::EntityId;

    // Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
    #[test]
    fn test_compute_edge_weight_formula() {
        let score = compute_edge_weight(0.3, 0.7, 0.5);
        assert!((score - 0.5).abs() < f32::EPSILON);
    }

    // Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
    #[test]
    fn test_cooccurrence_reinforcement_decay() {
        let mut edge = CsrEdge::new(EntityId::new(1), 1.0);
        edge.cooccurrence_weight = 1.0;
        let config = EdgeReinforcementConfig::default();

        for _ in 0..1000 {
            apply_cooccurrence_reinforcement(&mut edge, 0.0, &config);
        }
        assert!(edge.cooccurrence_weight < 0.4);
        assert!(edge.cooccurrence_weight >= 0.0);
    }

    // Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
    #[test]
    fn test_weight_normalization_caps_w_max() {
        let mut edges = vec![
            CsrEdge::new(EntityId::new(1), 1.0),
            CsrEdge::new(EntityId::new(2), 1.0),
        ];
        edges[0].cooccurrence_weight = 8.0;
        edges[1].cooccurrence_weight = 8.0;

        let w_max = 10.0;
        apply_weight_normalization(&mut edges, w_max);

        let sum: f32 = edges.iter().map(|e| e.cooccurrence_weight).sum();
        assert!((sum - w_max).abs() < 1e-5);
        assert_eq!(edges[0].cooccurrence_weight, 5.0);
        assert_eq!(edges[1].cooccurrence_weight, 5.0);
    }

    // Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
    #[test]
    fn test_traversal_reinforcement_evaporation() {
        let mut edge = CsrEdge::new(EntityId::new(1), 1.0);
        edge.traversal_weight = 1.0;
        let config = EdgeReinforcementConfig::default();

        let initial_traversal = edge.traversal_weight;
        apply_traversal_reinforcement(&mut edge, 0, &config);
        assert!(edge.traversal_weight < initial_traversal);
        assert!((edge.traversal_weight - 0.95).abs() < 1e-5);
        assert!(edge.traversal_weight >= 0.0);
    }

    // Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
    #[test]
    fn test_compute_edge_weight_extreme_alpha() {
        assert_eq!(compute_edge_weight(0.8, 0.2, 1.0), 0.8);
        assert_eq!(compute_edge_weight(0.8, 0.2, 0.0), 0.2);
    }

    // Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
    #[test]
    fn test_apply_weight_normalization_zero_and_below_max() {
        let mut edges = vec![
            CsrEdge::new(EntityId::new(1), 1.0),
            CsrEdge::new(EntityId::new(2), 1.0),
        ];
        edges[0].cooccurrence_weight = 2.0;
        edges[1].cooccurrence_weight = 3.0;

        apply_weight_normalization(&mut edges, 10.0);
        assert_eq!(edges[0].cooccurrence_weight, 2.0);
        assert_eq!(edges[1].cooccurrence_weight, 3.0);

        let mut empty_edges: Vec<CsrEdge> = vec![];
        apply_weight_normalization(&mut empty_edges, 10.0);
    }

    // Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
    #[test]
    fn test_cooccurrence_reinforcement_triggers_normalization() {
        let mut edge = CsrEdge::new(EntityId::new(1), 1.0);
        edge.cooccurrence_weight = 9.9;
        let config = EdgeReinforcementConfig {
            eta: 1.0,
            w_max: 10.0,
            ..Default::default()
        };

        let needs_norm = apply_cooccurrence_reinforcement(&mut edge, 1.0, &config);
        assert!(needs_norm);
    }
}

#[inline]
// ANCHOR[contextra-graph:COMPUTE_EDGE-0199]
/// Funktion `compute_edge_weight` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub fn compute_edge_weight(cooccurrence_weight: f32, traversal_weight: f32, alpha: f32) -> f32 {
    alpha * cooccurrence_weight + (1.0 - alpha) * traversal_weight
}

// ANCHOR[contextra-graph:APPLY_COOCCU-0200]
/// Funktion `apply_cooccurrence_reinforcement` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub fn apply_cooccurrence_reinforcement(
    edge: &mut CsrEdge,
    co_activation: f32,
    config: &EdgeReinforcementConfig,
) -> bool {
    let delta = config.eta * co_activation - config.delta * edge.cooccurrence_weight;
    edge.cooccurrence_weight = (edge.cooccurrence_weight + delta).max(0.0);
    edge.cooccurrence_weight > config.w_max
}

// ANCHOR[contextra-graph:APPLY_TRAVER-0201]
/// Funktion `apply_traversal_reinforcement` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub fn apply_traversal_reinforcement(
    edge: &mut CsrEdge,
    path_length: usize,
    config: &EdgeReinforcementConfig,
) {
    let reinforcement = if path_length > 0 {
        config.q / (path_length as f32)
    } else {
        0.0
    };
    edge.traversal_weight = ((1.0 - config.rho) * edge.traversal_weight + reinforcement).max(0.0);
}

// ANCHOR[contextra-graph:APPLY_WEIGHT-0202]
/// Funktion `apply_weight_normalization` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub fn apply_weight_normalization(outgoing_edges: &mut [CsrEdge], w_max: f32) {
    let sum_w: f32 = outgoing_edges.iter().map(|e| e.cooccurrence_weight).sum();
    if sum_w > w_max && sum_w > 0.0 {
        let scale = w_max / sum_w;
        for edge in outgoing_edges {
            edge.cooccurrence_weight = (edge.cooccurrence_weight * scale).max(0.0);
        }
    }
}
