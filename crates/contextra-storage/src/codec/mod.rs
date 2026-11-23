//! # Codec Modul
//!
//! Grundlegende Formate, Kodierungen und Prüfsummen.

pub mod bloom;
pub mod bytes;
pub mod comparator;
pub mod crc32c;
pub mod format_header;
pub mod keyrange;
pub mod limits;
pub mod tag;
pub mod varint;
pub mod xxh64;

pub use bloom::BloomFilter;
pub use bytes::Bytes;
pub use comparator::{EntryKey, CLUSTER_NONE};
pub use crc32c::crc32c;
pub use format_header::{decode_format_plain, encode_format_plain, FormatCodecError, FormatFields};
pub use keyrange::KeyRange;
pub use tag::{InternalKey, Tag, TagKind};
pub use varint::{decode_u64, encode_u64, VarintError};
pub use xxh64::xxh64;
