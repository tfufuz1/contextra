// FILE-CONTEXT
// ZWECK: Persistenz-Schicht für HNSW-Dateiserialisierung (`.hnsw`) - NodeRecord-Format.
// INVARIANTEN: Zero-Panic bei Deserialisierung.

use contextra_core::{ContextraError, Result};

/// Represents a node's metadata in the flat file.
#[derive(Debug, Clone, Copy)]
pub struct NodeRecord {
    #[cfg(not(feature = "docid-128"))]
    pub doc_id: u64,
    #[cfg(feature = "docid-128")]
    pub doc_id: u128,
    pub max_layer: u8,
    pub vector_offset: u64,
    pub connections_offset: u64,
}

impl NodeRecord {
    #[cfg(not(feature = "docid-128"))]
    pub const SIZE: usize = 8 + 1 + 8 + 8; // 25 bytes
    #[cfg(feature = "docid-128")]
    pub const SIZE: usize = 16 + 1 + 8 + 8; // 33 bytes

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(ContextraError::Storage("NodeRecord bytes too small".into()));
        }
        #[cfg(not(feature = "docid-128"))]
        let doc_id = u64::from_le_bytes(
            bytes
                .get(0..8)
                .ok_or_else(|| ContextraError::Storage("Invalid doc_id offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid doc_id bytes".into()))?,
        );
        #[cfg(feature = "docid-128")]
        let doc_id = u128::from_le_bytes(
            bytes
                .get(0..16)
                .ok_or_else(|| ContextraError::Storage("Invalid doc_id offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid doc_id bytes".into()))?,
        );

        let doc_id_bytes = std::mem::size_of_val(&doc_id);
        let max_layer = *bytes
            .get(doc_id_bytes)
            .ok_or_else(|| ContextraError::Storage("Invalid max_layer offset".into()))?;
        let vector_offset = u64::from_le_bytes(
            bytes
                .get(doc_id_bytes + 1..doc_id_bytes + 9)
                .ok_or_else(|| ContextraError::Storage("Invalid vector_offset offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid vector_offset bytes".into()))?,
        );
        let connections_offset = u64::from_le_bytes(
            bytes
                .get(doc_id_bytes + 9..doc_id_bytes + 17)
                .ok_or_else(|| ContextraError::Storage("Invalid connections_offset offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid connections_offset bytes".into()))?,
        );
        Ok(Self {
            doc_id,
            max_layer,
            vector_offset,
            connections_offset,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = vec![0u8; Self::SIZE];
        let doc_id_bytes = std::mem::size_of_val(&self.doc_id);
        buf[0..doc_id_bytes].copy_from_slice(&self.doc_id.to_le_bytes());
        buf[doc_id_bytes] = self.max_layer;
        buf[doc_id_bytes + 1..doc_id_bytes + 9].copy_from_slice(&self.vector_offset.to_le_bytes());
        buf[doc_id_bytes + 9..doc_id_bytes + 17]
            .copy_from_slice(&self.connections_offset.to_le_bytes());
        buf
    }
}
