// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Fault-injected Virtual File System (FaultVfs) for crash recovery and I/O error testing.
// INVARIANTEN: Zero-panic, fail-closed error injection, configurable error rate / limits.

use std::collections::HashMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use parking_lot::RwLock;

/// Configuration parameters for controlling fault injection behavior in [`FaultVfs`].
#[derive(Debug, Clone, Default)]
pub struct FaultConfig {
    /// Seed for deterministic RNG operations (if enabled).
    pub seed: Option<u64>,
    /// Simulate write I/O failure after N write operations.
    pub fail_writes_after: Option<usize>,
    /// Simulate read I/O failure after N read operations.
    pub fail_reads_after: Option<usize>,
    /// Simulate fsync failure after N sync operations.
    pub fail_syncs_after: Option<usize>,
    /// Simulate torn write (partial write) fault after this many write operations.
    pub torn_write_after: Option<usize>,
    /// Simulate bitrot byte corruption after this many write operations.
    pub bitrot_write_after: Option<usize>,
    /// Simulate bitrot byte corruption after this many read operations.
    pub bitrot_read_after: Option<usize>,
    /// Simulate lost fsync (returns Ok without persisting) after this many syncs.
    pub lost_syncs_after: Option<usize>,
    /// Global kill-switch: fail all operations immediately.
    pub fail_all: bool,
}

/// Simulated in-memory file system with deterministic fault injection capabilities.
///
/// Supports torn writes, bitrot byte corruptions, lost fsyncs, and seed-based determinism.
#[derive(Debug, Clone, Default)]
pub struct FaultVfs {
    files: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
    durable_files: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
    unpersisted_files: Arc<RwLock<HashMap<PathBuf, Vec<u8>>>>,
    read_count: Arc<AtomicUsize>,
    write_count: Arc<AtomicUsize>,
    sync_count: Arc<AtomicUsize>,
    config: Arc<RwLock<FaultConfig>>,
    seed_log: Arc<RwLock<Vec<u64>>>,
    crashed: Arc<AtomicBool>,
}

impl FaultVfs {
    /// Creates a new `FaultVfs` with default empty configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the active fault configuration and resets counters and crash state.
    pub fn set_config(&self, config: FaultConfig) {
        if let Some(s) = config.seed {
            tracing::info!("FaultVfs configured with seed {}", s);
            self.seed_log.write().push(s);
        }
        *self.config.write() = config;
        self.reset_counters();
    }

    /// Returns a list of all seeds logged during execution.
    pub fn seed_log(&self) -> Vec<u64> {
        self.seed_log.read().clone()
    }

    /// Simulates a system crash (disables further operations and drops unpersisted writes).
    pub fn trigger_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
        // Revert files map to last durable checkpoint on crash if lost_syncs configured
        let cfg = self.config.read();
        if cfg.lost_syncs_after.is_some() {
            *self.files.write() = self.durable_files.read().clone();
        }
        self.unpersisted_files.write().clear();
    }

    /// Resets crash state and counters.
    pub fn reset_counters(&self) {
        self.read_count.store(0, Ordering::SeqCst);
        self.write_count.store(0, Ordering::SeqCst);
        self.sync_count.store(0, Ordering::SeqCst);
        self.crashed.store(false, Ordering::SeqCst);
    }

    /// Checks if a write is allowed or if a fault should be triggered, returning 0-based op index.
    fn check_write_fault(&self) -> io::Result<usize> {
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
        Ok(current)
    }

    /// Checks if a read is allowed or if a fault should be triggered, returning 0-based op index.
    fn check_read_fault(&self) -> io::Result<usize> {
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
        Ok(current)
    }

    /// Writes data to a path in volatile buffer, respecting fault policies (torn write, bitrot, write limits).
    pub fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
        let current_writes = self.check_write_fault()?;
        let cfg = self.config.read();

        let mut write_buf = data.to_vec();

        // Handle bitrot fault on write
        if let Some(bitrot_at) = cfg.bitrot_write_after {
            if current_writes == bitrot_at && !write_buf.is_empty() {
                write_buf[0] ^= 0xFF; // Flip byte
            }
        }

        // Handle torn write fault
        if let Some(torn_at) = cfg.torn_write_after {
            if current_writes == torn_at {
                let partial_len = write_buf.len() / 2;
                write_buf.truncate(partial_len);
                self.unpersisted_files
                    .write()
                    .insert(path.to_path_buf(), write_buf);
                return Err(io::Error::new(
                    ErrorKind::WriteZero,
                    "Simulated torn write fault",
                ));
            }
        }

        self.unpersisted_files
            .write()
            .insert(path.to_path_buf(), write_buf.clone());
        self.files.write().insert(path.to_path_buf(), write_buf);
        Ok(())
    }

    /// Reads data from a path, respecting fault policies (bitrot, read limits).
    pub fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
        let current_reads = self.check_read_fault()?;
        let cfg = self.config.read();

        let files = self.files.read();

        let mut data = files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "File not found in FaultVfs"))?;

        // Handle bitrot fault on read
        if let Some(bitrot_at) = cfg.bitrot_read_after {
            if current_reads == bitrot_at && !data.is_empty() {
                data[0] ^= 0xFF; // Flip byte
            }
        }

        Ok(data)
    }

    /// Syncs / flushes a path or the entire VFS, respecting fault policies (lost fsync, sync limits).
    pub fn sync(&self) -> io::Result<()> {
        if self.crashed.load(Ordering::SeqCst) {
            return Err(io::Error::new(ErrorKind::BrokenPipe, "VFS simulated crash"));
        }
        let cfg = self.config.read();
        if cfg.fail_all {
            return Err(io::Error::other("Simulated injected failure (fail_all)"));
        }
        let current = self.sync_count.fetch_add(1, Ordering::SeqCst);

        // Handle lost fsync: returns Ok(()) but reverts files to durable_files
        if let Some(lost_at) = cfg.lost_syncs_after {
            if current >= lost_at {
                tracing::info!("Simulated lost fsync for seed {:?}", cfg.seed);
                *self.files.write() = self.durable_files.read().clone();
                self.unpersisted_files.write().clear();
                return Ok(());
            }
        }

        if let Some(limit) = cfg.fail_syncs_after {
            if current >= limit {
                return Err(io::Error::other("Simulated fsync I/O failure"));
            }
        }

        // Persist unpersisted changes to durable file map
        let unpersisted = self.unpersisted_files.read().clone();
        for (k, v) in &unpersisted {
            self.durable_files.write().insert(k.clone(), v.clone());
            self.files.write().insert(k.clone(), v.clone());
        }
        self.unpersisted_files.write().clear();
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

    #[test]
    fn test_fault_vfs_torn_write() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            torn_write_after: Some(0),
            ..Default::default()
        });

        let res = vfs.write_file(Path::new("torn.txt"), b"12345678");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::WriteZero);
    }

    #[test]
    fn test_fault_vfs_bitrot() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            bitrot_write_after: Some(0),
            ..Default::default()
        });

        assert!(vfs.write_file(Path::new("bitrot.txt"), b"hello").is_ok());
        let read = vfs.read_file(Path::new("bitrot.txt")).unwrap();
        // First byte was bit-flipped
        assert_ne!(read[0], b'h');
    }

    #[test]
    fn test_fault_vfs_seed_logging() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            seed: Some(42),
            ..Default::default()
        });

        assert_eq!(vfs.seed_log(), vec![42]);
    }

    #[test]
    fn test_fault_vfs_lost_fsync() {
        let vfs = FaultVfs::new();
        vfs.set_config(FaultConfig {
            lost_syncs_after: Some(0),
            ..Default::default()
        });

        let path = Path::new("lost_sync.txt");
        assert!(vfs.write_file(path, b"unpersisted data").is_ok());
        // Lost fsync returns Ok but does not persist
        assert!(vfs.sync().is_ok());

        // On crash, unpersisted data is lost
        vfs.trigger_crash();
        vfs.reset_counters();
        assert!(vfs.read_file(path).is_err());
    }

    #[test]
    fn test_fault_vfs_concurrent_operations_integrity() {
        let vfs = Arc::new(FaultVfs::new());
        vfs.set_config(FaultConfig {
            fail_writes_after: Some(50),
            ..Default::default()
        });

        let mut handles = Vec::new();
        for i in 0..10 {
            let vfs_clone = vfs.clone();
            handles.push(std::thread::spawn(move || {
                for j in 0..10 {
                    let filename = format!("file_{}_{}.txt", i, j);
                    let _ = vfs_clone.write_file(Path::new(&filename), b"data");
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // Exactly 100 write operations attempted
        assert_eq!(vfs.write_count(), 100);
    }
}
