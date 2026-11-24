//! Authenticated, immutable FORMAT manifest. Wire v1 is provisional until G-F.

use contextra_storage_ports::traits::{Kdf, Mac};
use contextra_storage_ports::{Error, Key32};

const MAGIC: &[u8; 8] = b"CTXFMT01";
const VERSION: u16 = 1;
const BODY_LEN: usize = 8 + 2 + 1 + 4 + 16;
const CRC_LEN: usize = 4;
const MAC_LEN: usize = 32;
const MAC_LABEL: &[u8] = b"ctxs format mac v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IntegrityMode { Plain = 1, Authenticated = 2, Encrypted = 3 }

impl TryFrom<u8> for IntegrityMode {
    type Error = Error;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value { 1 => Ok(Self::Plain), 2 => Ok(Self::Authenticated), 3 => Ok(Self::Encrypted), _ => Err(Error::Unsupported { what: "FORMAT integrity mode" }) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatRecord {
    pub integrity: IntegrityMode,
    pub feature_bits: u32,
    pub instance_id: [u8; 16],
}

pub fn encode(record: FormatRecord, root_key: &Key32, kdf: &dyn Kdf, mac: &dyn Mac) -> Result<Vec<u8>, Error> {
    let mut out = Vec::with_capacity(BODY_LEN + CRC_LEN + MAC_LEN);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&VERSION.to_le_bytes());
    out.push(record.integrity as u8);
    out.extend_from_slice(&record.feature_bits.to_le_bytes());
    out.extend_from_slice(&record.instance_id);
    let crc = super::checksums::crc32c(&out);
    out.extend_from_slice(&crc.to_le_bytes());
    let mac_key = kdf.derive(root_key, MAC_LABEL)?;
    let tag = mac.tag(&mac_key, &out)?;
    out.extend_from_slice(&tag);
    Ok(out)
}

pub fn decode(input: &[u8], root_key: &Key32, kdf: &dyn Kdf, mac: &dyn Mac, supported_features: u32) -> Result<FormatRecord, Error> {
    if input.len() != BODY_LEN + CRC_LEN + MAC_LEN { return Err(Error::Corrupt { place: "FORMAT length" }); }
    if input.get(..8) != Some(MAGIC.as_slice()) { return Err(Error::Unsupported { what: "FORMAT magic" }); }
    let version = u16::from_le_bytes(input.get(8..10).ok_or(Error::Corrupt { place: "FORMAT version" })?
        .try_into().map_err(|_| Error::Corrupt { place: "FORMAT version" })?);
    if version != VERSION { return Err(Error::Unsupported { what: "FORMAT version" }); }
    let integrity = IntegrityMode::try_from(*input.get(10).ok_or(Error::Corrupt { place: "FORMAT integrity" })?)?;
    let feature_bits = u32::from_le_bytes(input.get(11..15).ok_or(Error::Corrupt { place: "FORMAT features" })?
        .try_into().map_err(|_| Error::Corrupt { place: "FORMAT features" })?);
    if feature_bits & !supported_features != 0 { return Err(Error::Unsupported { what: "FORMAT feature bits" }); }
    let crc_end = BODY_LEN;
    let crc = u32::from_le_bytes(input.get(crc_end..crc_end + 4).ok_or(Error::Corrupt { place: "FORMAT CRC" })?
        .try_into().map_err(|_| Error::Corrupt { place: "FORMAT CRC" })?);
    if super::checksums::crc32c(input.get(..crc_end).ok_or(Error::Corrupt { place: "FORMAT CRC input" })?) != crc {
        return Err(Error::Corrupt { place: "FORMAT CRC" });
    }
    let authenticated = input.get(..crc_end + CRC_LEN).ok_or(Error::Corrupt { place: "FORMAT MAC input" })?;
    let mac_key = kdf.derive(root_key, MAC_LABEL)?;
    let expected = mac.tag(&mac_key, authenticated)?;
    let received = input.get(crc_end + CRC_LEN..).ok_or(Error::Corrupt { place: "FORMAT MAC" })?;
    if !constant_time_eq(received, &expected) { return Err(Error::Corrupt { place: "FORMAT MAC" }); }
    let instance_id = input.get(15..31).ok_or(Error::Corrupt { place: "FORMAT instance id" })?
        .try_into().map_err(|_| Error::Corrupt { place: "FORMAT instance id" })?;
    Ok(FormatRecord { integrity, feature_bits, instance_id })
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() { return false; }
    left.iter().zip(right).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
}
