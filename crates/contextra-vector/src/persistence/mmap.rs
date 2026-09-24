// FILE-CONTEXT
// ZWECK: mmap-basiertes Lesen für HNSW Vektor-Indizes.
// INVARIANTEN: Zero-Panic bei Deserialisierung; mmap-Reads überleben POSIX file replace.
// NICHT-OFFENSICHTLICH: MmapIndex hält read-only FD; Schreibvorgänge laufen atomar über .tmp und Rename.
// HOTSPOTS: persistence/mmap.rs (MmapIndex::open)

use super::{HnswHeader, NodeRecord};
use contextra_core::{ContextraError, Result};

#[derive(Clone)]
/// A reader for memory-mapped HNSW indices.
pub struct MmapIndex {
    pub mmap: std::sync::Arc<memmap2::Mmap>,
    pub header: HnswHeader,
}

impl MmapIndex {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let file = std::fs::File::open(path)
            .map_err(|e| ContextraError::Storage(format!("Failed to open HNSW file: {}", e)))?;

        let mmap = contextra_sys::mmap_readonly(&file)
            .map_err(|e| ContextraError::Storage(format!("Failed to mmap HNSW: {}", e)))?;

        let header_slice = mmap
            .get(0..HnswHeader::SIZE)
            .or_else(|| mmap.get(0..76))
            .or_else(|| mmap.get(0..64))
            .ok_or_else(|| ContextraError::Storage("HNSW file too small for header".into()))?;
        let header = HnswHeader::try_from_bytes(header_slice)?;

        let index_obj = Self {
            mmap: std::sync::Arc::new(mmap),
            header,
        };

        if index_obj.header.node_count() > 0 {
            let expected_nodes_bytes = (index_obj.header.node_count() as usize)
                .checked_mul(NodeRecord::SIZE)
                .ok_or_else(|| ContextraError::Storage("Node records total size overflow".into()))?;
            let actual_nodes_bytes = (index_obj.header.connections_offset() as usize)
                .checked_sub(index_obj.header.nodes_offset() as usize)
                .ok_or_else(|| {
                    ContextraError::Storage(
                        "Invalid nodes_offset / connections_offset relative position".into(),
                    )
                })?;
            if actual_nodes_bytes < expected_nodes_bytes {
                return Err(ContextraError::Storage(format!(
                    "NodeRecord size mismatch / DocId bit-width mismatch: expected at least {} bytes for {} nodes ({} bytes per record), found {} bytes between nodes_offset and connections_offset",
                    expected_nodes_bytes,
                    index_obj.header.node_count(),
                    NodeRecord::SIZE,
                    actual_nodes_bytes
                )));
            }

            if index_obj.header.version() == 2 {
                let rec0 = index_obj.get_node_record(0)?;
                let expected_vector_offset = index_obj
                    .header
                    .nodes_offset()
                    .checked_add(
                        index_obj
                            .header
                            .node_count()
                            .checked_mul(NodeRecord::SIZE as u64)
                            .ok_or_else(|| {
                                ContextraError::Storage("Node records total size overflow".into())
                            })?,
                    )
                    .ok_or_else(|| ContextraError::Storage("Vector offset overflow".into()))?;
                if rec0.vector_offset != expected_vector_offset {
                    return Err(ContextraError::Storage(format!(
                        "NodeRecord layout mismatch or DocId bit-width mismatch: record 0 vector_offset is {}, expected {}",
                        rec0.vector_offset, expected_vector_offset
                    )));
                }
            }
        }

        Ok(index_obj)
    }

    /// Opens an HNSW file without requiring external runtime blocking task spawns.
    pub async fn open_async(path: impl AsRef<std::path::Path> + Send) -> Result<Self> {
        Self::open(path)
    }

    pub fn get_node_record(&self, index: usize) -> Result<NodeRecord> {
        let byte_offset = index
            .checked_mul(NodeRecord::SIZE)
            .ok_or_else(|| ContextraError::Storage("Node index overflow in mmap".into()))?;
        let nodes_offset = self.header.nodes_offset() as usize;
        let offset = nodes_offset
            .checked_add(byte_offset)
            .ok_or_else(|| ContextraError::Storage("Node record offset overflow in mmap".into()))?;
        let record_end = offset
            .checked_add(NodeRecord::SIZE)
            .ok_or_else(|| ContextraError::Storage("Node record end overflow in mmap".into()))?;
        let slice = self
            .mmap
            .get(offset..record_end)
            .ok_or_else(|| ContextraError::Storage("Node record out of bounds".into()))?;
        NodeRecord::from_bytes(slice)
    }

    pub fn get_vector(&self, record: &NodeRecord) -> Result<&[u8]> {
        let dim = self.header.dimension() as usize;
        let size = if self.header.is_quantized() {
            dim
        } else {
            dim.checked_mul(4)
                .ok_or_else(|| ContextraError::Storage("Vector size overflow in mmap".into()))?
        };
        let offset = record.vector_offset as usize;
        let vector_end = offset
            .checked_add(size)
            .ok_or_else(|| ContextraError::Storage("Vector offset overflow in mmap".into()))?;
        let slice = self
            .mmap
            .get(offset..vector_end)
            .ok_or_else(|| ContextraError::Storage("Vector data out of bounds".into()))?;
        Ok(slice)
    }

    pub fn get_connections(&self, record: &NodeRecord, layer: usize) -> Result<Vec<u32>> {
        let offset = record.connections_offset as usize;
        let num_layers_byte = *self
            .mmap
            .get(offset)
            .ok_or_else(|| ContextraError::Storage("Connections offset out of bounds".into()))?;

        let num_layers = num_layers_byte as usize;
        if layer >= num_layers {
            return Ok(Vec::new());
        }

        let mut current_pos = offset
            .checked_add(1)
            .ok_or_else(|| ContextraError::Storage("Connection offset overflow".into()))?;
        for _ in 0..layer {
            let check_pos = current_pos.checked_add(4).ok_or_else(|| {
                ContextraError::Storage("Connection length position overflow".into())
            })?;
            let len_bytes = self
                .mmap
                .get(current_pos..check_pos)
                .ok_or_else(|| ContextraError::Storage("Connection length out of bounds".into()))?;
            let len = u32::from_le_bytes(
                len_bytes
                    .try_into()
                    .map_err(|_| ContextraError::Storage("Corrupt connection length".into()))?,
            ) as usize;

            let connections_byte_size = len.checked_mul(4).ok_or_else(|| {
                ContextraError::Storage("Connection count overflow in node record".into())
            })?;
            let new_pos = current_pos
                .checked_add(4)
                .and_then(|p| p.checked_add(connections_byte_size))
                .ok_or_else(|| {
                    ContextraError::Storage("Cumulative offset overflow in node connections".into())
                })?;
            current_pos = new_pos;
        }

        let check_pos = current_pos
            .checked_add(4)
            .ok_or_else(|| ContextraError::Storage("Connection length position overflow".into()))?;
        let len_bytes = self
            .mmap
            .get(current_pos..check_pos)
            .ok_or_else(|| ContextraError::Storage("Connection length out of bounds".into()))?;

        let len = u32::from_le_bytes(
            len_bytes
                .try_into()
                .map_err(|_| ContextraError::Storage("Corrupt connection length".into()))?,
        ) as usize;

        let connections_byte_size = len.checked_mul(4).ok_or_else(|| {
            ContextraError::Storage("Connection count overflow in node record".into())
        })?;
        let start = check_pos;
        let end = start
            .checked_add(connections_byte_size)
            .ok_or_else(|| ContextraError::Storage("Connection end offset overflow".into()))?;

        let raw = self
            .mmap
            .get(start..end)
            .ok_or_else(|| ContextraError::Storage("Connection data out of bounds".into()))?;

        let mut connections = Vec::with_capacity(len);
        for i in 0..len {
            let elem_offset = i
                .checked_mul(4)
                .ok_or_else(|| ContextraError::Storage("Connection index overflow".into()))?;
            let elem_end = elem_offset
                .checked_add(4)
                .ok_or_else(|| ContextraError::Storage("Connection elem end overflow".into()))?;
            let elem_slice = raw
                .get(elem_offset..elem_end)
                .ok_or_else(|| ContextraError::Storage("Connection slice out of bounds".into()))?;
            let val = u32::from_le_bytes(
                elem_slice
                    .try_into()
                    .map_err(|_| ContextraError::Storage("Corrupt connection value".into()))?,
            );
            connections.push(val);
        }

        Ok(connections)
    }
}
