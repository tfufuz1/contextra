//! # Integrationstests für exklusiven Verzeichnis-Lock (DF-05, T-LOCK-1)
//!
//! Prüft exklusives Sperren, Fehlertexte und Prozesskonflikte gemäß SPEC DF-05, D-50.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::duplicate_mod
)]

#[path = "../src/dir_lock.rs"]
mod dir_lock;

#[path = "support/lock_child.rs"]
mod lock_child;

use dir_lock::{DirLock, DIR_LOCKED_MSG};
use lock_child::{
    run_child_if_requested, ENV_LOCK_CHILD_MODE, ENV_LOCK_DIR_PATH, EXIT_LOCK_ACQUIRED,
    EXIT_LOCK_CONFLICT,
};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_df_05_dir_lock_exclusive_same_process() {
    run_child_if_requested();

    let temp = TempDir::new().expect("TempDir failed");
    let dir = temp.path();

    // 1. Erster Lock muss erfolgreich sein
    let lock1 = DirLock::acquire(dir).expect("First lock must succeed");
    assert!(lock1.lock_path().exists());

    // 2. Zweiter Versuch im selben Prozess muss scheitern
    let lock2_res = DirLock::acquire(dir);
    assert!(
        lock2_res.is_err(),
        "Second lock acquisition on same directory must fail"
    );
    let err = lock2_res.expect_err("Error expected");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains(DIR_LOCKED_MSG),
        "Expected error message '{DIR_LOCKED_MSG}', got: '{err_msg}'"
    );

    // 3. Nach unlock des ersten Locks muss erneuter Erwerb gelingen
    lock1.unlock();
    let lock3 = DirLock::acquire(dir).expect("Re-acquisition after unlock must succeed");
    drop(lock3);
}

#[test]
fn test_t_lock_1_dir_lock_exclusive_subprocess() {
    run_child_if_requested();

    let temp = TempDir::new().expect("TempDir failed");
    let dir = temp.path();

    // 1. Elternprozess erwirbt die Sperre
    let lock_guard = DirLock::acquire(dir).expect("Parent lock acquisition must succeed");

    // 2. Kindprozess starten: muss mit Code 42 (EXIT_LOCK_CONFLICT) abgewiesen werden
    let current_exe = std::env::current_exe().expect("current_exe failed");
    let status_conflict = Command::new(&current_exe)
        .arg("--nocapture")
        .env(ENV_LOCK_CHILD_MODE, "1")
        .env(ENV_LOCK_DIR_PATH, dir.as_os_str())
        .status()
        .expect("Failed to execute child process");

    assert_eq!(
        status_conflict.code(),
        Some(EXIT_LOCK_CONFLICT),
        "Child process must fail with conflict code 42 (T-LOCK-1)"
    );

    // 3. Elternprozess gibt die Sperre frei
    drop(lock_guard);

    // 4. Kindprozess erneut starten: muss jetzt mit 0 (EXIT_LOCK_ACQUIRED) gelingen
    let status_success = Command::new(&current_exe)
        .arg("--nocapture")
        .env(ENV_LOCK_CHILD_MODE, "1")
        .env(ENV_LOCK_DIR_PATH, dir.as_os_str())
        .status()
        .expect("Failed to execute second child process");

    assert_eq!(
        status_success.code(),
        Some(EXIT_LOCK_ACQUIRED),
        "Child process must acquire lock successfully after parent releases it"
    );
}

#[test]
fn test_df_05_dir_lock_permissions() {
    run_child_if_requested();

    let temp = TempDir::new().expect("TempDir failed");
    let dir = temp.path();

    let lock = DirLock::acquire(dir).expect("Lock acquire failed");
    let lock_file = lock.lock_path();

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::metadata(lock_file).expect("Metadata failed");
        let mode = meta.mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "LOCK file must be created with 0600 permissions (DF-08)"
        );
    }

    drop(lock);
}
