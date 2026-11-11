//! # Support Module für Compaction Differenztests (Ring 1)
//!
//! Herkunfts-Commit: fcd387021e19923facf673ca4de28fc7f5863d2d

use contextra_store::compaction::adaptive::{CostBasedAdaptivePlanner, WorkloadMetricsSnapshot};

/// Referenz-Auswertung des Lese-Schreib-Verhältnisses aus dem Ursprungs-Commit.
pub fn reference_read_write_ratio(snapshot: &WorkloadMetricsSnapshot) -> f64 {
    if snapshot.write_count == 0 {
        if snapshot.read_count > 0 {
            f64::INFINITY
        } else {
            1.0
        }
    } else {
        snapshot.read_count as f64 / snapshot.write_count as f64
    }
}

/// Referenz-Erstellung eines Standard CostBasedAdaptivePlanner.
pub fn reference_default_planner() -> CostBasedAdaptivePlanner {
    CostBasedAdaptivePlanner::new(0.70, 4, 4.0)
}
