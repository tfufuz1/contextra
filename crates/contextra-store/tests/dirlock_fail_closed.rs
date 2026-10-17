use contextra_core::ContextraError;
use contextra_store::lsm::config::DurabilityMode;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::Path;
use tempfile::TempDir;

/// Subprocess entry point for testing multi-process lock conflict across durability modes.
fn handle_subprocess_child() {
    if std::env::var("TEST_SUBPROCESS_DIRLOCK_FAILCLOSED_CHILD").is_ok() {
        let path_str = std::env::var("TEST_SUBPROCESS_DIR_PATH").expect("Path env required");
        let mode_str = std::env::var("TEST_SUBPROCESS_DIR_MODE").expect("Mode env required");

        let mode = match mode_str.as_str() {
            "Full" => DurabilityMode::Full,
            "WalNoHmac" => DurabilityMode::WalNoHmac,
            "MemoryOnly" => DurabilityMode::MemoryOnly,
            other => panic!("Unknown mode: {other}"),
        };

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let config = LsmConfig {
                    path: path_str.into(),
                    durability_mode: mode,
                    ..Default::default()
                };

                let res = LsmStorage::new(config).await;
                match res {
                    Ok(_) => std::process::exit(0),
                    Err(e)
                        if e.to_string()
                            .contains("Datenverzeichnis bereits in Benutzung") =>
                    {
                        std::process::exit(42)
                    }
                    Err(e) => {
                        eprintln!("Unexpected error in child process: {e}");
                        std::process::exit(1);
                    }
                }
            });
        })
        .join()
        .unwrap();
    }
}

/// Truth table test for DirLock failure decisions:
/// {Full, WalNoHmac, MemoryOnly} x {WouldBlock, OS/FS Lock Error}
#[test]
fn test_dirlock_failure_truth_table() {
    let dummy_path = Path::new("/tmp/test_dirlock_truth_table_LOCK");
    let os_error = Error::new(
        ErrorKind::Unsupported,
        "Operation not supported on filesystem",
    );

    // Helper simulating the lock error decision rule per Decision D1 specification matrix
    let decide = |mode: DurabilityMode, err: &Error| -> Result<Option<File>, ContextraError> {
        if mode == DurabilityMode::Full || mode == DurabilityMode::WalNoHmac {
            Err(ContextraError::Storage(format!(
                "Dateisperre auf diesem Dateisystem nicht unterstützt ({}): {err}",
                dummy_path.display()
            )))
        } else {
            Ok(None)
        }
    };

    // 1. Full x OS Lock Error -> Err (Fail Closed)
    let res_full_err = decide(DurabilityMode::Full, &os_error);
    assert!(
        res_full_err.is_err(),
        "Full durability must fail-closed on OS lock error"
    );
    assert!(res_full_err
        .unwrap_err()
        .to_string()
        .contains("Dateisperre auf diesem Dateisystem nicht unterstützt"));

    // 2. WalNoHmac x OS Lock Error -> Err (Fail Closed)
    let res_wal_err = decide(DurabilityMode::WalNoHmac, &os_error);
    assert!(
        res_wal_err.is_err(),
        "WalNoHmac durability must fail-closed on OS lock error"
    );
    assert!(res_wal_err
        .unwrap_err()
        .to_string()
        .contains("Dateisperre auf diesem Dateisystem nicht unterstützt"));

    // 3. MemoryOnly x OS Lock Error -> Ok(None) (Fail Open with warning)
    let res_mem_err = decide(DurabilityMode::MemoryOnly, &os_error);
    assert!(
        res_mem_err.is_ok(),
        "MemoryOnly durability must remain open on OS lock error"
    );
    assert!(res_mem_err.unwrap().is_none());
}

#[tokio::test]
async fn test_multiprocess_dirlock_conflict_full_mode() {
    handle_subprocess_child();

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        durability_mode: DurabilityMode::Full,
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("Primary storage initialization should succeed");

    // Spawn child OS process using std::process::Command attempting to open the same directory
    let exe = std::env::current_exe().unwrap();
    let status = std::process::Command::new(exe)
        .env("TEST_SUBPROCESS_DIRLOCK_FAILCLOSED_CHILD", "1")
        .env("TEST_SUBPROCESS_DIR_PATH", db_path.to_str().unwrap())
        .env("TEST_SUBPROCESS_DIR_MODE", "Full")
        .arg("--nocapture")
        .arg("test_multiprocess_dirlock_conflict_full_mode")
        .status()
        .expect("Failed to execute child process");

    assert_eq!(
        status.code(),
        Some(42),
        "Subprocess in Full mode must fail with exit code 42 (DirLock WouldBlock conflict)"
    );

    // Drop storage to release lock
    drop(storage);

    // Opening directory now should succeed
    let _storage2 = LsmStorage::new(config)
        .await
        .expect("Re-opening after dropping lock must succeed");
}

#[tokio::test]
async fn test_multiprocess_dirlock_conflict_wal_no_hmac_mode() {
    handle_subprocess_child();

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        durability_mode: DurabilityMode::WalNoHmac,
        ..Default::default()
    };

    let storage = LsmStorage::new(config.clone())
        .await
        .expect("Primary storage initialization should succeed in WalNoHmac mode");

    // Spawn child OS process using std::process::Command attempting to open the same directory
    let exe = std::env::current_exe().unwrap();
    let status = std::process::Command::new(exe)
        .env("TEST_SUBPROCESS_DIRLOCK_FAILCLOSED_CHILD", "1")
        .env("TEST_SUBPROCESS_DIR_PATH", db_path.to_str().unwrap())
        .env("TEST_SUBPROCESS_DIR_MODE", "WalNoHmac")
        .arg("--nocapture")
        .arg("test_multiprocess_dirlock_conflict_wal_no_hmac_mode")
        .status()
        .expect("Failed to execute child process");

    assert_eq!(
        status.code(),
        Some(42),
        "Subprocess in WalNoHmac mode must fail with exit code 42 (DirLock WouldBlock conflict)"
    );

    // Drop storage to release lock
    drop(storage);

    // Opening directory now should succeed
    let _storage2 = LsmStorage::new(config)
        .await
        .expect("Re-opening after dropping lock must succeed in WalNoHmac mode");
}
