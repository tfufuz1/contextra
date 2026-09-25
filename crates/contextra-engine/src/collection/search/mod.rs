// FILE-CONTEXT
// ZWECK: Suchoperationen (Vector-, Text-, Graph- & Hybrid-Retrieval) für Collection modularisiert nach Familien.
// INVARIANTEN: Snapshot-Pinning garantiert Isolation während gefilterter Suche; MAX_SEARCH_K Obergrenze.
// NICHT-OFFENSICHTLICH: Multi-Signal RRF vereint Ergebnisse ohne inkompatible Score-Skalen. Collection::query() ist der empfohlene Einstiegspunkt.
// STAND: TS:2026-08-30T21:15:00Z (SESSION: 0dcb9f3b)

//! Search operations for `Collection`.
//!
//! **Empfohlener Einstiegspunkt**: [`Collection::query()`] liefert einen [`HybridQueryBuilder`](crate::HybridQueryBuilder)
//! als Fluent-API für Vektor-, Text-, Graph- und Hybrid-Suchen. Die direkten `search_*`-Methoden sind deprecated.

mod basic;
mod checkpoint;
mod filtered;
mod hybrid;
mod hydrate;

pub use checkpoint::{
    with_pinned_checkpoint, with_pinned_checkpoint_and_guard, with_pinned_checkpoint_at_latest,
    CheckpointPinGuard,
};

use super::{extract_effective_importance, Collection, StoredDocument, StoredDocumentMeta};
pub use crate::temporal_filter::{
    apply_temporal_validity_filter, apply_temporal_validity_filter_at, FusionResult,
};
