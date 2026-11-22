//! # Feld-Dekodierer und Primitive für Katalog-TLV
//!
//! Hilfsfunktionen zur Kodierung und Dekodierung von primitiven Datentypen.

use crate::catalog::types::{CatalogError, CatalogKeyRange};
use crate::codec::varint::{decode_u64, encode_u64};

pub(crate) fn put_varint(buf: &mut Vec<u8>, val: u64) {
    let mut v = [0u8; 10];
    let len = encode_u64(val, &mut v);
    let (slice, _) = v.split_at(len);
    buf.extend_from_slice(slice);
}

pub(crate) fn get_varint(buf: &[u8]) -> Result<(u64, usize), CatalogError> {
    decode_u64(buf).map_err(|_| CatalogError::InvalidFormat("Ungültiger Varint"))
}

pub(crate) fn put_bytes(buf: &mut Vec<u8>, b: &[u8]) {
    let len_u64 = u64::try_from(b.len()).unwrap_or(0);
    put_varint(buf, len_u64);
    buf.extend_from_slice(b);
}

pub(crate) fn get_bytes(buf: &[u8]) -> Result<(&[u8], usize), CatalogError> {
    let (len_u64, c1) = get_varint(buf)?;
    let len = usize::try_from(len_u64).map_err(|_| CatalogError::InvalidFormat("Längenüberlauf"))?;
    let (_, rest) = buf.split_at(c1);
    if rest.len() < len {
        return Err(CatalogError::UnexpectedEof);
    }
    let (data, _) = rest.split_at(len);
    let total = c1.checked_add(len).ok_or(CatalogError::InvalidFormat("Überlauf"))?;
    Ok((data, total))
}

pub(crate) fn put_range(buf: &mut Vec<u8>, r: &CatalogKeyRange) {
    put_bytes(buf, &r.start);
    match &r.end {
        None => buf.push(0),
        Some(e) => {
            buf.push(1);
            put_bytes(buf, e);
        }
    }
}

pub(crate) fn get_range(mut buf: &[u8]) -> Result<(CatalogKeyRange, usize), CatalogError> {
    let (start_b, c1) = get_bytes(buf)?;
    let (_, r) = buf.split_at(c1);
    buf = r;

    let flag = buf.first().copied().ok_or(CatalogError::UnexpectedEof)?;
    let (_, r) = buf.split_at(1);
    buf = r;

    let mut consumed = c1.checked_add(1).ok_or(CatalogError::InvalidFormat("Überlauf"))?;

    let end = if flag == 0 {
        None
    } else {
        let (end_b, c2) = get_bytes(buf)?;
        consumed = consumed.checked_add(c2).ok_or(CatalogError::InvalidFormat("Überlauf"))?;
        Some(end_b.to_vec())
    };

    Ok((CatalogKeyRange { start: start_b.to_vec(), end }, consumed))
}

pub(crate) fn read_u8(buf: &mut &[u8]) -> Result<u8, CatalogError> {
    let b = buf.first().copied().ok_or(CatalogError::UnexpectedEof)?;
    let (_, r) = buf.split_at(1);
    *buf = r;
    Ok(b)
}

pub(crate) fn read_u16(buf: &mut &[u8]) -> Result<u16, CatalogError> {
    if buf.len() < 2 { return Err(CatalogError::UnexpectedEof); }
    let (s, r) = buf.split_at(2);
    let mut b = [0u8; 2];
    b.copy_from_slice(s);
    *buf = r;
    Ok(u16::from_le_bytes(b))
}

pub(crate) fn read_u32(buf: &mut &[u8]) -> Result<u32, CatalogError> {
    if buf.len() < 4 { return Err(CatalogError::UnexpectedEof); }
    let (s, r) = buf.split_at(4);
    let mut b = [0u8; 4];
    b.copy_from_slice(s);
    *buf = r;
    Ok(u32::from_le_bytes(b))
}

pub(crate) fn read_u64(buf: &mut &[u8]) -> Result<u64, CatalogError> {
    if buf.len() < 8 { return Err(CatalogError::UnexpectedEof); }
    let (s, r) = buf.split_at(8);
    let mut b = [0u8; 8];
    b.copy_from_slice(s);
    *buf = r;
    Ok(u64::from_le_bytes(b))
}

pub(crate) fn read_i64(buf: &mut &[u8]) -> Result<i64, CatalogError> {
    if buf.len() < 8 { return Err(CatalogError::UnexpectedEof); }
    let (s, r) = buf.split_at(8);
    let mut b = [0u8; 8];
    b.copy_from_slice(s);
    *buf = r;
    Ok(i64::from_le_bytes(b))
}

pub(crate) fn read_vec(buf: &mut &[u8]) -> Result<Vec<u8>, CatalogError> {
    let (slice, c) = get_bytes(buf)?;
    let (_, r) = buf.split_at(c);
    *buf = r;
    Ok(slice.to_vec())
}
