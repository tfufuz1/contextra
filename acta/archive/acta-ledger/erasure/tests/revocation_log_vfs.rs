#![cfg(feature = "signing")]

//! # Integrationstest: `revocation_log_vfs.rs`
//!
//! Validiert Persistenz, Neustart, Fehler-Injektion und Tamper-Detection für `RevocationLog`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use acta_erasure::error::ErasureError;
use acta_erasure::revocation_log::{RevocationLog, RevocationTarget};
use acta_erasure::types_local::{Clock, SystemClock, Vfs, VfsFile, VfsOpenOptions};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::collections::BTreeMap;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MemoryVfs {
    files: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
    fail_writes: AtomicBool,
}

impl MemoryVfs {
    fn new() -> Self {
        Self::default()
    }

    fn set_fail_writes(&self, fail: bool) {
        self.fail_writes.store(fail, Ordering::SeqCst);
    }
}

impl Vfs for MemoryVfs {
    fn open(
        &self,
        _path: &Path,
        _options: &VfsOpenOptions,
    ) -> Result<Box<dyn VfsFile>, ErasureError> {
        Err(ErasureError::Io(io::Error::new(
            ErrorKind::Unsupported,
            "open not supported in memory vfs",
        )))
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>, ErasureError> {
        let files = self.files.lock().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| ErasureError::Io(io::Error::new(ErrorKind::NotFound, "file not found")))
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<(), ErasureError> {
        if self.fail_writes.load(Ordering::SeqCst) {
            return Err(ErasureError::Io(io::Error::new(
                ErrorKind::Other,
                "injected I/O failure",
            )));
        }
        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> Result<(), ErasureError> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<(), ErasureError> {
        let mut files = self.files.lock().unwrap();
        files.remove(path);
        Ok(())
    }

    fn exists(&self, path: &Path) -> Result<bool, ErasureError> {
        let files = self.files.lock().unwrap();
        Ok(files.contains_key(path))
    }

    fn len(&self, path: &Path) -> Result<u64, ErasureError> {
        let files = self.files.lock().unwrap();
        Ok(files.get(path).map(|v| v.len() as u64).unwrap_or(0))
    }

    fn sync_dir(&self, _path: &Path) -> Result<(), ErasureError> {
        Ok(())
    }

    fn list_dir(&self, _path: &Path) -> Result<Vec<String>, ErasureError> {
        Ok(Vec::new())
    }
}

fn test_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

#[test]
fn test_vfs_roundtrip_across_restarts() {
    let vfs: Arc<dyn Vfs> = Arc::new(MemoryVfs::new());
    let path = PathBuf::from("/vault/revocation.log");
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let (sk, vk) = test_keypair();

    let target_kek = RevocationTarget::Kek("kek-prod-42".into());
    let target_group = RevocationTarget::Group(888);

    // Initial session: open and append 2 entries
    {
        let log = RevocationLog::open_or_create(vfs.clone(), &path, clock.clone(), Some(sk), vk)
            .expect("open_or_create must succeed");
        assert_eq!(log.len(), 0);
        assert!(log.is_empty());

        log.append(target_kek.clone()).expect("append kek");
        log.append(target_group.clone()).expect("append group");

        assert_eq!(log.len(), 2);
        assert!(log.is_revoked(&target_kek));
        assert!(log.is_revoked(&target_group));
        assert!(log.verify_integrity().is_ok());
    }

    // Restart session: reopen without signing key
    {
        let reopened = RevocationLog::open_or_create(vfs, &path, clock, None, vk)
            .expect("open_or_create after restart must succeed");

        assert_eq!(reopened.len(), 2);
        assert!(reopened.is_revoked(&target_kek));
        assert!(reopened.is_revoked(&target_group));
        assert!(!reopened.is_revoked(&RevocationTarget::Dek("dek-active".into())));
        assert!(reopened.verify_integrity().is_ok());
    }
}

#[test]
fn test_vfs_fault_injection_write_atomic_preserves_consistency() {
    let vfs_mock = Arc::new(MemoryVfs::new());
    let vfs: Arc<dyn Vfs> = vfs_mock.clone();
    let path = PathBuf::from("/vault/revocation.log");
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let (sk, vk) = test_keypair();

    let initial_target = RevocationTarget::Group(1);
    let failing_target = RevocationTarget::Group(2);

    let log = RevocationLog::open_or_create(vfs.clone(), &path, clock.clone(), Some(sk), vk)
        .expect("open_or_create must succeed");

    // 1. Initial write succeeds
    log.append(initial_target.clone())
        .expect("initial append must succeed");
    assert_eq!(log.len(), 1);
    assert!(log.is_revoked(&initial_target));

    // 2. Inject fault for subsequent writes
    vfs_mock.set_fail_writes(true);

    // 3. Attempting append MUST fail
    let append_res = log.append(failing_target.clone());
    assert!(
        append_res.is_err(),
        "append MUST fail when write_atomic fails"
    );

    // Fail-Closed verification: In-Memory state must NOT have updated!
    assert_eq!(
        log.len(),
        1,
        "Log length in RAM must remain 1 after failed append"
    );
    assert!(
        !log.is_revoked(&failing_target),
        "failing_target must NOT be marked as revoked in RAM"
    );
    assert!(
        log.is_revoked(&initial_target),
        "initial_target must remain revoked"
    );

    // 4. Disable fault and simulate reopen to check persisted state
    vfs_mock.set_fail_writes(false);

    let reopened = RevocationLog::open_or_create(vfs, &path, clock, None, vk)
        .expect("reopen after failed append");
    assert_eq!(
        reopened.len(),
        1,
        "Persisted log must contain only the initial entry"
    );
    assert!(reopened.is_revoked(&initial_target));
    assert!(
        !reopened.is_revoked(&failing_target),
        "Persisted log must NOT contain the failed entry"
    );
    assert!(reopened.verify_integrity().is_ok());
}

#[test]
fn test_vfs_tamper_detection_and_integrity_verification() {
    let vfs: Arc<dyn Vfs> = Arc::new(MemoryVfs::new());
    let path = PathBuf::from("/vault/revocation.log");
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let (sk, vk) = test_keypair();

    let log = RevocationLog::open_or_create(vfs.clone(), &path, clock.clone(), Some(sk), vk)
        .expect("open_or_create must succeed");

    for i in 0..5 {
        log.append(RevocationTarget::Group(i))
            .expect("append group");
    }
    assert_eq!(log.len(), 5);
    assert!(log.verify_integrity().is_ok());

    // Tamper with serialized file contents in VFS
    let mut raw_bytes = vfs.read(&path).expect("read raw bytes");
    assert!(!raw_bytes.is_empty());
    // Flip a bit in the middle of the byte payload
    let mid_idx = raw_bytes.len() / 2;
    raw_bytes[mid_idx] ^= 0x40;
    vfs.write_atomic(&path, &raw_bytes).expect("write tampered");

    // Reopen MUST fail with IntegrityViolation / Corrupt / Crypto
    let open_res = RevocationLog::open_or_create(vfs.clone(), &path, clock.clone(), None, vk);
    assert!(
        matches!(
            open_res,
            Err(ErasureError::IntegrityViolation)
                | Err(ErasureError::Corrupt)
                | Err(ErasureError::Crypto(_))
        ),
        "Opening tampered log file in VFS MUST fail with Corrupt or Crypto error"
    );

    // Truncate file in VFS
    let truncated_bytes = &raw_bytes[..raw_bytes.len() / 4];
    vfs.write_atomic(&path, truncated_bytes)
        .expect("write truncated");

    let open_trunc = RevocationLog::open_or_create(vfs, &path, clock, None, vk);
    assert!(
        matches!(
            open_trunc,
            Err(ErasureError::IntegrityViolation)
                | Err(ErasureError::Corrupt)
                | Err(ErasureError::Crypto(_))
        ),
        "Opening truncated log file in VFS MUST fail fail-closed"
    );
}
