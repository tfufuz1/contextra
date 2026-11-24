//! Protected two-slot disk reserve for bounded purge rewrites (SP-PURGE E-63).

use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::Error;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub const REQUIRED_PURGE_RESERVE_BYTES: u64 = 128 * 1024 * 1024;
const SLOT_COUNT: usize = 2;
const IO: IoCtx = IoCtx { role: FileRole::PurgeScratch, purpose: "purge_scratch_reserve" };

/// A crash-recovery slot is held aside; ordinary writers can lease only slot 0.
/// Both backing files are durably preallocated when this reserve opens.
pub struct ScratchReserve {
    root: PathBuf,
    slot_bytes: u64,
    writer_leased: Arc<AtomicBool>,
}

impl ScratchReserve {
    pub fn open(fs: &dyn Fs, root: &Path, slot_bytes: u64) -> Result<Self, Error> {
        let caps = fs.caps();
        if !caps.durable_file_sync || !caps.durable_dir_sync || !caps.atomic_rename
            || !caps.preallocation_reserves_space
        {
            return Err(Error::Unsupported { what: "filesystem cannot provide durable purge scratch" });
        }
        let minimum_slot = REQUIRED_PURGE_RESERVE_BYTES / SLOT_COUNT as u64;
        if slot_bytes < minimum_slot {
            return Err(Error::Limit { limit: "purge scratch slot must be at least 64 MiB" });
        }
        let mut files: Vec<Option<Box<dyn contextra_storage_ports::traits::File>>> = Vec::with_capacity(SLOT_COUNT);
        let mut needed = 0u64;
        for slot in 0..SLOT_COUNT {
            let path = slot_path(root, slot);
            let file = match fs.open(&path, IO) {
                Ok(file) => {
                    let current_size = file.size()?;
                    if current_size < slot_bytes {
                        needed = needed.checked_add(slot_bytes - current_size)
                            .ok_or(Error::Limit { limit: "purge scratch reserve size overflow" })?;
                    }
                    Some(file)
                }
                Err(Error::Io { kind: std::io::ErrorKind::NotFound, .. }) => {
                    needed = needed.checked_add(slot_bytes)
                        .ok_or(Error::Limit { limit: "purge scratch reserve size overflow" })?;
                    None
                }
                Err(error) => return Err(error),
            };
            files.push(file);
        }
        if fs.free_space(root)? < needed { return Err(Error::DiskFull); }
        for (slot, existing) in files.into_iter().enumerate() {
            let file = match existing {
                Some(file) => file,
                None => fs.create_new(&slot_path(root, slot), IO)?,
            };
            if file.size()? < slot_bytes { file.preallocate(slot_bytes, IO)?; }
            file.sync_all(IO)?;
        }
        fs.sync_dir(root, IO)?;
        Ok(Self { root: root.to_path_buf(), slot_bytes, writer_leased: Arc::new(AtomicBool::new(false)) })
    }

    pub fn slot_bytes(&self) -> u64 { self.slot_bytes }

    pub fn writer_slot(&self) -> Result<ScratchSlot, Error> {
        self.writer_leased.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error::Limit { limit: "purge rewrite concurrency is one" })?;
        Ok(ScratchSlot { path: slot_path(&self.root, 0), leased: Some(Arc::clone(&self.writer_leased)) })
    }

    /// Reserved for recovery from a killed writer; not available to concurrent
    /// flush/compaction or another purge operation.
    pub fn recovery_path(&self) -> PathBuf { slot_path(&self.root, 1) }
}

pub struct ScratchSlot {
    path: PathBuf,
    leased: Option<Arc<AtomicBool>>,
}

impl ScratchSlot {
    pub fn path(&self) -> &Path { &self.path }
}

impl Drop for ScratchSlot {
    fn drop(&mut self) {
        if let Some(leased) = self.leased.take() { leased.store(false, Ordering::Release); }
    }
}

fn slot_path(root: &Path, slot: usize) -> PathBuf {
    root.join(format!("PURGE-SCRATCH-{slot}"))
}
