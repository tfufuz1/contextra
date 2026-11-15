// FILE-CONTEXT: Deutsche Morphologie & Komposita-Zerlegung.
// ZWECK: Umlaut-Normalisierung (ä->ae, ö->oe, ü->ue, ß->ss) und Zerlegung deutscher Zusammensetzungen.
// INVARIANTEN: Decompose arbeitet ausschließlich auf Kleinbuchstaben und prüft Mindestkomponentenlänge.
// NICHT-OFFENSICHTLICH: KMU-Wörterbuch aus data/german_words.txt via include_str! geladen.
// HOTSPOTS: GermanCompoundSplitter::decompose, normalize_umlauts
// STAND: TS:2026-09-10T19:21:35Z (SESSION: 4dd1c98c)

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
