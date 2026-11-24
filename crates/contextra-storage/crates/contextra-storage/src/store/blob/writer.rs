//! Blob file writer and position-independent survivor repack (SPEC §5.7, SP-BLOB WP-08.2, E-58).
#![forbid(unsafe_code)]

use super::record::BlobRecord;
use crate::codec::header::{FileHeader, HEADER_LEN, HEADER_VERSION};
use crate::codec::seal::{seal_frame, Aad, SealMode};
use contextra_storage_ports::traits::{Checksum, Cipher, File, FileRole, Fs, IoCtx, Mac};
use contextra_storage_ports::{ClusterId, Error, Key32, SpaceId};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

pub const BLOB_FILE_MAGIC: [u8; 8] = *b"CTXBLB01";
pub const BLOB_FILE_KIND: u8 = 2;
pub const BLOB_PREALLOC_BYTES: u64 = 1024 * 1024;

fn blob_io(purpose: &'static str) -> IoCtx {
    IoCtx {
        role: FileRole::Blob,
        purpose,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobRef {
    pub blob_id: u64,
    pub incarnation: u64,
    pub offset: u64,
    pub len: u32,
    pub record_uid: [u8; 16],
}

pub struct BlobSealContext<'a> {
    pub instance_id: &'a [u8; 16],
    pub space: SpaceId,
    pub cluster: ClusterId,
    pub key_epoch: u32,
    pub mode: SealMode,
    pub key: Option<&'a Key32>,
    pub cipher: &'a dyn Cipher,
    pub mac: &'a dyn Mac,
    pub checksum: &'a dyn Checksum,
}

pub fn blob_filename(blob_id: u64, incarnation: u64) -> String {
    format!("{blob_id:016x}_{incarnation:016x}.blob")
}

pub struct BlobWriter {
    blob_id: u64,
    incarnation: u64,
    temp_path: PathBuf,
    final_path: PathBuf,
    blobs_dir: PathBuf,
    file: Option<Box<dyn File>>,
    offset: u64,
    allocated_until: u64,
    poisoned: bool,
}

impl BlobWriter {
    pub fn create(
        fs: &dyn Fs,
        blobs_dir: &Path,
        blob_id: u64,
        incarnation: u64,
        instance_id: [u8; 16],
    ) -> Result<Self, Error> {
        let final_name = blob_filename(blob_id, incarnation);
        let temp_path = blobs_dir.join(format!(".building.{final_name}"));
        let final_path = blobs_dir.join(&final_name);

        let file = fs.create_new(&temp_path, blob_io("blob_create_temp"))?;
        if let Err(e) = file.preallocate(BLOB_PREALLOC_BYTES, blob_io("blob_prealloc_init")) {
            let _ = fs.unlink(&temp_path, blob_io("blob_cleanup_temp"));
            return Err(e);
        }

        let header = FileHeader {
            magic: BLOB_FILE_MAGIC,
            file_kind: BLOB_FILE_KIND,
            format_ver: HEADER_VERSION,
            feature_bits: 0,
            file_id: blob_id,
            instance_id,
        };
        let header_bytes = header.encode()?;
        if let Err(e) = file.append_prealloc(&header_bytes, blob_io("blob_write_header")) {
            let _ = fs.unlink(&temp_path, blob_io("blob_cleanup_temp"));
            return Err(e);
        }

        Ok(Self {
            blob_id,
            incarnation,
            temp_path,
            final_path,
            blobs_dir: blobs_dir.to_path_buf(),
            file: Some(file),
            offset: HEADER_LEN as u64,
            allocated_until: BLOB_PREALLOC_BYTES,
            poisoned: false,
        })
    }

    pub fn blob_id(&self) -> u64 {
        self.blob_id
    }

    pub fn incarnation(&self) -> u64 {
        self.incarnation
    }

    pub fn append_sealed(
        &mut self,
        ctx: &BlobSealContext<'_>,
        record_uid: [u8; 16],
        nonce: &[u8; 24],
        plaintext: &[u8],
        max_frame_bytes: usize,
    ) -> Result<BlobRef, Error> {
        let aad = Aad::BlobRecord {
            instance: ctx.instance_id,
            space: ctx.space,
            cluster: ctx.cluster,
            key_epoch: ctx.key_epoch,
            record_uid: &record_uid,
        };
        let sealed = seal_frame(
            ctx.mode,
            ctx.key_epoch,
            ctx.key,
            nonce,
            aad,
            plaintext,
            ctx.cipher,
            ctx.mac,
            ctx.checksum,
        )?;
        let record = BlobRecord {
            record_uid,
            seal_frame: Zeroizing::new(sealed),
        };
        self.append_raw_record(&record, max_frame_bytes)
    }

    pub fn append_raw_record(
        &mut self,
        record: &BlobRecord,
        max_frame_bytes: usize,
    ) -> Result<BlobRef, Error> {
        if self.poisoned || self.file.is_none() {
            return Err(Error::Poisoned {
                cause: "blob writer is poisoned or closed",
            });
        }
        let encoded = record.encode(max_frame_bytes)?;
        let len_u32 =
            u32::try_from(encoded.len()).map_err(|_| Error::Limit { limit: "blob record len" })?;
        let record_offset = self.offset;
        let next_offset = record_offset
            .checked_add(encoded.len() as u64)
            .ok_or(Error::Limit { limit: "blob file offset overflow" })?;

        self.ensure_capacity(next_offset)?;
        let file = self.file.as_ref().ok_or(Error::Poisoned {
            cause: "blob writer closed",
        })?;
        if let Err(e) = file.append_prealloc(&encoded, blob_io("blob_append_record")) {
            self.poisoned = true;
            return Err(e);
        }
        self.offset = next_offset;
        Ok(BlobRef {
            blob_id: self.blob_id,
            incarnation: self.incarnation,
            offset: record_offset,
            len: len_u32,
            record_uid: record.record_uid,
        })
    }

    pub fn finish(mut self, fs: &dyn Fs) -> Result<PathBuf, Error> {
        if self.poisoned {
            return Err(Error::Poisoned {
                cause: "blob writer is poisoned",
            });
        }
        let file = self.file.take().ok_or(Error::Poisoned {
            cause: "blob writer already finished",
        })?;
        file.sync_all(blob_io("blob_sync_all")).map_err(|_| Error::Poisoned {
            cause: "blob file sync failed",
        })?;
        drop(file);

        fs.rename(&self.temp_path, &self.final_path, blob_io("blob_rename"))?;
        fs.sync_dir(&self.blobs_dir, blob_io("blob_sync_dir"))?;
        Ok(self.final_path)
    }

    fn ensure_capacity(&mut self, required: u64) -> Result<(), Error> {
        if required <= self.allocated_until {
            return Ok(());
        }
        let rounded = required
            .checked_add(BLOB_PREALLOC_BYTES - 1)
            .map(|v| (v / BLOB_PREALLOC_BYTES) * BLOB_PREALLOC_BYTES)
            .ok_or(Error::Limit {
                limit: "blob prealloc overflow",
            })?;
        let file = self.file.as_ref().ok_or(Error::Poisoned {
            cause: "blob writer closed",
        })?;
        file.preallocate(rounded, blob_io("blob_prealloc_grow"))?;
        self.allocated_until = rounded;
        Ok(())
    }
}

/// Repacks surviving blob records into a new incarnation without decrypting or
/// re-sealing (E-58 position-independent BlobRecord AAD).
pub fn repack_survivors(
    src: &super::reader::BlobReader,
    survivor_refs: &[BlobRef],
    dst: &mut BlobWriter,
    max_frame_bytes: usize,
) -> Result<Vec<BlobRef>, Error> {
    let mut new_refs = Vec::with_capacity(survivor_refs.len());
    for old_ref in survivor_refs {
        let record = src.read_raw_record(old_ref.offset, old_ref.len, max_frame_bytes)?;
        if record.record_uid != old_ref.record_uid {
            return Err(Error::Corrupt {
                place: "blob repack record_uid mismatch",
            });
        }
        let new_ref = dst.append_raw_record(&record, max_frame_bytes)?;
        new_refs.push(new_ref);
    }
    Ok(new_refs)
}
