// FILE-CONTEXT
// ZWECK: VFS-basierte Persistenz-, Neustart- und Crash-Konsistenz-Tests für RevocationLog.
// INVARIANTEN: Atomarität bei write_atomic: Im Fehlerfall bleibt der Speicherzustand unverändert.
// Tamper-Detection beim Öffnen via Vfs; Integritätsprüfung unverändert.

#![forbid(unsafe_code)]

use contextra_crypto::{
    error::CryptoError,
    revocation_log::{RevocationLog, RevocationTarget},
};
use contextra_ports::{Clock, SystemClock, Vfs};
use contextra_testkit::{FaultConfig, FaultVfs};
use contextra_types::ContextraError;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct FaultInjectingVfs {
    inner: FaultVfs,
}

impl FaultInjectingVfs {
    fn new() -> Self {
        Self {
            inner: FaultVfs::new(),
        }
    }

    fn set_config(&self, config: FaultConfig) {
        self.inner.set_config(config);
    }
}

impl Vfs for FaultInjectingVfs {
    fn read(&self, path: &Path) -> contextra_ports::Result<Vec<u8>> {
        self.inner.read_file(path).map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                ContextraError::NotFound(path.display().to_string())
            } else {
                ContextraError::Io(e)
            }
        })
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> contextra_ports::Result<()> {
        self.inner
            .write_file(path, data)
            .map_err(ContextraError::Io)
    }

    fn create_dir_all(&self, _path: &Path) -> contextra_ports::Result<()> {
        Ok(())
    }

    fn remove_file(&self, _path: &Path) -> contextra_ports::Result<()> {
        Ok(())
    }

    fn exists(&self, path: &Path) -> contextra_ports::Result<bool> {
        match self.inner.read_file(path) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
            Err(e) => Err(ContextraError::Io(e)),
        }
    }

    fn len(&self, path: &Path) -> contextra_ports::Result<u64> {
        let bytes = self.read(path)?;
        Ok(bytes.len() as u64)
    }

    fn list_dir(&self, _path: &Path) -> contextra_ports::Result<Vec<String>> {
        Ok(Vec::new())
    }

    fn sync_dir(&self, _path: &Path) -> contextra_ports::Result<()> {
        self.inner.sync().map_err(ContextraError::Io)
    }
}

fn test_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

#[test]
fn test_vfs_roundtrip_across_restarts() {
    let vfs: Arc<dyn Vfs> = Arc::new(FaultInjectingVfs::new());
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
    let vfs_mock = Arc::new(FaultInjectingVfs::new());
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
    vfs_mock.set_config(FaultConfig {
        fail_writes_after: Some(0), // any new write fails
        ..Default::default()
    });

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
    vfs_mock.set_config(FaultConfig::default());

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
    let vfs: Arc<dyn Vfs> = Arc::new(FaultInjectingVfs::new());
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

    // Reopen MUST fail with IntegrityViolation
    let open_res = RevocationLog::open_or_create(vfs.clone(), &path, clock.clone(), None, vk);
    assert!(
        matches!(open_res, Err(CryptoError::IntegrityViolation)),
        "Opening tampered log file in VFS MUST fail with CryptoError::IntegrityViolation"
    );

    // Truncate file in VFS
    let truncated_bytes = &raw_bytes[..raw_bytes.len() / 4];
    vfs.write_atomic(&path, truncated_bytes)
        .expect("write truncated");
    let trunc_res = RevocationLog::open_or_create(vfs, &path, clock, None, vk);
    assert!(
        trunc_res.is_err(),
        "Opening truncated log file in VFS MUST fail with error"
    );
}
