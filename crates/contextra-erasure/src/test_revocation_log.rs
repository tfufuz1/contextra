#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use super::*;
use crate::types_local::{Clock, SystemClock, Vfs};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
struct TestMemVfs {
    files: Mutex<BTreeMap<PathBuf, Vec<u8>>>,
}

impl Vfs for TestMemVfs {
    fn read(&self, path: &Path) -> std::result::Result<Vec<u8>, ErasureError> {
        let files = self
            .files
            .lock()
            .map_err(|_| ErasureError::Internal("lock poisoned".to_string()))?;
        files.get(path).cloned().ok_or_else(|| {
            ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        })
    }

    fn write_atomic(&self, path: &Path, data: &[u8]) -> std::result::Result<(), ErasureError> {
        let mut files = self
            .files
            .lock()
            .map_err(|_| ErasureError::Internal("lock poisoned".to_string()))?;
        files.insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> std::result::Result<(), ErasureError> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> std::result::Result<(), ErasureError> {
        let mut files = self
            .files
            .lock()
            .map_err(|_| ErasureError::Internal("lock poisoned".to_string()))?;
        files.remove(path);
        Ok(())
    }

    fn exists(&self, path: &Path) -> std::result::Result<bool, ErasureError> {
        let files = self
            .files
            .lock()
            .map_err(|_| ErasureError::Internal("lock poisoned".to_string()))?;
        Ok(files.contains_key(path))
    }

    fn len(&self, path: &Path) -> std::result::Result<u64, ErasureError> {
        let files = self
            .files
            .lock()
            .map_err(|_| ErasureError::Internal("lock poisoned".to_string()))?;
        files.get(path).map(|v| v.len() as u64).ok_or_else(|| {
            ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found",
            ))
        })
    }

    fn list_dir(&self, _path: &Path) -> std::result::Result<Vec<String>, ErasureError> {
        Ok(Vec::new())
    }

    fn sync_dir(&self, _path: &Path) -> std::result::Result<(), ErasureError> {
        Ok(())
    }
}

#[test]
fn test_revocation_log_lifecycle() {
    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let log = RevocationLog::new_in_memory(clock, Some(signing_key), verifying_key);
    let target = RevocationTarget::Group(42);
    assert!(!log.is_revoked(&target));
    log.append(target.clone()).unwrap();
    assert!(log.is_revoked(&target));
    assert_eq!(log.len(), 1);
    assert!(log.verify_integrity().is_ok());
}

#[test]
fn test_revocation_log_persistence_roundtrip() {
    let mut rng = OsRng;
    let signing_key = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let vfs: Arc<dyn Vfs> = Arc::new(TestMemVfs::default());
    let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
    let path = PathBuf::from("/revocation.log");

    let log = RevocationLog::open_or_create(
        vfs.clone(),
        &path,
        clock.clone(),
        Some(signing_key.clone()),
        verifying_key,
    )
    .unwrap();

    log.append(RevocationTarget::Group(101)).unwrap();
    log.append(RevocationTarget::Kek("kek-1".to_string()))
        .unwrap();
    assert_eq!(log.len(), 2);

    let signing_key2 = SigningKey::generate(&mut rng);
    let verifying_key = signing_key.verifying_key();
    let loaded =
        RevocationLog::open_or_create(vfs, &path, clock, Some(signing_key2), verifying_key)
            .unwrap();

    assert_eq!(loaded.len(), 2);
    assert!(loaded.is_revoked(&RevocationTarget::Group(101)));
    assert!(loaded.is_revoked(&RevocationTarget::Kek("kek-1".to_string())));
    assert!(loaded.verify_integrity().is_ok());
}
