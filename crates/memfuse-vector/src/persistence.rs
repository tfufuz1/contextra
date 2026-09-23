// FILE-CONTEXT
// ZWECK: Persistenz-Schicht für HNSW-Dateiserialisierung (`.hnsw`) und mmap-basiertes Lesen.
// INVARIANTEN: Zero-Panic bei Deserialisierung; mmap-Reads überleben POSIX file replace.
// NICHT-OFFENSICHTLICH: MmapIndex hält read-only FD; Schreibvorgänge laufen atomar über .tmp und Rename.
// HOTSPOTS: persistence.rs (HnswHeader::try_from_bytes, MmapIndex::open)
// STAND: TS:2026-08-30T18:53:53Z (SESSION: 37b1d991)

//! HNSW Persistence Layer — Serialisierung und mmap-Mapping für Vektor-Indizes.

use crate::hnsw::Sq8Bias;
use memfuse_core::{MemFuseError, Result};

/// Magic number for HNSW files (0x484E5357 = "HNSW").
pub const HNSW_MAGIC: u32 = 0x484E5357;
/// Current file format version.
pub const HNSW_VERSION: u16 = 2;

/// The header of an HNSW persistent file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HnswHeader {
    magic: u32,
    version: u16,
    dimension: u32,
    m: u32,
    metric: u8,
    quantized: u8,
    q_min: f32, // Legacy v1 field
    q_max: f32, // Legacy v1 field
    node_count: u64,
    entry_point: i64,
    nodes_offset: u64,
    connections_offset: u64,
    last_tx_id: u64,               // Added for Repair-on-Open
    quant_calibration_offset: u64, // Added in v2 for per-dimension calibration
    quant_calibration_len: u32,    // Added in v2 for per-dimension calibration
    sq8_bias_mean: f32,            // SQ8 quantization bias mean
    sq8_bias_variance: f32,        // SQ8 quantization bias variance
}

impl HnswHeader {
    pub const SIZE: usize = 84;

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dimension: u32,
        m: u32,
        metric: u8,
        quantized: u8,
        q_min: f32,
        q_max: f32,
        node_count: u64,
        entry_point: i64,
        nodes_offset: u64,
        connections_offset: u64,
        last_tx_id: u64,
    ) -> Self {
        Self::new_v2(
            dimension,
            m,
            metric,
            quantized,
            q_min,
            q_max,
            node_count,
            entry_point,
            nodes_offset,
            connections_offset,
            last_tx_id,
            0,
            0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_v2(
        dimension: u32,
        m: u32,
        metric: u8,
        quantized: u8,
        q_min: f32,
        q_max: f32,
        node_count: u64,
        entry_point: i64,
        nodes_offset: u64,
        connections_offset: u64,
        last_tx_id: u64,
        quant_calibration_offset: u64,
        quant_calibration_len: u32,
    ) -> Self {
        Self::new_v2_with_bias(
            dimension,
            m,
            metric,
            quantized,
            q_min,
            q_max,
            node_count,
            entry_point,
            nodes_offset,
            connections_offset,
            last_tx_id,
            quant_calibration_offset,
            quant_calibration_len,
            0.0,
            0.0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_v2_with_bias(
        dimension: u32,
        m: u32,
        metric: u8,
        quantized: u8,
        q_min: f32,
        q_max: f32,
        node_count: u64,
        entry_point: i64,
        nodes_offset: u64,
        connections_offset: u64,
        last_tx_id: u64,
        quant_calibration_offset: u64,
        quant_calibration_len: u32,
        sq8_bias_mean: f32,
        sq8_bias_variance: f32,
    ) -> Self {
        Self {
            magic: HNSW_MAGIC,
            version: HNSW_VERSION,
            dimension,
            m,
            metric,
            quantized,
            q_min,
            q_max,
            node_count,
            entry_point,
            nodes_offset,
            connections_offset,
            last_tx_id,
            quant_calibration_offset,
            quant_calibration_len,
            sq8_bias_mean,
            sq8_bias_variance,
        }
    }

    pub fn magic(&self) -> u32 {
        self.magic
    }

    pub fn version(&self) -> u16 {
        self.version
    }

    pub fn m(&self) -> u32 {
        self.m
    }

    pub fn metric(&self) -> u8 {
        self.metric
    }

    pub fn quantized(&self) -> u8 {
        self.quantized
    }

    pub fn q_min(&self) -> f32 {
        self.q_min
    }

    pub fn q_max(&self) -> f32 {
        self.q_max
    }

    pub fn nodes_offset(&self) -> u64 {
        self.nodes_offset
    }

    pub fn connections_offset(&self) -> u64 {
        self.connections_offset
    }

    pub fn set_connections_offset(&mut self, offset: u64) {
        self.connections_offset = offset;
    }

    pub fn node_count(&self) -> u64 {
        self.node_count
    }

    pub fn last_tx_id(&self) -> u64 {
        self.last_tx_id
    }

    pub fn quant_calibration_offset(&self) -> u64 {
        self.quant_calibration_offset
    }

    pub fn quant_calibration_len(&self) -> u32 {
        self.quant_calibration_len
    }

    pub fn sq8_bias_mean(&self) -> f32 {
        self.sq8_bias_mean
    }

    pub fn sq8_bias_variance(&self) -> f32 {
        self.sq8_bias_variance
    }

    pub fn sq8_bias(&self) -> Sq8Bias {
        Sq8Bias {
            mean_bias: self.sq8_bias_mean,
            variance_bias: self.sq8_bias_variance,
            sample_count: 0,
        }
    }

    pub fn dimension(&self) -> u32 {
        self.dimension
    }

    pub fn entry_point(&self) -> i64 {
        self.entry_point
    }

    pub fn is_quantized(&self) -> bool {
        self.quantized != 0
    }

    pub fn q_range(&self) -> (f32, f32) {
        (self.q_min, self.q_max)
    }

    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 64 {
            return Err(MemFuseError::Storage("Header too small".into()));
        }

        let magic = u32::from_le_bytes(
            bytes
                .get(0..4)
                .ok_or_else(|| MemFuseError::Storage("Invalid magic offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid magic bytes".into()))?,
        );
        if magic != HNSW_MAGIC {
            return Err(MemFuseError::Storage(
                "Not a valid HNSW file: bad magic".into(),
            ));
        }

        let version = u16::from_le_bytes(
            bytes
                .get(4..6)
                .ok_or_else(|| MemFuseError::Storage("Invalid version offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid version bytes".into()))?,
        );
        if version != 1 && version != 2 {
            return Err(MemFuseError::Storage(format!(
                "Unsupported HNSW version: {}, expected 1 or 2",
                version
            )));
        }

        let dimension = u32::from_le_bytes(
            bytes
                .get(6..10)
                .ok_or_else(|| MemFuseError::Storage("Invalid dimension offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid dimension bytes".into()))?,
        );

        let m = u32::from_le_bytes(
            bytes
                .get(10..14)
                .ok_or_else(|| MemFuseError::Storage("Invalid m offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid m bytes".into()))?,
        );

        let metric = *bytes
            .get(14)
            .ok_or_else(|| MemFuseError::Storage("Invalid metric offset".into()))?;

        let quantized = *bytes
            .get(15)
            .ok_or_else(|| MemFuseError::Storage("Invalid quantized offset".into()))?;

        let q_min = f32::from_le_bytes(
            bytes
                .get(16..20)
                .ok_or_else(|| MemFuseError::Storage("Invalid q_min offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid q_min bytes".into()))?,
        );

        let q_max = f32::from_le_bytes(
            bytes
                .get(20..24)
                .ok_or_else(|| MemFuseError::Storage("Invalid q_max offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid q_max bytes".into()))?,
        );

        let node_count = u64::from_le_bytes(
            bytes
                .get(24..32)
                .ok_or_else(|| MemFuseError::Storage("Invalid node_count offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid node_count bytes".into()))?,
        );

        let entry_point = i64::from_le_bytes(
            bytes
                .get(32..40)
                .ok_or_else(|| MemFuseError::Storage("Invalid entry_point offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid entry_point bytes".into()))?,
        );

        let nodes_offset = u64::from_le_bytes(
            bytes
                .get(40..48)
                .ok_or_else(|| MemFuseError::Storage("Invalid nodes_offset offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid nodes_offset bytes".into()))?,
        );

        let connections_offset = u64::from_le_bytes(
            bytes
                .get(48..56)
                .ok_or_else(|| MemFuseError::Storage("Invalid connections_offset offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid connections_offset bytes".into()))?,
        );

        let last_tx_id = u64::from_le_bytes(
            bytes
                .get(56..64)
                .ok_or_else(|| MemFuseError::Storage("Invalid last_tx_id offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid last_tx_id bytes".into()))?,
        );

        let (quant_calibration_offset, quant_calibration_len) = if version == 2 {
            if bytes.len() < 76 {
                return Err(MemFuseError::Storage("Header too small for v2".into()));
            }
            let off = u64::from_le_bytes(
                bytes
                    .get(64..72)
                    .ok_or_else(|| {
                        MemFuseError::Storage("Invalid quant_calibration_offset".into())
                    })?
                    .try_into()
                    .map_err(|_| {
                        MemFuseError::Storage("Invalid quant_calibration_offset bytes".into())
                    })?,
            );
            let len = u32::from_le_bytes(
                bytes
                    .get(72..76)
                    .ok_or_else(|| MemFuseError::Storage("Invalid quant_calibration_len".into()))?
                    .try_into()
                    .map_err(|_| {
                        MemFuseError::Storage("Invalid quant_calibration_len bytes".into())
                    })?,
            );
            (off, len)
        } else {
            tracing::warn!("Loading legacy HNSW file format version 1 — using degraded uniform quantization range fallback");
            (0, 0)
        };

        let (sq8_bias_mean, sq8_bias_variance) = if bytes.len() >= Self::SIZE {
            let mean = f32::from_le_bytes(
                bytes
                    .get(76..80)
                    .ok_or_else(|| MemFuseError::Storage("Invalid sq8_bias_mean".into()))?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("Invalid sq8_bias_mean bytes".into()))?,
            );
            let variance = f32::from_le_bytes(
                bytes
                    .get(80..84)
                    .ok_or_else(|| MemFuseError::Storage("Invalid sq8_bias_variance".into()))?
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("Invalid sq8_bias_variance bytes".into()))?,
            );
            (mean, variance)
        } else {
            (0.0, 0.0)
        };

        Ok(Self {
            magic,
            version,
            dimension,
            m,
            metric,
            quantized,
            q_min,
            q_max,
            node_count,
            entry_point,
            nodes_offset,
            connections_offset,
            last_tx_id,
            quant_calibration_offset,
            quant_calibration_len,
            sq8_bias_mean,
            sq8_bias_variance,
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic.to_le_bytes());
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6..10].copy_from_slice(&self.dimension.to_le_bytes());
        buf[10..14].copy_from_slice(&self.m.to_le_bytes());
        buf[14] = self.metric;
        buf[15] = self.quantized;
        buf[16..20].copy_from_slice(&self.q_min.to_le_bytes());
        buf[20..24].copy_from_slice(&self.q_max.to_le_bytes());
        buf[24..32].copy_from_slice(&self.node_count.to_le_bytes());
        buf[32..40].copy_from_slice(&self.entry_point.to_le_bytes());
        buf[40..48].copy_from_slice(&self.nodes_offset.to_le_bytes());
        buf[48..56].copy_from_slice(&self.connections_offset.to_le_bytes());
        buf[56..64].copy_from_slice(&self.last_tx_id.to_le_bytes());
        buf[64..72].copy_from_slice(&self.quant_calibration_offset.to_le_bytes());
        buf[72..76].copy_from_slice(&self.quant_calibration_len.to_le_bytes());
        buf[76..80].copy_from_slice(&self.sq8_bias_mean.to_le_bytes());
        buf[80..84].copy_from_slice(&self.sq8_bias_variance.to_le_bytes());
        buf
    }
}

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
            return Err(MemFuseError::Storage("NodeRecord bytes too small".into()));
        }
        #[cfg(not(feature = "docid-128"))]
        let doc_id = u64::from_le_bytes(
            bytes
                .get(0..8)
                .ok_or_else(|| MemFuseError::Storage("Invalid doc_id offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid doc_id bytes".into()))?,
        );
        #[cfg(feature = "docid-128")]
        let doc_id = u128::from_le_bytes(
            bytes
                .get(0..16)
                .ok_or_else(|| MemFuseError::Storage("Invalid doc_id offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid doc_id bytes".into()))?,
        );

        let doc_id_bytes = std::mem::size_of_val(&doc_id);
        let max_layer = *bytes
            .get(doc_id_bytes)
            .ok_or_else(|| MemFuseError::Storage("Invalid max_layer offset".into()))?;
        let vector_offset = u64::from_le_bytes(
            bytes
                .get(doc_id_bytes + 1..doc_id_bytes + 9)
                .ok_or_else(|| MemFuseError::Storage("Invalid vector_offset offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid vector_offset bytes".into()))?,
        );
        let connections_offset = u64::from_le_bytes(
            bytes
                .get(doc_id_bytes + 9..doc_id_bytes + 17)
                .ok_or_else(|| MemFuseError::Storage("Invalid connections_offset offset".into()))?
                .try_into()
                .map_err(|_| MemFuseError::Storage("Invalid connections_offset bytes".into()))?,
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

#[derive(Clone)]
/// A reader for memory-mapped HNSW indices.
pub struct MmapIndex {
    pub mmap: std::sync::Arc<memmap2::Mmap>,
    pub header: HnswHeader,
}

impl MmapIndex {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let file = std::fs::File::open(path)
            .map_err(|e| MemFuseError::Storage(format!("Failed to open HNSW file: {}", e)))?;

        let mmap = memfuse_sys::mmap_readonly(&file)
            .map_err(|e| MemFuseError::Storage(format!("Failed to mmap HNSW: {}", e)))?;

        let header_slice = mmap
            .get(0..HnswHeader::SIZE)
            .or_else(|| mmap.get(0..76))
            .or_else(|| mmap.get(0..64))
            .ok_or_else(|| MemFuseError::Storage("HNSW file too small for header".into()))?;
        let header = HnswHeader::try_from_bytes(header_slice)?;

        let index_obj = Self {
            mmap: std::sync::Arc::new(mmap),
            header,
        };

        if index_obj.header.node_count() > 0 {
            let expected_nodes_bytes = (index_obj.header.node_count() as usize)
                .checked_mul(NodeRecord::SIZE)
                .ok_or_else(|| MemFuseError::Storage("Node records total size overflow".into()))?;
            let actual_nodes_bytes = (index_obj.header.connections_offset() as usize)
                .checked_sub(index_obj.header.nodes_offset() as usize)
                .ok_or_else(|| {
                    MemFuseError::Storage(
                        "Invalid nodes_offset / connections_offset relative position".into(),
                    )
                })?;
            if actual_nodes_bytes < expected_nodes_bytes {
                return Err(MemFuseError::Storage(format!(
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
                                MemFuseError::Storage("Node records total size overflow".into())
                            })?,
                    )
                    .ok_or_else(|| MemFuseError::Storage("Vector offset overflow".into()))?;
                if rec0.vector_offset != expected_vector_offset {
                    return Err(MemFuseError::Storage(format!(
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
            .ok_or_else(|| MemFuseError::Storage("Node index overflow in mmap".into()))?;
        let nodes_offset = self.header.nodes_offset() as usize;
        let offset = nodes_offset
            .checked_add(byte_offset)
            .ok_or_else(|| MemFuseError::Storage("Node record offset overflow in mmap".into()))?;
        let record_end = offset
            .checked_add(NodeRecord::SIZE)
            .ok_or_else(|| MemFuseError::Storage("Node record end overflow in mmap".into()))?;
        let slice = self
            .mmap
            .get(offset..record_end)
            .ok_or_else(|| MemFuseError::Storage("Node record out of bounds".into()))?;
        NodeRecord::from_bytes(slice)
    }

    pub fn get_vector(&self, record: &NodeRecord) -> Result<&[u8]> {
        let dim = self.header.dimension() as usize;
        let size = if self.header.is_quantized() {
            dim
        } else {
            dim.checked_mul(4)
                .ok_or_else(|| MemFuseError::Storage("Vector size overflow in mmap".into()))?
        };
        let offset = record.vector_offset as usize;
        let vector_end = offset
            .checked_add(size)
            .ok_or_else(|| MemFuseError::Storage("Vector offset overflow in mmap".into()))?;
        let slice = self
            .mmap
            .get(offset..vector_end)
            .ok_or_else(|| MemFuseError::Storage("Vector data out of bounds".into()))?;
        Ok(slice)
    }

    pub fn get_connections(&self, record: &NodeRecord, layer: usize) -> Result<Vec<u32>> {
        let offset = record.connections_offset as usize;
        let num_layers_byte = *self
            .mmap
            .get(offset)
            .ok_or_else(|| MemFuseError::Storage("Connections offset out of bounds".into()))?;

        let num_layers = num_layers_byte as usize;
        if layer >= num_layers {
            return Ok(Vec::new());
        }

        let mut current_pos = offset
            .checked_add(1)
            .ok_or_else(|| MemFuseError::Storage("Connection offset overflow".into()))?;
        for _ in 0..layer {
            let check_pos = current_pos.checked_add(4).ok_or_else(|| {
                MemFuseError::Storage("Connection length position overflow".into())
            })?;
            let len_bytes = self
                .mmap
                .get(current_pos..check_pos)
                .ok_or_else(|| MemFuseError::Storage("Connection length out of bounds".into()))?;
            let len = u32::from_le_bytes(
                len_bytes
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("Corrupt connection length".into()))?,
            ) as usize;

            let connections_byte_size = len.checked_mul(4).ok_or_else(|| {
                MemFuseError::Storage("Connection count overflow in node record".into())
            })?;
            let new_pos = current_pos
                .checked_add(4)
                .and_then(|p| p.checked_add(connections_byte_size))
                .ok_or_else(|| {
                    MemFuseError::Storage("Cumulative offset overflow in node connections".into())
                })?;
            current_pos = new_pos;
        }

        let check_pos = current_pos
            .checked_add(4)
            .ok_or_else(|| MemFuseError::Storage("Connection length position overflow".into()))?;
        let len_bytes = self
            .mmap
            .get(current_pos..check_pos)
            .ok_or_else(|| MemFuseError::Storage("Connection length out of bounds".into()))?;

        let len = u32::from_le_bytes(
            len_bytes
                .try_into()
                .map_err(|_| MemFuseError::Storage("Corrupt connection length".into()))?,
        ) as usize;

        let connections_byte_size = len.checked_mul(4).ok_or_else(|| {
            MemFuseError::Storage("Connection count overflow in node record".into())
        })?;
        let start = check_pos;
        let end = start
            .checked_add(connections_byte_size)
            .ok_or_else(|| MemFuseError::Storage("Connection end offset overflow".into()))?;

        let raw = self
            .mmap
            .get(start..end)
            .ok_or_else(|| MemFuseError::Storage("Connection data out of bounds".into()))?;

        let mut connections = Vec::with_capacity(len);
        for i in 0..len {
            let elem_offset = i
                .checked_mul(4)
                .ok_or_else(|| MemFuseError::Storage("Connection index overflow".into()))?;
            let elem_end = elem_offset
                .checked_add(4)
                .ok_or_else(|| MemFuseError::Storage("Connection elem end overflow".into()))?;
            let elem_slice = raw
                .get(elem_offset..elem_end)
                .ok_or_else(|| MemFuseError::Storage("Connection slice out of bounds".into()))?;
            let val = u32::from_le_bytes(
                elem_slice
                    .try_into()
                    .map_err(|_| MemFuseError::Storage("Corrupt connection value".into()))?,
            );
            connections.push(val);
        }

        Ok(connections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_header_roundtrip_and_errors() -> Result<()> {
        let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 100, 5, 84, 256, 42);

        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), HnswHeader::SIZE);
        let parsed = HnswHeader::try_from_bytes(&bytes)?;
        assert_eq!(header, parsed);

        let small_bytes = vec![0u8; 10];
        let err_small = HnswHeader::try_from_bytes(&small_bytes);
        assert!(matches!(err_small, Err(MemFuseError::Storage(_))));

        let mut bad_magic_bytes = bytes;
        bad_magic_bytes[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        let err_magic = HnswHeader::try_from_bytes(&bad_magic_bytes);
        assert!(matches!(err_magic, Err(MemFuseError::Storage(_))));

        Ok(())
    }

    #[test]
    fn test_node_record_roundtrip_and_errors() -> Result<()> {
        let record = NodeRecord {
            doc_id: 12345,
            max_layer: 3,
            vector_offset: 500,
            connections_offset: 1500,
        };

        let bytes = record.to_bytes();
        assert_eq!(bytes.len(), NodeRecord::SIZE);
        let parsed = NodeRecord::from_bytes(&bytes)?;
        assert_eq!(record.doc_id, parsed.doc_id);
        assert_eq!(record.max_layer, parsed.max_layer);
        assert_eq!(record.vector_offset, parsed.vector_offset);
        assert_eq!(record.connections_offset, parsed.connections_offset);

        let small_bytes = vec![0u8; 10];
        let err = NodeRecord::from_bytes(&small_bytes);
        assert!(matches!(err, Err(MemFuseError::Storage(_))));

        Ok(())
    }

    #[tokio::test]
    async fn test_mmap_index_boundary_checks() -> memfuse_core::Result<()> {
        use crate::hnsw::{HnswConfig, HnswIndex};
        use memfuse_core::{DistanceMetric, DocId, TxId, VectorIndex};

        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("bounds_test.hnsw");

        let config = HnswConfig {
            dimension: 4,
            m: 16,
            distance_metric: DistanceMetric::Euclidean,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config.clone())?;
        let tx = TxId::new(1);
        index
            .insert(tx, DocId::new(1), &[1.0, 2.0, 3.0, 4.0])
            .await?;
        index.commit(tx).await?;
        index.save(&path).await?;

        let mmap_index = MmapIndex::open(&path)?;

        assert!(mmap_index.get_node_record(99999).is_err());

        let rec = mmap_index.get_node_record(0)?;
        let connections = mmap_index.get_connections(&rec, 100)?;
        assert!(connections.is_empty());

        let vec_bytes = mmap_index.get_vector(&rec)?;
        assert_eq!(vec_bytes.len(), 4 * 4);

        Ok(())
    }

    #[tokio::test]
    async fn test_mmap_open_async() -> memfuse_core::Result<()> {
        use crate::hnsw::{HnswConfig, HnswIndex};
        use memfuse_core::{DistanceMetric, DocId, TxId, VectorIndex};

        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("test_async.hnsw");

        let config = HnswConfig {
            dimension: 4,
            m: 16,
            ef_construction: 200,
            ef_search: 64,
            distance_metric: DistanceMetric::Euclidean,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config.clone())?;
        let tx = TxId::new(1);

        let mut vectors = Vec::with_capacity(100);
        for i in 1..=100u64 {
            let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32, (i * 4) as f32];
            index.insert(tx, DocId::from(i), &vec).await?;
            vectors.push((DocId::from(i), vec));
        }
        index.commit(tx).await?;

        index.save(&path).await?;

        let mmap_index = HnswIndex::try_new(config)?;
        mmap_index.load_mmap(&path).await?;

        let query = vec![50.1, 100.2, 150.3, 200.4];
        let search_results = mmap_index.search(&query, 5).await?;
        assert_eq!(search_results.len(), 5);

        let mut brute_force: Vec<(DocId, f32)> = vectors
            .iter()
            .map(|(doc_id, v)| {
                let dist: f32 = v
                    .iter()
                    .zip(query.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    .sqrt();
                (*doc_id, dist)
            })
            .collect();
        brute_force.sort_by(|a, b| a.1.total_cmp(&b.1));

        let expected_doc_ids: Vec<DocId> = brute_force.iter().take(5).map(|(id, _)| *id).collect();
        let returned_doc_ids: Vec<DocId> = search_results.iter().map(|r| r.doc_id).collect();

        assert_eq!(returned_doc_ids, expected_doc_ids);
        Ok(())
    }

    #[test]
    fn test_mmap_index_error_paths() {
        let res = MmapIndex::open("non_existent_path_memfuse_test.hnsw");
        assert!(res.is_err());

        let short_bytes = vec![0u8; 10];
        assert!(HnswHeader::try_from_bytes(&short_bytes).is_err());

        let invalid_magic = vec![0u8; HnswHeader::SIZE];
        assert!(HnswHeader::try_from_bytes(&invalid_magic).is_err());

        let short_node = vec![0u8; 10];
        assert!(NodeRecord::from_bytes(&short_node).is_err());
    }

    #[test]
    fn test_open_rejects_invalid_magic() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("invalid_magic.hnsw");
        let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
        let mut bytes = header.to_bytes().to_vec();
        bytes[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        std::fs::write(&path, &bytes).map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let res = MmapIndex::open(&path);
        assert!(res.is_err(), "Expected error for invalid magic");
        if let Err(MemFuseError::Storage(msg)) = res {
            assert!(
                msg.contains("bad magic"),
                "Unexpected error message: {}",
                msg
            );
        } else {
            panic!("Expected Storage error");
        }
        Ok(())
    }

    #[test]
    fn test_open_rejects_unsupported_version() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("unsupported_version.hnsw");
        let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
        let mut bytes = header.to_bytes().to_vec();
        let unsupported_version = 99u16;
        bytes[4..6].copy_from_slice(&unsupported_version.to_le_bytes());
        std::fs::write(&path, &bytes).map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let res = MmapIndex::open(&path);
        assert!(res.is_err(), "Expected error for unsupported version");
        if let Err(MemFuseError::Storage(msg)) = res {
            assert!(
                msg.contains("Unsupported HNSW version"),
                "Unexpected error message: {}",
                msg
            );
        } else {
            panic!("Expected Storage error");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_v1_legacy_migration_load_degraded_fallback() -> Result<()> {
        use crate::hnsw::{HnswConfig, HnswIndex};

        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("legacy_v1.hnsw");

        let v1_header_bytes = [
            0x57, 0x53, 0x4E, 0x48, // magic "HNSW"
            0x01, 0x00, // version 1
            0x02, 0x00, 0x00, 0x00, // dimension 2
            0x10, 0x00, 0x00, 0x00, // m 16
            0x01, // metric Euclidean
            0x01, // quantized = 1
            0x00, 0x00, 0x00, 0x00, // q_min = 0.0
            0x00, 0x00, 0x80, 0x3F, // q_max = 1.0
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // node_count = 1
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // entry_point = 0
            64, 0, 0, 0, 0, 0, 0, 0, // nodes_offset = 64
            89, 0, 0, 0, 0, 0, 0, 0, // connections_offset = 89
            0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // last_tx_id = 1
        ];

        let mut file_bytes = v1_header_bytes.to_vec();
        let record = NodeRecord {
            doc_id: 1,
            max_layer: 0,
            vector_offset: 89 + 1 + 4 + 4,
            connections_offset: 89,
        };
        file_bytes.extend_from_slice(&record.to_bytes());
        file_bytes.push(1);
        file_bytes.extend_from_slice(&1u32.to_le_bytes());
        file_bytes.extend_from_slice(&0u32.to_le_bytes());
        file_bytes.push(127);
        file_bytes.push(127);

        std::fs::write(&path, &file_bytes).map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let config = HnswConfig {
            dimension: 2,
            quantize: true,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config)?;
        index.load_mmap(&path).await?;

        let q = index
            .quantizer()
            .expect("Quantizer must be populated on mmap load");
        assert_eq!(q.mins(), &[0.0, 0.0]);
        assert_eq!(q.maxes(), &[1.0, 1.0]);

        Ok(())
    }

    #[tokio::test]
    async fn test_v2_save_load_roundtrip_preserves_per_dim_calibration() -> Result<()> {
        use crate::hnsw::{HnswConfig, HnswIndex};
        use memfuse_core::{DocId, TxId, VectorIndex};

        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("v2_roundtrip.hnsw");

        let config = HnswConfig {
            dimension: 2,
            quantize: true,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config.clone())?;
        let tx = TxId::new(1);

        for i in 0..60u64 {
            let v0 = (i as f32) / 59.0;
            let v1 = (i as f32) * 1000.0 / 59.0;
            index.insert(tx, DocId::from(i + 1), &[v0, v1]).await?;
        }
        index.commit(tx).await?;

        index.save(&path).await?;

        let loaded_index = HnswIndex::try_new(config)?;
        loaded_index.load_mmap(&path).await?;

        let q = loaded_index
            .quantizer()
            .expect("Quantizer must be restored");
        assert!(
            (q.mins()[0] - 0.0).abs() < 1e-2,
            "mins[0] was {}",
            q.mins()[0]
        );
        assert!(
            (q.mins()[1] - 0.0).abs() < 1e-2,
            "mins[1] was {}",
            q.mins()[1]
        );
        assert!(
            (q.maxes()[0] - 1.0).abs() < 1e-2,
            "maxes[0] was {}",
            q.maxes()[0]
        );
        assert!(
            (q.maxes()[1] - 1000.0).abs() < 10.0,
            "maxes[1] was {}",
            q.maxes()[1]
        );

        Ok(())
    }

    #[test]
    fn test_open_rejects_truncated_header() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("truncated_header.hnsw");
        let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
        let bytes = header.to_bytes();
        std::fs::write(&path, &bytes[..20]).map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let res = MmapIndex::open(&path);
        assert!(res.is_err(), "Expected error for truncated header");
        if let Err(MemFuseError::Storage(msg)) = res {
            assert!(
                msg.contains("too small"),
                "Unexpected error message: {}",
                msg
            );
        } else {
            panic!("Expected Storage error");
        }
        Ok(())
    }

    #[test]
    fn test_get_connections_rejects_out_of_bounds_offset() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("oob_connections.hnsw");
        let header = HnswHeader::new(
            128,
            16,
            1,
            0,
            -1.0,
            1.0,
            1,
            0,
            HnswHeader::SIZE as u64,
            (HnswHeader::SIZE + 25) as u64,
            1,
        );
        let mut file_bytes = header.to_bytes().to_vec();
        let record = NodeRecord {
            doc_id: 1,
            max_layer: 1,
            vector_offset: (HnswHeader::SIZE + 25) as u64,
            connections_offset: 9999,
        };
        file_bytes.extend_from_slice(&record.to_bytes());
        std::fs::write(&path, &file_bytes).map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let mmap_index = MmapIndex::open(&path)?;
        let res = mmap_index.get_connections(&record, 0);
        assert!(
            res.is_err(),
            "Expected error for out-of-bounds connections_offset"
        );
        if let Err(MemFuseError::Storage(msg)) = res {
            assert!(
                msg.contains("out of bounds"),
                "Unexpected error message: {}",
                msg
            );
        } else {
            panic!("Expected Storage error");
        }
        Ok(())
    }

    #[test]
    fn test_get_vector_rejects_out_of_bounds_offset() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("oob_vector.hnsw");
        let header = HnswHeader::new(
            128,
            16,
            1,
            0,
            -1.0,
            1.0,
            1,
            0,
            HnswHeader::SIZE as u64,
            (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
            1,
        );
        let mut file_bytes = header.to_bytes().to_vec();
        let record = NodeRecord {
            doc_id: 1,
            max_layer: 1,
            vector_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
            connections_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
        };
        file_bytes.extend_from_slice(&record.to_bytes());
        std::fs::write(&path, &file_bytes).map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let mmap_index = MmapIndex::open(&path)?;
        let mut bad_record = record;
        bad_record.vector_offset = 9999;
        let res = mmap_index.get_vector(&bad_record);
        assert!(
            res.is_err(),
            "Expected error for out-of-bounds vector_offset"
        );
        if let Err(MemFuseError::Storage(msg)) = res {
            assert!(
                msg.contains("out of bounds"),
                "Unexpected error message: {}",
                msg
            );
        } else {
            panic!("Expected Storage error");
        }
        Ok(())
    }

    #[test]
    fn test_get_connections_rejects_absurd_length_field() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(|e| MemFuseError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("absurd_length.hnsw");
        let conn_offset = HnswHeader::SIZE + NodeRecord::SIZE;
        let header = HnswHeader::new(
            4,
            16,
            1,
            0,
            -1.0,
            1.0,
            1,
            0,
            HnswHeader::SIZE as u64,
            conn_offset as u64,
            1,
        );
        let mut file_bytes = header.to_bytes().to_vec();
        let record = NodeRecord {
            doc_id: 1,
            max_layer: 1,
            vector_offset: conn_offset as u64,
            connections_offset: conn_offset as u64,
        };
        file_bytes.extend_from_slice(&record.to_bytes());
        file_bytes.push(1);
        file_bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        std::fs::write(&path, &file_bytes).map_err(|e| MemFuseError::Storage(e.to_string()))?;

        let mmap_index = MmapIndex::open(&path)?;
        let res = mmap_index.get_connections(&record, 0);
        assert!(res.is_err(), "Expected error for absurd length field");
        if let Err(MemFuseError::Storage(msg)) = res {
            assert!(
                msg.contains("overflow") || msg.contains("out of bounds"),
                "Unexpected error message: {}",
                msg
            );
        } else {
            panic!("Expected Storage error");
        }
        Ok(())
    }
}
