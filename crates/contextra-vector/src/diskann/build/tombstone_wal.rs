// FILE-CONTEXT
// ZWECK: Tombstone-Set und WAL-Verwaltung für gelöschte Dokumente (WP-VEC-07).
// INVARIANTEN: Zero-Panic bei WAL-Lesezugriff; HMAC-Integritätsprüfung.
// STAND: 2026-11-11

use std::io::Read;
use subtle::ConstantTimeEq;

use super::super::format::{
    DISKANN_INTEGRITY_KEY, TOMBSTONE_WAL_HEADER_SIZE, TOMBSTONE_WAL_MAGIC, TOMBSTONE_WAL_VERSION,
};
use super::super::types::DiskAnnIndex;
use crate::error::{Result, VectorError};
use crate::types::DocId;
#[cfg(not(feature = "docid-128"))]
pub(crate) type TombstoneSet = roaring::RoaringTreemap;

#[cfg(feature = "docid-128")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TombstoneSet {
    pub(crate) set: std::collections::BTreeSet<u128>,
}

#[cfg(feature = "docid-128")]
impl TombstoneSet {
    pub(crate) fn new() -> Self {
        Self {
            set: std::collections::BTreeSet::new(),
        }
    }

    pub(crate) fn insert(&mut self, id: u128) -> bool {
        self.set.insert(id)
    }

    pub(crate) fn contains(&self, id: u128) -> bool {
        self.set.contains(&id)
    }

    pub(crate) fn len(&self) -> usize {
        self.set.len()
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.set.clear();
    }
}

use std::sync::atomic::Ordering;

impl DiskAnnIndex {
    pub(crate) fn append_to_tombstone_wal_blocking(
        path: &std::path::Path,
        id: DocId,
    ) -> Result<()> {
        use std::io::Write;

        let id_bytes = id.inner().to_le_bytes();
        let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY)?;
        hmac.update(&id_bytes);
        let computed_hmac = hmac.finalize();

        crate::diskann::io::append_wal(
            path,
            TOMBSTONE_WAL_MAGIC,
            TOMBSTONE_WAL_VERSION,
            &id_bytes,
            &computed_hmac,
        )
    }

    /// Appends a tombstone record to `tombstone.wal` with HMAC-SHA256 integrity protection.
    ///
    /// # Binary Format
    /// - File Header (written on new/empty file creation): `[TWAL: 4 bytes] [version=1: 1 byte]`
    /// - Entry Layout:
    ///   - `id`: `u64` LE (8 bytes)
    ///   - `hmac`: `[u8; 32]` (32-byte HMAC-SHA256 computed over `id_bytes`)
    pub(crate) async fn append_to_tombstone_wal(path: &std::path::Path, id: DocId) -> Result<()> {
        Self::append_to_tombstone_wal_blocking(path, id)
    }

    /// Reads and verifies uncommitted tombstone entries from `tombstone.wal`.
    pub(crate) fn read_tombstone_wal(path: &std::path::Path) -> Result<TombstoneSet> {
        use std::io::Read;
        use subtle::ConstantTimeEq;

        let mut bitset = TombstoneSet::new();

        if !path.exists() {
            return Ok(bitset);
        }
        let mut file = match crate::diskann::io::open_read(path) {
            Ok(f) => f,
            Err(_) => return Ok(bitset),
        };

        let file_len = match file.metadata() {
            Ok(m) => m.len() as usize,
            Err(_) => return Ok(bitset),
        };

        if file_len == 0 {
            return Ok(bitset);
        }

        if file_len < TOMBSTONE_WAL_HEADER_SIZE {
            return Err(VectorError::Storage(
                "Unbekanntes oder veraltetes tombstone.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        let mut magic_buf = [0u8; 4];
        let mut version_buf = [0u8; 1];
        if file.read_exact(&mut magic_buf).is_err() || file.read_exact(&mut version_buf).is_err() {
            return Err(VectorError::Storage(
                "Unbekanntes oder veraltetes tombstone.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        if magic_buf != *TOMBSTONE_WAL_MAGIC || version_buf[0] != TOMBSTONE_WAL_VERSION {
            return Err(VectorError::Storage(
                "Unbekanntes oder veraltetes tombstone.wal-Format, bitte vor Upgrade leeren".into(),
            ));
        }

        let mut offset = TOMBSTONE_WAL_HEADER_SIZE;
        let doc_id_size = std::mem::size_of::<DocId>();
        let mut buf_id = vec![0u8; doc_id_size];

        while file.read_exact(&mut buf_id).is_ok() {
            let entry_offset = offset;
            offset += doc_id_size;

            #[cfg(not(feature = "docid-128"))]
            let doc_id = u64::from_le_bytes(
                buf_id
                    .as_slice()
                    .try_into()
                    .map_err(|_| VectorError::Storage("Corrupt DocId in WAL".into()))?,
            );
            #[cfg(feature = "docid-128")]
            let doc_id = u128::from_le_bytes(
                buf_id
                    .as_slice()
                    .try_into()
                    .map_err(|_| VectorError::Storage("Corrupt DocId in WAL".into()))?,
            );

            let mut buf_hmac = [0u8; 32];
            if file.read_exact(&mut buf_hmac).is_err() {
                tracing::warn!("Truncated HMAC in tombstone.wal");
                break;
            }
            offset += 32;

            let mut hmac = crate::diskann::integrity::DiskAnnHmac::new(DISKANN_INTEGRITY_KEY)?;
            hmac.update(&buf_id);
            let computed_hmac = hmac.finalize();

            if computed_hmac.ct_eq(&buf_hmac).into() {
                bitset.insert(doc_id);
            } else {
                tracing::error!(
                    offset = entry_offset,
                    doc_id = doc_id,
                    "tombstone.wal: HMAC-Mismatch bei Eintrag, Eintrag verworfen"
                );
            }
        }
        Ok(bitset)
    }
}
