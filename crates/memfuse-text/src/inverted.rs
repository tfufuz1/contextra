// FILE-CONTEXT: LSM-backed Inverted Index & Transactional Storage.
// ZWECK: Speichert Postings-Listen, Dokumentlängen und BM25-Statistiken transaktional im StorageEngine.
// INVARIANTEN: upsert_document und search_bm25_at beachten MAX_TEXT_BYTES; Lock-Hierarchie: commit_lock (parking_lot::Mutex) > staged_stats (parking_lot::Mutex); k <= MAX_SEARCH_K.
// NICHT-OFFENSICHTLICH: Key-Prefixes: "i:" (Inverted), "f:" (Forward), "dl:" (Doc Length), "fw:" (Forward Words), "meta:stats".
// HOTSPOTS: upsert_document, search_bm25_at, commit_stats
// STAND: TS:2026-09-10T19:25:46Z (SESSION: c844907e)

//! LSM-backed Inverted Index.
// CONSTRAINT: Inverted Index Key-Gen & Cache
// TARGET: < 20µs für upsert_document
// AKTUELL: ~18.6 µs (nach Optimierung)
// VORHER: 24.6 µs → NACHHER: 18.6 µs (~24% gain)
// BOTTLENECK: Heap-Allokationen (format!, Vec::new)
// OPTIMIERUNG: itoa::Buffer + Vec::with_capacity + doc_len_cache

mod index_struct;
mod morph_index;
mod types;

#[cfg(test)]
mod tests;

pub use index_struct::InvertedIndex;
pub use morph_index::BM25MorphIndex;
pub use types::{Language, TextIndexMetadata};
