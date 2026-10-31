//! # Testsuite: fault_injector_tests.rs
//!
//! Zweck: Chaos- und Fehlerinjektionstests für `FaultInjector`-Hooks bei Schreib-, Sync- und Rename-Operationen.

use contextra_durable_fs::{
    atomic_replace, atomic_replace_with_injector, durable_remove_with_injector,
    scrub_and_remove_with_injector, scrub_if_safe_with_injector, sync_dir_with_injector,
    DurableHandle, FaultInjector, NoFault, ScrubPolicy,
};
use contextra_sys::FlushKind;
use std::fs::File;
use std::io::{ErrorKind, Write};
use std::path::Path;
use tempfile::tempdir;

#[derive(Default)]
struct CountingFaultInjector {
    write_count: std::sync::atomic::AtomicUsize,
    fsync_count: std::sync::atomic::AtomicUsize,
    rename_count: std::sync::atomic::AtomicUsize,
    dir_fsync_count: std::sync::atomic::AtomicUsize,
}

impl FaultInjector for CountingFaultInjector {
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        self.write_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        self.fsync_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        self.rename_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        self.dir_fsync_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

struct FailInjector {
    fail_on_write: bool,
    fail_on_fsync: bool,
    fail_on_rename: bool,
    fail_on_dir_fsync: bool,
    error_kind: ErrorKind,
}

impl FaultInjector for FailInjector {
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        if self.fail_on_write {
            Err(std::io::Error::new(
                self.error_kind,
                "injected write failure (ENOSPC)",
            ))
        } else {
            Ok(())
        }
    }

    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        if self.fail_on_fsync {
            Err(std::io::Error::new(
                self.error_kind,
                "injected fsync failure",
            ))
        } else {
            Ok(())
        }
    }

    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        if self.fail_on_rename {
            Err(std::io::Error::new(
                self.error_kind,
                "injected rename failure",
            ))
        } else {
            Ok(())
        }
    }

    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        if self.fail_on_dir_fsync {
            Err(std::io::Error::new(
                self.error_kind,
                "injected dir fsync failure",
            ))
        } else {
            Ok(())
        }
    }
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-NO-FAULT-DEFAULT]
/// Verifiziert das no-op Standardverhalten von NoFault.
#[test]
fn test_no_fault_default_behavior() -> std::io::Result<()> {
    let no_fault = NoFault;
    assert!(no_fault.on_write(None, 100).is_ok());
    assert!(no_fault.on_fsync(None).is_ok());
    assert!(no_fault.on_rename(Path::new("a"), Path::new("b")).is_ok());
    assert!(no_fault.on_dir_fsync(Path::new(".")).is_ok());
    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-COUNTING]
/// Prüft die Zählung aller I/O-Hook-Aufrufe über CountingFaultInjector.
#[test]
fn test_counting_fault_injector_tracks_operations() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("test_count.txt");
    let injector = CountingFaultInjector::default();

    atomic_replace_with_injector(&target, b"counted data", &injector)?;

    assert_eq!(
        injector
            .write_count
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        injector
            .fsync_count
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        injector
            .rename_count
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        injector
            .dir_fsync_count
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-HANDLE-POISON]
/// Prüft das Vergiften des DurableHandle bei injiziertem fsync-Fehler.
#[test]
fn test_durable_handle_fsync_failure_poisons_handle() -> std::io::Result<()> {
    let dir = tempdir()?;
    let file_path = dir.path().join("handle_test.txt");
    let file = File::create(&file_path)?;

    let injector = FailInjector {
        fail_on_write: false,
        fail_on_fsync: true,
        fail_on_rename: false,
        fail_on_dir_fsync: false,
        error_kind: ErrorKind::Other,
    };

    let mut handle = DurableHandle::new_with_path_and_injector(file, file_path, injector);
    assert!(!handle.is_poisoned());

    let flush_res = handle.flush(FlushKind::DataAndMetadata);
    assert!(flush_res.is_err());
    assert!(handle.is_poisoned());
    assert!(handle.get_ref().is_none());
    assert!(handle.get_mut().is_none());

    // Subsequent write/flush on poisoned handle returns poisoned error
    let write_res = handle.write(b"data after poison");
    assert!(write_res.is_err());
    let err_msg = write_res.as_ref().err().map(|e| e.to_string());
    assert!(err_msg
        .as_deref()
        .unwrap_or("")
        .contains("handle is poisoned"));

    let second_flush = handle.flush(FlushKind::DataAndMetadata);
    assert!(second_flush.is_err());

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-ENOSPC-WRITE]
/// Prüft die Fehlerpropagation von ENOSPC bei Schreibzugriffen während atomic_replace.
#[test]
fn test_atomic_replace_enospc_write_failure_propagation() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("enospc.txt");

    let injector = FailInjector {
        fail_on_write: true,
        fail_on_fsync: false,
        fail_on_rename: false,
        fail_on_dir_fsync: false,
        error_kind: ErrorKind::StorageFull,
    };

    let res = atomic_replace_with_injector(&target, b"payload", &injector);
    assert_eq!(res.err().map(|e| e.kind()), Some(ErrorKind::StorageFull));
    assert!(!target.exists());

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-RENAME-FAIL]
/// Prüft die Fehlerpropagation bei Fehlern während des atomaren Umbenennens.
#[test]
fn test_atomic_replace_rename_failure_propagation() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("rename_fail.txt");

    let injector = FailInjector {
        fail_on_write: false,
        fail_on_fsync: false,
        fail_on_rename: true,
        fail_on_dir_fsync: false,
        error_kind: ErrorKind::PermissionDenied,
    };

    let res = atomic_replace_with_injector(&target, b"payload", &injector);
    assert_eq!(
        res.err().map(|e| e.kind()),
        Some(ErrorKind::PermissionDenied)
    );

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-REMOVE-DIR-FSYNC]
/// Prüft die Fehlerbehandlung bei Verzeichnis-Sync-Fehlern während durable_remove.
#[test]
fn test_durable_remove_dir_fsync_failure() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("remove_fail.txt");
    atomic_replace(&target, b"to remove")?;

    let injector = FailInjector {
        fail_on_write: false,
        fail_on_fsync: false,
        fail_on_rename: false,
        fail_on_dir_fsync: true,
        error_kind: ErrorKind::Other,
    };

    let res = durable_remove_with_injector(&target, &injector);
    assert!(res.is_err());

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-SYNC-DIR-FAIL]
/// Prüft die Fehlerpropagation bei Verzeichnis-Sync-Fehlern in sync_dir.
#[test]
fn test_sync_dir_fsync_failure() -> std::io::Result<()> {
    let dir = tempdir()?;

    let injector = FailInjector {
        fail_on_write: false,
        fail_on_fsync: false,
        fail_on_rename: false,
        fail_on_dir_fsync: true,
        error_kind: ErrorKind::Other,
    };

    let res = sync_dir_with_injector(dir.path(), &injector);
    assert!(res.is_err());

    Ok(())
}

// ANCHOR[contextra-durable-fs:TEST-FAULT-SCRUB-INJECTOR]
/// Prüft die Fehlerpropagation bei Fehlerinjektion während scrub_and_remove.
#[test]
fn test_scrub_and_remove_injector_fault() -> std::io::Result<()> {
    let dir = tempdir()?;
    let target = dir.path().join("scrub_fail.txt");
    atomic_replace(&target, b"data to scrub")?;

    let injector = FailInjector {
        fail_on_write: true,
        fail_on_fsync: false,
        fail_on_rename: false,
        fail_on_dir_fsync: false,
        error_kind: ErrorKind::StorageFull,
    };

    let res = scrub_and_remove_with_injector(&target, &injector);
    assert!(res.is_err());

    let res_safe = scrub_if_safe_with_injector(
        &target,
        ScrubPolicy::OverwriteZeros,
        false,
        false,
        &injector,
    );
    assert!(res_safe.is_err());

    Ok(())
}
