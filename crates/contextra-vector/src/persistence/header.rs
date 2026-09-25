// FILE-CONTEXT
// ZWECK: Persistenz-Schicht für HNSW-Dateiserialisierung (`.hnsw`) - Header-Format.
// INVARIANTEN: Zero-Panic bei Deserialisierung.

use crate::hnsw::Sq8Bias;
use contextra_core::{ContextraError, Result};

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
            return Err(ContextraError::Storage("Header too small".into()));
        }

        let magic = u32::from_le_bytes(
            bytes
                .get(0..4)
                .ok_or_else(|| ContextraError::Storage("Invalid magic offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid magic bytes".into()))?,
        );
        if magic != HNSW_MAGIC {
            return Err(ContextraError::Storage(
                "Not a valid HNSW file: bad magic".into(),
            ));
        }

        let version = u16::from_le_bytes(
            bytes
                .get(4..6)
                .ok_or_else(|| ContextraError::Storage("Invalid version offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid version bytes".into()))?,
        );
        if version != 1 && version != 2 {
            return Err(ContextraError::Storage(format!(
                "Unsupported HNSW version: {}, expected 1 or 2",
                version
            )));
        }

        let dimension = u32::from_le_bytes(
            bytes
                .get(6..10)
                .ok_or_else(|| ContextraError::Storage("Invalid dimension offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid dimension bytes".into()))?,
        );

        let m = u32::from_le_bytes(
            bytes
                .get(10..14)
                .ok_or_else(|| ContextraError::Storage("Invalid m offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid m bytes".into()))?,
        );

        let metric = *bytes
            .get(14)
            .ok_or_else(|| ContextraError::Storage("Invalid metric offset".into()))?;

        let quantized = *bytes
            .get(15)
            .ok_or_else(|| ContextraError::Storage("Invalid quantized offset".into()))?;

        let q_min = f32::from_le_bytes(
            bytes
                .get(16..20)
                .ok_or_else(|| ContextraError::Storage("Invalid q_min offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid q_min bytes".into()))?,
        );

        let q_max = f32::from_le_bytes(
            bytes
                .get(20..24)
                .ok_or_else(|| ContextraError::Storage("Invalid q_max offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid q_max bytes".into()))?,
        );

        let node_count = u64::from_le_bytes(
            bytes
                .get(24..32)
                .ok_or_else(|| ContextraError::Storage("Invalid node_count offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid node_count bytes".into()))?,
        );

        let entry_point = i64::from_le_bytes(
            bytes
                .get(32..40)
                .ok_or_else(|| ContextraError::Storage("Invalid entry_point offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid entry_point bytes".into()))?,
        );

        let nodes_offset = u64::from_le_bytes(
            bytes
                .get(40..48)
                .ok_or_else(|| ContextraError::Storage("Invalid nodes_offset offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid nodes_offset bytes".into()))?,
        );

        let connections_offset = u64::from_le_bytes(
            bytes
                .get(48..56)
                .ok_or_else(|| ContextraError::Storage("Invalid connections_offset offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid connections_offset bytes".into()))?,
        );

        let last_tx_id = u64::from_le_bytes(
            bytes
                .get(56..64)
                .ok_or_else(|| ContextraError::Storage("Invalid last_tx_id offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Storage("Invalid last_tx_id bytes".into()))?,
        );

        let (quant_calibration_offset, quant_calibration_len) = if version == 2 {
            if bytes.len() < 76 {
                return Err(ContextraError::Storage("Header too small for v2".into()));
            }
            let off = u64::from_le_bytes(
                bytes
                    .get(64..72)
                    .ok_or_else(|| {
                        ContextraError::Storage("Invalid quant_calibration_offset".into())
                    })?
                    .try_into()
                    .map_err(|_| {
                        ContextraError::Storage("Invalid quant_calibration_offset bytes".into())
                    })?,
            );
            let len = u32::from_le_bytes(
                bytes
                    .get(72..76)
                    .ok_or_else(|| ContextraError::Storage("Invalid quant_calibration_len".into()))?
                    .try_into()
                    .map_err(|_| {
                        ContextraError::Storage("Invalid quant_calibration_len bytes".into())
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
                    .ok_or_else(|| ContextraError::Storage("Invalid sq8_bias_mean".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Storage("Invalid sq8_bias_mean bytes".into()))?,
            );
            let variance = f32::from_le_bytes(
                bytes
                    .get(80..84)
                    .ok_or_else(|| ContextraError::Storage("Invalid sq8_bias_variance".into()))?
                    .try_into()
                    .map_err(|_| {
                        ContextraError::Storage("Invalid sq8_bias_variance bytes".into())
                    })?,
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
