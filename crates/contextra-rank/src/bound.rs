//! Berechnung oberer Schranken für noch ungesehene Dokumente (RK-07).

use crate::error::RankError;
use crate::rrf::{RrfScore, K_RRF};

/// Berechnet eine obere Schranke für den Score eines Dokuments, das in jeder
/// Teilliste erst nach der Tiefe `depths[i]` auftreten kann.
///
/// Formel: $\sum_i \lfloor 2^{40} / (K\_RRF + depths[i] + 1) \rfloor$ (ganzzahlig).
///
/// # Errors
/// Gibt `RankError::Overflow` zurück, falls die Berechnung überläuft.
pub fn unseen_bound(depths: &[usize]) -> Result<RrfScore, RankError> {
    let mut total: u64 = 0;
    for &depth in depths {
        let rank = (depth as u64).checked_add(1).ok_or(RankError::Overflow)?;
        let denom = (K_RRF as u64)
            .checked_add(rank)
            .ok_or(RankError::Overflow)?;
        let contrib = (1u64 << 40).checked_div(denom).ok_or(RankError::Overflow)?;
        total = total.checked_add(contrib).ok_or(RankError::Overflow)?;
    }
    Ok(RrfScore(total))
}
