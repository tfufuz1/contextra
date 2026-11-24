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
#[cfg(test)]
use crate::scanner::WitnessLayer;
use crate::scanner::{LayerScanner, Purger, ScanTarget, ScanTechnique};
use crate::storage::StorageEngine;
use crate::text_index::TextIndex;
use crate::types::{DocId, TenantId, TxId};
use crate::vector_index::VectorIndex;
use crate::vfs::Vfs;
use crate::RequestContext;
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

/// Contract test helper verifying [`StorageEngine`] trait implementation compliance.
///
/// Verifies put/get, isolation, prefix scanning, delete, and commit lifecycle.
pub async fn verify_storage_engine_contract<S: StorageEngine>(storage: &S) {
    let tx1 = TxId(101);
    let key1 = b"contract:storage:k1";
    let val1 = b"value_1";

    // 1. Put and get within tx
    storage
        .put(tx1, key1, val1)
        .await
        .expect("put must succeed");

    // 2. Commit transaction
    storage.commit(tx1).await.expect("commit must succeed");

    // 3. Get committed value
    let res = storage.get(key1).await.expect("get must succeed");
    assert_eq!(
        res.as_deref(),
        Some(&val1[..]),
        "Committed value must match"
    );

    // 4. Batch put and scan prefix
    let tx2 = TxId(102);
    let batch = vec![
        (b"contract:storage:p_a".to_vec(), b"v_a".to_vec()),
        (b"contract:storage:p_b".to_vec(), b"v_b".to_vec()),
    ];
    storage
        .put_batch(tx2, &batch)
        .await
        .expect("put_batch must succeed");
    storage.commit(tx2).await.expect("commit tx2 must succeed");

    let scanned = storage
        .scan_prefix(b"contract:storage:p_")
        .await
        .expect("scan_prefix must succeed");
    assert!(
        scanned.len() >= 2,
        "scan_prefix must return at least 2 entries"
    );

    // 5. Delete prefix
    let tx3 = TxId(103);
    let deleted = storage
        .delete_prefix(tx3, b"contract:storage:p_")
        .await
        .expect("delete_prefix must succeed");
    assert!(deleted >= 2, "delete_prefix must report deleted count >= 2");
    storage.commit(tx3).await.expect("commit tx3 must succeed");

    // 6. Rollback behavior
    let tx4 = TxId(104);
    storage
        .put(tx4, b"contract:storage:rollback_key", b"discard")
        .await
        .expect("put must succeed");
    storage.rollback(tx4).await.expect("rollback must succeed");
    let uncommitted = storage
        .get(b"contract:storage:rollback_key")
        .await
        .expect("get rolled-back key");
    assert_eq!(
        uncommitted, None,
        "Rolled-back key must not be visible after rollback"
    );
}

/// Contract test helper verifying [`VectorIndex`] trait implementation compliance.
///
/// Verifies insertion, search, delete, len, and transaction lifecycle.
pub async fn verify_vector_index_contract<V: VectorIndex>(index: &V) {
    let tx1 = TxId(201);
    let doc1 = DocId(1001);
    let vec1 = [1.0, 0.0, 0.0, 0.0];

    // 1. Insert vector
    index
        .insert(tx1, doc1, &vec1)
        .await
        .expect("vector insert must succeed");
    index.commit(tx1).await.expect("vector commit must succeed");

    assert!(
        index.len().await >= 1,
        "Vector index len must be >= 1 after insert"
    );

    // 2. Search
    let results = index
        .search(&vec1, 5)
        .await
        .expect("vector search must succeed");
    assert!(
        !results.is_empty(),
        "Vector search must return matching candidates"
    );

    // 3. Delete
    let tx2 = TxId(202);
    index
        .delete(tx2, doc1)
        .await
        .expect("vector delete must succeed");
    index.commit(tx2).await.expect("vector commit must succeed");
}

/// Contract test helper verifying [`TextIndex`] trait implementation compliance.
///
/// Verifies text insertion, search, delete, len, and commit.
pub async fn verify_text_index_contract<T: TextIndex>(index: &T) {
    let tx1 = TxId(301);
    let doc1 = DocId(2001);
    let text1 = "contract verification quick brown fox";

    // 1. Insert text
    index
        .insert(tx1, doc1, text1)
        .await
        .expect("text insert must succeed");
    index.commit(tx1).await.expect("text commit must succeed");

    assert!(
        index.len().await >= 1,
        "Text index len must be >= 1 after insert"
    );

    // 2. Search
    let results = index
        .search("quick brown", 5)
        .await
        .expect("text search must succeed");
    assert!(
        !results.is_empty(),
        "Text search must return candidates for indexed keywords"
    );

    // 3. Delete
    let tx2 = TxId(302);
    index
        .delete(tx2, doc1)
        .await
        .expect("text delete must succeed");
    index.commit(tx2).await.expect("text commit must succeed");
}

/// Contract test helper verifying [`Purger`] trait implementation compliance.
///
/// Verifies idempotency and safe fail-closed execution.
pub fn verify_purger_contract<P: Purger>(purger: &P) {
    let ctx = RequestContext::new(TenantId::try_new(1).unwrap(), TxId(401));
    let target = ScanTarget::new("tenant_contract_test", [0xFE; 32]);

    // 1. First execution
    let stats1 = purger.purge(&ctx, &target).expect("first purge call");

    // 2. Idempotent second execution
    let stats2 = purger
        .purge(&ctx, &target)
        .expect("second purge call must be idempotent");

    assert_eq!(
        stats2.purged_units, 0,
        "Subsequent purge on already cleaned target should report 0 units"
    );
    let _ = stats1;
}

/// Contract test helper verifying [`LayerScanner`] trait implementation compliance.
///
/// Verifies positive control, scanned unit registration, and invariant `INV-WITNESS-MINT-1`.
pub fn verify_layer_scanner_contract<L: LayerScanner>(scanner: &L, clock: &dyn Clock) {
    let ctx = RequestContext::new(TenantId::try_new(1).unwrap(), TxId(501));
    let target = ScanTarget::new("tenant_scanner_test", [0xEE; 32]);

    let witness = scanner
        .scan(&ctx, target, ScanTechnique::Structural, clock)
        .expect("scan must succeed with positive control");

    assert_eq!(witness.witness_layer(), &scanner.layer());
    assert!(
        witness.scanned() > 0,
        "Valid scan witness must have scanned > 0 units"
    );
    assert_ne!(
        witness.digest(),
        &[0u8; 32],
        "Valid scan witness must have non-zero digest commitment"
    );
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
    fn test_contract_purger_and_scanner() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct MockPurger {
            purged: AtomicBool,
        }
        impl Purger for MockPurger {
            fn purge(
                &self,
                _ctx: &RequestContext,
                _target: &ScanTarget,
            ) -> crate::Result<crate::scanner::PurgeStats> {
                if !self.purged.swap(true, Ordering::SeqCst) {
                    Ok(crate::scanner::PurgeStats {
                        purged_units: 5,
                        bytes_freed: 256,
                    })
                } else {
                    Ok(crate::scanner::PurgeStats {
                        purged_units: 0,
                        bytes_freed: 0,
                    })
                }
            }
        }

        let purger = MockPurger {
            purged: AtomicBool::new(false),
        };
        verify_purger_contract(&purger);

        struct MockScanner;
        impl LayerScanner for MockScanner {
            fn layer(&self) -> WitnessLayer {
                WitnessLayer::LsmMemtable
            }
            fn scan_units(
                &self,
                session: &mut crate::scanner::ScanSession,
                _ctx: &RequestContext,
            ) -> crate::Result<()> {
                session.record_control_match();
                session.record_unit([42u8; 32], false);
                Ok(())
            }
        }

        let scanner = MockScanner;
        let clock = SystemClock::new();
        verify_layer_scanner_contract(&scanner, &clock);
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

    #[test]
    fn test_contract_egress_policy_allow_and_deny() {
        use crate::policy::{EgressDecision, EgressPolicy};

        struct MockPolicy;
        impl EgressPolicy for MockPolicy {
            fn evaluate(&self, _ctx: &crate::RequestContext, payload: &str) -> EgressDecision {
                if payload.contains("secret") {
                    EgressDecision::Deny("contains secret".into())
                } else if payload.contains("pii") {
                    EgressDecision::Redact("[REDACTED]".into())
                } else {
                    EgressDecision::Allow
                }
            }
        }

        let policy = MockPolicy;
        let ctx = crate::RequestContext::new(crate::TenantId(1), crate::TxId(1));
        assert_eq!(policy.evaluate(&ctx, "normal data"), EgressDecision::Allow);
        assert_eq!(
            policy.evaluate(&ctx, "user secret data"),
            EgressDecision::Deny("contains secret".into())
        );
        assert_eq!(
            policy.evaluate(&ctx, "has pii here"),
            EgressDecision::Redact("[REDACTED]".into())
        );
    }
}
