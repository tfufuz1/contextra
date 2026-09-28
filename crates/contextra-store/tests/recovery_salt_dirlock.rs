use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_salt_unreadable_fails_open() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    // Create a directory named SALT inside the db path. Reading a directory with tokio::fs::read will fail with EISDIR
    let salt_path = db_path.join("SALT");
    fs::create_dir_all(&salt_path).unwrap();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let res = LsmStorage::new(config).await;
    assert!(
        res.is_err(),
        "LsmStorage::new must fail when SALT is unreadable"
    );
    let err = res.err().unwrap();
    let err_str = err.to_string();
    assert!(
        err_str.contains("SALT nicht lesbar") || err_str.contains("SALT"),
        "Expected SALT unreadable error, got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_salt_missing_non_pristine_fails_open() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    // Create MANIFEST file or WAL file without SALT
    fs::write(db_path.join("wal-0.log"), b"dummy wal").unwrap();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let res = LsmStorage::new(config).await;
    assert!(
        res.is_err(),
        "LsmStorage::new must fail when SALT is missing in non-pristine directory"
    );
    let err = res.err().unwrap();
    assert!(
        err.to_string().contains("SALT") || err.to_string().contains("non-pristine"),
        "Expected missing salt in non-pristine directory error, got: {}",
        err
    );
}

#[tokio::test]
async fn test_salt_missing_pristine_creates_salt() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let storage = LsmStorage::new(config)
        .await
        .expect("LsmStorage::new should succeed on pristine directory");
    let salt_path = db_path.join("SALT");
    assert!(salt_path.exists(), "SALT file should have been created");
    let salt_bytes = fs::read(&salt_path).unwrap();
    assert_eq!(salt_bytes.len(), 32);

    drop(storage);
}

#[tokio::test]
async fn test_dirlock_same_process_conflict() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let _storage1 = LsmStorage::new(config.clone())
        .await
        .expect("First LsmStorage::new should succeed");

    let res2 = LsmStorage::new(config).await;
    assert!(
        res2.is_err(),
        "Second LsmStorage::new on same directory must fail with DirLock conflict"
    );
    let err_str = res2.err().unwrap().to_string();
    assert!(
        err_str.contains("Datenverzeichnis bereits in Benutzung"),
        "Expected 'Datenverzeichnis bereits in Benutzung', got: {}",
        err_str
    );
}

#[tokio::test]
async fn test_dirlock_subprocess_conflict() {
    if std::env::var("TEST_SUBPROCESS_DIRLOCK_CHILD").is_ok() {
        let path_str = std::env::var("TEST_SUBPROCESS_DIR_PATH").unwrap();
        let config = LsmConfig {
            path: path_str.into(),
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
                eprintln!("Unexpected error in child process: {}", e);
                std::process::exit(1);
            }
        }
    }

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().to_path_buf();

    let config = LsmConfig {
        path: db_path.clone(),
        ..Default::default()
    };

    let _storage = LsmStorage::new(config)
        .await
        .expect("First open in main process should succeed");

    let exe = std::env::current_exe().unwrap();
    let status = std::process::Command::new(exe)
        .env("TEST_SUBPROCESS_DIRLOCK_CHILD", "1")
        .env("TEST_SUBPROCESS_DIR_PATH", db_path.to_str().unwrap())
        .arg("--nocapture")
        .arg("test_dirlock_subprocess_conflict")
        .status()
        .expect("Failed to execute child test process");

    assert_eq!(
        status.code(),
        Some(42),
        "Subprocess should fail with exit code 42 (DirLock conflict)"
    );
}
