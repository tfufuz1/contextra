// FILE-CONTEXT
// ZWECK: Dekodieren und Laden einzelner Graphknoten aus dem Mmap (WP-VEC-07).
// INVARIANTEN: Zero-Panic bei unvollständigen Knoten oder korrupten Nachbarlisten.
// STAND: 2026-11-11

use super::super::types::{CachedNode, DiskAnnIndex};
use crate::error::{Result, VectorError};
use crate::types::DocId;

impl DiskAnnIndex {
    pub(crate) fn load_node(&self, index: u32) -> Result<CachedNode> {
        use std::sync::atomic::Ordering;
        if let Some(node) = self.inner.cache.read().get(&index) {
            return Ok(node.clone());
        }

        let mmap_guard = self.inner.mmap.read();
        let mmap = mmap_guard
            .as_ref()
            .ok_or_else(|| VectorError::Index("Index not loaded".into()))?;
        let header_guard = self.inner.header.read();
        let header = header_guard
            .as_ref()
            .ok_or_else(|| VectorError::Index("Header missing".into()))?;

        let sector_size = header.sector_size as usize;
        let node_size = self.inner.node_size_bytes.load(Ordering::SeqCst) as usize;
        let read_size = node_size;
        if !read_size.is_multiple_of(sector_size) {
            return Err(VectorError::Index(
                "Read size must be a multiple of sector_size".into(),
            ));
        }

        let start_offset = DiskAnnHeader::SIZE.div_ceil(sector_size) * sector_size;
        let index_offset = (index as usize)
            .checked_mul(node_size)
            .ok_or_else(|| VectorError::Index("Node offset multiplication overflow".into()))?;
        let node_offset = start_offset
            .checked_add(index_offset)
            .ok_or_else(|| VectorError::Index("Node offset addition overflow".into()))?;
        if node_offset % sector_size != 0 {
            return Err(VectorError::Index(
                "Node read offset must be sector-aligned".into(),
            ));
        }

        let end_offset = node_offset
            .checked_add(node_size)
            .ok_or_else(|| VectorError::Index("Node end offset addition overflow".into()))?;

        if end_offset > mmap.len() {
            return Err(VectorError::Index("Node offset out of bounds".into()));
        }

        let node_data = mmap
            .get(node_offset..end_offset)
            .ok_or_else(|| VectorError::Index("Node data out of bounds".into()))?;
        let mut cursor: usize = 0;

        let vector = if header.quantized != 0 {
            let dim = header.dimension as usize;
            let next_cursor = cursor
                .checked_add(dim)
                .ok_or_else(|| VectorError::Index("Cursor overflow in quantized vector".into()))?;
            let slice = node_data
                .get(cursor..next_cursor)
                .ok_or_else(|| VectorError::Index("Truncated quantized vector data".into()))?;
            cursor = next_cursor;
            VectorData::U8(slice.to_vec())
        } else {
            let dim = header.dimension as usize;
            let mut v = Vec::with_capacity(dim);
            for _ in 0..dim {
                let next_cursor = cursor
                    .checked_add(4)
                    .ok_or_else(|| VectorError::Index("Cursor overflow in f32 vector".into()))?;
                let slice = node_data
                    .get(cursor..next_cursor)
                    .ok_or_else(|| VectorError::Index("Truncated f32 vector data".into()))?;
                v.push(f32::from_le_bytes(slice.try_into().map_err(|_| {
                    VectorError::Index("Invalid vector data".into())
                })?));
                cursor = next_cursor;
            }
            VectorData::F32(v)
        };

        let next_cursor = cursor
            .checked_add(4)
            .ok_or_else(|| VectorError::Index("Cursor overflow in neighbor count".into()))?;
        let count_bytes = node_data
            .get(cursor..next_cursor)
            .ok_or_else(|| VectorError::Index("Truncated neighbor count".into()))?;
        let neighbor_count = u32::from_le_bytes(
            count_bytes
                .try_into()
                .map_err(|_| VectorError::Index("Invalid neighbor count".into()))?,
        ) as usize;
        cursor = next_cursor;

        if neighbor_count > header.max_degree as usize {
            return Err(VectorError::Index(format!(
                "Corrupt DiskANN node: neighbor_count {} > max_degree {}",
                neighbor_count, header.max_degree
            )));
        }

        let mut neighbors = Vec::with_capacity(neighbor_count);
        for _ in 0..neighbor_count {
            let nxt = cursor
                .checked_add(4)
                .ok_or_else(|| VectorError::Index("Cursor overflow in neighbor ID".into()))?;
            let slice = node_data
                .get(cursor..nxt)
                .ok_or_else(|| VectorError::Index("Truncated neighbor ID".into()))?;
            neighbors.push(u32::from_le_bytes(
                slice
                    .try_into()
                    .map_err(|_| VectorError::Index("Invalid neighbor ID".into()))?,
            ));
            cursor = nxt;
        }

        let skip_bytes = (header.max_degree as usize)
            .checked_sub(neighbor_count)
            .and_then(|diff| diff.checked_mul(4))
            .ok_or_else(|| VectorError::Index("Padding overflow".into()))?;
        cursor = cursor
            .checked_add(skip_bytes)
            .ok_or_else(|| VectorError::Index("Cursor overflow in padding".into()))?;

        let doc_id_size = std::mem::size_of::<DocId>();
        let doc_id_end = cursor
            .checked_add(doc_id_size)
            .ok_or_else(|| VectorError::Index("Cursor overflow in DocId".into()))?;
        let doc_id_bytes = node_data
            .get(cursor..doc_id_end)
            .ok_or_else(|| VectorError::Index("Truncated DocId".into()))?;
        #[cfg(not(feature = "docid-128"))]
        let doc_id = DocId::from(u64::from_le_bytes(
            doc_id_bytes
                .try_into()
                .map_err(|_| VectorError::Index("Invalid DocId".into()))?,
        ));
        #[cfg(feature = "docid-128")]
        let doc_id = DocId::from(u128::from_le_bytes(
            doc_id_bytes
                .try_into()
                .map_err(|_| VectorError::Index("Invalid DocId".into()))?,
        ));

        let node = CachedNode {
            vector,
            neighbors,
            doc_id,
        };

        let mut cache = self.inner.cache.write();
        if cache.len() * node_size >= self.inner.config.memory_budget {
            // FIND-IND-004: Replace full cache wipe with 25% partial eviction
            let to_remove = cache.len() / 4;
            let keys: Vec<_> = cache.keys().take(to_remove).cloned().collect();
            for k in keys {
                cache.remove(&k);
            }
        }
        cache.insert(index, node.clone());

        Ok(node)
    }
}
