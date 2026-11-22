//! # Vollständige Crash-Matrix für atomare Schreiboperationen (`INV-VFS-ATOMIC-1`)

#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::panic, clippy::io_other_error)]

use contextra_durable_fs::fault_injector::FaultInjector;
use contextra_durable_fs::{atomic_replace_with_injector, DurableFsError};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use tempfile::tempdir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CrashPoint {
    DuringWrite,
    DuringFsync,
    BeforeRename,
    DuringDirFsync,
}

#[derive(Default)]
struct MatrixInjector {
    fail_at: Mutex<Option<CrashPoint>>,
    hit_count: AtomicUsize,
}

impl MatrixInjector {
    fn set_fail_at(&self, point: CrashPoint) {
        if let Ok(mut g) = self.fail_at.lock() {
            *g = Some(point);
        }
    }

    fn should_fail(&self, point: CrashPoint) -> bool {
        if let Ok(g) = self.fail_at.lock() {
            *g == Some(point)
        } else {
            false
        }
    }
}

impl FaultInjector for MatrixInjector {
    fn on_write(&self, _path: Option<&Path>, _bytes: usize) -> std::io::Result<()> {
        if self.should_fail(CrashPoint::DuringWrite) {
            self.hit_count.fetch_add(1, Ordering::SeqCst);
            return Err(Error::new(ErrorKind::StorageFull, "injected ENOSPC on write"));
        }
        Ok(())
    }

    fn on_fsync(&self, _path: Option<&Path>) -> std::io::Result<()> {
        if self.should_fail(CrashPoint::DuringFsync) {
            self.hit_count.fetch_add(1, Ordering::SeqCst);
            return Err(Error::other("injected EIO on fsync"));
        }
        Ok(())
    }

    fn on_rename(&self, _from: &Path, _to: &Path) -> std::io::Result<()> {
        if self.should_fail(CrashPoint::BeforeRename) {
            self.hit_count.fetch_add(1, Ordering::SeqCst);
            return Err(Error::new(ErrorKind::PermissionDenied, "injected EPERM on rename"));
        }
        Ok(())
    }

    fn on_dir_fsync(&self, _dir: &Path) -> std::io::Result<()> {
        if self.should_fail(CrashPoint::DuringDirFsync) {
            self.hit_count.fetch_add(1, Ordering::SeqCst);
            return Err(Error::other("injected EIO on dir fsync"));
        }
        Ok(())
    }
}

#[test]
fn test_atomic_replace_exhaustive_crash_matrix() {
    let crash_points = [
        CrashPoint::DuringWrite,
        CrashPoint::DuringFsync,
        CrashPoint::BeforeRename,
        CrashPoint::DuringDirFsync,
    ];

    let old_content = b"original-rock-solid-v1";
    let new_content = b"new-replacement-content-v2";

    for point in crash_points {
        let dir = tempdir().expect("tempdir");
        let target = dir.path().join("guaranteed_atomic.txt");

        fs::write(&target, old_content).expect("init write");
        assert_eq!(fs::read(&target).expect("read initial"), old_content);

        let injector = MatrixInjector::default();
        injector.set_fail_at(point);

        let res = atomic_replace_with_injector(&target, new_content, &injector);
        assert!(res.is_err(), "Operation must fail at point {point:?}");

        let post_crash = fs::read(&target).expect("target file must exist after crash");
        let is_old = post_crash == old_content;
        let is_new = post_crash == new_content;

        assert!(
            is_old || is_new,
            "State corruption at point {point:?}: neither old nor new content present! Read: {post_crash:?}"
        );

        if point != CrashPoint::DuringDirFsync {
            assert!(
                is_old,
                "State should remain old content when failing before/during rename (point: {point:?})"
            );
        }

        let entries: Vec<PathBuf> = fs::read_dir(dir.path())
            .expect("read_dir")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        for entry in entries {
            if entry != target {
                let name = entry.file_name().and_then(|n| n.to_str()).unwrap_or("");
                assert!(
                    !name.contains(".tmp."),
                    "Leaked temporary file found: {entry:?}"
                );
            }
        }
    }
}

#[test]
fn test_journal_replay_idempotency_property() {
    use contextra_durable_fs::journal::{
        AppendOnlyJournal, JournalSyncPolicy, RecordRecoveryPolicy,
    };
    use std::sync::Arc;

    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("idempotent.journal");

    {
        let mut journal = AppendOnlyJournal::open(&path).expect("open");
        journal.append(b"item-alpha").expect("append");
        journal.append(b"item-beta").expect("append");
        journal.append(b"item-gamma").expect("append");
    }

    let mut j1 = AppendOnlyJournal::open_without_recovery(
        &path,
        JournalSyncPolicy::Manual,
        Arc::new(contextra_durable_fs::AtomicHealthState::new()),
    )
    .expect("open 1");
    let recs1 = j1
        .recover(RecordRecoveryPolicy::FailClosed)
        .expect("recover 1");

    let mut j2 = AppendOnlyJournal::open_without_recovery(
        &path,
        JournalSyncPolicy::Manual,
        Arc::new(contextra_durable_fs::AtomicHealthState::new()),
    )
    .expect("open 2");
    let recs2 = j2
        .recover(RecordRecoveryPolicy::FailClosed)
        .expect("recover 2");

    let mut j3 = AppendOnlyJournal::open_without_recovery(
        &path,
        JournalSyncPolicy::Manual,
        Arc::new(contextra_durable_fs::AtomicHealthState::new()),
    )
    .expect("open 3");
    let recs3 = j3
        .recover(RecordRecoveryPolicy::FailClosed)
        .expect("recover 3");

    assert_eq!(recs1.len(), 3);
    assert_eq!(recs1, recs2);
    assert_eq!(recs2, recs3);
}

#[test]
fn test_scrub_detects_deliberate_corruption() {
    use contextra_durable_fs::journal::{AppendOnlyJournal, RecordRecoveryPolicy};
    use std::fs::OpenOptions;
    use std::io::{Seek, SeekFrom, Write};

    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("scrub_target.journal");

    {
        let mut journal = AppendOnlyJournal::open(&path).expect("open");
        journal.append(b"payload-to-be-corrupted").expect("append");
    }

    {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .expect("open");
        file.seek(SeekFrom::Start(22)).expect("seek");
        file.write_all(&[0xEE]).expect("corrupt");
    }

    let mut journal = AppendOnlyJournal::open_without_recovery(
        &path,
        contextra_durable_fs::journal::JournalSyncPolicy::Manual,
        std::sync::Arc::new(contextra_durable_fs::AtomicHealthState::new()),
    )
    .expect("open");

    let err = journal.recover(RecordRecoveryPolicy::FailClosed);
    assert!(err.is_err());
    match err {
        Err(DurableFsError::Corruption(msg)) => {
            assert!(msg.contains("checksum mismatch"));
        }
        other => panic!("Expected Corruption error, got: {other:?}"),
    }
}
