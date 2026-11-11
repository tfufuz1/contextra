#![allow(deprecated)]
// FILE-CONTEXT: LSM-backed Inverted Index & Transactional Storage.
// ZWECK: Speichert Postings-Listen, Dokumentlängen und BM25-Statistiken transaktional im StorageEngine.
// INVARIANTEN: upsert_document und search_bm25_at beachten MAX_TEXT_BYTES; Lock-Hierarchie: commit_lock (parking_lot::Mutex) > staged_stats (parking_lot::Mutex); k <= MAX_SEARCH_K.
// NICHT-OFFENSICHTLICH: Key-Prefixes: "i:" (Inverted), "f:" (Forward), "dl:" (Doc Length), "fw:" (Forward Words), "meta:stats".
// HOTSPOTS: upsert_document, search_bm25_at, commit_stats
// STAND: TS:2026-09-10T19:25:46Z (SESSION: c844907e)

//! LSM-backed Inverted Index.

mod index_struct;
#[cfg(feature = "workspace")]
pub use crate::compat::morph_index;
#[cfg(feature = "workspace")]
pub use crate::compat::purge;
#[cfg(feature = "workspace")]
pub use crate::compat::scanner;
pub(crate) mod types;

// Tests moved to compat/inverted_tests.rs

pub use index_struct::InvertedIndex;
#[cfg(feature = "workspace")]
pub use morph_index::BM25MorphIndex;
#[cfg(feature = "workspace")]
pub use purge::{text_purger, PurgeReport, TextPurger};
#[cfg(feature = "workspace")]
pub use scanner::{text_scanner, ParsedTextTarget, TextScanner};
pub use types::{Language, TextIndexMetadata};
