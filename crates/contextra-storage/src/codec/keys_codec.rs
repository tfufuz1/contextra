//! KEYS manifest codec. Contains wrapped root/space keys only, never cluster keys.

use contextra_storage_ports::traits::Mac;
use contextra_storage_ports::{Error, Key32, SpaceId};
use std::collections::BTreeMap;

const MAGIC: &[u8; 8] = b"CTXKEY01";
const VERSION: u16 = 1;
const MAC_LEN: usize = 32;
const MAX_WRAPPED_KEY: usize = 4096;
const MAX_SPACE_KEYS: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeysRecord {
    pub wrapped_root: Vec<u8>,
    pub wrapped_space_deks: BTreeMap<SpaceId, Vec<u8>>,
}

pub fn encode(record: &KeysRecord, root_key: &Key32, mac: &dyn Mac) -> Result<Vec<u8>, Error> {
    check_record(record)?;
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    put_blob(&mut out, &record.wrapped_root)?;
    let count = u32::try_from(record.wrapped_space_deks.len()).map_err(|_| Error::Limit { limit: "KEYS entry count" })?;
    out.extend_from_slice(&count.to_le_bytes());
    for (space, wrapped) in &record.wrapped_space_deks {
        out.extend_from_slice(&space.0.to_be_bytes());
        put_blob(&mut out, wrapped)?;
    }
    let tag = mac.tag(root_key, &out)?;
    out.extend_from_slice(&tag);
    Ok(out)
}

pub fn decode(input: &[u8], root_key: &Key32, mac: &dyn Mac) -> Result<KeysRecord, Error> {
    if input.len() < 8 + 2 + 4 + 4 + MAC_LEN || input.len() > 16 * 1024 * 1024 {
        return Err(Error::Corrupt { place: "KEYS length" });
    }
    let signed_len = input.len().checked_sub(MAC_LEN).ok_or(Error::Corrupt { place: "KEYS MAC" })?;
    let signed = input.get(..signed_len).ok_or(Error::Corrupt { place: "KEYS signed bytes" })?;
    let received = input.get(signed_len..).ok_or(Error::Corrupt { place: "KEYS MAC" })?;
    let expected = mac.tag(root_key, signed)?;
    if !constant_time_eq(received, &expected) { return Err(Error::Corrupt { place: "KEYS MAC" }); }
    let mut c = Cursor { input: signed, offset: 0 };
    if c.take(8)? != MAGIC { return Err(Error::Unsupported { what: "KEYS magic" }); }
    if c.u16()? != VERSION { return Err(Error::Unsupported { what: "KEYS version" }); }
    let wrapped_root = c.blob()?.to_vec();
    let count = c.u32()? as usize;
    if count > MAX_SPACE_KEYS { return Err(Error::Limit { limit: "KEYS entry count" }); }
    let mut wrapped_space_deks = BTreeMap::new();
    for _ in 0..count {
        let space = SpaceId(c.u64_be()?);
        let key = c.blob()?.to_vec();
        if wrapped_space_deks.insert(space, key).is_some() { return Err(Error::Corrupt { place: "duplicate KEYS space" }); }
    }
    if !c.empty() { return Err(Error::Corrupt { place: "KEYS trailing bytes" }); }
    let record = KeysRecord { wrapped_root, wrapped_space_deks };
    check_record(&record)?;
    Ok(record)
}

fn check_record(record: &KeysRecord) -> Result<(), Error> {
    if record.wrapped_root.is_empty() || record.wrapped_root.len() > MAX_WRAPPED_KEY {
        return Err(Error::Limit { limit: "wrapped root key length" });
    }
    if record.wrapped_space_deks.len() > MAX_SPACE_KEYS { return Err(Error::Limit { limit: "KEYS entry count" }); }
    if record.wrapped_space_deks.values().any(|v| v.is_empty() || v.len() > MAX_WRAPPED_KEY) {
        return Err(Error::Limit { limit: "wrapped space key length" });
    }
    Ok(())
}
fn put_blob(out: &mut Vec<u8>, bytes: &[u8]) -> Result<(), Error> {
    let len = u32::try_from(bytes.len()).map_err(|_| Error::Limit { limit: "KEYS blob length" })?;
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(bytes);
    Ok(())
}
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() { return false; }
    left.iter().zip(right).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
}
struct Cursor<'a> { input: &'a [u8], offset: usize }
impl<'a> Cursor<'a> {
    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        let end = self.offset.checked_add(len).ok_or(Error::Corrupt { place: "KEYS offset" })?;
        let value = self.input.get(self.offset..end).ok_or(Error::Corrupt { place: "KEYS truncated" })?;
        self.offset = end;
        Ok(value)
    }
    fn u16(&mut self) -> Result<u16, Error> { Ok(u16::from_le_bytes(self.take(2)?.try_into().map_err(|_| Error::Corrupt { place: "KEYS u16" })?)) }
    fn u64_be(&mut self) -> Result<u64, Error> { Ok(u64::from_be_bytes(self.take(8)?.try_into().map_err(|_| Error::Corrupt { place: "KEYS space id" })?)) }
    fn u32(&mut self) -> Result<u32, Error> { Ok(u32::from_le_bytes(self.take(4)?.try_into().map_err(|_| Error::Corrupt { place: "KEYS u32" })?)) }
    fn blob(&mut self) -> Result<&'a [u8], Error> {
        let len = self.u32()? as usize;
        if len == 0 || len > MAX_WRAPPED_KEY { return Err(Error::Limit { limit: "wrapped key length" }); }
        self.take(len)
    }
    fn empty(&self) -> bool { self.offset == self.input.len() }
}
