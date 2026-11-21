#![allow(deprecated)]

//! LSM-backed Inverted Index.

mod index_struct;
#[cfg(feature = "types-compat")]
// MIGRATION(S-70): morph_index moved to compat/
pub use crate::compat::morph_index;
#[cfg(feature = "types-compat")]
// MIGRATION(S-70): purge moved to compat/
pub use crate::compat::purge;
#[cfg(feature = "types-compat")]
// MIGRATION(S-70): scanner moved to compat/
pub use crate::compat::scanner;
pub(crate) mod types;



pub use index_struct::InvertedIndex;
#[cfg(feature = "types-compat")]
pub use morph_index::BM25MorphIndex;
#[cfg(feature = "types-compat")]
pub use purge::{text_purger, PurgeReport, TextPurger};
#[cfg(feature = "types-compat")]
pub use scanner::{text_scanner, ParsedTextTarget, TextScanner};
pub use types::{Language, TextIndexMetadata};
