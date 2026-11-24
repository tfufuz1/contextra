#![deny(missing_docs)]
#![allow(deprecated)]
#![deny(dead_code)]
// FILE-CONTEXT: Layer 1 text search integration facade (contextra-text).
// ZWECK: Exportiert Bm25Scorer, InvertedIndex, Morphologie-Tools und BM25-Modelle für DB-Hybrid-Suche.
// INVARIANTEN: #![forbid(unsafe_code)], TextIndex-Trait Implementierung ist fully async & transaction-aware.
// NICHT-OFFENSICHTLICH: Scorer delegiert direkt an InvertedIndex; MVCC & Lock-Free Storage durch StorageEngine.
// HOTSPOTS: Bm25Scorer::search, Bm25Scorer::insert
// STAND: TS:2026-09-09T13:22:45Z (SESSION: dc71d70)

//! Hybrid Search Engine & BM25 Scoring (WP-2.1)
//!
//! Evaluates Inverse Document Frequencies integrating natively into the
//! `fusion.rs` layer in `contextra-db`.
//!
//! # Einordnung in die Gesamtarchitektur (Ring-Modell)
//! Dieses Modul gehört zu **Ring 0 (Kern)** von Contextra. Es enthält grundlegende
//! Textindizierungs- und Volltextsuch-Primitive (BM25, WAND, morphologische Zerlegung
//! für Deutsch, invertierte Indizes und Posting-Listen).
//!
//! # Abhängigkeiten
//! - **Normal**: `contextra-ports`, `contextra-types`, sowie Hilfs-Bibliotheken (`bincode`, `blake3`,
//!   `bytes`, `futures`, `itoa`, `parking_lot`, `regex`, `serde`, `serde_json`, `thiserror`, `tracing`,
//!   `unicode-segmentation`).
//! - **Wird genutzt von**: `contextra-engine`, `contextra-db`, `contextra-router`, `contextra-agent`
//!   sowie Benchmarks (`contextra-bench`).
//!
//! # Haupttypen
//! - `Bm25Scorer`: Öffentliche Fassade zur Anbindung an den Port `TextIndex` (unter Feature `workspace`).
//! - [`InvertedIndex`]: Kern des persistenten invertierten Index mit MVCC-Snapshot-Isolation.
//! - [`BM25`] & [`BM25F`]: Mathematische Scoring-Modelle für Dokument-Relevanz.
//! - [`Bm25CandidateStream`]: Asynchroner Pull-Stream zur iterativen Ermittlung von Treffern.
//!
//! ## Scoring Models & Opt-in Features
//! - **Standard BM25**: Default scoring algorithm used for keyword and morphological search.
//! - **BM25F (Field-Weighted BM25)**: Opt-in field-weighted scoring model available via the
//!   `bm25f` Cargo feature.
//!
//! # Schnellstart (Standalone)
//! ```rust
//! use contextra_text::bm25::BM25;
//!
//! let bm25 = BM25::default();
//! let score = bm25.score_term(2, 50, 100.0, 5, 100);
//! assert!(score > 0.0);
//! ```

#![forbid(unsafe_code)]

#[cfg(feature = "types-compat")]
pub mod compat;
pub mod contract;
pub mod error;
pub mod memory_index;
pub mod storage;
pub mod traits;
pub mod types;

pub use contract::{Clock, DocKey, Epoch, Error as ContractError, Result as ContractResult, Rng};

pub use error::{Result as LocalResult, TextError};
pub use memory_index::{CompactStats, MemoryTextIndex};
pub use storage::PostingStore;
pub use traits::{Index, Maintain};
pub use types::{DocId as LocalDocId, TextHit, MAX_SEARCH_K as LOCAL_MAX_SEARCH_K};

pub mod bm25;
pub mod domain;
pub mod inverted;
pub mod morphology;
pub mod posting_list;
pub mod stream;
pub mod tokenizer;
pub mod wand;

pub use bm25::{score_term_bm25f, FieldId, FieldWeight, BM25, BM25F};
pub use domain::{DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary};
pub use inverted::{InvertedIndex, Language};
#[cfg(feature = "types-compat")]
pub use inverted::{
    text_purger, text_scanner, BM25MorphIndex, PurgeReport, TextPurger, TextScanner,
};
pub use morphology::{normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer};
pub use posting_list::{Posting, PostingList, ResidentPostingIndex};
pub use stream::{Bm25CandidateStream, DEFAULT_STREAM_BATCH_SIZE};
pub use tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};

#[cfg(feature = "types-compat")]
pub use crate::compat::scorer::Bm25Scorer;

