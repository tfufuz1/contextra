//! # Katalog-Edit und TLV-Kodierung
//!
//! SPEC §9.7: Ein Edit ist ein einzelner Record aus TLV-Feldern (P-2).
//! Bit 0x8000 markiert optionale Felder; unbekannte Pflichtfelder führen zu Unsupported.

use crate::catalog::codec::*;
use crate::catalog::field_decode;
use crate::catalog::field_encode::*;
use crate::catalog::types::{
    CatalogError, CatalogFileKind, CatalogSpaceOptions, ErasePendingMeta, ErasedRangeMeta,
    FileMeta, HoldMeta,
};

// Feld-IDs exakt nach SPEC §9.7
/// Feld-ID 0x0001: Hinzugefügte Dateien (`[FileMeta]`).
pub const FIELD_ADD_FILES: u16 = 0x0001;
/// Feld-ID 0x0002: Entfernte Dateien (`[(kind, id)]`).
pub const FIELD_REMOVE_FILES: u16 = 0x0002;
/// Feld-ID 0x0003: Delta der Blob-Lebendbytes (`[(blob_no, delta)]`).
pub const FIELD_BLOB_LIVE_DELTA: u16 = 0x0003;
/// Feld-ID 0x0004: Generationstausch für Blobs nach Sweep (`[(blob_no, gen_alt, gen_neu)]`).
pub const FIELD_BLOB_REPLACE: u16 = 0x0004;
/// Feld-ID 0x0005: Geflushter Stamp je Space (`[(space, stamp)]`).
pub const FIELD_SPACE_FLUSHED: u16 = 0x0005;
/// Feld-ID 0x0006: Redo-Log-Grenzen (`floor | floor_stamp | hwm`).
pub const FIELD_REDO: u16 = 0x0006;
/// Feld-ID 0x0007: ID-Hochwassermarken (`run | blob | segment | tmp | keys_gen`).
pub const FIELD_RESERVE: u16 = 0x0007;
/// Feld-ID 0x0008: Neue Spaces (`[(id, name, options)]`).
pub const FIELD_SPACES_ADD: u16 = 0x0008;
/// Feld-ID 0x0009: Entfernte Spaces (`[id]`).
pub const FIELD_SPACES_DROP: u16 = 0x0009;
/// Feld-ID 0x000A: Consumer-Cursor aktualisieren (`[(name, stamp, max_lag)]`).
pub const FIELD_CONSUMER_SET: u16 = 0x000A;
/// Feld-ID 0x000B: Consumer-Cursor löschen (`[name]`).
pub const FIELD_CONSUMER_DROP: u16 = 0x000B;
/// Feld-ID 0x000C: Nachweis vernichteter Domänenschlüssel (`[(domain, stamp)]`).
pub const FIELD_DOMAIN_RETIRED: u16 = 0x000C;
/// Feld-ID 0x000D: Ausstehende physische Löschung hinzufügen (`[(id, space, range, level)]`).
pub const FIELD_ERASE_PENDING_ADD: u16 = 0x000D;
/// Feld-ID 0x000E: Löschung abgeschlossen (`[id]`).
pub const FIELD_ERASE_DONE: u16 = 0x000E;
/// Feld-ID 0x000F: Löschung durch Blob-Sweep erfasst (`[id]`).
pub const FIELD_ERASE_SWEPT: u16 = 0x000F;
/// Feld-ID 0x0010: Gelöschten Bereich registrieren (`[(space, range, stamp)]`).
pub const FIELD_ERASED_RANGES_ADD: u16 = 0x0010;
/// Feld-ID 0x0011: Gelöschten Bereich deregistrieren (`[(space, range, stamp)]`).
pub const FIELD_ERASED_RANGES_REMOVE: u16 = 0x0011;
/// Feld-ID 0x0012: Dauerhaften Hold setzen (`[(hold_id, stamp, owner, expires_ms)]`).
pub const FIELD_HOLD_SET: u16 = 0x0012;
/// Feld-ID 0x0013: Dauerhaften Hold aufheben (`[hold_id]`).
pub const FIELD_HOLD_DROP: u16 = 0x0013;
/// Feld-ID 0x0014: Schlüsselepoch je Space inkrementieren (`[(space, epoch)]`).
pub const FIELD_KEY_EPOCH: u16 = 0x0014;

/// Bit-Maske für optionale Felder (SPEC §9.7).
pub const FIELD_OPTIONAL_FLAG: u16 = 0x8000;

/// Ein atomarer Katalog-Edit nach SPEC §9.7 (P-2).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogEdit {
    /// Hinzugefügte Dateien (0x0001).
    pub add_files: Vec<FileMeta>,
    /// Entfernte Dateien (0x0002).
    pub remove_files: Vec<(CatalogFileKind, u64)>,
    /// Delta der Lebendbytes von Blobs (0x0003).
    pub blob_live_delta: Vec<(u64, i64)>,
    /// Generationstausch für Blobs (0x0004).
    pub blob_replace: Vec<(u64, u32, u32)>,
    /// Flush-Fortschritt je Space (0x0005).
    pub space_flushed: Vec<(u16, u64)>,
    /// Redo-Log-Grenzen (0x0006).
    pub redo: Option<(u64, u64, u64)>,
    /// ID-Hochwassermarken (0x0007).
    pub reserve: Option<(u64, u64, u64, u64, u64)>,
    /// Hinzugefügte Spaces (0x0008).
    pub spaces_add: Vec<(u16, Vec<u8>, CatalogSpaceOptions)>,
    /// Entfernte Spaces (0x0009).
    pub spaces_drop: Vec<u16>,
    /// Gesetzte Consumer-Cursor (0x000A).
    pub consumer_set: Vec<(Vec<u8>, u64, u64)>,
    /// Entfernte Consumer-Cursor (0x000B).
    pub consumer_drop: Vec<Vec<u8>>,
    /// Vernichtete Domänenschlüssel (0x000C).
    pub domain_retired: Vec<(u32, u64)>,
    /// Neue anhängige physische Löschungen (0x000D).
    pub erase_pending_add: Vec<ErasePendingMeta>,
    /// Abgeschlossene physische Löschungen (0x000E).
    pub erase_done: Vec<u64>,
    /// Im Sweep abgearbeitete Löschungen (0x000F).
    pub erase_swept: Vec<u64>,
    /// Neue gelöschte Schlüsselbereiche (0x0010).
    pub erased_ranges_add: Vec<ErasedRangeMeta>,
    /// Bereinigte gelöschte Schlüsselbereiche (0x0011).
    pub erased_ranges_remove: Vec<ErasedRangeMeta>,
    /// Gesetzte dauerhafte Holds (0x0012).
    pub hold_set: Vec<HoldMeta>,
    /// Aufgehobene Holds (0x0013).
    pub hold_drop: Vec<u64>,
    /// Schlüsselepoch je Space (0x0014).
    pub key_epoch: Vec<(u16, u32)>,
}

impl CatalogEdit {
    /// Kodiert den Edit in die TLV-Bytes eines Records (SPEC §9.7).
    pub fn encode(&self) -> Vec<u8> {
        encode_edit(self)
    }

    /// Dekodiert einen Edit aus TLV-Bytes.
    ///
    /// Gesetztes Bit 0x8000 markiert ein optionales Feld (unbekannt => überspringen).
    /// Unbekannte Pflichtfelder führen zu CatalogError::UnsupportedField.
    pub fn decode(mut buf: &[u8]) -> Result<Self, CatalogError> {
        let mut edit = Self::default();

        while !buf.is_empty() {
            if buf.len() < 2 {
                return Err(CatalogError::UnexpectedEof);
            }
            let (fid_bytes, r) = buf.split_at(2);
            let mut fb = [0u8; 2];
            fb.copy_from_slice(fid_bytes);
            let field_id = u16::from_le_bytes(fb);

            let (len_u64, consumed) = get_varint(r)?;
            let (_, r2) = r.split_at(consumed);
            let len = usize::try_from(len_u64)
                .map_err(|_| CatalogError::InvalidFormat("Längenüberlauf"))?;

            if r2.len() < len {
                return Err(CatalogError::UnexpectedEof);
            }
            let (payload, remaining) = r2.split_at(len);
            buf = remaining;

            let is_optional = (field_id & FIELD_OPTIONAL_FLAG) != 0;
            let actual_id = field_id & !FIELD_OPTIONAL_FLAG;

            match actual_id {
                FIELD_ADD_FILES => field_decode::decode_add_files(&mut edit, payload)?,
                FIELD_REMOVE_FILES => field_decode::decode_remove_files(&mut edit, payload)?,
                FIELD_BLOB_LIVE_DELTA => field_decode::decode_blob_live_delta(&mut edit, payload)?,
                FIELD_BLOB_REPLACE => field_decode::decode_blob_replace(&mut edit, payload)?,
                FIELD_SPACE_FLUSHED => field_decode::decode_space_flushed(&mut edit, payload)?,
                FIELD_REDO => field_decode::decode_redo(&mut edit, payload)?,
                FIELD_RESERVE => field_decode::decode_reserve(&mut edit, payload)?,
                FIELD_SPACES_ADD => field_decode::decode_spaces_add(&mut edit, payload)?,
                FIELD_SPACES_DROP => field_decode::decode_spaces_drop(&mut edit, payload)?,
                FIELD_CONSUMER_SET => field_decode::decode_consumer_set(&mut edit, payload)?,
                FIELD_CONSUMER_DROP => field_decode::decode_consumer_drop(&mut edit, payload)?,
                FIELD_DOMAIN_RETIRED => field_decode::decode_domain_retired(&mut edit, payload)?,
                FIELD_ERASE_PENDING_ADD => field_decode::decode_erase_pending_add(&mut edit, payload)?,
                FIELD_ERASE_DONE => field_decode::decode_erase_done(&mut edit, payload)?,
                FIELD_ERASE_SWEPT => field_decode::decode_erase_swept(&mut edit, payload)?,
                FIELD_ERASED_RANGES_ADD => field_decode::decode_erased_ranges_add(&mut edit, payload)?,
                FIELD_ERASED_RANGES_REMOVE => field_decode::decode_erased_ranges_remove(&mut edit, payload)?,
                FIELD_HOLD_SET => field_decode::decode_hold_set(&mut edit, payload)?,
                FIELD_HOLD_DROP => field_decode::decode_hold_drop(&mut edit, payload)?,
                FIELD_KEY_EPOCH => field_decode::decode_key_epoch(&mut edit, payload)?,
                _ => {
                    if !is_optional {
                        return Err(CatalogError::UnsupportedField(field_id));
                    }
                }
            }
        }

        Ok(edit)
    }
}
