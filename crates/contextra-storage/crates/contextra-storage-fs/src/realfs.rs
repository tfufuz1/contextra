//! RealFs implementation using std + rustix (SP-ENV §3.5, SPEC §3.7.3).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{File, FileRole, Fs, FsCaps, FsLock, IoCtx};
use contextra_storage_ports::Error;
use std::fs::{File as StdFile, OpenOptions, TryLockError};
use std::io::ErrorKind;
use std::os::unix::fs::FileExt;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

fn role_name(role: FileRole) -> &'static str {
    match role {
        FileRole::Catalog => "catalog",
        FileRole::Redo => "redo",
        FileRole::Run => "run",
        FileRole::Blob => "blob",
        FileRole::Keys => "keys",
        FileRole::Anchor => "anchor",
        FileRole::Ledger => "ledger",
        FileRole::Format => "format",
        FileRole::PurgeScratch => "purge_scratch",
        FileRole::Other => "other",
    }
}

fn map_io_error(e: std::io::Error, role: FileRole) -> Error {
    if e.kind() == ErrorKind::StorageFull {
        return Error::DiskFull;
    }
    Error::Io {
        role: role_name(role),
        kind: e.kind(),
    }
}

#[cfg(unix)]
fn map_errno(err: rustix::io::Errno, role: FileRole) -> Error {
    let io_err = std::io::Error::from_raw_os_error(err.raw_os_error());
    map_io_error(io_err, role)
}

struct RealFileState {
    logical_len: u64,
    allocated: u64,
}

pub struct RealFile {
    file: StdFile,
    state: Mutex<RealFileState>,
}

impl RealFile {
    pub fn new(file: StdFile, role: FileRole) -> Result<Self, Error> {
        let len = file.metadata().map_err(|e| map_io_error(e, role))?.len();
        Ok(Self {
            file,
            state: Mutex::new(RealFileState {
                logical_len: len,
                allocated: len,
            }),
        })
    }

    fn zero_fill_range(&self, start: u64, end: u64, ctx: IoCtx) -> Result<(), Error> {
        const ZEROS: [u8; 8192] = [0u8; 8192];
        let mut offset = start;
        while offset < end {
            let chunk = ((end - offset) as usize).min(ZEROS.len());
            self.file
                .write_all_at(&ZEROS[..chunk], offset)
                .map_err(|e| map_io_error(e, ctx.role))?;
            offset += chunk as u64;
        }
        Ok(())
    }
}

impl File for RealFile {
    fn read_at(&self, off: u64, buf: &mut [u8], ctx: IoCtx) -> Result<usize, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned {
            cause: "RealFile state lock",
        })?;
        if off >= state.logical_len {
            return Ok(0);
        }
        let available = (state.logical_len - off) as usize;
        let count = buf.len().min(available);
        self.file
            .read_at(&mut buf[..count], off)
            .map_err(|e| map_io_error(e, ctx.role))
    }

    fn append_prealloc(&self, data: &[u8], ctx: IoCtx) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned {
            cause: "RealFile state lock",
        })?;
        let end = state
            .logical_len
            .checked_add(data.len() as u64)
            .ok_or(Error::Limit { limit: "file size" })?;
        if end > state.allocated {
            return Err(Error::Io {
                role: role_name(ctx.role),
                kind: ErrorKind::InvalidData,
            });
        }
        self.file
            .write_all_at(data, state.logical_len)
            .map_err(|e| map_io_error(e, ctx.role))?;
        state.logical_len = end;
        Ok(())
    }

    /// Reserves physical blocks via `fallocate` where available, falling back to
    /// explicit zero-blocks so ENOSPC surfaces during preallocation (AN-10).
    fn preallocate(&self, len: u64, ctx: IoCtx) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::Poisoned {
            cause: "RealFile state lock",
        })?;
        if len <= state.allocated {
            return Ok(());
        }
        let start = state.allocated;
        #[cfg(target_os = "linux")]
        {
            let delta = len - start;
            match rustix::fs::fallocate(
                &self.file,
                rustix::fs::FallocateFlags::empty(),
                start,
                delta,
            ) {
                Ok(()) => {
                    state.allocated = len;
                    return Ok(());
                }
                Err(rustix::io::Errno::OPNOTSUPP | rustix::io::Errno::NOSYS) => {}
                Err(err) => return Err(map_errno(err, ctx.role)),
            }
        }
        self.zero_fill_range(start, len, ctx)?;
        state.allocated = len;
        Ok(())
    }

    fn sync_data(&self, ctx: IoCtx) -> Result<(), Error> {
        self.file
            .sync_data()
            .map_err(|e| map_io_error(e, ctx.role))
    }

    fn sync_all(&self, ctx: IoCtx) -> Result<(), Error> {
        self.file
            .sync_all()
            .map_err(|e| map_io_error(e, ctx.role))
    }

    fn size(&self) -> Result<u64, Error> {
        let state = self.state.lock().map_err(|_| Error::Poisoned {
            cause: "RealFile state lock",
        })?;
        Ok(state.logical_len)
    }
}

pub struct RealFsLock {
    _file: StdFile,
}

impl FsLock for RealFsLock {}

#[derive(Default)]
pub struct RealFs;

impl RealFs {
    pub fn new() -> Self {
        Self
    }
}

impl Fs for RealFs {
    fn open(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .map_err(|e| map_io_error(e, ctx.role))?;
        Ok(Box::new(RealFile::new(file, ctx.role)?))
    }

    fn create_new(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error> {
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create_new(true);
        #[cfg(unix)]
        opts.mode(0o600);
        let file = opts.open(path).map_err(|e| map_io_error(e, ctx.role))?;
        Ok(Box::new(RealFile::new(file, ctx.role)?))
    }

    fn rename(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error> {
        std::fs::rename(from, to).map_err(|e| map_io_error(e, ctx.role))
    }

    fn unlink(&self, path: &Path, ctx: IoCtx) -> Result<(), Error> {
        std::fs::remove_file(path).map_err(|e| map_io_error(e, ctx.role))
    }

    fn link(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error> {
        std::fs::hard_link(from, to).map_err(|e| map_io_error(e, ctx.role))
    }

    fn punch_hole(&self, path: &Path, offset: u64, length: u64, ctx: IoCtx) -> Result<(), Error> {
        #[cfg(target_os = "linux")]
        {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(|e| map_io_error(e, ctx.role))?;
            let flags = rustix::fs::FallocateFlags::PUNCH_HOLE
                | rustix::fs::FallocateFlags::KEEP_SIZE;
            return rustix::fs::fallocate(&file, flags, offset, length).map_err(|err| {
                if err == rustix::io::Errno::OPNOTSUPP || err == rustix::io::Errno::NOSYS {
                    Error::Unsupported {
                        what: "hole punching",
                    }
                } else {
                    map_errno(err, ctx.role)
                }
            });
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (path, offset, length, ctx);
            Err(Error::Unsupported {
                what: "hole punching",
            })
        }
    }

    fn lock(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn FsLock>, Error> {
        let mut opts = OpenOptions::new();
        opts.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        opts.mode(0o600);
        let file = opts.open(path).map_err(|e| map_io_error(e, ctx.role))?;
        file.try_lock().map_err(|e| match e {
            TryLockError::WouldBlock => Error::Io {
                role: role_name(ctx.role),
                kind: ErrorKind::WouldBlock,
            },
            TryLockError::Error(io_err) => map_io_error(io_err, ctx.role),
        })?;
        Ok(Box::new(RealFsLock { _file: file }))
    }

    fn sync_dir(&self, dir: &Path, ctx: IoCtx) -> Result<(), Error> {
        let d = OpenOptions::new()
            .read(true)
            .open(dir)
            .map_err(|e| map_io_error(e, ctx.role))?;
        d.sync_all().map_err(|e| map_io_error(e, ctx.role))
    }

    fn list(&self, dir: &Path) -> Result<Vec<PathBuf>, Error> {
        let entries = std::fs::read_dir(dir).map_err(|e| map_io_error(e, FileRole::Other))?;
        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| map_io_error(e, FileRole::Other))?;
            paths.push(entry.path());
        }
        paths.sort();
        Ok(paths)
    }

    fn free_space(&self, dir: &Path) -> Result<u64, Error> {
        #[cfg(unix)]
        {
            let stat = rustix::fs::statvfs(dir).map_err(|err| map_errno(err, FileRole::Other))?;
            let avail = stat.f_bavail as u64;
            let block = stat.f_frsize as u64;
            return Ok(avail.saturating_mul(block));
        }
        #[cfg(not(unix))]
        {
            if !dir.exists() {
                return Err(Error::Io {
                    role: "other",
                    kind: ErrorKind::NotFound,
                });
            }
            Err(Error::Unsupported {
                what: "free_space on non-unix host",
            })
        }
    }

    fn caps(&self) -> FsCaps {
        FsCaps {
            durable_file_sync: true,
            durable_dir_sync: true,
            atomic_rename: true,
            preallocation_reserves_space: true,
            hard_link: true,
            reflink: false,
            punch_hole: cfg!(target_os = "linux"),
        }
    }
}
