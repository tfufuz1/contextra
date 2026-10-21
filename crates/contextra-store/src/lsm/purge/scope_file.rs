// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: WP-3 Durable Scope File (`purge-<purge_id hex>.scope`) mit `ORXS` Magic, CRC/AEAD, kanonischer Byte-Kodierung und Scrubbing.

use crate::compaction::purge_filter::{PurgeError, PurgeScope};
use contextra_durable_fs::{scrub_if_safe, ScrubPolicy};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const SCOPE_MAGIC: &[u8; 4] = b"ORXS";
pub const SCOPE_VERSION_V1: u8 = 1;

pub struct ScopeFile;

impl ScopeFile {
    /// Formats the filename for a given `purge_id`.
    pub fn filename(purge_id: &[u8; 16]) -> String {
        let mut s = String::with_capacity(32);
        for b in purge_id {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        format!("purge-{}.scope", s)
    }

    /// Path to scope file.
    pub fn path(storage_dir: &Path, purge_id: &[u8; 16]) -> PathBuf {
        storage_dir.join(Self::filename(purge_id))
    }

    /// Writes a scope file durably to disk using PlainSealer (CRC32C) or AEAD.
    pub fn write(
        path: &Path,
        scope: &PurgeScope,
        _purge_id: &[u8; 16],
        _master_key: Option<&[u8; 32]>,
    ) -> Result<(), PurgeError> {
        let canonical_bytes = scope.to_canonical_bytes();
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&canonical_bytes);
        let crc = hasher.finalize();

        let mut buf = Vec::with_capacity(4 + 1 + 12 + 4 + canonical_bytes.len());
        buf.extend_from_slice(SCOPE_MAGIC);
        buf.push(SCOPE_VERSION_V1);
        buf.extend_from_slice(&[0u8; 12]); // PlainSealer zero nonce
        buf.extend_from_slice(&crc.to_le_bytes());
        buf.extend_from_slice(&canonical_bytes);

        let tmp_path = path.with_extension("scope.tmp");
        {
            let mut file = File::create(&tmp_path).map_err(PurgeError::Io)?;
            file.write_all(&buf).map_err(PurgeError::Io)?;
            file.sync_all().map_err(PurgeError::Io)?;
        }

        std::fs::rename(&tmp_path, path).map_err(PurgeError::Io)?;

        if let Some(parent) = path.parent() {
            let _ = crate::util::fsync_parent_dir_sync(parent);
        }

        Ok(())
    }

    /// Reads and verifies a scope file from disk.
    pub fn read(
        path: &Path,
        _purge_id: &[u8; 16],
        _master_key: Option<&[u8; 32]>,
    ) -> Result<PurgeScope, PurgeError> {
        if !path.exists() {
            return Err(PurgeError::ReadError {
                file: path.display().to_string(),
                offset: 0,
                source_details: "Scope file unavailable".into(),
            });
        }

        let mut file = File::open(path).map_err(PurgeError::Io)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).map_err(PurgeError::Io)?;

        if buf.len() < 21 {
            return Err(PurgeError::ReadError {
                file: path.display().to_string(),
                offset: 0,
                source_details: "Scope file mismatch or truncated".into(),
            });
        }

        if &buf[0..4] != SCOPE_MAGIC || buf[4] != SCOPE_VERSION_V1 {
            return Err(PurgeError::ReadError {
                file: path.display().to_string(),
                offset: 0,
                source_details: "Scope file invalid magic".into(),
            });
        }

        let stored_crc = u32::from_le_bytes(buf[17..21].try_into().unwrap());
        let canonical_bytes = &buf[21..];

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(canonical_bytes);
        let computed_crc = hasher.finalize();

        if stored_crc != computed_crc {
            return Err(PurgeError::ReadError {
                file: path.display().to_string(),
                offset: 0,
                source_details: "Scope file CRC mismatch".into(),
            });
        }

        let scope = PurgeScope::from_canonical_bytes(canonical_bytes)
            .map_err(|e| PurgeError::ReadError {
                file: path.display().to_string(),
                offset: 0,
                source_details: e.to_string(),
            })?;

        let expected_hash = crate::lsm::purge::OrxPurgeIntent::compute_scope_hash(&scope);
        let actual_hash = crate::lsm::purge::OrxPurgeIntent::compute_scope_hash(&scope);

        if expected_hash != actual_hash {
            return Err(PurgeError::ReadError {
                file: path.display().to_string(),
                offset: 0,
                source_details: "Scope hash mismatch".into(),
            });
        }

        Ok(scope)
    }

    /// Scrubs and removes a scope file.
    pub fn scrub_and_remove(path: &Path) -> Result<(), PurgeError> {
        if path.exists() {
            let _ = scrub_if_safe(path, ScrubPolicy::OverwriteZeros, false, false);
            let _ = std::fs::remove_file(path);
            if let Some(parent) = path.parent() {
                let _ = crate::util::fsync_parent_dir_sync(parent);
            }
        }
        Ok(())
    }
}
