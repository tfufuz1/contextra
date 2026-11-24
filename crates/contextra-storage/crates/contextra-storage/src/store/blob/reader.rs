//! Blob file reader with generation-fenced unsealing (SPEC §5.7, SP-BLOB WP-08.3, INV-KEY-2).
#![forbid(unsafe_code)]

use super::record::BlobRecord;
use super::writer::{BlobRef, BLOB_FILE_KIND, BLOB_FILE_MAGIC};
use crate::codec::header::{FileHeader, HEADER_LEN};
use crate::codec::seal::{unseal_frame, Aad, SealMode};
use crate::keys::cache::Key32Handle;
use crate::store::run::table_cache::TableCache;
use contextra_storage_ports::traits::{Checksum, Cipher, File, FileRole, Fs, IoCtx, Mac};
use contextra_storage_ports::{ClusterId, Error, SpaceId};
use std::path::Path;
use std::sync::Arc;
use zeroize::Zeroizing;

fn blob_read_io() -> IoCtx {
    IoCtx {
        role: FileRole::Blob,
        purpose: "blob_read",
    }
}

pub struct BlobReader {
    file: Arc<dyn File>,
    header: FileHeader,
    file_len: u64,
}

impl BlobReader {
    pub fn open(
        fs: &dyn Fs,
        cache: &TableCache,
        path: &Path,
        expected_instance: [u8; 16],
    ) -> Result<Self, Error> {
        let file = cache.pin(fs, path)?;
        let file_len = file.size()?;
        if file_len < HEADER_LEN as u64 {
            return Err(Error::Corrupt {
                place: "blob file header truncated",
            });
        }
        let mut hdr_buf = [0u8; HEADER_LEN];
        read_exact_at(file.as_ref(), 0, &mut hdr_buf)?;
        let header = FileHeader::decode(&hdr_buf, BLOB_FILE_MAGIC, BLOB_FILE_KIND, 0)?;
        if header.instance_id != expected_instance {
            return Err(Error::Corrupt {
                place: "blob file instance_id mismatch",
            });
        }
        Ok(Self {
            file,
            header,
            file_len,
        })
    }

    pub fn header(&self) -> &FileHeader {
        &self.header
    }

    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    pub fn read_raw_record(
        &self,
        offset: u64,
        len: u32,
        max_frame_bytes: usize,
    ) -> Result<BlobRecord, Error> {
        let end = offset
            .checked_add(u64::from(len))
            .ok_or(Error::Corrupt {
                place: "blob read range overflow",
            })?;
        if offset < HEADER_LEN as u64 || end > self.file_len {
            return Err(Error::Corrupt {
                place: "blob read outside file bounds",
            });
        }
        let mut buf = vec![0u8; len as usize];
        read_exact_at(self.file.as_ref(), offset, &mut buf)?;
        BlobRecord::decode(&buf, max_frame_bytes)
    }

    pub fn read_sealed(
        &self,
        blob_ref: &BlobRef,
        mode: SealMode,
        space: SpaceId,
        cluster: ClusterId,
        key_handle: Option<&Key32Handle>,
        cipher: &dyn Cipher,
        mac: &dyn Mac,
        checksum: &dyn Checksum,
        max_frame_bytes: usize,
    ) -> Result<Zeroizing<Vec<u8>>, Error> {
        let record = self.read_raw_record(blob_ref.offset, blob_ref.len, max_frame_bytes)?;
        if record.record_uid != blob_ref.record_uid {
            return Err(Error::Corrupt {
                place: "blob record_uid mismatch",
            });
        }
        if record.seal_frame.len() < 4 {
            return Err(Error::Corrupt {
                place: "blob seal frame truncated",
            });
        }
        let key_epoch = u32::from_le_bytes(
            record.seal_frame[..4]
                .try_into()
                .map_err(|_| Error::Corrupt {
                    place: "blob seal frame epoch",
                })?,
        );
        let aad = Aad::BlobRecord {
            instance: &self.header.instance_id,
            space,
            cluster,
            key_epoch,
            record_uid: &record.record_uid,
        };

        match mode {
            SealMode::Plain => {
                let (_, pt) = unseal_frame(
                    mode,
                    None,
                    aad,
                    &record.seal_frame,
                    cipher,
                    mac,
                    checksum,
                )?;
                Ok(pt)
            }
            SealMode::Authenticated | SealMode::Encrypted => {
                let handle = key_handle.ok_or(Error::KeyUnavailable {
                    retry_after_secs: None,
                })?;
                handle.with_key(|k| {
                    let (_, pt) = unseal_frame(
                        mode,
                        Some(k),
                        aad,
                        &record.seal_frame,
                        cipher,
                        mac,
                        checksum,
                    )?;
                    Ok(pt)
                })
            }
        }
    }
}

fn read_exact_at(file: &dyn File, offset: u64, out: &mut [u8]) -> Result<(), Error> {
    let mut read = 0usize;
    while read < out.len() {
        let pos = offset
            .checked_add(read as u64)
            .ok_or(Error::Corrupt {
                place: "blob read offset overflow",
            })?;
        let chunk = file.read_at(pos, &mut out[read..], blob_read_io())?;
        if chunk == 0 {
            return Err(Error::Corrupt {
                place: "blob short read",
            });
        }
        read += chunk;
    }
    Ok(())
}
