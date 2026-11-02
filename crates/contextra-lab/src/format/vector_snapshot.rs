//! F.8 Vektor-Snapshot Parser (`kind = 5`).
//!
//! SPEC Anhang F.8:
//! - Kopf nach F.1 (`kind = 5`) -> 64 Bytes
//! - Header:
//!   `dim(u16) || metric(u8) || M(u16) || ef_c(u16) || slot_count(u32) || entry_slot(u32) || max_level(u8)`
//!   = 2 + 1 + 2 + 2 + 4 + 4 + 1 = 16 Bytes
//! - Slot-Abschnitt: je Slot `slot(u32) || envelope` (Envelope v2 unter dem DEK des Dokuments;
//!   Klartext: Vektor in allen Repräsentationen nach D-86 || cluster_tag || doc_tag || chunk(u32))
//! - Graph-Abschnitt: Adjazenzlisten je Level, nur Slotnummern, unter dem Dateischlüssel

use crate::error::LabError;
use crate::format::file_header::{FileHeader, FileKind};

/// Größe des Vektor-Snapshot Headers.
pub const VECTOR_SNAPSHOT_HEADER_SIZE: usize = 16;

/// Distanzmetrik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VectorMetric {
    /// 0: Kosinus (Cosine)
    Cosine = 0,
    /// 1: Euklidisch (L2)
    Euclidean = 1,
    /// 2: Skalarprodukt (Dot Product)
    Dot = 2,
}

impl VectorMetric {
    /// Parst die Metrik.
    pub fn from_u8(val: u8) -> Result<Self, LabError> {
        match val {
            0 => Ok(Self::Cosine),
            1 => Ok(Self::Euclidean),
            2 => Ok(Self::Dot),
            _ => Err(LabError::Corrupted("Unbekannte Vektormetrik")),
        }
    }
}

/// Kopfdaten eines Vektor-Snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorSnapshotHeader {
    /// Dimension der Vektoren.
    pub dim: u16,
    /// Metrik.
    pub metric: VectorMetric,
    /// HNSW Parameter M.
    pub m: u16,
    /// HNSW Parameter ef_construction.
    pub ef_c: u16,
    /// Anzahl belegter Slots.
    pub slot_count: u32,
    /// Einstiegsslot des HNSW-Graphen.
    pub entry_slot: u32,
    /// Maximales Level im Graphen.
    pub max_level: u8,
}

impl VectorSnapshotHeader {
    /// Parst die 16 Byte des Headers.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < VECTOR_SNAPSHOT_HEADER_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: VECTOR_SNAPSHOT_HEADER_SIZE,
                actual: bytes.len(),
            });
        }

        let dim = u16::from_be_bytes(
            bytes
                .get(0..2)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("dim fehlt"))?,
        );

        let metric_byte = *bytes.get(2).ok_or(LabError::Corrupted("metric fehlt"))?;
        let metric = VectorMetric::from_u8(metric_byte)?;

        let m = u16::from_be_bytes(
            bytes
                .get(3..5)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("M fehlt"))?,
        );

        let ef_c = u16::from_be_bytes(
            bytes
                .get(5..7)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("ef_c fehlt"))?,
        );

        let slot_count = u32::from_be_bytes(
            bytes
                .get(7..11)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("slot_count fehlt"))?,
        );

        let entry_slot = u32::from_be_bytes(
            bytes
                .get(11..15)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("entry_slot fehlt"))?,
        );

        let max_level = *bytes
            .get(15)
            .ok_or(LabError::Corrupted("max_level fehlt"))?;

        Ok(Self {
            dim,
            metric,
            m,
            ef_c,
            slot_count,
            entry_slot,
            max_level,
        })
    }
}

/// Strukturierte Repräsentation eines Vektor-Snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorSnapshotFile {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Snapshot-Header.
    pub snapshot_header: VectorSnapshotHeader,
}

impl VectorSnapshotFile {
    /// Parst einen Vektor-Snapshot.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        let min_size = 64 + VECTOR_SNAPSHOT_HEADER_SIZE;
        if bytes.len() < min_size {
            return Err(LabError::UnexpectedEof {
                expected: min_size,
                actual: bytes.len(),
            });
        }

        let header = FileHeader::parse(bytes)?;
        if header.kind != FileKind::VectorSnapshot {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

        let snap_head_bytes = bytes
            .get(64..64 + VECTOR_SNAPSHOT_HEADER_SIZE)
            .ok_or(LabError::Corrupted("Snapshot Header fehlt"))?;
        let snapshot_header = VectorSnapshotHeader::parse(snap_head_bytes)?;

        Ok(Self {
            header,
            snapshot_header,
        })
    }
}
