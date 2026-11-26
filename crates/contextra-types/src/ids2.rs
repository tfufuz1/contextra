//! Foundation identifiers (SPEC-02 §2.2.1, §2.7, §11.5; IMPL-01 §3).
//!
//! Every identifier here is a transparent newtype. Raw values only enter
//! through an explicitly named constructor (`from_raw`, `from_le_bytes`), so
//! that accidental mix-ups (a `Seq` passed where a `Gid` is expected) are
//! type errors at the call site.
//!
//! Byte encodings:
//! - Scalar values use little-endian (`to_le_bytes`), matching the file
//!   headers in Anhang B.
//! - Keys that are stored in ordered trees use big-endian prefixes, so that
//!   lexicographic byte order equals numeric order (see [`CellId::key_bytes`]).

use core::fmt;
use core::marker::PhantomData;

/// Declares a transparent numeric identifier with canonical little-endian codecs.
macro_rules! numeric_id {
    ($(#[$doc:meta])* $name:ident($inner:ty), $width:literal) => {
        $(#[$doc])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
        pub struct $name(pub $inner);

        impl $name {
            /// Wraps a raw value. Use only at a boundary that owns the value.
            #[inline]
            pub const fn from_raw(raw: $inner) -> Self {
                Self(raw)
            }

            /// Returns the raw value.
            #[inline]
            pub const fn get(self) -> $inner {
                self.0
            }

            /// Canonical little-endian bytes for file formats.
            #[inline]
            pub const fn to_le_bytes(self) -> [u8; $width] {
                self.0.to_le_bytes()
            }

            /// Parses canonical little-endian bytes.
            #[inline]
            pub const fn from_le_bytes(bytes: [u8; $width]) -> Self {
                Self(<$inner>::from_le_bytes(bytes))
            }
        }
    };
}

numeric_id! {
    /// Dense, monotonic global index address space (SPEC-02 §2.2.1).
    ///
    /// Allocation is **not** part of L0 (it belongs to the alias tree, IMPL-05).
    /// A `Gid` is never reused once allocated.
    Gid(u64), 8
}

impl Gid {
    /// Invalid sentinel. Never allocated.
    pub const NONE: Gid = Gid(0);

    /// Returns `true` unless this is the reserved sentinel `0`.
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

numeric_id! {
    /// Namespace identifier.
    NsId(u32), 4
}

numeric_id! {
    /// Branch identifier.
    BranchId(u32), 4
}

numeric_id! {
    /// Physical segment identifier.
    SegmentId(u64), 8
}

numeric_id! {
    /// WAL log position. Strictly monotonic.
    Lsn(u64), 8
}

numeric_id! {
    /// Global commit counter.
    Seq(u64), 8
}

numeric_id! {
    /// Segment-local document identifier.
    ///
    /// The SPEC name is `LocalDocId`. The code name is `DocId32` until the
    /// legacy `DocId` (IMPL-01 §2) is removed.
    DocId32(u32), 4
}

impl Lsn {
    /// Returns the successor position, or `None` on overflow.
    #[inline]
    pub const fn next(self) -> Option<Lsn> {
        match self.0.checked_add(1) {
            Some(v) => Some(Lsn(v)),
            None => None,
        }
    }
}

impl Seq {
    /// Returns the successor sequence number, or `None` on overflow.
    #[inline]
    pub const fn next(self) -> Option<Seq> {
        match self.0.checked_add(1) {
            Some(v) => Some(Seq(v)),
            None => None,
        }
    }
}

/// 16-byte cell key, ULID-compatible or random (SPEC-02 §2.2.1).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellKey(pub [u8; 16]);

impl CellKey {
    /// Wraps raw key bytes.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Borrows the key bytes.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for CellKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CellKey(")?;
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        f.write_str(")")
    }
}

/// Logical, branch- and version-stable cell identity (SPEC-02 §2.2.1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct CellId {
    /// Namespace the cell belongs to.
    pub ns: NsId,
    /// Cell key within the namespace.
    pub key: CellKey,
}

impl CellId {
    /// Creates a cell identity.
    #[inline]
    pub const fn new(ns: NsId, key: CellKey) -> Self {
        Self { ns, key }
    }

    /// Ordered 20-byte key for the primary tree: big-endian namespace, then key.
    #[inline]
    pub fn key_bytes(&self) -> [u8; 20] {
        let mut out = [0u8; 20];
        out[..4].copy_from_slice(&self.ns.0.to_be_bytes());
        out[4..].copy_from_slice(&self.key.0);
        out
    }
}

/// BLAKE3-256 content address (SPEC-02 §2.2.1).
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkId(pub [u8; 32]);

impl ChunkId {
    /// Hashes `data` with BLAKE3-256.
    pub fn of(data: &[u8]) -> Self {
        Self(*blake3::hash(data).as_bytes())
    }

    /// Wraps raw digest bytes.
    #[inline]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Borrows the digest bytes.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ChunkId(")?;
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        f.write_str(")")
    }
}

/// Typed content hash: a [`ChunkId`] tagged with the type it addresses.
///
/// The phantom parameter prevents, for example, a hash of a `CellContent`
/// from being used where a hash of a `CellMeta` is expected (SPEC-02 §2.2.2).
pub struct ContentHash<T> {
    id: ChunkId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ContentHash<T> {
    /// Tags an existing digest.
    #[inline]
    pub const fn from_chunk(id: ChunkId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Hashes `data` and tags the digest.
    pub fn of(data: &[u8]) -> Self {
        Self::from_chunk(ChunkId::of(data))
    }

    /// Returns the untyped digest.
    #[inline]
    pub const fn chunk_id(&self) -> ChunkId {
        self.id
    }
}

impl<T> Clone for ContentHash<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ContentHash<T> {}

impl<T> PartialEq for ContentHash<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for ContentHash<T> {}

impl<T> core::hash::Hash for ContentHash<T> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T> fmt::Debug for ContentHash<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ContentHash").field(&self.id).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gid_none_is_invalid_and_others_valid() {
        assert!(!Gid::NONE.is_valid());
        assert!(Gid::from_raw(1).is_valid());
        assert_eq!(Gid::NONE.get(), 0);
    }

    #[test]
    fn scalar_le_roundtrip() {
        let g = Gid::from_raw(0x0102_0304_0506_0708);
        assert_eq!(g.to_le_bytes()[0], 0x08);
        assert_eq!(Gid::from_le_bytes(g.to_le_bytes()), g);
        assert_eq!(NsId::from_le_bytes(NsId(7).to_le_bytes()), NsId(7));
        assert_eq!(DocId32::from_le_bytes(DocId32(9).to_le_bytes()), DocId32(9));
    }

    #[test]
    fn lsn_and_seq_successors_are_checked() {
        assert_eq!(Lsn(5).next(), Some(Lsn(6)));
        assert_eq!(Lsn(u64::MAX).next(), None);
        assert_eq!(Seq(0).next(), Some(Seq(1)));
        assert_eq!(Seq(u64::MAX).next(), None);
    }

    #[test]
    fn cell_key_bytes_are_order_preserving() {
        let a = CellId::new(NsId(1), CellKey([0xFF; 16]));
        let b = CellId::new(NsId(2), CellKey([0x00; 16]));
        assert!(a < b);
        assert!(a.key_bytes() < b.key_bytes());
        assert_eq!(&a.key_bytes()[..4], &1u32.to_be_bytes());
    }

    #[test]
    fn chunk_id_matches_blake3_reference() {
        let id = ChunkId::of(b"abc");
        assert_eq!(id.as_bytes(), blake3::hash(b"abc").as_bytes());
    }

    #[test]
    fn content_hash_is_typed_but_comparable() {
        struct Content;
        struct Meta;
        let a: ContentHash<Content> = ContentHash::of(b"x");
        let b: ContentHash<Content> = ContentHash::from_chunk(ChunkId::of(b"x"));
        assert_eq!(a, b);
        let m: ContentHash<Meta> = ContentHash::of(b"x");
        assert_eq!(a.chunk_id(), m.chunk_id());
    }

    #[test]
    fn debug_hides_nothing_structural_and_is_hex() {
        let s = format!("{:?}", CellKey([0xAB; 16]));
        assert!(s.starts_with("CellKey(ab"));
        assert_eq!(format!("{:?}", Gid::from_raw(5)), "Gid(5)");
    }
}
