//! F.7 Textsegment Parser (`kind = 4`).
//!
//! SPEC Anhang F.7:
//! - Kopf nach F.1 (`kind = 4`) -> 64 Bytes
//! - Segmentheader-Block:
//!   `bucket(u8) || bucket_count(u8) || analyzer_id(u8) || analyzer_version(u16) || lang(u8[3]) || flags(u8: bit0 positions) || doc_count(u32) || term_count(u32) || total_len(u64) || reserved(8)`
//!   = 1 + 1 + 1 + 2 + 3 + 1 + 4 + 4 + 8 + 8 = 32 Bytes
//! - Wörterbuch: sortierte Einträge `term_tag(16) || df(u32) || posting_off(u64) || posting_len(u32)` (32 B)
//! - Postings
//! - Dokumenttabelle: je `doc_ord(u32)`: `doclen(u32) || binding_len(u16) || binding_envelope`
//!   - `binding_envelope` = Envelope v2 unter dem DEK des Dokuments
//!   - Klartext: `cluster_tag || doc_tag`
//!   - AAD = "ctx/tbind/v1" || file_id || u32(doc_ord)
//! - Lösch-Bitmap (`ceil(doc_count/8)` Byte), Statistikblock

use crate::error::LabError;
use crate::format::file_header::{FileHeader, FileKind};

/// Größe des Textsegment-Kopfs.
pub const TEXT_SEGMENT_HEADER_SIZE: usize = 32;

/// Größe eines Wörterbucheintrags (32 Byte).
pub const TERM_ENTRY_SIZE: usize = 32;

/// Kopfdaten eines Textsegments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSegmentHeader {
    /// Bucket Index.
    pub bucket: u8,
    /// Gesamtanzahl Buckets.
    pub bucket_count: u8,
    /// Analyzer ID.
    pub analyzer_id: u8,
    /// Analyzer Version.
    pub analyzer_version: u16,
    /// ISO Sprache (3 Byte ASCII).
    pub lang: [u8; 3],
    /// Flags (Bit 0: positions).
    pub flags: u8,
    /// Anzahl Dokumente im Segment.
    pub doc_count: u32,
    /// Anzahl unterschiedlicher Terme.
    pub term_count: u32,
    /// Gesamtlänge aller Dokumente.
    pub total_len: u64,
}

impl TextSegmentHeader {
    /// Parst die 32 Byte des Segmentheaders.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < TEXT_SEGMENT_HEADER_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: TEXT_SEGMENT_HEADER_SIZE,
                actual: bytes.len(),
            });
        }

        let bucket = *bytes.first().ok_or(LabError::Corrupted("bucket fehlt"))?;
        let bucket_count = *bytes
            .get(1)
            .ok_or(LabError::Corrupted("bucket_count fehlt"))?;
        let analyzer_id = *bytes
            .get(2)
            .ok_or(LabError::Corrupted("analyzer_id fehlt"))?;

        let analyzer_version = u16::from_be_bytes(
            bytes
                .get(3..5)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("analyzer_version fehlt"))?,
        );

        let lang: [u8; 3] = bytes
            .get(5..8)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("lang fehlt"))?;

        let flags = *bytes.get(8).ok_or(LabError::Corrupted("flags fehlt"))?;

        let doc_count = u32::from_be_bytes(
            bytes
                .get(9..13)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("doc_count fehlt"))?,
        );

        let term_count = u32::from_be_bytes(
            bytes
                .get(13..17)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("term_count fehlt"))?,
        );

        let total_len = u64::from_be_bytes(
            bytes
                .get(17..25)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("total_len fehlt"))?,
        );

        let reserved = bytes
            .get(25..32)
            .ok_or(LabError::Corrupted("reserved fehlt"))?;
        if reserved.iter().any(|&b| b != 0) {
            return Err(LabError::Corrupted(
                "Textsegment Header Reserved ist nicht 0",
            ));
        }

        Ok(Self {
            bucket,
            bucket_count,
            analyzer_id,
            analyzer_version,
            lang,
            flags,
            doc_count,
            term_count,
            total_len,
        })
    }
}

/// Wörterbucheintrag (32 Byte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermEntry {
    /// Term-Tag (16 Byte BLAKE3/HMAC Präfix).
    pub term_tag: [u8; 16],
    /// Dokumentfrequenz df.
    pub df: u32,
    /// Offset der Posting-Liste.
    pub posting_off: u64,
    /// Länge der Posting-Liste in Bytes.
    pub posting_len: u32,
}

impl TermEntry {
    /// Parst einen Wörterbucheintrag.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        if bytes.len() < TERM_ENTRY_SIZE {
            return Err(LabError::UnexpectedEof {
                expected: TERM_ENTRY_SIZE,
                actual: bytes.len(),
            });
        }

        let term_tag: [u8; 16] = bytes
            .get(0..16)
            .and_then(|s| s.try_into().ok())
            .ok_or(LabError::Corrupted("term_tag fehlt"))?;

        let df = u32::from_be_bytes(
            bytes
                .get(16..20)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("df fehlt"))?,
        );

        let posting_off = u64::from_be_bytes(
            bytes
                .get(20..28)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("posting_off fehlt"))?,
        );

        let posting_len = u32::from_be_bytes(
            bytes
                .get(28..32)
                .and_then(|s| s.try_into().ok())
                .ok_or(LabError::Corrupted("posting_len fehlt"))?,
        );

        Ok(Self {
            term_tag,
            df,
            posting_off,
            posting_len,
        })
    }
}

/// Strukturierte Repräsentation eines Textsegments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextSegmentFile {
    /// F.1 Dateikopf.
    pub header: FileHeader,
    /// Segmentheader.
    pub segment_header: TextSegmentHeader,
}

impl TextSegmentFile {
    /// Parst ein Textsegment.
    pub fn parse(bytes: &[u8]) -> Result<Self, LabError> {
        let min_size = 64 + TEXT_SEGMENT_HEADER_SIZE;
        if bytes.len() < min_size {
            return Err(LabError::UnexpectedEof {
                expected: min_size,
                actual: bytes.len(),
            });
        }

        let header = FileHeader::parse(bytes)?;
        if header.kind != FileKind::TextSegment {
            return Err(LabError::UnsupportedKind(header.kind as u8));
        }

        let seg_head_bytes = bytes
            .get(64..64 + TEXT_SEGMENT_HEADER_SIZE)
            .ok_or(LabError::Corrupted("Segmentheader fehlt"))?;
        let segment_header = TextSegmentHeader::parse(seg_head_bytes)?;

        Ok(Self {
            header,
            segment_header,
        })
    }
}
