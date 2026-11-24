//! Checkpoint creation and consistent snapshot manifest capture (SPEC §10.2, SP-CKPT).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::{Error, InstanceId, Stamp};
use std::path::{Path, PathBuf};

fn ckpt_ctx(purpose: &'static str) -> IoCtx {
    IoCtx {
        role: FileRole::Other,
        purpose,
    }
}

pub const CHECKPOINT_MANIFEST_MAGIC: &[u8; 8] = b"CTXSCKP1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointFileEntry {
    pub relative_path: PathBuf,
    pub size_bytes: u64,
    pub checksum: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointManifest {
    pub instance_id: InstanceId,
    pub checkpoint_id: u64,
    pub created_at_utc: u64,
    pub catalog_epoch: u64,
    pub durable_stamp: Stamp,
    pub erase_epoch: Stamp,
    pub tomb_epoch: u32,
    pub files: Vec<CheckpointFileEntry>,
}

impl CheckpointManifest {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(256 + self.files.len() * 64);
        buf.extend_from_slice(CHECKPOINT_MANIFEST_MAGIC);
        buf.extend_from_slice(self.instance_id.as_bytes());
        buf.extend_from_slice(&self.checkpoint_id.to_be_bytes());
        buf.extend_from_slice(&self.created_at_utc.to_be_bytes());
        buf.extend_from_slice(&self.catalog_epoch.to_be_bytes());
        buf.extend_from_slice(&self.durable_stamp.to_be_bytes());
        buf.extend_from_slice(&self.erase_epoch.to_be_bytes());
        buf.extend_from_slice(&self.tomb_epoch.to_be_bytes());
        buf.extend_from_slice(&(self.files.len() as u32).to_be_bytes());

        for f in &self.files {
            let path_str = f.relative_path.to_string_lossy();
            let path_bytes = path_str.as_bytes();
            buf.extend_from_slice(&(path_bytes.len() as u32).to_be_bytes());
            buf.extend_from_slice(path_bytes);
            buf.extend_from_slice(&f.size_bytes.to_be_bytes());
            buf.extend_from_slice(&f.checksum.to_be_bytes());
        }
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 8 + 16 + 8 * 5 + 4 || &bytes[..8] != CHECKPOINT_MANIFEST_MAGIC {
            return Err(Error::Corrupt {
                place: "checkpoint manifest magic",
            });
        }
        let mut inst_raw = [0u8; 16];
        inst_raw.copy_from_slice(&bytes[8..24]);
        let instance_id = InstanceId::from_wire(inst_raw);

        let checkpoint_id = u64::from_be_bytes(bytes[24..32].try_into().map_err(|_| {
            Error::Corrupt {
                place: "checkpoint id",
            }
        })?);
        let created_at_utc = u64::from_be_bytes(bytes[32..40].try_into().map_err(|_| {
            Error::Corrupt {
                place: "checkpoint timestamp",
            }
        })?);
        let catalog_epoch = u64::from_be_bytes(bytes[40..48].try_into().map_err(|_| {
            Error::Corrupt {
                place: "catalog epoch",
            }
        })?);
        let durable_stamp = u64::from_be_bytes(bytes[48..56].try_into().map_err(|_| {
            Error::Corrupt {
                place: "durable stamp",
            }
        })?);
        let erase_epoch = u64::from_be_bytes(bytes[56..64].try_into().map_err(|_| {
            Error::Corrupt {
                place: "erase epoch",
            }
        })?);
        let tomb_epoch = u32::from_be_bytes(bytes[64..68].try_into().map_err(|_| {
            Error::Corrupt {
                place: "tomb epoch",
            }
        })?);
        let num_files = u32::from_be_bytes(bytes[68..72].try_into().map_err(|_| {
            Error::Corrupt {
                place: "manifest files count",
            }
        })?) as usize;

        let mut offset = 72;
        let mut files = Vec::with_capacity(num_files);
        for _ in 0..num_files {
            if offset + 4 > bytes.len() {
                return Err(Error::Corrupt {
                    place: "file entry path len",
                });
            }
            let path_len = u32::from_be_bytes(bytes[offset..offset + 4].try_into().map_err(
                |_| Error::Corrupt {
                    place: "path length",
                },
            )?) as usize;
            offset += 4;
            let entry_end = offset
                .checked_add(path_len)
                .and_then(|n| n.checked_add(12))
                .ok_or(Error::Corrupt {
                    place: "file entry length overflow",
                })?;
            if entry_end > bytes.len() {
                return Err(Error::Corrupt {
                    place: "file entry body truncated",
                });
            }
            let path_str = std::str::from_utf8(&bytes[offset..offset + path_len]).map_err(
                |_| Error::Corrupt {
                    place: "path utf8",
                },
            )?;
            offset += path_len;
            let size_bytes = u64::from_be_bytes(bytes[offset..offset + 8].try_into().map_err(
                |_| Error::Corrupt {
                    place: "file size",
                },
            )?);
            offset += 8;
            let checksum = u32::from_be_bytes(bytes[offset..offset + 4].try_into().map_err(
                |_| Error::Corrupt {
                    place: "file checksum",
                },
            )?);
            offset += 4;
            files.push(CheckpointFileEntry {
                relative_path: PathBuf::from(path_str),
                size_bytes,
                checksum,
            });
        }
        if offset != bytes.len() {
            return Err(Error::Corrupt {
                place: "checkpoint manifest trailing bytes",
            });
        }

        Ok(Self {
            instance_id,
            checkpoint_id,
            created_at_utc,
            catalog_epoch,
            durable_stamp,
            erase_epoch,
            tomb_epoch,
            files,
        })
    }
}

pub struct CheckpointCreator;

impl CheckpointCreator {
    /// Captures a checkpoint: writes manifest into target directory using atomic two-phase write.
    pub fn write_manifest(
        fs: &dyn Fs,
        target_dir: &Path,
        manifest: &CheckpointManifest,
    ) -> Result<PathBuf, Error> {
        let temp_path = target_dir.join(".building.manifest");
        let final_path = target_dir.join("manifest.ckpt");

        let encoded = manifest.encode();
        let file = fs.create_new(&temp_path, ckpt_ctx("ckpt_manifest_create"))?;
        if let Err(e) = file
            .preallocate(encoded.len() as u64, ckpt_ctx("ckpt_manifest_prealloc"))
            .and_then(|_| file.append_prealloc(&encoded, ckpt_ctx("ckpt_manifest_write")))
            .and_then(|_| file.sync_all(ckpt_ctx("ckpt_manifest_sync")))
        {
            let _ = fs.unlink(&temp_path, ckpt_ctx("ckpt_manifest_cleanup"));
            return Err(e);
        }
        drop(file);

        if let Err(e) = fs.rename(&temp_path, &final_path, ckpt_ctx("ckpt_manifest_rename")) {
            let _ = fs.unlink(&temp_path, ckpt_ctx("ckpt_manifest_cleanup"));
            return Err(e);
        }

        fs.sync_dir(target_dir, ckpt_ctx("ckpt_manifest_dir_sync"))?;
        Ok(final_path)
    }

    /// Reads and decodes a checkpoint manifest from `target_dir/manifest.ckpt`.
    pub fn read_manifest(fs: &dyn Fs, target_dir: &Path) -> Result<CheckpointManifest, Error> {
        let path = target_dir.join("manifest.ckpt");
        let file = fs.open(&path, ckpt_ctx("ckpt_manifest_open"))?;
        let size = usize::try_from(file.size()?)
            .map_err(|_| Error::Limit { limit: "checkpoint manifest size" })?;
        let mut buf = vec![0u8; size];
        let read = file.read_at(0, &mut buf, ckpt_ctx("ckpt_manifest_read"))?;
        if read != size {
            return Err(Error::Corrupt {
                place: "checkpoint manifest short read",
            });
        }
        CheckpointManifest::decode(&buf)
    }
}
