//! # Integrationstests für FaultVfs und Vfs-Conformance-Suite
//!
//! Prüft die Anforderungen aus SPEC 02-contextra-ports.md (PO-12, T-FSYNC-1)
//! und 05-contextra-durable-fs.md (DF-02, DF-03).

#![allow(clippy::expect_used, clippy::indexing_slicing)]

use contextra_ports::{Vfs, VfsFile, VfsOpenOptions};
use contextra_testkit::conformance::vfs::run_vfs_suite;
use contextra_testkit::fault_vfs::{FaultOperation, FaultPlan, FaultVfs};
use std::path::Path;

/// PO-12: Generische Conformance-Suite läuft erfolgreich gegen FaultVfs (Selbsttest)
#[test]
fn po_12_vfs_conformance_suite_green() {
    let res = run_vfs_suite(FaultVfs::new);
    assert!(res.is_ok(), "FaultVfs must pass Vfs conformance: {res:?}");
}

/// T-FSYNC-1: Ein fsync-Fehler (sync_data/sync_all) verbraucht das Handle dauerhaft (DF-03)
#[test]
fn t_fsync_1_handle_consumed_on_sync_error() {
    let vfs = FaultVfs::new();
    let file_path = Path::new("sync_error_test.dat");

    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        ..Default::default()
    };
    let file = vfs.open(file_path, &opts).expect("open should succeed");

    // Fehlerinjektion für nächsten sync konfigurieren
    vfs.set_plan(FaultPlan::exact(
        FaultOperation::Sync,
        0,
        std::io::ErrorKind::Other,
    ));

    // Erster sync muss fehlschlagen
    let sync_res = file.sync_all();
    assert!(
        sync_res.is_err(),
        "sync_all should fail due to injected error"
    );

    // Nach dem fsync-Fehler muss das Handle verbraucht sein: Folgeoperationen liefern Fehler (DF-03)
    let write_res = file.write_at(b"extra", 0);
    assert!(
        write_res.is_err(),
        "Handle must be poisoned/consumed after sync failure"
    );

    let sync_again = file.sync_all();
    assert!(
        sync_again.is_err(),
        "Handle cannot retry sync after failure (T-FSYNC-1)"
    );
}

/// DF-02: Crash-Matrix für atomaren Austausch / Abläufe
#[test]
fn df_02_crash_matrix_atomic_replace() {
    let outcomes = FaultVfs::for_each_crash_point(|vfs, crash_idx| {
        let path = Path::new("atomic_target.bin");
        let res = vfs.write_atomic(path, b"crash_matrix_payload");
        (crash_idx, res.is_ok())
    });

    assert!(
        !outcomes.is_empty(),
        "Crash matrix outcomes must not be empty"
    );
    let first = outcomes.first();
    assert!(
        matches!(first, Some((0, (_, false)))),
        "Operation at point 0 must fail"
    );
    let last = outcomes.last();
    assert!(
        matches!(last, Some((_, (_, true)))),
        "Operation at last point without interruption must succeed"
    );
}

/// PO-12: Mutationstest: Eine mutierte Vfs, die den Locking-Vertrag bricht (kein Konflikt bei create_new),
/// muss von der Conformance-Suite zuverlässig als rot erkannt werden.
#[test]
fn po_12_vfs_conformance_mutant_red() {
    // Mutant: Verhält sich wie FaultVfs, aber ignoriert create_new und erlaubt Mehrfach-Locks
    #[derive(Clone, Default)]
    struct NonExclusiveLockVfs {
        inner: FaultVfs,
    }

    struct NonExclusiveFile {
        inner: Box<dyn VfsFile>,
    }

    impl VfsFile for NonExclusiveFile {
        fn read_at(&self, buf: &mut [u8], offset: u64) -> std::io::Result<usize> {
            self.inner.read_at(buf, offset)
        }
        fn write_at(&self, buf: &[u8], offset: u64) -> std::io::Result<usize> {
            self.inner.write_at(buf, offset)
        }
        fn set_len(&self, len: u64) -> std::io::Result<()> {
            self.inner.set_len(len)
        }
        fn len(&self) -> std::io::Result<u64> {
            self.inner.len()
        }
        fn sync_data(&self) -> std::io::Result<()> {
            self.inner.sync_data()
        }
        fn sync_all(&self) -> std::io::Result<()> {
            self.inner.sync_all()
        }
    }

    impl Vfs for NonExclusiveLockVfs {
        fn open(
            &self,
            path: &Path,
            opts: &VfsOpenOptions,
        ) -> contextra_types::Result<Box<dyn VfsFile>> {
            // BUG-MUTANT: create_new wird auf create herabgestuft, verhindert Konflikt
            let mut modified_opts = opts.clone();
            if modified_opts.create_new {
                modified_opts.create_new = false;
                modified_opts.create = true;
            }
            let file = self.inner.open(path, &modified_opts)?;
            Ok(Box::new(NonExclusiveFile { inner: file }))
        }
        fn read(&self, path: &Path) -> contextra_types::Result<Vec<u8>> {
            self.inner.read(path)
        }
        fn write_atomic(&self, path: &Path, data: &[u8]) -> contextra_types::Result<()> {
            self.inner.write_atomic(path, data)
        }
        fn create_dir_all(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.create_dir_all(path)
        }
        fn remove_file(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.remove_file(path)
        }
        fn rename(&self, from: &Path, to: &Path) -> contextra_types::Result<()> {
            self.inner.rename(from, to)
        }
        fn exists(&self, path: &Path) -> contextra_types::Result<bool> {
            self.inner.exists(path)
        }
        fn len(&self, path: &Path) -> contextra_types::Result<u64> {
            self.inner.len(path)
        }
        fn list_dir(&self, path: &Path) -> contextra_types::Result<Vec<String>> {
            self.inner.list_dir(path)
        }
        fn sync_dir(&self, path: &Path) -> contextra_types::Result<()> {
            self.inner.sync_dir(path)
        }
    }

    let res = run_vfs_suite(NonExclusiveLockVfs::default);
    assert!(
        res.is_err(),
        "Mutant Vfs with broken lock contract must fail conformance suite"
    );
}
