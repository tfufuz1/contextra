//! # Tags und interne Schlüssel
//!
//! Definitionen für `tag = (stamp << 8) | kind` und Sortierregeln (SPEC §6.2, Anhang A.3).
//! Sortierung: Nutzerschlüssel aufsteigend, Tag absteigend (neueste Version zuerst).
//! Keine Integer-Casts (M-02 = 0).

use std::cmp::Ordering;
use std::fmt;

/// Fehler bei Überschreitung des maximalen Stamps (INV-STAMP-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exhausted;

impl fmt::Display for Exhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "INV-STAMP-2: Stamp erschöpft (>= 2^56)")
    }
}

impl std::error::Error for Exhausted {}

/// Fehler bei unbekanntem oder nicht unterstütztem Tag-Kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagKindError {
    /// Unbekanntes Byte.
    Unknown(u8),
    /// Reservierter Merge-Operator (SD-12, Version 1 unterstützt dies nicht).
    UnsupportedMerge,
}

impl fmt::Display for TagKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown(b) => write!(f, "Unbekanntes TagKind-Byte: {}", b),
            Self::UnsupportedMerge => write!(f, "Merge-Operator ist in Version 1 reserviert (SD-12)"),
        }
    }
}

impl std::error::Error for TagKindError {}

/// Art der Operation / des Eintrags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TagKind {
    /// Normales Schreiben (Header + Körper).
    Put = 1,
    /// Punktuelles Löschen.
    Delete = 2,
    /// Bereichslöschung (Tombstone).
    RangeDelete = 3,
    /// Physische Bereichslöschung.
    EraseRange = 4,
    /// Referenz auf ausgelagerten Blob-Körper (nur in Runs).
    PutRef = 5,
    /// Reserviert für spätere Merge-Operatoren (SD-12).
    Merge = 6,
}

impl TagKind {
    /// Versucht ein Byte in ein `TagKind` zu dekodieren.
    pub fn try_from_u8(b: u8) -> Result<Self, TagKindError> {
        match b {
            1 => Ok(Self::Put),
            2 => Ok(Self::Delete),
            3 => Ok(Self::RangeDelete),
            4 => Ok(Self::EraseRange),
            5 => Ok(Self::PutRef),
            6 => Err(TagKindError::UnsupportedMerge),
            other => Err(TagKindError::Unknown(other)),
        }
    }

    /// Gibt das Byte zurück.
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Put => 1,
            Self::Delete => 2,
            Self::RangeDelete => 3,
            Self::EraseRange => 4,
            Self::PutRef => 5,
            Self::Merge => 6,
        }
    }
}

/// Ein 64-Bit-Tag: `(stamp << 8) | kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tag(pub u64);

impl Tag {
    /// Erstellt ein neues Tag aus Stamp und Kind.
    ///
    /// Prüft `stamp < 2^56` (INV-STAMP-2) und liefert `Exhausted` bei Überlauf.
    pub fn try_new(stamp: u64, kind: TagKind) -> Result<Self, Exhausted> {
        if stamp >= (1 << 56) {
            return Err(Exhausted);
        }
        let kind_u64 = u64::from(kind.to_u8());
        Ok(Self((stamp << 8) | kind_u64))
    }

    /// Ungeprüfte Erstellung für bekannte Werte oder Tests.
    pub fn new(stamp: u64, kind: TagKind) -> Self {
        Self::try_new(stamp, kind).unwrap_or(Self(0))
    }

    /// Liefert den Stamp.
    pub const fn stamp(self) -> u64 {
        self.0 >> 8
    }

    /// Liefert das Kind.
    pub fn kind(self) -> Result<TagKind, TagKindError> {
        let b = u8::try_from(self.0 & 0xFF).unwrap_or(0);
        TagKind::try_from_u8(b)
    }

    /// Liefert die Rohbytes in Little-Endian.
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    /// Liest ein Tag aus Little-Endian-Bytes.
    pub const fn from_le_bytes(bytes: [u8; 8]) -> Self {
        Self(u64::from_le_bytes(bytes))
    }
}

/// Interner Schlüssel: Nutzerschlüssel aufsteigend, Tag absteigend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternalKey {
    /// Nutzerschlüssel.
    pub user_key: Vec<u8>,
    /// Tag (Stamp + Kind).
    pub tag: Tag,
}

impl InternalKey {
    /// Erstellt einen neuen internen Schlüssel.
    pub fn new(user_key: Vec<u8>, tag: Tag) -> Self {
        Self { user_key, tag }
    }
}

impl Ord for InternalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        // Nutzerschlüssel aufsteigend
        match self.user_key.cmp(&other.user_key) {
            Ordering::Equal => {
                // Tag absteigend (neueste Version zuerst)
                other.tag.0.cmp(&self.tag.0)
            }
            ord => ord,
        }
    }
}

impl PartialOrd for InternalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_golden_a3_tag() {
        // stamp = 1, Put (1)
        let t1 = Tag::new(1, TagKind::Put);
        assert_eq!(t1.to_le_bytes(), [0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // stamp = 7, Delete (2)
        let t2 = Tag::new(7, TagKind::Delete);
        assert_eq!(t2.to_le_bytes(), [0x02, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

        // stamp = 2^56 - 1, PutRef (5)
        let max_stamp = (1u64 << 56) - 1;
        let t3 = Tag::new(max_stamp, TagKind::PutRef);
        assert_eq!(t3.to_le_bytes(), [0x05, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

        // Sortierung: ("k", tag(7, Delete)) steht VOR ("k", tag(1, Put))
        let k_del = InternalKey::new(b"k".to_vec(), t2);
        let k_put = InternalKey::new(b"k".to_vec(), t1);
        assert!(k_del < k_put);

        // Überlaufprüfung
        assert_eq!(Tag::try_new(1u64 << 56, TagKind::Put), Err(Exhausted));

        // Merge reserviert
        assert_eq!(TagKind::try_from_u8(6), Err(TagKindError::UnsupportedMerge));
    }
}
