//! Extension blocks, builders, and query-visibility enforcement (SPEC §12.4, SP-EXT, INV-VIS-1).
#![forbid(unsafe_code)]

use crate::erase::gate::GateIndex;
use contextra_storage_ports::{ClusterId, Error, Stamp};

pub const EXTENSION_BLOCK_MAGIC: &[u8; 8] = b"CTXSEXT1";

/// Manifest pruning contract required before `BlockPruner` may skip blocks (SP-EXT WP-17.6, K-29).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PruneContract {
    ShadowSafe { reason: &'static str },
    NotShadowSafe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionBlock {
    pub owner_tag: u32,
    pub block_index: u32,
    pub payload: Vec<u8>,
}

impl ExtensionBlock {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16 + self.payload.len());
        buf.extend_from_slice(EXTENSION_BLOCK_MAGIC);
        buf.extend_from_slice(&self.owner_tag.to_be_bytes());
        buf.extend_from_slice(&self.block_index.to_be_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 20 || &bytes[..8] != EXTENSION_BLOCK_MAGIC {
            return Err(Error::Corrupt {
                place: "invalid extension block header",
            });
        }
        let owner_tag = u32::from_be_bytes(bytes[8..12].try_into().map_err(|_| {
            Error::Corrupt {
                place: "extension owner tag",
            }
        })?);
        let block_index = u32::from_be_bytes(bytes[12..16].try_into().map_err(|_| {
            Error::Corrupt {
                place: "extension block index",
            }
        })?);
        let payload_len = u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| {
            Error::Corrupt {
                place: "extension payload length",
            }
        })?) as usize;
        let expected_len = 20usize
            .checked_add(payload_len)
            .ok_or(Error::Corrupt {
                place: "extension payload length overflow",
            })?;
        if bytes.len() < expected_len {
            return Err(Error::Corrupt {
                place: "extension block payload truncated",
            });
        }
        if bytes.len() > expected_len {
            return Err(Error::Corrupt {
                place: "extension block trailing bytes",
            });
        }
        let payload = bytes[20..expected_len].to_vec();
        Ok(Self {
            owner_tag,
            block_index,
            payload,
        })
    }
}

/// Enforces query visibility filtering on derived extension candidate results (INV-VIS-1).
/// All extension queries MUST run candidates through this check before returning to caller.
pub fn filter_visible(
    cluster: ClusterId,
    view_stamp: Stamp,
    gate_index: &GateIndex,
) -> Result<(), Error> {
    gate_index.check_visible(cluster, view_stamp)
}
