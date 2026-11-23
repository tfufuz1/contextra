//! Interner MVCC-Tag: `(stamp << 8) | kind` (R8 §4.2, INV-FMT-1).

use contextra_storage_ports::{Stamp, STAMP_MAX};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    Put = 1,
    Delete = 2,
    RangeDelete = 3,
    EraseRange = 4,
    PutRef = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagError {
    StampZero,
    StampOutOfRange,
    UnknownKind(u8),
    ReservedMerge,
}

/// Kodiert Stamp und Operationstyp. Stampwerte oberhalb der 56-Bit-Grenze
/// werden abgewiesen, damit ein Überlauf nie einen alten Tag reproduziert.
pub fn tag(stamp: Stamp, kind: Kind) -> Result<u64, TagError> {
    if stamp == 0 {
        return Err(TagError::StampZero);
    }
    if stamp > STAMP_MAX {
        return Err(TagError::StampOutOfRange);
    }
    Ok((stamp << 8) | kind as u64)
}

/// Dekodiert nur bekannte kanonische Tags; Typ 6 ist explizit reserviert.
pub fn untag(value: u64) -> Result<(Stamp, Kind), TagError> {
    let stamp = value >> 8;
    if stamp == 0 {
        return Err(TagError::StampZero);
    }
    let raw_kind = (value & 0xff) as u8;
    let kind = match raw_kind {
        1 => Kind::Put,
        2 => Kind::Delete,
        3 => Kind::RangeDelete,
        4 => Kind::EraseRange,
        5 => Kind::PutRef,
        6 => return Err(TagError::ReservedMerge),
        other => return Err(TagError::UnknownKind(other)),
    };
    Ok((stamp, kind))
}
