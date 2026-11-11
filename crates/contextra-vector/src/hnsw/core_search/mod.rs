// FILE-CONTEXT
// ZWECK: Suchtraversierung auf Schichten (Layer Search), gefilterte Suche und adaptive ef-Suche.
// INVARIANTEN: Zero-Panic; deterministische Sortierung (Score absteigend, DocId aufsteigend); k <= MAX_SEARCH_K.
// HOTSPOTS: search_layer_with_context, search_filtered_internal, search_adaptive_with_stats
// STAND: 2026-11-12

//! Suchpfad-Implementierung für den HNSW-Index.
//!
//! Dieses Modul gliedert die Suchtraversierung in modulare Phasen:
//! - [`validate`]: Prüfung von Dimension, K-Wert und Floating-Point-Gültigkeit (NaN/Inf).
//! - [`layer`]: Schichtbasierte Graph-Traversierung mit Kontext und Backlink-Tracking.
//! - [`filtered`]: Gefilterte Suche unter Berücksichtigung von Soft-Deletes und MVCC-Snapshots.
//! - [`adaptive`]: Adaptive ef-Suche (Ada-ef) zur dynamischen Tiefenanpassung.
//! - [`rerank`]: Exaktes Nachranking und Score-Berechnung nach Distanzmetrik.

pub(crate) mod adaptive;
pub(crate) mod filtered;
pub(crate) mod layer;
pub(crate) mod rerank;
pub(crate) mod validate;
