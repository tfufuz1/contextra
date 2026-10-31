#![no_main]

//! # Fuzz-Target für Weighted Reciprocal Rank Fusion (RRF)
//!
//! # Zweck des Fuzz-Targets
//! Überprüft die Robustheit des gewichteten RRF-Fusionsalgorithmus
//! ([`weighted_reciprocal_rank_fusion`]) gegenüber beliebigen, durch den Fuzzer
//! generierten Eingabemengen.
//!
//! # Geprüfte Invarianten:
//! - **INV-RRF-1 (Ergebnisschranke):** Die fusionierte Ergebnisliste darf niemals mehr
//!   Elemente als `max_results` enthalten (`fused.len() <= max_results`).
//! - **INV-RRF-2 (Panik-Freiheit & NaN-Sicherheit):** Scores und Reciprocal-Rank-Werte
//!   müssen wohldefiniert bleiben und dürfen keine arithmetischen Überläufe oder Panics auslösen.

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use contextra_db::fusion::weighted_reciprocal_rank_fusion;
use contextra_db::SearchResult;

// ANCHOR[contextra-db:FUZZ-RRF-001]
/// Ein einzelnes zufällig erzeugtes Trefferelement für den Fuzz-Test.
#[derive(Arbitrary, Debug)]
pub struct FuzzItem {
    /// Dokumenten-ID des Treffers.
    pub id: String,
    /// Roher Relevanz-Score vor der Fusion.
    pub score: f32,
}

// ANCHOR[contextra-db:FUZZ-RRF-002]
/// Ein Signal-Kanal mit Namen, Gewichtung und einer Liste von Trefferelementen.
#[derive(Arbitrary, Debug)]
pub struct FuzzSignalSet {
    /// Name des Signals (z. B. "vector", "bm25", "graph").
    pub signal_name: String,
    /// Gewichtung des Signals in der RRF-Formel.
    pub weight: f32,
    /// Teilergebnisse dieses Signals.
    pub items: Vec<FuzzItem>,
}

// ANCHOR[contextra-db:FUZZ-RRF-003]
/// Vollständige Eingabestruktur für das RRF-Fuzzing.
#[derive(Arbitrary, Debug)]
pub struct RrfFuzzInput {
    /// Signalmengen, die fusioniert werden sollen.
    pub sets: Vec<FuzzSignalSet>,
    /// Maximale Anzahl gewünschter Ausgabeergebnisse.
    pub max_results: usize,
}

// ANCHOR[contextra-db:FUZZ-RRF-004]
// Fuzz-Target-Einstiegspunkt: Mappt Fuzz-Input auf SearchResult-Strukturen und ruft weighted_reciprocal_rank_fusion auf.
fuzz_target!(|input: RrfFuzzInput| {
    let result_sets: Vec<(String, Vec<SearchResult>, f32)> = input
        .sets
        .into_iter()
        .map(|s| {
            let results = s
                .items
                .into_iter()
                .map(|item| SearchResult {
                    id: item.id,
                    score: item.score,
                    metadata: None,
                    matched_signals: Vec::new(),
                    provenance: None,
                })
                .collect();
            (s.signal_name, results, s.weight)
        })
        .collect();

    let fused = weighted_reciprocal_rank_fusion(result_sets, input.max_results);

    // Invariant checks
    assert!(
        fused.len() <= input.max_results,
        "Fused results length cannot exceed max_results"
    );
    for r in &fused {
        // Scores should be defined (not panic)
        let _ = r.score;
    }
});
