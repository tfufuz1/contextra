// FILE-CONTEXT: LSM-backed Inverted Index & Transactional Storage.
// ZWECK: Speichert Postings-Listen, Dokumentlängen und BM25-Statistiken transaktional im StorageEngine.
// INVARIANTEN: upsert_document und search_bm25_at beachten MAX_TEXT_BYTES; Lock-Hierarchie: commit_lock (parking_lot::Mutex) > staged_stats (parking_lot::Mutex); k <= MAX_SEARCH_K.
// NICHT-OFFENSICHTLICH: Key-Prefixes: "i:" (Inverted), "f:" (Forward), "dl:" (Doc Length), "fw:" (Forward Words), "meta:stats".
// HOTSPOTS: upsert_document, search_bm25_at, commit_stats
// STAND: TS:2026-09-10T19:25:46Z (SESSION: c844907e)

//! LSM-backed Inverted Index.
//! Persistenter, LSM-basierter invertierter Volltextindex mit MVCC-Snapshot-Isolation.
//!
//! # Ring-Zuordnung
//! Ring 0 (Kern).
//!
//! # Funktionsweise
//! Verwaltet Dokumente, Posting-Listen, Vorwärtsindex und BM25-Statistiken über dem
//! [`contextra_ports::StorageEngine`]-Port. Unterstützt Transaktionen (Staging, Commit, Rollback),
//! physische Löschung (Purge) und Scans für Verifiable Deletion.

mod index_struct;
mod morph_index;
pub mod purge;
mod types;

#[cfg(test)]
mod tests;

pub use index_struct::InvertedIndex;
pub use morph_index::BM25MorphIndex;
pub use purge::{ParsedTextTarget, PurgeReport, PurgeStats, ScanTarget, TextPurger};
pub use types::{Language, TextIndexMetadata};
