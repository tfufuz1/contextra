// FILE-CONTEXT
// ZWECK: Persistenz-Schicht für HNSW-Dateiserialisierung (`.hnsw`) - NodeRecord-Format.
// INVARIANTEN: Zero-Panic bei Deserialisierung.

use contextra_types::{ContextraError, DocId, DocIdRaw, Result};

/// Represents a node's metadata in the flat file.
#[derive(Debug, Clone, Copy)]
pub struct NodeRecord {
    pub doc_id: DocIdRaw,
    pub max_layer: u8,
    pub vector_offset: u64,
    pub connections_offset: u64,
}

impl NodeRecord {
    pub const SIZE: usize = DocId::BYTES + 1 + 8 + 8;

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(ContextraError::Storage("NodeRecord bytes too small".into()));
        }
        let doc_id_bytes: [u8; DocId::BYTES] = bytes
            .get(0..DocId::BYTES)
            .ok_or_else(|| ContextraError::Storage("Invalid doc_id offset".into()))?
            .try_into()
            .map_err(|_| ContextraError::Storage("Invalid doc_id bytes".into()))?;
        let doc_id = DocId::from_le_bytes(doc_id_bytes).raw();

        let doc_id_len = DocId::BYTES;
        let max_layer = *bytes
            .get(doc_id_len)
            .ok_or_else(|| ContextraError::Storage("Invalid max_layer offset".into()))?;
        let vector_offset = u64::from_le_bytes(
            bytes
                .get(doc_id_len + 1..doc_id_len + 9)
                .ok_or_else(|| ContextraError::Storage("Invalid vector_offset offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid vector_offset bytes".into()))?,
        );
        let connections_offset = u64::from_le_bytes(
            bytes
                .get(doc_id_len + 9..doc_id_len + 17)
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
        let doc_id = DocId::from(self.doc_id);
        let doc_id_bytes = doc_id.to_le_bytes();
        let doc_id_len = DocId::BYTES;
        buf[0..doc_id_len].copy_from_slice(&doc_id_bytes);
        buf[doc_id_len] = self.max_layer;
        buf[doc_id_len + 1..doc_id_len + 9].copy_from_slice(&self.vector_offset.to_le_bytes());
        buf[doc_id_len + 9..doc_id_len + 17]
            .copy_from_slice(&self.connections_offset.to_le_bytes());
        buf
    }
}
