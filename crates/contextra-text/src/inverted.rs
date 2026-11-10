#![allow(deprecated)]
// FILE-CONTEXT: LSM-backed Inverted Index & Transactional Storage.
// ZWECK: Speichert Postings-Listen, Dokumentlängen und BM25-Statistiken transaktional im StorageEngine.
// INVARIANTEN: upsert_document und search_bm25_at beachten MAX_TEXT_BYTES; Lock-Hierarchie: commit_lock (parking_lot::Mutex) > staged_stats (parking_lot::Mutex); k <= MAX_SEARCH_K.
// NICHT-OFFENSICHTLICH: Key-Prefixes: "i:" (Inverted), "f:" (Forward), "dl:" (Doc Length), "fw:" (Forward Words), "meta:stats".
// HOTSPOTS: upsert_document, search_bm25_at, commit_stats
// STAND: TS:2026-09-10T19:25:46Z (SESSION: c844907e)

//! LSM-backed Inverted Index.

mod index_struct;
// MIGRATION(S-70): morph_index moved to compat/
pub use crate::compat::morph_index;
// MIGRATION(S-70): purge moved to compat/
pub use crate::compat::purge;
// MIGRATION(S-70): scanner moved to compat/
pub use crate::compat::scanner;
pub(crate) mod types;

#[cfg(test)]
mod tests;

pub use index_struct::InvertedIndex;
pub use morph_index::BM25MorphIndex;
pub use purge::{text_purger, PurgeReport, TextPurger};
pub use scanner::{text_scanner, ParsedTextTarget, TextScanner};
pub use types::{Language, TextIndexMetadata};
