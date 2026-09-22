use arc_swap::ArcSwap;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use crate::consistency_enforcement::{ConsistencyEnforcer, EdgeAssertion};
use crate::GraphIndexExt;
use memfuse_core::{
    BoxFuture, DocId, Entity, EntityId, GraphIndex, GraphIndexStats, MemFuseError, Result,
    StorageEngine, TxId,
};

use super::types::Edge;

/// Untere Schranke für Wall-Clock-abgeleitete TxId-Heuristik.
///
/// Unix-Nanosekunden seit Epoch lagen am 01-01-2014 bei ca. 1.39×10¹⁸.
/// TxIds, die in diesen Bereich fallen **und** unterhalb von `INTERNAL_BASE`
/// liegen, sind höchstwahrscheinlich wall-clock-abgeleitet und verletzen das
/// TxId-Origin-Invariant (AGT-GRAPH-001).
///
/// Wert gewählt als `1_400_000_000 * 1_000_000_000` (1. Jan 2014 UTC in ns).
pub const WALLCLOCK_TX_HEURISTIC_MIN: u64 = 1_400_000_000_000_000_000;

/// Prüft, ob `tx` aus einem verdächtigen (wall-clock-ähnlichen) Bereich stammt.
///
/// Gibt `true` zurück, wenn `tx` zwischen [`WALLCLOCK_TX_HEURISTIC_MIN`] und
/// `TxId::INTERNAL_BASE` liegt — ein Bereich, in dem keine kanonische
/// Collection-Sequenz operiert, aber Unix-Nanosekunden-Werte liegen würden.
///
/// # Invarianten-Kontext (AGT-GRAPH-001)
/// Die Invariante AGT-GRAPH-001 ist graph-spezifisch: Graph-Traversal, Personalized PageRank (PPR),
/// bi-temporale Gültigkeitsfenster (`valid_from`, `valid_to`) und kausale `rollback_to_tx()`-Operationen
/// hängen strikt von geordneten Transaktions-Sequenzen ab. VectorIndex (HNSW) und TextIndex verwalten
/// Dokumenten-Updates hingegen über `TxBuffer` und atomare Snapshots/Tombstones.
///
/// # AI-NOTE[BOUNDARY-MISSING][MAJOR]
/// KONTEXT: AGT-GRAPH-001 — add_entity/add_edge/commit akzeptieren TxIds ohne
///   Laufzeit-Fehlerablehnung, erzwingen jedoch in Debug-Builds `debug_assert!(tx.is_valid_origin())`.
/// ANWEISUNG: Bei verdächtigen TxIds => tracing::warn! loggen. In Debug-Builds schlägt debug_assert! fehl.
/// ID: AGT-GRAPH-001
#[inline]
pub fn is_suspicious_tx_id(tx: TxId) -> bool {
    let v = tx.inner();
    v == 0 || (WALLCLOCK_TX_HEURISTIC_MIN..TxId::INTERNAL_BASE).contains(&v)
}

/// Prüft ob eine Kante bezogen auf die Transaktionszeit (MVCC / Systemzeit) zum Zeitpunkt `as_of` sichtbar ist.
#[inline]
pub fn is_edge_visible(
    tx_valid_from: Option<TxId>,
    tx_valid_to: Option<TxId>,
    as_of: TxId,
) -> bool {
    tx_valid_from.is_none_or(|vf| vf <= as_of) && tx_valid_to.is_none_or(|vt| as_of < vt)
}

/// Prüft ob eine Kante bezogen auf die Businesszeit zum Zeitpunkt `business_as_of` gültig ist.
#[inline]
pub fn is_edge_visible_business(
    business_valid_from: Option<i64>,
    business_valid_to: Option<i64>,
    business_as_of: i64,
) -> bool {
    business_valid_from.is_none_or(|vf| vf <= business_as_of)
        && business_valid_to.is_none_or(|vt| business_as_of < vt)
}

/// Prüft bi-temporale Sichtbarkeit einer Kante (unabhängige Auswertung von System- und Businesszeit).
///
/// Business-Zeit wird nur ausgewertet, wenn `business_as_of` angegeben ist UND mindestens
/// ein Business-Zeit-Feld (`business_valid_from` oder `business_valid_to`) auf der Kante gesetzt ist.
#[inline]
pub fn is_edge_visible_bitemporal(
    tx_valid_from: Option<TxId>,
    tx_valid_to: Option<TxId>,
    as_of_tx: TxId,
    business_valid_from: Option<i64>,
    business_valid_to: Option<i64>,
    as_of_business: Option<i64>,
) -> bool {
    if !is_edge_visible(tx_valid_from, tx_valid_to, as_of_tx) {
        return false;
    }
    if let Some(b_as_of) = as_of_business {
        if business_valid_from.is_some() || business_valid_to.is_some() {
            return is_edge_visible_business(business_valid_from, business_valid_to, b_as_of);
        }
    }
    true
}
