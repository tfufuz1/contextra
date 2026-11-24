//! PackMeta codec for Pure, Packed, and Staging run placement metadata (SPEC §5.1, §5.6, SP-RUN).
#![forbid(unsafe_code)]

use crate::codec::checksums::crc32c;
use contextra_storage_ports::{Error, Stamp};

const PACK_META_MAGIC: [u8; 8] = *b"CTXPMA01";
const PACK_META_LEN: usize = 8 + 1 + 1 + 16 + 1 + 8 + 8 + 4; // 47 bytes

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PackKind {
    Pure = 1,
    Packed = 2,
    Staging = 3,
}

impl TryFrom<u8> for PackKind {
    type Error = Error;
    fn try_from(v: u8) -> Result<Self, Error> {
        match v {
            1 => Ok(Self::Pure),
            2 => Ok(Self::Packed),
            3 => Ok(Self::Staging),
            _ => Err(Error::Unsupported {
                what: "run pack kind",
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackMeta {
    pub kind: PackKind,
    pub pack_key: Option<[u8; 16]>,
    pub level: u8,
    pub min_stamp: Stamp,
    pub max_stamp: Stamp,
}

impl PackMeta {
    pub fn encode(&self) -> Result<[u8; PACK_META_LEN], Error> {
        if self.min_stamp > self.max_stamp {
            return Err(Error::Limit {
                limit: "pack meta stamp range",
            });
        }
        let mut out = [0u8; PACK_META_LEN];
        out[..8].copy_from_slice(&PACK_META_MAGIC);
        out[8] = self.kind as u8;
        match self.pack_key {
            Some(pk) => {
                out[9] = 1;
                out[10..26].copy_from_slice(&pk);
            }
            None => {
                out[9] = 0;
            }
        }
        out[26] = self.level;
        out[27..35].copy_from_slice(&self.min_stamp.to_le_bytes());
        out[35..43].copy_from_slice(&self.max_stamp.to_le_bytes());
        let crc = crc32c(&out[..43]);
        out[43..47].copy_from_slice(&crc.to_le_bytes());
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != PACK_META_LEN {
            return Err(Error::Corrupt {
                place: "pack meta length",
            });
        }
        if bytes[..8] != PACK_META_MAGIC {
            return Err(Error::Unsupported {
                what: "pack meta magic",
            });
        }
        let expected_crc = u32::from_le_bytes([bytes[43], bytes[44], bytes[45], bytes[46]]);
        if crc32c(&bytes[..43]) != expected_crc {
            return Err(Error::Corrupt {
                place: "pack meta CRC",
            });
        }
        let kind = PackKind::try_from(bytes[8])?;
        let pack_key = match bytes[9] {
            0 => None,
            1 => {
                let mut pk = [0u8; 16];
                pk.copy_from_slice(&bytes[10..26]);
                Some(pk)
            }
            _ => {
                return Err(Error::Corrupt {
                    place: "pack meta pack_key flag",
                })
            }
        };
        let level = bytes[26];
        let min_stamp = u64::from_le_bytes(
            bytes[27..35]
                .try_into()
                .map_err(|_| Error::Corrupt { place: "pack meta min_stamp" })?,
        );
        let max_stamp = u64::from_le_bytes(
            bytes[35..43]
                .try_into()
                .map_err(|_| Error::Corrupt { place: "pack meta max_stamp" })?,
        );
        if min_stamp > max_stamp {
            return Err(Error::Corrupt {
                place: "pack meta stamp range",
            });
        }
        Ok(Self {
            kind,
            pack_key,
            level,
            min_stamp,
            max_stamp,
        })
    }
}
