//! # Adaptive Kompaktierungsstrategie (Ring 1)
//!
//! Berechnet optimale Kompaktierungsplaene basierend auf Schreib-/Leselast und Verstaerkung.
//!
//! INVARIANTEN:
//! - P28 Determinismus-Zwang (WorkloadMetrics über TxId/seq_no & Ops-Counter, keine Wanduhrzeit)
//! - INV-COMPACTION-ADAPTIVE-1: Tombstone retention for active MVCC snapshots (min_active_seqno)

pub mod cost_based;
pub mod metrics;
pub mod planner;

pub use cost_based::*;
pub use metrics::*;
pub use planner::*;
