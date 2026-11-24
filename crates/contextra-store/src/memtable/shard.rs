//! MemTable Shard-Struktur und Schlüssel-zu-Shard-Abbildung (Ring 1)
//!
//! Partitioniert den Schlüsselraum deterministisch auf `SHARD_COUNT` Shards,
//! um Schreibkonkurrenz bei parallelen Schreibern zu minimieren.

use bytes::Bytes;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::ops::Bound;

pub type SequenceNumber = u64;
pub type TransactionId = u64;
pub type MemTableEntry = (SequenceNumber, Bytes, TransactionId);
pub type MemTableMap = BTreeMap<Bytes, Vec<MemTableEntry>>;

/// Number of independent shards. 16 provides sufficient parallelism for the
/// 8-concurrent-embed pipeline while keeping memory overhead negligible.
/// Must be > 0 (compile-time const, enforced by type system).
pub const SHARD_COUNT: usize = 16;

/// List of known reserved system key prefixes.
pub const RESERVED_PREFIXES: &[&[u8]] = &[
    b"__col:",
    b"__col_idx:",
    b"__meta:",
    b"__rel:",
    b"__idx:",
    b"__graph:",
    b"__txt:",
    b"__docid:",
    b"__tx_intent:",
    b"__kv_spill:",
    b"__session_dag:",
    b"__maintenance_intent:",
];

/// Helper to detect if a key matches a reserved system prefix.
#[inline]
pub fn matching_reserved_prefix(key: &[u8]) -> Option<&'static [u8]> {
    if !key.starts_with(b"__") {
        return None;
    }
    for &prefix in RESERVED_PREFIXES {
        if key.starts_with(prefix) {
            return Some(prefix);
        }
    }
    Some(b"__")
}

/// Helper to compute a deterministic hash for reserved system key remainder.
#[inline]
pub fn hash_remainder(remainder: &[u8]) -> usize {
    let mut hash = 0xcbf29ce484222325u64;
    for &byte in remainder {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3u64);
    }
    (hash as usize) % SHARD_COUNT
}

/// A single shard of the MemTable, holding a subset of the key space.
#[derive(Debug)]
pub struct MemTableShard {
    pub entries: RwLock<MemTableMap>,
}

impl MemTableShard {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(BTreeMap::new()),
        }
    }
}

impl Default for MemTableShard {
    fn default() -> Self {
        Self::new()
    }
}

/// Maps a key to its owning shard index `0..SHARD_COUNT`.
#[inline]
pub fn shard_for(key: &[u8]) -> usize {
    if let Some(prefix) = matching_reserved_prefix(key) {
        let remainder = &key[prefix.len()..];
        hash_remainder(remainder)
    } else {
        let b0 = key.first().copied().unwrap_or(0) as u16;
        let b1 = key.get(1).copied().unwrap_or(0) as u16;
        let val = (b0 << 8) | b1;
        ((val as usize) * SHARD_COUNT) / 65536
    }
}

/// Computes the contiguous shard range `[start, end]` that may contain keys with the given prefix.
#[inline]
pub fn shard_range_for_prefix(prefix: &[u8]) -> std::ops::RangeInclusive<usize> {
    if prefix.first() == Some(&b'_') {
        0..=SHARD_COUNT - 1
    } else if prefix.len() >= 2 {
        let shard = shard_for(prefix);
        shard..=shard
    } else if prefix.len() == 1 {
        let start = shard_for(&[prefix[0], 0x00]);
        let end = shard_for(&[prefix[0], 0xFF]);
        start..=end
    } else {
        0..=SHARD_COUNT - 1
    }
}

/// Computes the contiguous shard range `[start, end]` that covers the given bounds.
#[inline]
pub fn shard_range_for_bounds(
    start: Bound<&[u8]>,
    end: Bound<&[u8]>,
) -> std::ops::RangeInclusive<usize> {
    let is_system = match (start, end) {
        (Bound::Included(s), _) | (Bound::Excluded(s), _) if s.first() == Some(&b'_') => true,
        (_, Bound::Included(e)) | (_, Bound::Excluded(e)) if e.first() == Some(&b'_') => true,
        (Bound::Unbounded, _) | (_, Bound::Unbounded) => true,
        (Bound::Included(s), Bound::Included(e))
        | (Bound::Included(s), Bound::Excluded(e))
        | (Bound::Excluded(s), Bound::Included(e))
        | (Bound::Excluded(s), Bound::Excluded(e)) => {
            s <= b"__".as_slice() && e >= b"__".as_slice()
        }
    };

    if is_system {
        0..=SHARD_COUNT - 1
    } else {
        let start_shard = match start {
            Bound::Included(s) | Bound::Excluded(s) => shard_for(s),
            Bound::Unbounded => 0,
        };
        let end_shard = match end {
            Bound::Included(e) | Bound::Excluded(e) => shard_for(e),
            Bound::Unbounded => SHARD_COUNT - 1,
        };
        start_shard..=end_shard
    }
}
