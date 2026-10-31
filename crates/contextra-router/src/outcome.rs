// FILE-CONTEXT
// STAND: 2026-10-29
// ZWECK: Outcome-Typen und DecisionId-Identifier für konformale Router-Kalibrierung.
// INVARIANTEN: DecisionId-Monotonie via AtomicU64; Non-Conformity-Scores in [0.0, 1.0].
// SIEHE AUCH: docs/decisions/ADR-020-contextra-brain.md

//! # Outcome-Typen und Entscheidungs-Tracking für Router-Kalibrierung
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Eindeutige Nachverfolgung von Routing-Entscheidungen über instanzgebundene [`DecisionId`]s
//! und Protokollierung des tatsächlichen Ergebnisses ([`RoutingOutcome`]) nach erfolgter Modell-Generierung.
//!
//! **Hintergrund:**
//! Um Conformal Prediction (konforme Fehlerkalibrierung) zu betreiben, muss jede Entscheidung mit ihrem
//! späteren Ausgang (Erfolg, Eskalation oder Ablehnung) verknüpft werden. Der Non-Conformity-Score
//! quantifiziert hierbei den Fehlergrad im Bereich `[0.0, 1.0]`.
//!
//! **Abhängigkeiten:**
//! - Upstream: `serde`, `std::sync::atomic::AtomicU64`.
//! - Downstream: [`crate::router::RouterEngine`], Feedback-Loops in `contextra-agent`.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

// ANCHOR[contextra-router:OUT-001]
/// Eindeutige ID einer Routing-Entscheidung.
///
/// Wird von `RouterEngine::route()` erzeugt und vergeben und später von
/// `RouterEngine::record_outcome()` konsumiert, um das Feedback dem richtigen
/// Modellprofil zuzuordnen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DecisionId(u64);

impl DecisionId {
    // ANCHOR[contextra-router:OUT-002]
    /// Erzeugt eine [`DecisionId`] aus einem rohen 64-Bit-Ganzzahlwert.
    ///
    /// # Parameter
    /// - `raw`: Rohwert der ID.
    pub const fn from_raw(raw: u64) -> Self {
        DecisionId(raw)
    }

    // ANCHOR[contextra-router:OUT-003]
    /// Liefert den zugrundeliegenden `u64`-Wert der [`DecisionId`].
    pub const fn inner(self) -> u64 {
        self.0
    }
}

// ANCHOR[contextra-router:OUT-004]
/// Instanzgebundener Generator für monoton steigende [`DecisionId`]s (Spec §3 P29).
///
/// Garantiert Thread-Sicherheit über atomare Inkrement-Operationen ohne globale Sperren.
#[derive(Debug)]
pub struct DecisionIdGenerator {
    next: AtomicU64,
}

impl DecisionIdGenerator {
    // ANCHOR[contextra-router:OUT-005]
    /// Erstellt einen neuen Generator mit dem angegebenen Startwert.
    ///
    /// # Parameter
    /// - `start`: Anfangswert für die Sequenz (meist 0).
    pub fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }

    // ANCHOR[contextra-router:OUT-006]
    /// Erzeugt die nächste monotone [`DecisionId`].
    ///
    /// # Ablauf
    /// 1. Atomares Inkrement des Zählers mittels `fetch_add` mit `Ordering::Relaxed`.
    /// 2. Kapselung des alten Werts in einer `DecisionId`.
    ///
    /// # Vorbedingungen
    /// Keine.
    ///
    /// # Nachbedingungen
    /// Liefert eine innerhalb dieser Generator-Instanz strikt monotone ID zurück.
    ///
    /// # Nebenwirkungen
    /// Erhöht den internen Zählerzustand.
    // NOTE: Relaxed ordering genügt, da es sich um einen reinen Sequenzzähler ohne Synchronisationspflicht für andere Speicherstellen handelt.
    pub fn next(&self) -> DecisionId {
        DecisionId::from_raw(self.next.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for DecisionIdGenerator {
    // ANCHOR[contextra-router:OUT-007]
    /// Standardkonstruktor: Startet die Zählung bei 0.
    fn default() -> Self {
        Self::new(0)
    }
}

// ANCHOR[contextra-router:OUT-008]
/// Tatsächliches Ergebnis einer getroffenen Routing-Entscheidung.
///
/// Wird vom Agent-Orchestrator oder Evaluator NACH dem SLM-Aufruf an den Router zurückgemeldet.
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
    // ANCHOR[contextra-router:OUT-009]
    /// Berechnet den Non-Conformity-Score: 0.0 = perfekt konform, 1.0 = maximal non-konform / fehlerhaft.
    ///
    /// # Werte
    /// - [`RoutingOutcome::Success`] => `0.0`
    /// - [`RoutingOutcome::Escalated`] => `0.7`
    /// - [`RoutingOutcome::Rejected`] => `1.0`
    ///
    /// # Nachbedingungen
    /// Der Score liegt garantiert im Intervall `[0.0, 1.0]`.
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
    use std::collections::BTreeSet;
    use std::sync::Arc;

    // ANCHOR[contextra-router:OUT-T01]
    // Prüft, dass unabhängige Generator-Instanzen eigene Zähler führen und sich nicht gegenseitig beeinflussen.
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

    // ANCHOR[contextra-router:OUT-T02]
    // Prüft die nebenläufige Eindeutigkeit erzeugter IDs über mehrere Threads hinweg (keine Kollisionen).
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

        let mut all_ids = BTreeSet::new();
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
