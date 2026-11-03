//! # Crash-Matrix- und Durability-Tests für atomaren Austausch (DF-02, DF-08, T-CRASH-2, T-CRASH-6)
//!
//! Prüft die Crash-Sicherheit bei Fehlerinjektion an jedem einzelnen Schritt:
//! `write`, `sync_data`, `rename`, `sync_dir`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods
)]

#[path = "../src/atomic.rs"]
mod atomic;

use atomic::{clean_tmp, write_atomic, write_atomic_with_injector, AtomicFaultInjector};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

/// Test-Injektor zur gezielten Fehlererzeugung an einzelnen Teilschritten.
#[derive(Debug, Default)]
struct StepFaultInjector {
    fail_at_write: bool,
    fail_at_sync_data: bool,
    fail_at_rename: bool,
    fail_at_sync_dir: bool,
    fail_at_unlink: bool,
    op_counter: AtomicUsize,
}

impl AtomicFaultInjector for StepFaultInjector {
    fn on_write(&self, _path: &Path, _bytes: usize) -> std::io::Result<()> {
        self.op_counter.fetch_add(1, Ordering::SeqCst);
        if self.fail_at_write {
            return Err(std::io::Error::other("simulated crash at write"));
        }
        Ok(())
    }

    fn on_sync_data(&self, _path: &Path) -> std::io::Result<()> {
        self.op_counter.fetch_add(1, Ordering::SeqCst);
        if self.fail_at_sync_data {
            return Err(std::io::Error::other("simulated crash at sync_data"));
        }
        Ok(())
    }

    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        self.op_counter.fetch_add(1, Ordering::SeqCst);
        if self.fail_at_rename {
            return Err(std::io::Error::other("simulated crash at rename"));
        }
        Ok(())
    }

    fn on_sync_dir(&self, _dir: &Path) -> std::io::Result<()> {
        self.op_counter.fetch_add(1, Ordering::SeqCst);
        if self.fail_at_sync_dir {
            return Err(std::io::Error::other("simulated crash at sync_dir"));
        }
        Ok(())
    }

    fn on_unlink(&self, _path: &Path) -> std::io::Result<()> {
        self.op_counter.fetch_add(1, Ordering::SeqCst);
        if self.fail_at_unlink {
            return Err(std::io::Error::other("simulated crash at unlink"));
        }
        Ok(())
    }
}

#[test]
fn test_df_02_atomic_replace_crash_matrix() {
    let old_data = b"VERSION_1_ORIGINAL_CONTENT_CANARY_VALUE_0123456789";
    let new_data = b"VERSION_2_UPDATED_CONTENT_CANARY_VALUE_9876543210";

    // Matrix aller Injektionspunkte:
    // - Fall 0: Normaler Durchlauf (kein Fehler)
    // - Fall 1: Fehler beim Schreiben
    // - Fall 2: Fehler bei sync_data
    // - Fall 3: Fehler beim Rename
    // - Fall 4: Fehler beim sync_dir
    for phase in 0..=4 {
        let temp = TempDir::new().expect("TempDir failed");
        let target = temp.path().join("target.dat");

        // NOTE: Ausgangszustand herstellen: V1 anlegen
        write_atomic(&target, old_data).expect("Initial write failed");
        assert_eq!(fs::read(&target).expect("Read failed"), old_data);

        let injector = StepFaultInjector {
            fail_at_write: phase == 1,
            fail_at_sync_data: phase == 2,
            fail_at_rename: phase == 3,
            fail_at_sync_dir: phase == 4,
            fail_at_unlink: false,
            op_counter: AtomicUsize::new(0),
        };

        let result = write_atomic_with_injector(&target, new_data, &injector);

        if phase == 0 {
            assert!(result.is_ok(), "Baseline write must succeed");
            assert_eq!(
                fs::read(&target).expect("Read failed"),
                new_data,
                "Target must contain new data"
            );
        } else {
            assert!(
                result.is_err(),
                "Injected phase {phase} must report error unchanged"
            );

            // NOTE: Nach Absturz/Fehler (T-CRASH-2) ist die Datei ENTWEDER alt ODER neu, NIE halb!
            let current = fs::read(&target).expect("Target file must still exist and be readable");
            let is_old = current == old_data;
            let is_new = current == new_data;
            assert!(
                is_old || is_new,
                "Phase {phase}: Target must be either V1 or V2, but was partially written / corrupted"
            );

            // Bei Fehlern vor dem Rename muss die Zieldatei zwingend V1 sein
            if phase <= 2 {
                assert!(
                    is_old,
                    "Phase {phase}: Prior to rename, target must strictly remain V1"
                );
            }
        }

        // Bereinigung verwaister temporärer Dateien
        let cleaned = clean_tmp(temp.path()).expect("clean_tmp failed");
        let _ = cleaned;

        // Keine verwaisten Zwischendateien im Verzeichnis
        for entry in fs::read_dir(temp.path()).expect("read_dir failed") {
            let entry = entry.expect("entry failed");
            let name = entry.file_name().to_string_lossy().to_string();
            assert!(
                !name.contains(".tmp."),
                "Orphan temp file '{name}' remaining after clean_tmp!"
            );
        }
    }
}

#[test]
fn test_df_08_t_crash_6_clean_tmp_orphans() {
    let temp = TempDir::new().expect("TempDir failed");
    let base = temp.path();

    // 1. Erstelle eine reguläre Datei
    let target = base.join("stable_file.bin");
    write_atomic(&target, b"STABLE_PAYLOAD").expect("write_atomic failed");

    // 2. Erzeuge verwaiste temporäre Dateien
    let orphan1 = base.join(".target.tmp.9999.1");
    let orphan2 = base.join(".other.tmp.8888.2");
    let orphan3 = base.join("tmp.orphan.old");
    fs::write(&orphan1, b"PARTIAL_DATA_1").expect("Write orphan1 failed");
    fs::write(&orphan2, b"PARTIAL_DATA_2").expect("Write orphan2 failed");
    fs::write(&orphan3, b"PARTIAL_DATA_3").expect("Write orphan3 failed");

    // 3. Erzeuge Unterordner tmp/ mit Waisen
    let tmp_sub = base.join("tmp");
    fs::create_dir_all(&tmp_sub).expect("create_dir_all failed");
    let sub_orphan = tmp_sub.join("abandoned_chunk.dat");
    fs::write(&sub_orphan, b"ABANDONED").expect("Write sub_orphan failed");

    // 4. clean_tmp aufrufen
    let removed = clean_tmp(base).expect("clean_tmp failed");
    assert_eq!(removed, 4, "Expected 4 orphan files to be cleaned");

    // 5. Reguläre Datei muss unversehrt sein
    assert_eq!(
        fs::read(&target).expect("Read stable file failed"),
        b"STABLE_PAYLOAD"
    );

    // 6. Waisen müssen verschwunden sein
    assert!(!orphan1.exists());
    assert!(!orphan2.exists());
    assert!(!orphan3.exists());
    assert!(!sub_orphan.exists());
}

#[test]
fn test_df_08_file_and_directory_permissions() {
    let temp = TempDir::new().expect("TempDir failed");
    let sub_dir = temp.path().join("secure_sub");
    let target = sub_dir.join("payload.dat");

    write_atomic(&target, b"CONFIDENTIAL_BYTES").expect("write_atomic failed");

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let dir_meta = fs::metadata(&sub_dir).expect("Directory metadata failed");
        let dir_mode = dir_meta.mode() & 0o777;
        assert_eq!(
            dir_mode, 0o700,
            "Directory must be created with mode 0700 (DF-08)"
        );

        let file_meta = fs::metadata(&target).expect("File metadata failed");
        let file_mode = file_meta.mode() & 0o777;
        assert_eq!(
            file_mode, 0o600,
            "File must be created with mode 0600 (DF-08)"
        );
    }
}
