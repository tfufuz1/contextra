// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: CopyRegEntry und CopyKind Datentypen für CopyRegistry.
// INVARIANTEN: Zero-Panic, Fail-Closed.

use contextra_types::{ContextraError, Result};

/// Kind of copy recorded in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyKind {
    /// Physical OS hardlink clone.
    Hardlink = 1,
    /// Reflink / Copy-on-Write clone.
    Reflink = 2,
    /// Byte-for-byte full copy.
    FullCopy = 3,
}

/// An entry in the copy registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyRegEntry {
    /// Unique identifier for this copy entry.
    pub copy_id: [u8; 16],
    /// The kind of copy (Hardlink, Reflink, or FullCopy).
    pub kind: CopyKind,
    /// Identifiers of files included in this copy.
    pub file_ids: Vec<[u8; 16]>,
    /// Transaction ID when this copy was created.
    pub created_tx: u64,
}

impl CopyRegEntry {
    /// Serializes the copy registry entry into a byte vector.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.copy_id);
        let kind_u8 = match self.kind {
            CopyKind::Hardlink => 1u8,
            CopyKind::Reflink => 2u8,
            CopyKind::FullCopy => 3u8,
        };
        buf.push(kind_u8);
        #[allow(clippy::cast_possible_truncation)]
        buf.extend_from_slice(&(self.file_ids.len() as u32).to_be_bytes());
        for fid in &self.file_ids {
            buf.extend_from_slice(fid);
        }
        buf.extend_from_slice(&self.created_tx.to_be_bytes());
        Ok(buf)
    }

    /// Deserializes a copy registry entry from a byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 16 + 1 + 4 + 8 {
            return Err(ContextraError::Serialization(
                "CopyRegEntry too short".into(),
            ));
        }

        let mut copy_id = [0u8; 16];
        copy_id.copy_from_slice(&bytes[0..16]);

        let kind = match bytes[16] {
            1 => CopyKind::Hardlink,
            2 => CopyKind::Reflink,
            3 => CopyKind::FullCopy,
            k => {
                return Err(ContextraError::Serialization(format!(
                    "Unknown CopyKind {}",
                    k
                )))
            }
        };

        let file_count = u32::from_be_bytes(bytes[17..21].try_into().unwrap_or([0; 4])) as usize;
        let expected_len = 16 + 1 + 4 + (file_count * 16) + 8;
        if bytes.len() < expected_len {
            return Err(ContextraError::Serialization(
                "CopyRegEntry truncated file_ids".into(),
            ));
        }

        let mut offset = 21;
        let mut file_ids = Vec::with_capacity(file_count);
        for _ in 0..file_count {
            let mut fid = [0u8; 16];
            fid.copy_from_slice(&bytes[offset..offset + 16]);
            file_ids.push(fid);
            offset += 16;
        }

        let created_tx = u64::from_be_bytes(bytes[offset..offset + 8].try_into().unwrap_or([0; 8]));

        Ok(Self {
            copy_id,
            kind,
            file_ids,
            created_tx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_reg_entry_roundtrip() {
        let entry = CopyRegEntry {
            copy_id: [1u8; 16],
            kind: CopyKind::Hardlink,
            file_ids: vec![[2u8; 16], [3u8; 16]],
            created_tx: 42,
        };

        let bytes = entry.to_bytes().expect("serialization failed");
        let restored = CopyRegEntry::from_bytes(&bytes).expect("deserialization failed");

        assert_eq!(entry, restored);
    }
}
