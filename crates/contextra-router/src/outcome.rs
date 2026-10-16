// FILE-CONTEXT
// STAND: 2026-09-10T19:16:25Z (SESSION: 3f3e4637)
// ZWECK: Outcome-Typen und DecisionId-Identifier für konformale Router-Kalibrierung.
// INVARIANTEN: DecisionId-Monotonie via AtomicU64; Non-Conformity-Scores in [0.0, 1.0].
// SIEHE AUCH: docs/decisions/ADR-020-contextra-brain.md, rules/tag_taxonomy.md

//! Outcome types and decision tracking identifiers for router calibration.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Eindeutige ID einer Routing-Entscheidung.
/// Wird von route() zurückgegeben und von record_outcome() konsumiert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DecisionId(u64);

impl DecisionId {
    /// Erzeugt eine `DecisionId` aus einem Rohwert.
    pub const fn from_raw(raw: u64) -> Self {
        DecisionId(raw)
    }

    /// Liefert den zugrundeliegenden `u64`-Wert der `DecisionId`.
    pub const fn inner(self) -> u64 {
        self.0
    }
}

/// Instanzgebundener Generator für monoton steigende `DecisionId`s (Spec §3 P29).
#[derive(Debug)]
pub struct DecisionIdGenerator {
    next: AtomicU64,
}

impl DecisionIdGenerator {
    /// Erstellt einen neuen Generator mit dem angegebenen Startwert.
    pub fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }

    /// Erzeugt die nächste monotone `DecisionId`.
    pub fn next(&self) -> DecisionId {
        DecisionId::from_raw(self.next.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for DecisionIdGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Tatsächliches Ergebnis einer getroffenen Routing-Entscheidung.
/// Wird vom Agent-Orchestrator NACH dem SLM-Aufruf geliefert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingOutcome {
    /// SLM hat die Anfrage vollständig und korrekt beantwortet.
    Success,
    /// SLM-Antwort war unzureichend — Eskalation zu größerem Modell war nötig.
    Escalated { escalated_to: String },
    /// Nachgelagerter Evaluator/Judge hat die SLM-Antwort als falsch markiert.
    Rejected { reason: Option<String> },
}

impl RoutingOutcome {
    /// Non-Conformity-Score: 0.0 = perfekt, 1.0 = komplett falsch.
    /// Diese Werte sind Expertenschätzungen und sollten durch A/B-Tests kalibriert werden.
    pub fn non_conformity_score(&self) -> f32 {
        match self {
            RoutingOutcome::Success => 0.0,
            RoutingOutcome::Escalated { .. } => 0.7,
            RoutingOutcome::Rejected { .. } => 1.0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;

    #[test]
    fn test_decision_id_generator_instance_independence() {
        let gen_a = DecisionIdGenerator::new(0);
        let gen_b = DecisionIdGenerator::new(0);

        let id_a0 = gen_a.next();
        let id_b0 = gen_b.next();

        assert_eq!(id_a0.inner(), 0);
        assert_eq!(id_b0.inner(), 0);

        let id_a1 = gen_a.next();
        let id_b1 = gen_b.next();

        assert_eq!(id_a1.inner(), 1);
        assert_eq!(id_b1.inner(), 1);
    }

    #[test]
    fn test_decision_id_generator_concurrency() {
        let gen = Arc::new(DecisionIdGenerator::new(0));
        let num_threads = 8;
        let ids_per_thread = 1_000;
        let mut handles = Vec::new();

        for _ in 0..num_threads {
            let gen_clone = Arc::clone(&gen);
            handles.push(std::thread::spawn(move || {
                let mut local_ids = Vec::with_capacity(ids_per_thread);
                for _ in 0..ids_per_thread {
                    local_ids.push(gen_clone.next().inner());
                }
                local_ids
            }));
        }

        let mut all_ids = HashSet::new();
        for handle in handles {
            let thread_ids = handle.join().expect("thread join ok");
            for id in thread_ids {
                assert!(
                    all_ids.insert(id),
                    "Duplicate DecisionId generated under concurrency: {id}"
                );
            }
        }

        assert_eq!(all_ids.len(), num_threads * ids_per_thread);
        for expected in 0..(num_threads * ids_per_thread) as u64 {
            assert!(
                all_ids.contains(&expected),
                "Missing expected DecisionId: {expected}"
            );
        }
    }
}
