//! # Fehlereinspritzendes Dateisystem (FaultFs)
//!
//! Deterministische Fehlereinspritzung nach SPEC §5.1, §10.5, §19.2 und IMPL §6 WP-1.3.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

pub mod config;
pub use config::{CrashMode, FaultConfig, XorShift64};

use crate::testing::MemFs;
use contextra_storage_ports::{DirEntry, File, Fs, FsCaps, LockGuard, LockMode, OpenMode, Perm, ReadReq};
use std::io;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Dateisystem-Wrapper mit deterministischer Fehlereinspritzung nach SPEC §19.2.
pub struct FaultFs<F> {
    inner: F,
    config: Arc<FaultConfig>,
}

impl<F: Fs> FaultFs<F> {
    /// Erstellt ein neues `FaultFs` mit gegebener Konfiguration.
    pub fn new(inner: F, config: Arc<FaultConfig>) -> Self {
        Self { inner, config }
    }

    /// Erstellt ein neues `FaultFs` mit einem Zufalls-Seed.
    pub fn with_seed(inner: F, seed: u64) -> Self {
        Self::new(inner, Arc::new(FaultConfig::new(seed)))
    }

    /// Liefert eine Referenz auf die Konfiguration.
    pub fn config(&self) -> &Arc<FaultConfig> {
        &self.config
    }

    /// Liefert den Seed der Fehlereinspritzung.
    pub fn seed(&self) -> u64 {
        self.config.seed
    }

    fn check_op(&self, op_name: &str) -> io::Result<()> {
        if self.config.fail_io.load(Ordering::Relaxed) {
            self.config.record_event(&format!("{op_name}:Err(EIO)"));
            return Err(io::Error::new(io::ErrorKind::Other, "Injected EIO"));
        }
        let limit = self.config.fail_after_ops.load(Ordering::Relaxed);
        if limit > 0 {
            let current = self.config.op_counter.fetch_add(1, Ordering::SeqCst) + 1;
            if current >= limit {
                self.config.trigger_crash();
                self.config.record_event(&format!("{op_name}:CrashAtOp({current})"));
                return Err(io::Error::new(io::ErrorKind::Other, "Injected crash after N operations"));
            }
        }
        self.config.record_event(op_name);
        Ok(())
    }

    fn check_write_op(&self, op_name: &str) -> io::Result<()> {
        if self.config.fail_enospc.load(Ordering::Relaxed) {
            self.config.record_event(&format!("{op_name}:Err(ENOSPC)"));
            return Err(io::Error::new(io::ErrorKind::StorageFull, "Injected ENOSPC"));
        }
        self.check_op(op_name)
    }
}

impl FaultFs<MemFs> {
    /// Initialisiert ein `FaultFs` über `MemFs` und bindet den automatischen Absturz-Handler an.
    pub fn new_mem(inner: MemFs, config: Arc<FaultConfig>) -> Self {
        let mem_clone = inner.clone();
        let cfg_clone = Arc::clone(&config);
        config.set_crash_handler(Arc::new(move || {
            let mode = *cfg_clone.crash_mode.lock().unwrap();
            match mode {
                CrashMode::Lost => mem_clone.crash(),
                CrashMode::Torn(sec) => mem_clone.crash_torn(sec),
                CrashMode::Reordered => {
                    let seed = cfg_clone.rng.lock().unwrap().next_u64();
                    mem_clone.crash_reordered(seed);
                }
            }
        }));
        Self { inner, config }
    }

    /// Simuliert einen manuellen Absturz des zugrundeliegenden `MemFs`.
    pub fn crash(&self) {
        self.config.record_event("ManualCrash");
        self.config.trigger_crash();
        self.inner.crash();
    }

    /// Erstellt einen Schnappschuss des gesamten dauerhaften Zustands (F-10).
    pub fn take_snapshot(&self) -> u64 {
        self.config.record_event("TakeSnapshot");
        self.inner.take_snapshot()
    }

    /// Setzt den gesamten Zustand auf einen früheren Schnappschuss zurück (F-10).
    pub fn rollback_to_snapshot(&self, snap_id: u64) -> io::Result<()> {
        self.config.record_event(&format!("RollbackSnapshot:{snap_id}"));
        self.inner.restore_snapshot(snap_id)
    }
}

struct FaultFile {
    inner: Box<dyn File>,
    config: Arc<FaultConfig>,
}

impl File for FaultFile {
    fn read_at(&self, buf: &mut [u8], off: u64) -> io::Result<usize> {
        if self.config.fail_io.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Other, "Injected EIO"));
        }
        if self.config.fail_read_eio.load(Ordering::Relaxed) {
            let target_off = self.config.fail_read_offset.load(Ordering::Relaxed);
            let end_off = off.saturating_add(buf.len() as u64);
            if target_off == u64::MAX || (target_off >= off && target_off < end_off) {
                return Err(io::Error::new(io::ErrorKind::Other, "Injected EIO at offset"));
            }
        }

        let limit = self.config.fail_after_ops.load(Ordering::Relaxed);
        if limit > 0 {
            let current = self.config.op_counter.fetch_add(1, Ordering::SeqCst) + 1;
            if current >= limit {
                self.config.trigger_crash();
                return Err(io::Error::new(io::ErrorKind::Other, "Injected crash"));
            }
        }

        let n = self.inner.read_at(buf, off)?;
        if self.config.enable_bit_rot.load(Ordering::Relaxed) {
            let rot_off = self.config.bit_rot_offset.load(Ordering::Relaxed);
            let end_off = off.saturating_add(n as u64);
            if rot_off >= off && rot_off < end_off {
                let idx = (rot_off - off) as usize;
                if idx < buf.len() {
                    buf[idx] ^= 0x01;
                    self.config.record_event(&format!("BitRotAt:{rot_off}"));
                }
            }
        }
        Ok(n)
    }

    fn read_batch(&self, reqs: &mut [ReadReq]) -> io::Result<()> {
        for req in reqs.iter_mut() {
            let n = self.read_at(req.buf, req.offset)?;
            if n < req.buf.len() { req.buf[n..].fill(0); }
        }
        Ok(())
    }

    fn write_all_at(&self, buf: &[u8], off: u64) -> io::Result<()> {
        if self.config.fail_enospc.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::StorageFull, "Injected ENOSPC"));
        }
        if self.config.fail_io.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Other, "Injected EIO"));
        }
        let limit = self.config.fail_after_ops.load(Ordering::Relaxed);
        if limit > 0 {
            let current = self.config.op_counter.fetch_add(1, Ordering::SeqCst) + 1;
            if current >= limit {
                self.config.trigger_crash();
                return Err(io::Error::new(io::ErrorKind::Other, "Injected crash"));
            }
        }

        if self.config.lost_write.load(Ordering::Relaxed) {
            self.config.record_event(&format!("LostWrite:off={off},len={}", buf.len()));
            return Ok(());
        }

        let actual_off = if self.config.misdirected_write.load(Ordering::Relaxed) {
            let delta = self.config.misdirected_offset_delta.load(Ordering::Relaxed);
            self.config.record_event(&format!("MisdirectedWrite:off={off}->{}", off.wrapping_add(delta)));
            off.wrapping_add(delta)
        } else {
            off
        };

        let short_lim = self.config.short_write_limit.load(Ordering::Relaxed) as usize;
        if short_lim > 0 && buf.len() > short_lim {
            self.inner.write_all_at(&buf[..short_lim], actual_off)?;
            return Err(io::Error::new(io::ErrorKind::StorageFull, "Injected short write ENOSPC"));
        }

        let torn_lim = self.config.torn_write_limit.load(Ordering::Relaxed) as usize;
        if torn_lim > 0 && buf.len() > torn_lim {
            self.inner.write_all_at(&buf[..torn_lim], actual_off)?;
            return Err(io::Error::new(io::ErrorKind::WriteZero, "Injected torn write"));
        }

        self.inner.write_all_at(buf, actual_off)
    }

    fn len(&self) -> io::Result<u64> { self.inner.len() }
    fn set_len(&self, len: u64) -> io::Result<()> { self.inner.set_len(len) }

    fn sync_data(&self) -> io::Result<()> {
        if self.config.fail_sync.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Other, "Injected sync failure"));
        }
        self.inner.sync_data()
    }

    fn sync_all(&self) -> io::Result<()> {
        if self.config.fail_sync.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Other, "Injected sync failure"));
        }
        self.inner.sync_all()
    }

    fn preallocate(&self, len: u64) -> io::Result<()> {
        if self.config.fail_enospc.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::StorageFull, "Injected ENOSPC"));
        }
        self.inner.preallocate(len)
    }
}

impl<F: Fs> Fs for FaultFs<F> {
    fn create_dir_all(&self, p: &Path, perm: Perm) -> io::Result<()> {
        self.check_write_op("create_dir_all")?;
        self.inner.create_dir_all(p, perm)
    }

    fn open(&self, p: &Path, mode: OpenMode) -> io::Result<Box<dyn File>> {
        match mode {
            OpenMode::CreateNew(_) => self.check_write_op("open_create")?,
            OpenMode::Read | OpenMode::ReadWrite => self.check_op("open_read")?,
        }
        let file = self.inner.open(p, mode)?;
        Ok(Box::new(FaultFile { inner: file, config: Arc::clone(&self.config) }))
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.check_write_op("rename")?;
        self.inner.rename(from, to)
    }

    fn remove_file(&self, p: &Path) -> io::Result<()> {
        self.check_write_op("remove_file")?;
        self.inner.remove_file(p)
    }

    fn remove_dir(&self, p: &Path) -> io::Result<()> {
        self.check_write_op("remove_dir")?;
        self.inner.remove_dir(p)
    }

    fn list_dir(&self, p: &Path) -> io::Result<Vec<DirEntry>> {
        self.check_op("list_dir")?;
        self.inner.list_dir(p)
    }

    fn sync_dir(&self, p: &Path) -> io::Result<()> {
        if self.config.fail_sync.load(Ordering::Relaxed) {
            return Err(io::Error::new(io::ErrorKind::Other, "Injected sync failure"));
        }
        self.inner.sync_dir(p)
    }

    fn lock(&self, p: &Path, mode: LockMode) -> io::Result<Box<dyn LockGuard>> {
        self.check_op("lock")?;
        self.inner.lock(p, mode)
    }

    fn caps(&self) -> FsCaps { self.inner.caps() }

    fn hard_link(&self, from: &Path, to: &Path) -> io::Result<()> {
        self.check_write_op("hard_link")?;
        self.inner.hard_link(from, to)
    }
}
