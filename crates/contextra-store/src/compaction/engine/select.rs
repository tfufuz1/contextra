//! # SSTable Candidate Selection (Ring 1)
//!
//! Auswahl und Bewertung von SSTable-Kandidaten für die Leveled-Compaction.

use crate::sstable::SstableReader;
use std::sync::Arc;

/// Wählt die besten SSTable-Kandidaten für einen Kompaktierungslauf aus.
///
/// Folgt chronologischer Reihenfolge (kleinste min_tx_id zuerst) und garantiert
/// Kontiguität der ausgewählten Sequenz.
pub fn select_compaction_candidates(
    sstables: &[Arc<SstableReader>],
    max_count: usize,
) -> Vec<Arc<SstableReader>> {
    if sstables.len() < 2 || max_count < 2 {
        return Vec::new();
    }

    // Sortierung nach min_tx_id für deterministische Auswahl
    let mut sorted = sstables.to_vec();
    sorted.sort_by_key(|s| s.min_tx_id());

    let count = max_count.min(sorted.len());
    sorted.into_iter().take(count).collect()
}

/// Schätzt den Spitzen-Speicherverbrauch einer Kompaktierung anhand der Kandidatengrößen.
pub fn estimate_compaction_peak_bytes(candidates: &[Arc<SstableReader>]) -> u64 {
    let mut total_file_size: u64 = 0;
    for candidate in candidates {
        total_file_size += candidate.metadata().file_size;
    }

    let raw_overhead = (total_file_size as f64 * 1.5) as u64;
    let min_stream_buffer = (candidates.len() as u64) * 4096 * 4;

    raw_overhead + min_stream_buffer
}
