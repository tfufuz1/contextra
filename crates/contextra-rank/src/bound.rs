//! Berechnung oberer Schranken für noch ungesehene Dokumente (RK-07).

use crate::rrf::{RrfScore, K_RRF};

/// Berechnet eine obere Schranke für den Score eines Dokuments, das in jeder
/// Teilliste erst nach der Tiefe `depths[i]` auftreten kann.
///
/// Formel: $\sum_i \lfloor 2^{40} / (K\_RRF + depths[i] + 1) \rfloor$ (ganzzahlig).
#[must_use]
pub fn unseen_bound(depths: &[usize]) -> RrfScore {
    let mut total: u64 = 0;
    for &depth in depths {
        let rank = (depth as u64).saturating_add(1);
        let denom = (K_RRF as u64).saturating_add(rank);
        let contrib = if denom > 0 { (1u64 << 40) / denom } else { 0 };
        total = total.saturating_add(contrib);
    }
    RrfScore(total)
}
