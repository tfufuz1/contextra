// FILE-CONTEXT
// STAND: 2026-09-19T20:12:00Z (SESSION: 01c5be8b)
// ZWECK: Deterministic I/O error injector and Virtual File System test utility.
// INVARIANTEN: Zero real data corruption, deterministic fault triggers based on operation count.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Configuration for fault injection.
#[derive(Debug, Clone, Default)]
pub struct FaultConfig {
    /// Fail write operations after this many successful writes.
    pub fail_writes_after: Option<usize>,
    /// Fail sync / flush operations after this many successful syncs.
    pub fail_syncs_after: Option<usize>,
    /// Fail read operations after this many successful reads.
    pub fail_reads_after: Option<usize>,
    /// Global kill-switch: fail all operations immediately.
    pub fail_all: bool,
}

/// Simulated in-memory file system with deterministic fault injection capabilities.
#[derive(Debug, Clone, Default)]
pub struct FaultVfs {
    files: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
    read_count: Arc<AtomicUsize>,
    write_count: Arc<AtomicUsize>,
    sync_count: Arc<AtomicUsize>,
    config: Arc<RwLock<FaultConfig>>,
    crashed: Arc<AtomicBool>,
}

impl FaultVfs {
    /// Creates a new, clean `FaultVfs`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the active fault configuration.
    pub fn set_config(&self, config: FaultConfig) {
        *self.config.write() = config;
    }

    /// Simulates a system crash (disables further operations until reset).
    pub fn trigger_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
    }

    /// Resets crash state and counters.
    pub fn reset_counters(&self) {
        self.read_count.store(0, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.sync_count.store(0, Ordering::SeqCst);
        self.crashed.store(false, Ordering::SeqCst);
    }

    /// Checks if a write is allowed or if a fault should be triggered.
    fn check_write_fault(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }
        let current = self.write_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_writes_after {
            if current >= limit {
                return Err(io::Error::new(
                    ErrorKind::WriteZero,
                    "Simulated disk write failure",
                ));
            }
        }
        Ok(())
    }

    /// Checks if a read is allowed or if a fault should be triggered.
    fn check_read_fault(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }
        let current = self.read_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_reads_after {
            if current >= limit {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    "Simulated disk read failure",
                ));
            }
        }
        Ok(())
    }

    /// Writes data to a path, respecting fault policies.
    pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        self.check_write_fault()?;
        self.files.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    /// Reads data from a path, respecting fault policies.
    pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.check_read_fault()?;
        self.files
            .read()
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found in FaultVfs"))
    }

    /// Syncs / flushes a path or the entire VFS, respecting fault policies.
    pub fn sync(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }
        let current = self.sync_count.fetch_add(1, Ordering::SeqCst);
        if let Some(limit) = cfg.fail_syncs_after {
            if current >= limit {
                return Err(io::Error::other("Simulated fsync I/O failure"));
            }
        }
        Ok(())
    }

    /// Returns the number of write operations attempted.
    pub fn write_count(&self) -> usize {
        self.write_count.load(Ordering::SeqCst)
    }

    /// Returns the number of read operations attempted.
    pub fn read_count(&self) -> usize {
        self.read_count.load(Ordering::SeqCst)
    }

    /// Returns the number of sync operations attempted.
    pub fn sync_count(&self) -> usize {
        self.sync_count.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_fault_vfs_write_limit() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            fail_writes_after: Some(2),
            ..Default::default()
        });

        let p1 = Path::new("file1.txt");
        let p2 = Path::new("file2.txt");
        let p3 = Path::new("file3.txt");

        assert!(vfs.write_file(p1, b"hello").is_ok());
        assert!(vfs.write_file(p2, b"world").is_ok());
        let res3 = vfs.write_file(p3, b"fail");
        assert!(res3.is_err());
        assert_eq!(res3.unwrap_err().kind(), ErrorKind::WriteZero);
    }

    #[test]
    fn test_fault_vfs_sync_failure() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            fail_syncs_after: Some(1),
            ..Default::default()
        });

        assert!(vfs.sync().is_ok());
        assert!(vfs.sync().is_err());
    }
}
