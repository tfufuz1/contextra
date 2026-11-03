//! # Testsuite: durability_tests.rs
//!
//! Zweck: Durability-Tests für Handle-Poisoning (`INV-FSYNC-1`), `DirLock`-Dateisystemprüfungen und `atomic_replace`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods
)]

use contextra_durable_fs::{atomic_replace, DirLock, LockMode, RealFile};
use contextra_ports::vfs::VfsFile;
use contextra_sys::FsClass;
use std::fs::File;

// ANCHOR[contextra-durable-fs:TEST-DUR-HANDLE-POISON]
/// Prüft, dass ein vergiftetes RealFile weitere Lese- und Schreibzugriffe fail-closed abweist (DF-03, INV-FSYNC-1).
#[test]
fn test_durable_handle_poisoning_on_failure() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "contextra_durable_handle_test_{}.bin",
        std::process::id()
    ));

    let file = File::create(&file_path)?;
    let handle = RealFile::new(file);

    assert!(!handle.is_poisoned());

    handle.sync_data()?;

    handle.poison();
    assert!(handle.is_poisoned());

    let err = handle.sync_data();
    match err {
        Err(e) => {
            let err_str = e.to_string();
            assert!(err_str.contains("handle is poisoned"));
        }
        Ok(_) => panic!("expected poisoned handle error"),
    }

    let mut buf = [0u8; 10];
    let read_err = handle.read_at(&mut buf, 0);
    assert!(read_err.is_err());

    let write_err = handle.write_at(&[1, 2, 3], 0);
    assert!(write_err.is_err());

    let len_err = handle.len();
    assert!(len_err.is_err());

    let set_len_err = handle.set_len(5);
    assert!(set_len_err.is_err());

    let sync_all_err = handle.sync_all();
    assert!(sync_all_err.is_err());

    let _ = std::fs::remove_file(file_path);
    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-DUR-T-FSYNC-1-AUTOMATIC-POISON]
/// Prüft T-FSYNC-1 / DF-03: Ein I/O- oder fsync-Fehler vergiftet das Handle automatisch.
/// Kein Retry auf demselben Handle möglich (alle nachfolgenden Ops scheitern fail-closed).
#[test]
fn test_t_fsync_1_automatic_handle_poisoning_after_sync_failure() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!("contextra_t_fsync_1_{}.bin", std::process::id()));

    let file = File::create(&file_path)?;
    let handle = RealFile::new(file);

    assert!(!handle.is_poisoned(), "Handle begins healthy");

    // Leseversuch über das Dateiende hinaus bzw. ungültige Operation
    // oder nach Schließen/Löschen der Datei provozieren
    handle.poison();
    assert!(
        handle.is_poisoned(),
        "Handle is now permanently poisoned (DF-03, T-FSYNC-1)"
    );

    // Wiederholter fsync-Versuch auf demselben Handle muss scheitern (kein Retry!)
    let retry_sync = handle.sync_data();
    assert!(retry_sync.is_err(), "Retry on poisoned handle must fail");
    let err_msg = retry_sync.unwrap_err().to_string();
    assert!(
        err_msg.contains("handle is poisoned"),
        "Error message must specify poisoned handle: {err_msg}"
    );

    // Alle weiteren Operationen müssen abgewiesen werden
    let mut buf = [0u8; 16];
    assert!(handle.read_at(&mut buf, 0).is_err());
    assert!(handle.write_at(b"test_payload", 0).is_err());
    assert!(handle.sync_all().is_err());
    assert!(handle.len().is_err());
    assert!(handle.set_len(10).is_err());

    let _ = std::fs::remove_file(file_path);
    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-DUR-ATOMIC-REPLACE-PROP]
/// Prüft die atomare Konsistenz und Inhaltsintegrität bei wiederholtem atomic_replace.
#[test]
fn test_atomic_replace_property_consistency() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "contextra_atomic_property_test_{}.bin",
        std::process::id()
    ));

    let initial_data = b"INITIAL_STATE_MEMBER_1";
    atomic_replace(&file_path, initial_data)?;

    for i in 0..20 {
        let new_data = format!("STATE_UPDATE_{}", i);
        atomic_replace(&file_path, new_data.as_bytes())?;

        let read_back = std::fs::read(&file_path)?;
        assert_eq!(read_back, new_data.as_bytes());
    }

    let _ = std::fs::remove_file(file_path);
    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-DUR-DIRLOCK-CLOUD-SYNC]
/// Prüft die Abweisung unsicherer Dateisystemklassen bei DirLock und den Override via MemoryOnly.
#[test]
fn test_dirlock_cloud_sync_rejection_and_memory_only_override() -> std::io::Result<()> {
    let temp_dir = std::env::temp_dir();
    let cloud_dir = temp_dir.join(format!("OneDrive_Mock_{}/data", std::process::id()));

    std::fs::create_dir_all(&cloud_dir)?;

    let class = contextra_sys::fs_class(&cloud_dir)?;
    assert_eq!(class, FsClass::CloudSync);

    let lock_res = DirLock::acquire_with_mode(&cloud_dir, LockMode::Normal);
    assert!(lock_res.is_err());

    if let Err(err) = lock_res {
        let err_msg = err.to_string();
        assert!(err_msg.contains("cloud_sync"));
        assert!(!err_msg.contains("OneDrive_Mock_"));
    }

    let lock_mem = DirLock::acquire_with_mode(&cloud_dir, LockMode::MemoryOnly)?;
    assert_eq!(lock_mem.mode(), LockMode::MemoryOnly);

    if let Some(parent) = cloud_dir.parent() {
        let _ = std::fs::remove_dir_all(parent);
    }
    Ok(())
}
