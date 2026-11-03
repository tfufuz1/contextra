// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> vfs_file -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Positional VFS-Lesevorgänge über contextra_ports::Vfs.
use contextra_crypto::hierarchy::IdxKey;
use contextra_crypto::secret::SecretKey;
use contextra_types::{ContextraError, Result};

pub(crate) const DISKANN_MAGIC: &[u8; 4] = b"DANN";
pub(crate) const DISKANN_VERSION: u16 = 1;
pub(crate) const DISKANN_FOOTER_MAGIC: &[u8; 4] = b"DANF";
pub(crate) const DISKANN_INTEGRITY_KEY: &[u8; 32] = b"contextra_diskann_integri_key_32";

/// Streaming BLAKE3 Keyed Tag Berechner für O(1) Speicherverbrauch.
pub(crate) struct StreamingTag {
    hasher: blake3::Hasher,
}

impl StreamingTag {
    pub(crate) fn new() -> Self {
        let key = IdxKey(SecretKey::new(*DISKANN_INTEGRITY_KEY));
        Self {
            hasher: blake3::Hasher::new_keyed(key.0.as_bytes()),
        }
    }

    pub(crate) fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub(crate) fn finalize(self) -> [u8; 16] {
        let mut out = [0u8; 16];
        let hash = self.hasher.finalize();
        out.copy_from_slice(&hash.as_bytes()[..16]);
        out
    }
}

/// Berechnet einen 16-Byte BLAKE3 Keyed Tag über die übergebenen Slices.
pub(crate) fn compute_diskann_tag(slices: &[&[u8]]) -> [u8; 16] {
    let mut stream = StreamingTag::new();
    for slice in slices {
        stream.update(slice);
    }
    stream.finalize()
}

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
            return Err(ContextraError::Index("Header too small".into()));
        }
        let magic_bytes = bytes
            .get(0..4)
            .ok_or_else(|| ContextraError::Storage("DiskANN header magic out of bounds".into()))?;
        if magic_bytes != DISKANN_MAGIC {
            return Err(ContextraError::Storage(
                "Invalid DiskANN file: bad magic".into(),
            ));
        }
        let version = u16::from_le_bytes(
            bytes
                .get(4..6)
                .ok_or_else(|| ContextraError::Index("Invalid version offset".into()))?
                .try_into()
                .map_err(|_| ContextraError::Index("Invalid version".into()))?,
        );
        if version != DISKANN_VERSION {
            return Err(ContextraError::Storage(format!(
                "DiskANN version mismatch: expected {}, got {}",
                DISKANN_VERSION, version
            )));
        }
        Ok(Self {
            magic: *DISKANN_MAGIC,
            version,
            node_count: u64::from_le_bytes(
                bytes
                    .get(6..14)
                    .ok_or_else(|| ContextraError::Index("Invalid node_count offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid node_count".into()))?,
            ),
            dimension: u32::from_le_bytes(
                bytes
                    .get(14..18)
                    .ok_or_else(|| ContextraError::Index("Invalid dimension offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid dimension".into()))?,
            ),
            max_degree: u32::from_le_bytes(
                bytes
                    .get(18..22)
                    .ok_or_else(|| ContextraError::Index("Invalid max_degree offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid max_degree".into()))?,
            ),
            sector_size: u32::from_le_bytes(
                bytes
                    .get(22..26)
                    .ok_or_else(|| ContextraError::Index("Invalid sector_size offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid sector_size".into()))?,
            ),
            entry_point: u32::from_le_bytes(
                bytes
                    .get(26..30)
                    .ok_or_else(|| ContextraError::Index("Invalid entry_point offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid entry_point".into()))?,
            ),
            metric: *bytes
                .get(30)
                .ok_or_else(|| ContextraError::Index("Invalid metric offset".into()))?,
            quantized: *bytes
                .get(31)
                .ok_or_else(|| ContextraError::Index("Invalid quantized offset".into()))?,
            q_min: f32::from_le_bytes(
                bytes
                    .get(32..36)
                    .ok_or_else(|| ContextraError::Index("Invalid q_min offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid q_min".into()))?,
            ),
            q_max: f32::from_le_bytes(
                bytes
                    .get(36..40)
                    .ok_or_else(|| ContextraError::Index("Invalid q_max offset".into()))?
                    .try_into()
                    .map_err(|_| ContextraError::Index("Invalid q_max".into()))?,
            ),
        })
    }
}

/// Footer for DiskANN index file (20 bytes: 4 bytes DISKANN_FOOTER_MAGIC + 16 bytes Tag).
#[derive(Debug, Clone, Copy)]
pub(crate) struct DiskAnnFooter {
    pub(crate) magic: [u8; 4],
    pub(crate) tag: [u8; 16],
}

impl DiskAnnFooter {
    pub(crate) const SIZE: usize = 20;

    pub(crate) fn to_bytes(self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..20].copy_from_slice(&self.tag);
        buf
    }

    #[cfg(feature = "experimental-diskann")]
    pub(crate) fn try_from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < Self::SIZE {
            return Err(ContextraError::Storage("DiskANN footer too small".into()));
        }
        let magic_bytes = bytes
            .get(0..4)
            .ok_or_else(|| ContextraError::Storage("DiskANN footer magic out of bounds".into()))?;
        if magic_bytes != DISKANN_FOOTER_MAGIC {
            return Err(ContextraError::Storage(
                "Invalid DiskANN file: missing or incomplete build_complete footer".into(),
            ));
        }
        let tag_bytes: [u8; 16] = bytes
            .get(4..20)
            .ok_or_else(|| ContextraError::Storage("DiskANN footer tag out of bounds".into()))?
            .try_into()
            .map_err(|_| ContextraError::Storage("Invalid DiskANN footer tag length".into()))?;
        Ok(Self {
            magic: *DISKANN_FOOTER_MAGIC,
            tag: tag_bytes,
        })
    }
}
