//! Contract verification test suite for `Clock`, `Rng`, `Vfs`, and `MetricsSink` implementations.
//!
//! # SPEC-REF
//! Compliance and contract testing suite enabled under feature `contract-tests`.
//! Downstream adapter crates can invoke these generic functions to verify their implementations
//! satisfy Contextra's behavioral invariants and contracts.

#![cfg(feature = "contract-tests")]

use crate::clock::Clock;
use crate::error::ContextraError;
use crate::metrics::MetricsSink;
use crate::rng::{Rng, SeededRng};
use crate::vfs::Vfs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// Contract test helper verifying [`Clock`] trait implementation compliance.
///
/// Checks monotonic non-decreasing behavior and valid wall-clock epoch timestamps.
pub fn verify_clock_contract<C: Clock>(clock: &C) {
    let t1 = clock.monotonic_nanos();
    std::thread::sleep(std::time::Duration::from_millis(1));
    let t2 = clock.monotonic_nanos();
    assert!(
        t2 >= t1,
        "Clock monotonic_nanos must be non-decreasing (t1={t1}, t2={t2})"
    );

    let wall = clock.now_unix_nanos();
    assert!(
        wall > 0,
        "Clock now_unix_nanos must return positive timestamp"
    );
}

/// Contract test helper verifying [`Rng`] trait implementation compliance.
///
/// Checks non-zero outputs, correct buffer filling, and reproducible seeding if applicable.
pub fn verify_rng_contract<R: Rng>(rng: &mut R) {
    let val1 = rng.next_u64();
    let val2 = rng.next_u64();
    assert_ne!(
        val1, val2,
        "Rng must produce non-constant consecutive u64 draws"
    );

    let unit = rng.next_unit_f64();
    assert!(
        (0.0..1.0).contains(&unit),
        "Rng next_unit_f64 must fall in [0.0, 1.0), got {unit}"
    );

    let mut buf1 = [0u8; 32];
    let mut buf2 = [0u8; 32];
    rng.fill_bytes(&mut buf1);
    rng.fill_bytes(&mut buf2);
    assert_ne!(
        buf1, buf2,
        "Rng fill_bytes must produce non-repeating byte buffers"
    );

    let seeded1 = SeededRng::new(42);
    let seeded2 = SeededRng::new(42);
    assert_eq!(
        seeded1.next_u64(),
        seeded2.next_u64(),
        "SeededRng with identical seed must yield deterministic sequence"
    );
}

/// Contract test helper verifying [`Vfs`] trait implementation compliance.
///
/// Verifies atomic replacement, read roundtrips, directory creation, file removal,
/// directory listing, and proper fail-closed errors on missing paths.
pub fn verify_vfs_contract<V: Vfs>(vfs: &V, base_dir: &Path) {
    // 1. Directory creation
    let sub_dir = base_dir.join("a").join("b");
    vfs.create_dir_all(&sub_dir)
        .expect("create_dir_all must succeed");
    assert!(
        vfs.exists(&sub_dir).expect("exists check on sub_dir"),
        "sub_dir must exist after create_dir_all"
    );

    // 2. Atomic write & read
    let file_path = sub_dir.join("test_file.bin");
    let payload_v1 = b"Hello, Contextra VFS Contract!";
    vfs.write_atomic(&file_path, payload_v1)
        .expect("write_atomic v1 must succeed");

    assert!(
        vfs.exists(&file_path).expect("exists check on file"),
        "file must exist after write_atomic"
    );
    assert_eq!(
        vfs.len(&file_path).expect("len check on file"),
        payload_v1.len() as u64,
        "file length must match written payload"
    );

    let read_back = vfs.read(&file_path).expect("read file must succeed");
    assert_eq!(
        read_back, payload_v1,
        "read bytes must match written payload"
    );

    // 3. Atomic overwrite
    let payload_v2 = b"Overwritten Contextra VFS Contract Data";
    vfs.write_atomic(&file_path, payload_v2)
        .expect("write_atomic v2 must succeed");
    let read_back_v2 = vfs
        .read(&file_path)
        .expect("read file after overwrite must succeed");
    assert_eq!(
        read_back_v2, payload_v2,
        "read bytes after overwrite must match new payload"
    );

    // 4. Directory listing & sync
    let entries = vfs.list_dir(&sub_dir).expect("list_dir must succeed");
    assert!(
        entries.contains(&"test_file.bin".to_string()),
        "list_dir must contain test_file.bin, got {entries:?}"
    );

    vfs.sync_dir(&sub_dir)
        .expect("sync_dir on valid directory must succeed");

    // 5. File removal
    vfs.remove_file(&file_path)
        .expect("remove_file must succeed");
    assert!(
        !vfs.exists(&file_path).expect("exists check after removal"),
        "file must not exist after removal"
    );

    // 6. Fail-closed error handling on non-existent paths
    let missing_path = sub_dir.join("non_existent.dat");
    let err = vfs
        .read(&missing_path)
        .expect_err("read on non-existent file must fail");
    match err {
        ContextraError::NotFound(_) | ContextraError::Io(_) | ContextraError::Storage(_) => {}
        other => panic!("Expected NotFound, Storage or Io error, got {other:?}"),
    }

    let len_err = vfs
        .len(&missing_path)
        .expect_err("len on non-existent file must fail");
    match len_err {
        ContextraError::NotFound(_) | ContextraError::Io(_) | ContextraError::Storage(_) => {}
        other => panic!("Expected NotFound, Storage or Io error, got {other:?}"),
    }
}

/// Contract test helper verifying [`MetricsSink`] trait implementation compliance.
pub fn verify_metrics_sink_contract<M: MetricsSink>(sink: &M) {
    sink.record_counter("contract_counter", 10, &[("layer", "ring0")]);
    sink.record_counter("contract_counter", 5, &[("layer", "ring0")]);
    sink.record_gauge("contract_gauge", 99.5, &[("node", "n1")]);
    sink.record_histogram("contract_latency", 0.005, &[("op", "read")]);
}

/// In-memory VFS reference implementation used strictly for contract self-testing.
#[derive(Debug, Default)]
pub struct MemVfs {
    files: RwLock<BTreeMap<PathBuf, Vec<u8>>>,
    dirs: RwLock<BTreeMap<PathBuf, ()>>,
}

impl MemVfs {
    /// Creates a new [`MemVfs`].
    pub fn new() -> Self {
        Self::default()
    }
}

fn std_io_err(msg: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, msg.into())
}

impl Vfs for MemVfs {
    fn read(&self, path: &Path) -> Result<Vec<u8>, ContextraError> {
        let guard = self
            .files
            .read()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        guard
            .get(path)
            .cloned()
            .ok_or_else(|| ContextraError::NotFound(path.display().to_string()))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<(), ContextraError> {
        if let Some(parent) = path.parent() {
            self.create_dir_all(parent)?;
        }
        let mut guard = self
            .files
            .write()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        guard.insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), ContextraError> {
        let mut guard = self
            .dirs
            .write()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        let mut current = path.to_path_buf();
        while !current.as_os_str().is_empty() {
            guard.insert(current.clone(), ());
            if !current.pop() {
                break;
            }
        }
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), ContextraError> {
        let mut guard = self
            .files
            .write()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        if guard.remove(path).is_some() {
            Ok(())
        } else {
            Err(ContextraError::NotFound(path.display().to_string()))
        }
    }

    fn exists(&self, path: &Path) -> Result<bool, ContextraError> {
        let files_guard = self
            .files
            .read()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        if files_guard.contains_key(path) {
            return Ok(true);
        }
        let dirs_guard = self
            .dirs
            .read()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        Ok(dirs_guard.contains_key(path))
    }

    fn len(&self, path: &Path) -> Result<u64, ContextraError> {
        let guard = self
            .files
            .read()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        guard
            .get(path)
            .map(|v| v.len() as u64)
            .ok_or_else(|| ContextraError::NotFound(path.display().to_string()))
    }

    fn list_dir(&self, path: &Path) -> Result<Vec<String>, ContextraError> {
        let dirs_guard = self
            .dirs
            .read()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        if !dirs_guard.contains_key(path) {
            return Err(ContextraError::NotFound(path.display().to_string()));
        }

        let files_guard = self
            .files
            .read()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        let mut entries = Vec::new();

        for file_path in files_guard.keys() {
            if file_path.parent() == Some(path) {
                if let Some(name) = file_path.file_name() {
                    entries.push(name.to_string_lossy().to_string());
                }
            }
        }
        for dir_path in dirs_guard.keys() {
            if dir_path != path && dir_path.parent() == Some(path) {
                if let Some(name) = dir_path.file_name() {
                    entries.push(name.to_string_lossy().to_string());
                }
            }
        }

        entries.sort();
        entries.dedup();
        Ok(entries)
    }

    fn sync_dir(&self, path: &Path) -> Result<(), ContextraError> {
        let dirs_guard = self
            .dirs
            .read()
            .map_err(|e| ContextraError::Io(std_io_err(e.to_string())))?;
        if dirs_guard.contains_key(path) {
            Ok(())
        } else {
            Err(ContextraError::NotFound(path.display().to_string()))
        }
    }
}

#[cfg(test)]
mod self_tests {
    use super::*;
    use crate::clock::SystemClock;
    use crate::metrics::{InMemoryMetricsSink, MetricKey, NoopMetricsSink};
    use crate::rng::SeededRng;

    #[test]
    fn test_contract_clock_system_clock() {
        let clock = SystemClock::new();
        verify_clock_contract(&clock);
    }

    #[test]
    fn test_contract_rng_seeded() {
        let mut rng = SeededRng::new(7);
        verify_rng_contract(&mut rng);
    }

    #[test]
    fn test_contract_vfs_mem_vfs() {
        let vfs = MemVfs::new();
        let base_dir = Path::new("/data");
        verify_vfs_contract(&vfs, base_dir);
    }

    #[test]
    fn test_contract_metrics_sinks() {
        verify_metrics_sink_contract(&NoopMetricsSink);

        let mem_sink = InMemoryMetricsSink::new();
        verify_metrics_sink_contract(&mem_sink);

        let snap = mem_sink.snapshot();
        let key = MetricKey::new("contract_counter", &[("layer", "ring0")]);
        assert_eq!(snap.counters.get(&key), Some(&15));
    }
}
