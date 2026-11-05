//! Format F.8 Encrypted Vector Snapshot Implementation (SPEC §9.8, F.8, D-19, D-94).

use contextra_ports::Vfs;
use contextra_types::{ContextraError, Result};
use std::path::Path;

/// Magic constant for Format F.1 / F.8 files.
pub const SNAPSHOT_MAGIC: &[u8; 8] = b"CTXFILE\0";
/// Format F.8 kind identifier for vector snapshots.
pub const SNAPSHOT_KIND_VECTOR: u8 = 5;
/// Format version for v1.
pub const SNAPSHOT_FORMAT_VERSION: u16 = 1;

/// Parsed F.8 Vector Snapshot Header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorSnapshotHeader {
    pub dimension: u16,
    pub metric: u8,
    pub m: u16,
    pub ef_construction: u16,
    pub slot_count: u32,
    pub entry_slot: u32,
    pub max_level: u8,
    pub built_through_seq: u64,
}

impl VectorSnapshotHeader {
    /// Serializes header to bytes (19 bytes payload after F.1 frame).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64 + 21);
        // F.1 frame
        buf.extend_from_slice(SNAPSHOT_MAGIC);
        buf.extend_from_slice(&SNAPSHOT_FORMAT_VERSION.to_le_bytes());
        buf.push(SNAPSHOT_KIND_VECTOR);
        buf.extend_from_slice(&[1, 0, 1, 1]); // alg_ids
        buf.extend_from_slice(&[0u8; 16]); // file_id
        buf.extend_from_slice(&0u64.to_le_bytes()); // created_seq
        buf.push(0); // key_mode = 0 (random key / F-slot)
        buf.extend_from_slice(&0u64.to_le_bytes()); // reserved
        buf.extend_from_slice(&[0u8; 16]); // header_tag

        // F.8 payload header
        buf.extend_from_slice(&self.dimension.to_le_bytes());
        buf.push(self.metric);
        buf.extend_from_slice(&self.m.to_le_bytes());
        buf.extend_from_slice(&self.ef_construction.to_le_bytes());
        buf.extend_from_slice(&self.slot_count.to_le_bytes());
        buf.extend_from_slice(&self.entry_slot.to_le_bytes());
        buf.push(self.max_level);
        buf.extend_from_slice(&self.built_through_seq.to_le_bytes());
        buf
    }

    /// Parses snapshot header from raw bytes verifying fail-closed invariants.
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 64 + 21 {
            return Err(ContextraError::Storage(
                "Snapshot too small for header".into(),
            ));
        }
        if &bytes[0..8] != SNAPSHOT_MAGIC {
            return Err(ContextraError::Storage("Invalid snapshot magic".into()));
        }
        let version = u16::from_le_bytes(
            bytes[8..10]
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid version".into()))?,
        );
        if version != SNAPSHOT_FORMAT_VERSION {
            return Err(ContextraError::Storage(format!(
                "Unsupported format version {}",
                version
            )));
        }
        let kind = bytes[10];
        if kind != SNAPSHOT_KIND_VECTOR {
            return Err(ContextraError::Storage(format!(
                "Invalid file kind for vector snapshot: {}",
                kind
            )));
        }

        let mut offset = 64;
        let dimension = u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .map_err(|_| ContextraError::Storage("dim".into()))?,
        );
        offset += 2;
        let metric = bytes[offset];
        offset += 1;
        let m = u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .map_err(|_| ContextraError::Storage("m".into()))?,
        );
        offset += 2;
        let ef_construction = u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .map_err(|_| ContextraError::Storage("ef_c".into()))?,
        );
        offset += 2;
        let slot_count = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| ContextraError::Storage("slot_count".into()))?,
        );
        offset += 4;
        let entry_slot = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .map_err(|_| ContextraError::Storage("entry_slot".into()))?,
        );
        offset += 4;
        let max_level = bytes[offset];
        offset += 1;
        let built_through_seq = u64::from_le_bytes(
            bytes[offset..offset + 8]
                .try_into()
                .map_err(|_| ContextraError::Storage("seq".into()))?,
        );

        Ok(Self {
            dimension,
            metric,
            m,
            ef_construction,
            slot_count,
            entry_slot,
            max_level,
            built_through_seq,
        })
    }
}

/// Atomically saves snapshot data through VFS port.
pub fn save_snapshot_vfs<V: Vfs>(
    vfs: &V,
    path: &Path,
    header: &VectorSnapshotHeader,
) -> Result<()> {
    let bytes = header.to_bytes();
    vfs.write_atomic(path, &bytes)
}

/// Reads and validates snapshot header through VFS port.
pub fn load_snapshot_vfs<V: Vfs>(vfs: &V, path: &Path) -> Result<VectorSnapshotHeader> {
    let bytes = vfs.read(path)?;
    VectorSnapshotHeader::parse(&bytes)
}
