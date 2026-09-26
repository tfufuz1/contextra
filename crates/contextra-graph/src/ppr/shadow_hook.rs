//! Shadow mode hook for comparing TL-HFD PPR scores against baseline PPR scores (§12).

use ahash::AHashMap;
use contextra_adapt::{ShadowDiscrepancy, ShadowSink};
use contextra_types::EntityId;

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
}
