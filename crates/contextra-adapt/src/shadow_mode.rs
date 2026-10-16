//! Generic shadow mode discrepancy logging framework (§12 Integrationsregel).
//!
//! Stellt typunabhängige Datenstrukturen und Senken-Schnittstellen bereit, um
//! Diskrepanzen zwischen Baseline-Algorithmen und Kandidaten-Algorithmen im
//! `ShadowMode` diagnostisch zu protokollieren, ohne das dem Nutzer gelieferte
//! Ergebnis zu beeinflussen.

/// Diskrepanz-Eintrag: Baseline-Ergebnis vs. Kandidaten-Ergebnis, ohne Einfluss auf das
/// tatsächlich zurückgegebene Nutzerergebnis (§12 Integrationsregel).
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowDiscrepancy<T> {
    /// Das Ergebnis des etablierten Baseline-Algorithmus.
    pub baseline: T,
    /// Das Ergebnis des neuen Kandidaten-Algorithmus.
    pub candidate: T,
    /// Eindeutige Kontext- oder Anfrage-ID für das Tracing.
    pub context_id: u64,
}

/// Dyn-kompatibler Port (P27) für Diskrepanz-Senken (In-Memory für Tests, Tracing/Log für Produktion).
pub trait ShadowSink<T>: Send + Sync {
    /// Zeichnet einen Diskrepanz-Eintrag auf.
    fn record(&self, discrepancy: ShadowDiscrepancy<T>);
}

/// Referenzimplementierung über `tracing::debug!` — kein zusätzlicher I/O-Pfad, keine
/// Rng/Clock-Portverletzung (P28), da rein diagnostisch.
#[derive(Debug, Default, Clone, Copy)]
pub struct TracingShadowSink;

impl<T: std::fmt::Debug> ShadowSink<T> for TracingShadowSink {
    fn record(&self, d: ShadowDiscrepancy<T>) {
        tracing::debug!(
            context_id = d.context_id,
            baseline = ?d.baseline,
            candidate = ?d.candidate,
            "shadow_mode discrepancy"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test (a): TracingShadowSink protokolliert ohne zu paniken
    #[test]
    fn test_tracing_shadow_sink_does_not_panic() {
        let sink = TracingShadowSink;
        sink.record(ShadowDiscrepancy {
            baseline: 0,
            candidate: 1,
            context_id: 101,
        });
    }

    /// Test (b): Nicht-Beeinflussungs-Invariante
    /// `select_arm` mit konfiguriertem Shadow-Sink liefert exakt dasselbe Ergebnis wie ohne Sink.
    #[cfg(feature = "flow-corrected-thompson")]
    #[test]
    fn test_select_arm_non_influence_invariant() {
        use crate::flow_thompson::{
            FcTsArmSet, FcTsConfig, FlowCorrectedThompsonBandit, SplitMix64,
        };
        use std::sync::{Arc, Mutex};

        #[derive(Default)]
        struct MockRecordingSink {
            discrepancies: Arc<Mutex<Vec<ShadowDiscrepancy<u32>>>>,
        }

        impl ShadowSink<u32> for MockRecordingSink {
            fn record(&self, discrepancy: ShadowDiscrepancy<u32>) {
                if let Ok(mut lock) = self.discrepancies.lock() {
                    lock.push(discrepancy);
                }
            }
        }

        let config = FcTsConfig::default();
        let arm0_a = FlowCorrectedThompsonBandit::new(config.clone()).unwrap();
        let arm1_a = FlowCorrectedThompsonBandit::new(config.clone()).unwrap();
        let arm_set_baseline = FcTsArmSet {
            arms: vec![arm0_a, arm1_a],
        };

        let recording_sink = Arc::new(MockRecordingSink::default());

        let arm0_b = FlowCorrectedThompsonBandit::new(config.clone()).unwrap();
        let arm1_b = FlowCorrectedThompsonBandit::new(config.clone()).unwrap();
        let arm_set_shadow = FcTsArmSet {
            arms: vec![arm0_b, arm1_b],
        }
        .with_shadow_sink(recording_sink.clone());

        let mut rng_a = SplitMix64::new(999);
        let mut rng_b = SplitMix64::new(999);
        let ctx = vec![0.5f32; config.dim];

        for _ in 0..50 {
            let selected_baseline = arm_set_baseline
                .select_arm(&ctx, &mut rng_a)
                .expect("selection ok");
            let selected_shadow = arm_set_shadow
                .select_arm(&ctx, &mut rng_b)
                .expect("selection ok");

            assert_eq!(
                selected_baseline, selected_shadow,
                "Non-influence invariant violated: shadow mode changed selected arm"
            );
        }

        let recorded = recording_sink.discrepancies.lock().unwrap();
        assert_eq!(recorded.len(), 50);
    }
}
