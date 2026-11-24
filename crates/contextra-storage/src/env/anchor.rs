//! Monotone Anchors (`MemAnchor` and durable `FileAnchor`) for rollback detection (SP-ENV §3.8, INV-ANCH-1).

use crate::codec::checksums::crc32c;
use contextra_storage_ports::traits::{Anchor, AnchorStrength, AnchorValue, FileRole, Fs, IoCtx};
use contextra_storage_ports::Error;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const ANCHOR_MAGIC: [u8; 8] = *b"CTXANC01";
const ANCHOR_LEN: usize = 8 + 8 + 8 + 8 + 4;

fn anchor_io(purpose: &'static str) -> IoCtx {
    IoCtx {
        role: FileRole::Anchor,
        purpose,
    }
}

pub struct MemAnchor {
    value: Mutex<AnchorValue>,
}

impl MemAnchor {
    pub fn new(initial: AnchorValue) -> Self {
        Self {
            value: Mutex::new(initial),
        }
    }
}

impl Default for MemAnchor {
    fn default() -> Self {
        Self::new(AnchorValue::default())
    }
}

impl Anchor for MemAnchor {
    fn read(&self) -> Result<AnchorValue, Error> {
        self.value
            .lock()
            .map(|value| *value)
            .map_err(|_| Error::Poisoned {
                cause: "memory anchor poisoned",
            })
    }

    fn advance(&self, expect: AnchorValue, next: AnchorValue) -> Result<(), Error> {
        let mut current = self.value.lock().map_err(|_| Error::Poisoned {
            cause: "memory anchor poisoned",
        })?;
        if *current != expect {
            return Err(Error::CasLost);
        }
        validate_monotone(expect, next)?;
        *current = next;
        Ok(())
    }

    fn strength(&self) -> AnchorStrength {
        AnchorStrength::Weak
    }
}

/// Durable file-backed anchor stored outside the primary data root via atomic rename + `sync_dir`.
pub struct FileAnchor {
    fs: Arc<dyn Fs>,
    dir: PathBuf,
    guard: Mutex<()>,
}

impl FileAnchor {
    pub fn open_or_init(
        fs: Arc<dyn Fs>,
        dir: PathBuf,
        initial: AnchorValue,
    ) -> Result<Self, Error> {
        let anchor = Self {
            fs,
            dir,
            guard: Mutex::new(()),
        };
        let path = anchor.anchor_path();
        match anchor.fs.open(&path, anchor_io("anchor_probe")) {
            Ok(f) => {
                let mut buf = [0u8; ANCHOR_LEN];
                let n = f.read_at(0, &mut buf, anchor_io("anchor_read_init"))?;
                if n != ANCHOR_LEN || f.size()? != ANCHOR_LEN as u64 {
                    return Err(Error::Corrupt {
                        place: "file anchor length",
                    });
                }
                let _ = decode_anchor(&buf)?;
            }
            Err(Error::Io {
                kind: ErrorKind::NotFound,
                ..
            }) => {
                write_anchor_atomic(anchor.fs.as_ref(), &anchor.dir, initial)?;
            }
            Err(e) => return Err(e),
        }
        Ok(anchor)
    }

    fn anchor_path(&self) -> PathBuf {
        self.dir.join("ANCHOR")
    }
}

impl Anchor for FileAnchor {
    fn read(&self) -> Result<AnchorValue, Error> {
        let _lock = self.guard.lock().map_err(|_| Error::Poisoned {
            cause: "file anchor lock poisoned",
        })?;
        read_anchor_file(self.fs.as_ref(), &self.anchor_path())
    }

    fn advance(&self, expect: AnchorValue, next: AnchorValue) -> Result<(), Error> {
        let _lock = self.guard.lock().map_err(|_| Error::Poisoned {
            cause: "file anchor lock poisoned",
        })?;
        let current = read_anchor_file(self.fs.as_ref(), &self.anchor_path())?;
        if current != expect {
            return Err(Error::CasLost);
        }
        validate_monotone(expect, next)?;
        write_anchor_atomic(self.fs.as_ref(), &self.dir, next)
    }

    fn strength(&self) -> AnchorStrength {
        AnchorStrength::Weak
    }
}

fn validate_monotone(expect: AnchorValue, next: AnchorValue) -> Result<(), Error> {
    if next.erase_epoch < expect.erase_epoch
        || next.ledger_head < expect.ledger_head
        || next.catalog_epoch < expect.catalog_epoch
    {
        return Err(Error::Limit {
            limit: "anchor value must be monotone",
        });
    }
    Ok(())
}

fn encode_anchor(val: AnchorValue) -> [u8; ANCHOR_LEN] {
    let mut buf = [0u8; ANCHOR_LEN];
    buf[..8].copy_from_slice(&ANCHOR_MAGIC);
    buf[8..16].copy_from_slice(&val.erase_epoch.to_be_bytes());
    buf[16..24].copy_from_slice(&val.ledger_head.to_be_bytes());
    buf[24..32].copy_from_slice(&val.catalog_epoch.to_be_bytes());
    let crc = crc32c(&buf[..32]);
    buf[32..36].copy_from_slice(&crc.to_be_bytes());
    buf
}

fn decode_anchor(buf: &[u8; ANCHOR_LEN]) -> Result<AnchorValue, Error> {
    if buf[..8] != ANCHOR_MAGIC {
        return Err(Error::Unsupported {
            what: "file anchor magic",
        });
    }
    let expected_crc = u32::from_be_bytes([buf[32], buf[33], buf[34], buf[35]]);
    if crc32c(&buf[..32]) != expected_crc {
        return Err(Error::Corrupt {
            place: "file anchor CRC",
        });
    }
    Ok(AnchorValue {
        erase_epoch: u64::from_be_bytes(
            buf[8..16].try_into().map_err(|_| Error::Corrupt {
                place: "file anchor erase_epoch",
            })?,
        ),
        ledger_head: u64::from_be_bytes(
            buf[16..24].try_into().map_err(|_| Error::Corrupt {
                place: "file anchor ledger_head",
            })?,
        ),
        catalog_epoch: u64::from_be_bytes(
            buf[24..32].try_into().map_err(|_| Error::Corrupt {
                place: "file anchor catalog_epoch",
            })?,
        ),
    })
}

fn read_anchor_file(fs: &dyn Fs, path: &Path) -> Result<AnchorValue, Error> {
    let file = fs.open(path, anchor_io("anchor_read"))?;
    if file.size()? != ANCHOR_LEN as u64 {
        return Err(Error::Corrupt {
            place: "file anchor size",
        });
    }
    let mut buf = [0u8; ANCHOR_LEN];
    let n = file.read_at(0, &mut buf, anchor_io("anchor_read_bytes"))?;
    if n != ANCHOR_LEN {
        return Err(Error::Corrupt {
            place: "file anchor short read",
        });
    }
    decode_anchor(&buf)
}

fn write_anchor_atomic(fs: &dyn Fs, dir: &Path, val: AnchorValue) -> Result<(), Error> {
    let tmp = dir.join(".ANCHOR.tmp");
    let final_path = dir.join("ANCHOR");
    let _ = fs.unlink(&tmp, anchor_io("anchor_cleanup_stale_tmp"));
    let encoded = encode_anchor(val);
    let file = fs.create_new(&tmp, anchor_io("anchor_create_tmp"))?;
    if let Err(e) = file
        .preallocate(ANCHOR_LEN as u64, anchor_io("anchor_prealloc"))
        .and_then(|_| file.append_prealloc(&encoded, anchor_io("anchor_write")))
        .and_then(|_| file.sync_all(anchor_io("anchor_sync")))
    {
        let _ = fs.unlink(&tmp, anchor_io("anchor_cleanup_tmp"));
        return Err(e);
    }
    drop(file);
    fs.rename(&tmp, &final_path, anchor_io("anchor_rename"))?;
    fs.sync_dir(dir, anchor_io("anchor_sync_dir"))
}
