
//! # Morphologische Analyse & Komposita-Zerlegung (`morphology`)
//!
//! # Einordnung im Ring-Modell
//! Gehört zu **Ring 0 (Kern)**.
//!
//! # Zweck
//! Bereitstellung von Algorithmen zur linguistischen Vorverarbeitung deutscher Texte:
//! - Normalisierung von Umlauten (`ä -> ae`, `ö -> oe`, `ü -> ue`, `ß -> ss`).
//! - Zerlegung deutscher Mehrwortzusammensetzungen (Komposita) wie *Bundesverfassungsgericht*
//!   mittels Präfixbaum (Trie) und dynamischer Programmierung (DP).
//! - Verwaltung deutscher Stoppwortlisten und Messung von Token-Expansionsraten.

mod compound_splitter;
mod metrics;
mod passthrough;
mod stopwords;
mod trie;

#[cfg(test)]
mod tests;

pub use compound_splitter::{GermanCompoundSplitter, MorphologicalTokenizer};
pub use metrics::TokenReductionMetrics;
pub use passthrough::PassthroughTokenizer;
pub use stopwords::{get_german_stopwords, is_german_stopword, normalize_umlauts};
pub use trie::{Trie, TrieNode};
