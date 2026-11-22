//! # Run: Unveränderliches Sortierlauf-Format (SSTable)
//!
//! Normative Referenzen: SPEC §9.5, §6.3–6.5, §6.7, §11, §12.4, §18.
//! Keine Integer-Casts (M-02 = 0).

pub mod blob_ref;
pub mod block;
pub mod block_cache;
pub mod builder;
pub mod data;
pub mod directory;
pub mod error;
pub mod extension;
pub mod filter;
pub mod footer;
pub mod index;
pub mod meta;
pub mod rangedel;
pub mod reader;
pub mod table_cache;

pub use blob_ref::BlobRef;
pub use block::{block_kind, BlockCodec, BlockSecurity, CODEC_NONE};
pub use block_cache::{BlockCache, BlockKey};
pub use builder::{RunBuilder, RunBuilderOptions};
pub use data::{DataBlockBuilder, DataBlockReader, DataEntry};
pub use directory::{DirectoryBlock, DirectoryEntry};
pub use error::RunError;
pub use extension::Extension;
pub use filter::{
    FilterBlockBuilder, FilterBlockReader, PrefixFilterBlockBuilder, PrefixFilterBlockReader,
};
pub use footer::{Footer, RUN_MAGIC, RUN_VERSION_1};
pub use index::{IndexBlock, IndexEntry};
pub use meta::MetaBlock;
pub use rangedel::{RangeDelBlock, RangeMarker};
pub use reader::RunReader;
pub use table_cache::{FileQuota, LocalQuota, TableCache, DEFAULT_MAX_OPEN_FILES};
