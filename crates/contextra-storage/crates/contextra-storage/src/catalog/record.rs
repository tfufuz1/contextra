//! Catalog edit-record codec (provisorisch bis Formatfreeze G-F).

use super::ops::CatalogOp;
use contextra_storage_ports::{traits::Mac, Error, Key32};

const RECORD_FIXED: usize = 8 + 4;
const CRC_BYTES: usize = 4;
const MAC_BYTES: usize = 32;
pub(super) const MAX_RECORD_BYTES: usize = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogIntegrity { Plain, Authenticated }

#[derive(Debug, Clone)]
pub struct CatalogRecord {
    pub sequence: u64,
    pub previous_crc: u32,
    pub ops: Vec<CatalogOp>,
    pub crc: u32,
    pub chain_mac: Option<[u8; 32]>,
}

pub fn encode_record(
    sequence: u64,
    previous_crc: u32,
    ops: &[CatalogOp],
    integrity: CatalogIntegrity,
    key: Option<&Key32>,
    previous_mac: &[u8; 32],
    mac: Option<&dyn Mac>,
) -> Result<Vec<u8>, Error> {
    if ops.len() > 4096 { return Err(Error::Limit { limit: "catalog ops per edit" }); }
    if integrity == CatalogIntegrity::Authenticated && (key.is_none() || mac.is_none()) {
        return Err(Error::Limit { limit: "catalog authenticated key and MAC" });
    }
    let mut body = Vec::new();
    body.extend_from_slice(&sequence.to_le_bytes());
    body.extend_from_slice(&previous_crc.to_le_bytes());
    for op in ops { op.encode(&mut body)?; }
    let crc = crate::codec::checksums::crc32c(&body);
    body.extend_from_slice(&crc.to_le_bytes());
    let chain_mac = match (integrity, key, mac) {
        (CatalogIntegrity::Plain, _, _) => None,
        (CatalogIntegrity::Authenticated, Some(key), Some(mac)) => {
            let mut authenticated = Vec::with_capacity(previous_mac.len() + body.len());
            authenticated.extend_from_slice(previous_mac);
            authenticated.extend_from_slice(&body);
            Some(mac.tag(key, &authenticated)?)
        }
        _ => return Err(Error::Limit { limit: "catalog authenticated key and MAC" }),
    };
    if let Some(tag) = chain_mac { body.extend_from_slice(&tag); }
    let record_len = u32::try_from(body.len()).map_err(|_| Error::Limit { limit: "catalog record length" })?;
    if body.len() > MAX_RECORD_BYTES { return Err(Error::Limit { limit: "catalog record <= 1 MiB" }); }
    let mut output = Vec::with_capacity(4 + body.len());
    output.extend_from_slice(&record_len.to_le_bytes());
    output.extend_from_slice(&body);
    Ok(output)
}

pub fn decode_record(
    input: &[u8],
    expected_sequence: u64,
    expected_previous_crc: u32,
    integrity: CatalogIntegrity,
    key: Option<&Key32>,
    previous_mac: &[u8; 32],
    mac: Option<&dyn Mac>,
) -> Result<(CatalogRecord, usize), Error> {
    let length_bytes: [u8; 4] = input.get(..4).ok_or(Error::Corrupt { place: "catalog record length" })?
        .try_into().map_err(|_| Error::Corrupt { place: "catalog record length" })?;
    let record_len = u32::from_le_bytes(length_bytes) as usize;
    let total = record_len.checked_add(4).ok_or(Error::Corrupt { place: "catalog record length" })?;
    if record_len > MAX_RECORD_BYTES || record_len < RECORD_FIXED + CRC_BYTES + if integrity == CatalogIntegrity::Authenticated { MAC_BYTES } else { 0 } {
        return Err(Error::Corrupt { place: "catalog record length" });
    }
    let encoded = input.get(4..total).ok_or(Error::Corrupt { place: "catalog record truncated" })?;
    let mac_len = if integrity == CatalogIntegrity::Authenticated { MAC_BYTES } else { 0 };
    let mac_offset = record_len.checked_sub(mac_len).ok_or(Error::Corrupt { place: "catalog record length" })?;
    let crc_offset = mac_offset.checked_sub(CRC_BYTES).ok_or(Error::Corrupt { place: "catalog record length" })?;
    let mut cursor = Cursor::new(encoded.get(..crc_offset).ok_or(Error::Corrupt { place: "catalog record body" })?);
    let sequence = cursor.u64()?;
    let previous_crc = cursor.u32()?;
    if sequence != expected_sequence || previous_crc != expected_previous_crc {
        return Err(Error::Corrupt { place: "catalog record chain" });
    }
    let mut ops = Vec::new();
    while !cursor.is_empty() {
        if ops.len() >= 4096 { return Err(Error::Limit { limit: "catalog ops per edit" }); }
        let (op, used) = CatalogOp::decode(cursor.remaining()?)?;
        if used == 0 { return Err(Error::Corrupt { place: "catalog op length" }); }
        cursor.advance(used)?;
        ops.push(op);
    }
    let crc_input = encoded.get(..crc_offset).ok_or(Error::Corrupt { place: "catalog record CRC input" })?;
    let crc_bytes: [u8; 4] = encoded.get(crc_offset..mac_offset).ok_or(Error::Corrupt { place: "catalog record CRC" })?
        .try_into().map_err(|_| Error::Corrupt { place: "catalog record CRC" })?;
    let crc = u32::from_le_bytes(crc_bytes);
    if crate::codec::checksums::crc32c(crc_input) != crc { return Err(Error::Corrupt { place: "catalog record CRC" }); }
    let chain_mac = match integrity {
        CatalogIntegrity::Plain => None,
        CatalogIntegrity::Authenticated => {
            let key = key.ok_or(Error::Limit { limit: "catalog authenticated key" })?;
            let mac = mac.ok_or(Error::Limit { limit: "catalog MAC" })?;
            let mut authenticated = Vec::with_capacity(previous_mac.len() + mac_offset);
            authenticated.extend_from_slice(previous_mac);
            authenticated.extend_from_slice(encoded.get(..mac_offset).ok_or(Error::Corrupt { place: "catalog MAC input" })?);
            let actual = mac.tag(key, &authenticated)?;
            let received: [u8; 32] = encoded.get(mac_offset..).ok_or(Error::Corrupt { place: "catalog MAC" })?
                .try_into().map_err(|_| Error::Corrupt { place: "catalog MAC" })?;
            if actual != received { return Err(Error::Corrupt { place: "catalog chain MAC" }); }
            Some(received)
        }
    };
    Ok((CatalogRecord { sequence, previous_crc, ops, crc, chain_mac }, total))
}

struct Cursor<'a> { input: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
    fn new(input: &'a [u8]) -> Self { Self { input, offset: 0 } }
    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "catalog cursor overflow" })?;
        let bytes = self.input.get(self.offset..end).ok_or(Error::Corrupt { place: "catalog record truncated" })?;
        self.offset = end;
        Ok(bytes)
    }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(|_| Error::Corrupt { place: "catalog u32" })?)) }
    fn u64(&mut self) -> Result<u64, Error> { Ok(u64::from_le_bytes(self.take(8)?.try_into().map_err(|_| Error::Corrupt { place: "catalog u64" })?)) }
    fn is_empty(&self) -> bool { self.offset == self.input.len() }
    fn remaining(&self) -> Result<&'a [u8], Error> {
        self.input.get(self.offset..).ok_or(Error::Corrupt { place: "catalog cursor offset" })
    }
    fn advance(&mut self, len: usize) -> Result<(), Error> { self.take(len).map(|_| ()) }
}
