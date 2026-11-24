// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)
use crate::error::{Result, VectorError};

pub(crate) const DISKANN_MAGIC: &[u8; 4] = b"DANN";
pub(crate) const DISKANN_VERSION: u16 = 1;
pub(crate) const DISKANN_FOOTER_MAGIC: &[u8; 4] = b"DANF";
pub(crate) const DISKANN_INTEGRITY_KEY: &[u8; 32] = b"contextra_diskann_integri_key_32";

pub(crate) const PENDING_WAL_MAGIC: &[u8; 4] = b"PWAL";
pub(crate) const PENDING_WAL_VERSION: u8 = 1;
pub(crate) const PENDING_WAL_HEADER_SIZE: usize = 5;

pub(crate) const TOMBSTONE_WAL_MAGIC: &[u8; 4] = b"TWAL";
pub(crate) const TOMBSTONE_WAL_VERSION: u8 = 1;
pub(crate) const TOMBSTONE_WAL_HEADER_SIZE: usize = 5;
pub(crate) const MAX_PENDING_WAL_DIM: usize = 65_536;

pub(crate) const PENDING_FLUSH_THRESHOLD_MIN: u64 = 50;
pub(crate) const PENDING_FLUSH_THRESHOLD_MAX: u64 = 1_000;
pub(crate) const PENDING_FLUSH_THRESHOLD_FACTOR: f64 = 0.05;

#[inline]
pub(crate) fn compute_adaptive_flush_threshold(n_persisted: u64) -> u64 {
    let adaptive = (n_persisted as f64 * PENDING_FLUSH_THRESHOLD_FACTOR).floor() as u64;
    adaptive.clamp(PENDING_FLUSH_THRESHOLD_MIN, PENDING_FLUSH_THRESHOLD_MAX)
}

/// Header for DiskANN index file.
#[derive(Debug, Clone, Copy)]
pub(crate) struct DiskAnnHeader {
    pub(crate) magic: [u8; 4],
    pub(crate) version: u16,
    pub(crate) node_count: u64,
    pub(crate) dimension: u32,
    pub(crate) max_degree: u32,
    pub(crate) sector_size: u32,
    pub(crate) entry_point: u32,
    pub(crate) metric: u8,
    pub(crate) quantized: u8,
    pub(crate) q_min: f32,
    pub(crate) q_max: f32,
}

impl DiskAnnHeader {
    pub(crate) const SIZE: usize = 40;

    pub(crate) fn to_bytes(self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());
        buf[6..14].copy_from_slice(&self.node_count.to_le_bytes());
        buf[14..18].copy_from_slice(&self.dimension.to_le_bytes());
        buf[18..22].copy_from_slice(&self.max_degree.to_le_bytes());
        buf[22..26].copy_from_slice(&self.sector_size.to_le_bytes());
        buf[26..30].copy_from_slice(&self.entry_point.to_le_bytes());
        buf[30] = self.metric;
        buf[31] = self.quantized;
        buf[32..36].copy_from_slice(&self.q_min.to_le_bytes());
        buf[36..40].copy_from_slice(&self.q_max.to_le_bytes());
        buf
    }

    pub(crate) fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(VectorError::Index("Header too small".into()));
        }
        let magic_bytes = bytes
            .get(0..4)
            .ok_or_else(|| VectorError::Storage("DiskANN header magic out of bounds".into()))?;
        if magic_bytes != DISKANN_MAGIC {
            return Err(VectorError::Storage(
                "Invalid DiskANN file: bad magic".into(),
            ));
        }
        let version = u16::from_le_bytes(
            bytes
                .get(4..6)
                .ok_or_else(|| VectorError::Index("Invalid version offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid version".into()))?,
        );
        if version != DISKANN_VERSION {
            return Err(VectorError::Storage(format!(
                "DiskANN version mismatch: expected {}, got {}",
                DISKANN_VERSION, version
            )));
        }
        let node_count = u64::from_le_bytes(
            bytes
                .get(6..14)
                .ok_or_else(|| VectorError::Index("Invalid node_count offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid node_count".into()))?,
        );
        let dimension = u32::from_le_bytes(
            bytes
                .get(14..18)
                .ok_or_else(|| VectorError::Index("Invalid dimension offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid dimension".into()))?,
        );
        let max_degree = u32::from_le_bytes(
            bytes
                .get(18..22)
                .ok_or_else(|| VectorError::Index("Invalid max_degree offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid max_degree".into()))?,
        );
        let sector_size = u32::from_le_bytes(
            bytes
                .get(22..26)
                .ok_or_else(|| VectorError::Index("Invalid sector_size offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid sector_size".into()))?,
        );
        let entry_point = u32::from_le_bytes(
            bytes
                .get(26..30)
                .ok_or_else(|| VectorError::Index("Invalid entry_point offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid entry_point".into()))?,
        );
        let metric = *bytes
            .get(30)
            .ok_or_else(|| VectorError::Index("Invalid metric offset".into()))?;
        let quantized = *bytes
            .get(31)
            .ok_or_else(|| VectorError::Index("Invalid quantized offset".into()))?;
        let q_min = f32::from_le_bytes(
            bytes
                .get(32..36)
                .ok_or_else(|| VectorError::Index("Invalid q_min offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid q_min".into()))?,
        );
        let q_max = f32::from_le_bytes(
            bytes
                .get(36..40)
                .ok_or_else(|| VectorError::Index("Invalid q_max offset".into()))?
                .try_into()
                .map_err(|_| VectorError::Index("Invalid q_max".into()))?,
        );

        if !q_min.is_finite() || !q_max.is_finite() {
            return Err(VectorError::Storage(
                "DiskANN header quantizer bounds non-finite".into(),
            ));
        }

        Ok(Self {
            magic: *DISKANN_MAGIC,
            version,
            node_count,
            dimension,
            max_degree,
            sector_size,
            entry_point,
            metric,
            quantized,
            q_min,
            q_max,
        })
    }
}

/// Footer for DiskANN index file (36 bytes: 4 bytes DISKANN_FOOTER_MAGIC + 32 bytes HMAC).
#[derive(Debug, Clone, Copy)]
pub(crate) struct DiskAnnFooter {
    pub(crate) magic: [u8; 4],
    pub(crate) hmac: [u8; 32],
}

impl DiskAnnFooter {
    pub(crate) const SIZE: usize = 36;

    pub(crate) fn to_bytes(self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..36].copy_from_slice(&self.hmac);
        buf
    }

    #[cfg(feature = "experimental-diskann")]
    pub(crate) fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(VectorError::Storage("DiskANN footer too small".into()));
        }
        let magic_bytes = bytes
            .get(0..4)
            .ok_or_else(|| VectorError::Storage("DiskANN footer magic out of bounds".into()))?;
        if magic_bytes != DISKANN_FOOTER_MAGIC {
            return Err(VectorError::Storage(
                "Invalid DiskANN file: missing or incomplete build_complete footer".into(),
            ));
        }
        let hmac: [u8; 32] = bytes
            .get(4..36)
            .ok_or_else(|| VectorError::Storage("DiskANN footer HMAC out of bounds".into()))?
            .try_into()
            .map_err(|_| VectorError::Storage("Invalid DiskANN footer HMAC length".into()))?;
        Ok(Self {
            magic: *DISKANN_FOOTER_MAGIC,
            hmac,
        })
    }
}
