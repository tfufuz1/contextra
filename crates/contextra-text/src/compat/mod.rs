//! Kompatibilitätsmodul für Ports und Workspace-Integration (SPEC §6.2 text, B04).
// ANCHOR[contextra-text:COMPAT-002]

pub mod morph_index;
pub mod purge;
pub mod scanner;
pub mod scorer;
pub mod storage;

pub use morph_index::BM25MorphIndex;
pub use purge::{text_purger, PurgeReport, TextPurger};
pub use scanner::{text_scanner, ParsedTextTarget, TextScanner};
pub use scorer::Bm25Scorer;
