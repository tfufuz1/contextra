use contextra_crypto::revocation_log::{
    test_set_revocation_log_dir_sync_hook, RevocationLog,
};
use contextra_ports::Clock;
use contextra_types::DeterministicTimestamp;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

static DIR_SYNC_COUNT: AtomicUsize = AtomicUsize::new(0);

fn mock_dir_sync_hook_ok(_path: &Path) -> std::io::Result<()> {
    DIR_SYNC_COUNT.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

fn mock_dir_sync_hook_err(_path: &Path) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        "simulated revocation log dir fsync failure",
    ))
}

struct DummyClock;
impl Clock for DummyClock {
    fn now(&self) -> DeterministicTimestamp {
        DeterministicTimestamp::new(1000, 0)
    }
    fn monotonic_ticks(&self) -> u64 {
        1000
    }
}

#[test]
fn test_revocation_log_create_invokes_dir_fsync() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = tmp_dir.path().join("revocation.log");

    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let clock = Arc::new(DummyClock);

    DIR_SYNC_COUNT.store(0, Ordering::SeqCst);
    test_set_revocation_log_dir_sync_hook(Some(mock_dir_sync_hook_ok));

    let log_res = RevocationLog::create_new(&log_path, clock, Some(sk), vk);
    test_set_revocation_log_dir_sync_hook(None);

    if let Err(ref e) = log_res {
        eprintln!("create_new error: {e:?}");
    }

    assert!(log_res.is_ok(), "RevocationLog::create_new should succeed");
    assert!(
        DIR_SYNC_COUNT.load(Ordering::SeqCst) > 0,
        "RevocationLog::create_new MUST invoke parent directory fsync"
    );
}

#[test]
fn test_revocation_log_create_propagates_dir_fsync_error() {
    let tmp_dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = tmp_dir.path().join("revocation.log");

    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    let clock = Arc::new(DummyClock);

    test_set_revocation_log_dir_sync_hook(Some(mock_dir_sync_hook_err));

    let log_res = RevocationLog::create_new(&log_path, clock, Some(sk), vk);
    test_set_revocation_log_dir_sync_hook(None);

    if let Err(ref e) = log_res {
        eprintln!("create_new_err error: {e:?}");
    }

    assert!(
        log_res.is_err(),
        "RevocationLog::create_new MUST return Err when directory fsync fails"
    );
    assert!(
        log_res
            .unwrap_err()
            .to_string()
            .contains("simulated revocation log dir fsync failure"),
        "Error message must convey directory fsync failure"
    );
}
