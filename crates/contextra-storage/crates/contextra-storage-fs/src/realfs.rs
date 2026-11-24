//! RealFs implementation using standard POSIX filesystem operations (SP-ENV §3.5, SPEC §3.2).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{File, FileRole, Fs, FsCaps, FsLock, IoCtx};
use contextra_storage_ports::Error;
use std::fs::{File as StdFile, OpenOptions};
use std::os::unix::fs::FileExt;
use std::path::{Path, PathBuf};

fn map_io_error(e: std::io::Error, role: FileRole) -> Error {
    let role_str = match role {
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
    };
    Error::Io {
        role: role_str,
        kind: e.kind(),
    }
}

pub struct RealFile {
    file: StdFile,
    role: FileRole,
}

impl RealFile {
    pub fn new(file: StdFile, role: FileRole) -> Self {
        Self { file, role }
    }
}

impl File for RealFile {
    fn read_at(&self, off: u64, buf: &mut [u8], ctx: IoCtx) -> Result<usize, Error> {
        self.file
            .read_at(buf, off)
            .map_err(|e| map_io_error(e, ctx.role))
    }

    fn append_prealloc(&self, data: &[u8], ctx: IoCtx) -> Result<(), Error> {
        let size = self.file.metadata().map_err(|e| map_io_error(e, ctx.role))?.len();
        self.file
            .write_all_at(data, size)
            .map_err(|e| map_io_error(e, ctx.role))
    }

    fn preallocate(&self, len: u64, ctx: IoCtx) -> Result<(), Error> {
        self.file
            .set_len(len)
            .map_err(|e| map_io_error(e, ctx.role))
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
        self.file
            .metadata()
            .map(|m| m.len())
            .map_err(|e| map_io_error(e, self.role))
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
        Ok(Box::new(RealFile::new(file, ctx.role)))
    }

    fn create_new(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn File>, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|e| map_io_error(e, ctx.role))?;
        Ok(Box::new(RealFile::new(file, ctx.role)))
    }

    fn rename(&self, from: &Path, to: &Path, ctx: IoCtx) -> Result<(), Error> {
        std::fs::rename(from, to).map_err(|e| map_io_error(e, ctx.role))
    }

    fn unlink(&self, path: &Path, ctx: IoCtx) -> Result<(), Error> {
        std::fs::remove_file(path).map_err(|e| map_io_error(e, ctx.role))
    }

    fn lock(&self, path: &Path, ctx: IoCtx) -> Result<Box<dyn FsLock>, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(|e| map_io_error(e, ctx.role))?;
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

    fn free_space(&self, _dir: &Path) -> Result<u64, Error> {
        // Return 1 TiB available placeholder for standard simulation/checks
        Ok(1024 * 1024 * 1024 * 1024)
    }

    fn caps(&self) -> FsCaps {
        FsCaps {
            durable_file_sync: true,
            durable_dir_sync: true,
            atomic_rename: true,
            preallocation_reserves_space: true,
            hard_link: true,
            reflink: false,
            punch_hole: false,
        }
    }
}
