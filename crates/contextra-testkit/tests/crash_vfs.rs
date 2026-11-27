//! Selbsttests für `CrashVfs` (Spezifikation 13 §4, DoD §13).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_ports::{Vfs, VfsOpenOptions};
use contextra_testkit::crash_vfs::*;
use contextra_types::ContextraError;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

fn p(s: &str) -> &Path {
    Path::new(s)
}

fn rw() -> VfsOpenOptions {
    VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        ..Default::default()
    }
}

fn fresh(seed: u64) -> CrashVfs {
    let v = CrashVfs::new(CrashSeed(seed));
    v.create_dir_all(p("/d")).unwrap();
    v.sync_dir(p("/")).unwrap();
    v
}

fn kind<T>(r: contextra_types::Result<T>) -> ErrorKind {
    match r {
        Err(ContextraError::Io(e)) => e.kind(),
        Err(other) => panic!("unexpected error {other}"),
        Ok(_) => panic!("expected an error"),
    }
}

#[test]
fn durable_and_volatile_are_separate() {
    let v = fresh(1);
    let f = v.open(p("/d/f"), &rw()).unwrap();
    f.write_at(b"synced", 0).unwrap();
    f.sync_data().unwrap();
    v.sync_dir(p("/d")).unwrap();
    f.write_at(b"-lost", 6).unwrap();

    assert_eq!(v.read(p("/d/f")).unwrap(), b"synced-lost");
    let strict = v.restart(CrashPolicy::DropUnsynced);
    assert_eq!(strict.read(p("/d/f")).unwrap(), b"synced");
    let kill = v.restart(CrashPolicy::KeepAll);
    assert_eq!(kill.read(p("/d/f")).unwrap(), b"synced-lost");
}

#[test]
fn file_sync_without_dir_sync_loses_the_name() {
    let v = fresh(1);
    let f = v.open(p("/d/f"), &rw()).unwrap();
    f.write_at(b"x", 0).unwrap();
    f.sync_all().unwrap();
    let r = v.restart(CrashPolicy::DropUnsynced);
    assert!(!r.exists(p("/d/f")).unwrap());
}

#[test]
fn unsynced_parent_directory_loses_children() {
    let v = CrashVfs::new(CrashSeed(1));
    v.create_dir_all(p("/a/b")).unwrap();
    v.write_atomic(p("/a/b/f"), b"x").unwrap();
    let r = v.restart(CrashPolicy::DropUnsynced);
    assert!(!r.exists(p("/a/b/f")).unwrap());
}

#[test]
fn rename_without_sync_dir_is_lost_or_kept_by_policy() {
    let v = fresh(1);
    v.write_atomic(p("/d/old"), b"1").unwrap();
    v.rename(p("/d/old"), p("/d/new")).unwrap();

    let lost = v.restart(CrashPolicy::DropUnsynced);
    assert!(lost.exists(p("/d/old")).unwrap() && !lost.exists(p("/d/new")).unwrap());
    let kept = v.restart(CrashPolicy::KeepAll);
    assert!(!kept.exists(p("/d/old")).unwrap() && kept.exists(p("/d/new")).unwrap());

    let outcomes: std::collections::BTreeSet<bool> = (0..32)
        .map(|seed| {
            let v = fresh(seed);
            v.write_atomic(p("/d/old"), b"1").unwrap();
            v.rename(p("/d/old"), p("/d/new")).unwrap();
            let r = v.restart(CrashPolicy::PartialNamespace);
            let new = r.exists(p("/d/new")).unwrap();
            assert_ne!(new, r.exists(p("/d/old")).unwrap(), "rename is atomic");
            new
        })
        .collect();
    assert_eq!(outcomes.len(), 2);

    v.sync_dir(p("/d")).unwrap();
    let durable = v.restart(CrashPolicy::DropUnsynced);
    assert_eq!(durable.read(p("/d/new")).unwrap(), b"1");
    assert!(!durable.exists(p("/d/old")).unwrap());
}

#[test]
fn cross_directory_rename_needs_both_directories() {
    let v = fresh(1);
    v.create_dir_all(p("/e")).unwrap();
    v.sync_dir(p("/")).unwrap();
    v.write_atomic(p("/d/f"), b"1").unwrap();
    v.rename(p("/d/f"), p("/e/f")).unwrap();
    v.sync_dir(p("/e")).unwrap();
    let r = v.restart(CrashPolicy::DropUnsynced);
    assert!(r.exists(p("/d/f")).unwrap());
    v.sync_dir(p("/d")).unwrap();
    let r = v.restart(CrashPolicy::DropUnsynced);
    assert!(r.exists(p("/e/f")).unwrap() && !r.exists(p("/d/f")).unwrap());
}

#[test]
fn torn_write_is_a_sector_aligned_prefix() {
    for sector in [1u32, 512, 4096] {
        let mut lengths = std::collections::BTreeSet::new();
        for seed in 0..64 {
            let v = fresh(seed);
            let f = v.open(p("/d/f"), &rw()).unwrap();
            v.sync_dir(p("/d")).unwrap();
            let data: Vec<u8> = (0..10_000u32).map(|i| (i % 251) as u8 + 1).collect();
            f.write_at(&data, 0).unwrap();
            let got = v
                .restart(CrashPolicy::TornTail { sector })
                .read(p("/d/f"))
                .unwrap();
            assert_eq!(got.as_slice(), &data[..got.len()]);
            assert!(got.len() == data.len() || got.len() % sector as usize == 0);
            lengths.insert(got.len());
        }
        assert!(lengths.len() > 1, "sector {sector}: no variation");
    }
}

#[test]
fn hardlink_keeps_content_until_last_name_is_gone() {
    let v = fresh(1);
    v.write_atomic(p("/d/a"), b"payload").unwrap();
    v.link(p("/d/a"), p("/d/b")).unwrap();
    v.sync_dir(p("/d")).unwrap();

    // Ein Schreiben über einen Namen ist über den anderen sichtbar (ein Inode).
    let f = v.open(p("/d/a"), &rw()).unwrap();
    f.write_at(b"P", 0).unwrap();
    assert_eq!(v.read(p("/d/b")).unwrap(), b"Payload");

    v.remove_file(p("/d/a")).unwrap();
    v.sync_dir(p("/d")).unwrap();
    assert_eq!(
        v.snapshot_tree().find(b"ayload"),
        vec![PathBuf::from("/d/b")]
    );
    v.remove_file(p("/d/b")).unwrap();
    v.sync_dir(p("/d")).unwrap();
    assert!(v.snapshot_tree().find(b"ayload").is_empty());
}

#[test]
fn crash_after_and_before() {
    let v = CrashVfs::new(CrashSeed(1));
    v.crash_after(2);
    v.create_dir_all(p("/d")).unwrap();
    v.write_atomic(p("/d/f"), b"x").unwrap();
    assert_eq!(kind(v.read(p("/d/f"))), ErrorKind::Interrupted);
    assert_eq!(kind(v.sync_dir(p("/"))), ErrorKind::Interrupted);
    assert_eq!(v.op_count(), 2);

    let v = CrashVfs::new(CrashSeed(1));
    v.crash_before(2);
    v.create_dir_all(p("/d")).unwrap();
    assert_eq!(
        kind(v.write_atomic(p("/d/f"), b"x")),
        ErrorKind::Interrupted
    );
    assert_eq!(v.op_count(), 1);
    assert!(v.snapshot_tree().files.is_empty());
}

#[test]
fn write_atomic_is_all_or_nothing_under_every_policy() {
    let policies = [
        CrashPolicy::DropUnsynced,
        CrashPolicy::TornTail { sector: 1 },
        CrashPolicy::PartialNamespace,
        CrashPolicy::KeepAll,
    ];
    let report = for_each_crash_point(
        |v| {
            let v = v.clone();
            Scenario::new(move || {
                let e = |e: ContextraError| e.to_string();
                v.create_dir_all(p("/d")).map_err(e)?;
                v.sync_dir(p("/")).map_err(e)?;
                v.write_atomic(p("/d/f"), b"old-old").map_err(e)?;
                v.write_atomic(p("/d/f"), b"new").map_err(e)
            })
        },
        &policies,
        |r, point| match r.read(p("/d/f")) {
            Ok(c) if c == b"old-old" && point.n < 4 => Ok(()),
            Ok(c) if c == b"new" && point.n == 4 => Ok(()),
            Err(_) if point.n < 3 => Ok(()),
            other => Err(format!("unexpected state {other:?}")),
        },
    )
    .unwrap();
    assert_eq!(report.total_ops, 4);
    assert_eq!(report.runs, 5 * policies.len());
    assert!(!report.sampled);
}

/// Schreibt Daten, dann Commit-Marke. `sync` steuert die Write-Ahead-Disziplin.
fn commit_protocol(v: &CrashVfs, sync: bool) -> Scenario {
    let v = v.clone();
    Scenario::new(move || {
        let e = |e: ContextraError| e.to_string();
        let io = |e: std::io::Error| e.to_string();
        v.create_dir_all(p("/d")).map_err(e)?;
        v.sync_dir(p("/")).map_err(e)?;
        let f = v.open(p("/d/data"), &rw()).map_err(e)?;
        f.write_at(b"DATA", 0).map_err(io)?;
        if sync {
            f.sync_all().map_err(io)?;
            v.sync_dir(p("/d")).map_err(e)?;
        }
        v.write_atomic(p("/d/commit"), b"ok").map_err(e)
    })
}

fn commit_implies_data(r: &CrashVfs, _: CrashPoint) -> Result<(), String> {
    if !r.exists(p("/d/commit")).map_err(|e| e.to_string())? {
        return Ok(());
    }
    match r.read(p("/d/data")) {
        Ok(d) if d == b"DATA" => Ok(()),
        other => Err(format!("commit without data: {other:?}")),
    }
}

#[test]
fn crash_loop_passes_correct_and_fails_broken_protocol() {
    let policies = [CrashPolicy::DropUnsynced, CrashPolicy::PartialNamespace];
    for_each_crash_point(|v| commit_protocol(v, true), &policies, commit_implies_data).unwrap();
    let failure = for_each_crash_point(
        |v| commit_protocol(v, false),
        &policies,
        commit_implies_data,
    )
    .unwrap_err();
    assert!(failure.message.contains("commit without data"), "{failure}");
}

#[test]
fn crash_loop_reports_panics_and_samples_deterministically() {
    let failure = for_each_crash_point(
        |v| {
            let v = v.clone();
            Scenario::new(move || {
                if v.create_dir_all(p("/d")).is_err() || v.is_crashed() {
                    panic!("code under test panicked");
                }
                Ok(())
            })
        },
        &[CrashPolicy::KeepAll],
        |_, _| Ok(()),
    )
    .unwrap_err();
    assert!(failure.message.contains("panicked"));

    let config = CrashLoopConfig {
        seed: CrashSeed(7),
        max_points: 3,
    };
    let run = || {
        let seen = std::cell::RefCell::new(Vec::new());
        let report = for_each_crash_point_with(
            &config,
            |v| commit_protocol(v, true),
            &[CrashPolicy::KeepAll],
            |_, point| {
                seen.borrow_mut().push(point.n);
                Ok(())
            },
        )
        .unwrap();
        assert!(report.sampled && report.runs == 3);
        seen.into_inner()
    };
    assert_eq!(run(), run());
}

#[test]
fn injected_sync_and_write_faults() {
    let v = fresh(1);
    let f = v.open(p("/d/f"), &rw()).unwrap();
    v.fail_sync(p("/d"), 1, ErrKind::Io);
    f.write_at(b"a", 0).unwrap();
    f.sync_data().unwrap();
    f.write_at(b"b", 1).unwrap();
    assert!(f.sync_data().is_err());
    assert!(v.sync_dir(p("/d")).is_err());
    let tree = v.snapshot_tree();
    assert_eq!(tree.files[p("/d/f")].volatile.as_deref(), Some(&b"ab"[..]));

    v.fail_write(p("/d"), 0, ErrKind::NoSpace);
    assert_eq!(
        f.write_at(b"c", 2).unwrap_err().kind(),
        ErrorKind::StorageFull
    );
    assert_eq!(
        kind(v.write_atomic(p("/d/g"), b"x")),
        ErrorKind::StorageFull
    );
    assert!(!v.exists(p("/d/g")).unwrap());
}

#[test]
fn bit_flip_changes_exactly_one_bit_under_prefix() {
    let v = fresh(3);
    v.write_atomic(p("/d/f"), &[0u8; 64]).unwrap();
    let r = v.restart(CrashPolicy::BitFlip {
        path_prefix: "/d".into(),
    });
    let ones: u32 = r
        .read(p("/d/f"))
        .unwrap()
        .iter()
        .map(|b| b.count_ones())
        .sum();
    assert_eq!(ones, 1);
}

#[test]
fn vfs_surface_open_options_listing_and_log() {
    let v = fresh(1);
    assert_eq!(
        kind(v.open(p("/d/none"), &VfsOpenOptions::read_only()).map(drop)),
        ErrorKind::NotFound
    );
    assert_eq!(
        kind(v.write_atomic(p("/nodir/f"), b"")),
        ErrorKind::NotFound
    );
    let f = v.open(p("/d/f"), &VfsOpenOptions::write_create()).unwrap();
    assert!(f.read_at(&mut [0; 1], 0).is_err());
    f.write_at(b"hello", 0).unwrap();
    f.set_len(4).unwrap();
    assert_eq!(v.len(p("/d/f")).unwrap(), 4);
    let mut buf = [0; 8];
    assert_eq!(v.read_at(p("/d/f"), 1, &mut buf).unwrap(), 3);
    assert_eq!(v.list_dir(p("/d")).unwrap(), vec!["f".to_string()]);
    assert_eq!(v.read_dir(p("/d")).unwrap(), vec![PathBuf::from("/d/f")]);
    v.rename(p("/d"), p("/moved")).unwrap();
    assert_eq!(v.read(p("/moved/f")).unwrap(), b"hell");
    v.mark("end");
    assert_eq!(v.marks(), vec![("end".to_string(), v.op_count())]);
    let last = *v.last_ops().last().unwrap();
    assert_eq!(last.kind, OpKind::Rename);
    assert_eq!(last.path_hash, path_hash(p("/moved")));
}

#[test]
fn sync_dir_does_not_persist_its_own_name() {
    let v = CrashVfs::new(CrashSeed(1));
    v.create_dir_all(p("/d")).unwrap();
    v.sync_dir(p("/d")).unwrap();
    let r = v.restart(CrashPolicy::DropUnsynced);
    assert!(!r.exists(p("/d")).unwrap());
}

#[test]
fn older_pending_rename_never_overrides_newer_atomic_write() {
    let v = fresh(1);
    v.create_dir_all(p("/e")).unwrap();
    v.sync_dir(p("/")).unwrap();
    v.write_atomic(p("/e/x"), b"older").unwrap();
    v.rename(p("/e/x"), p("/d/f")).unwrap();
    v.write_atomic(p("/d/f"), b"newer").unwrap();
    v.sync_dir(p("/e")).unwrap();
    let r = v.restart(CrashPolicy::DropUnsynced);
    assert_eq!(r.read(p("/d/f")).unwrap(), b"newer");
    assert!(!r.exists(p("/e/x")).unwrap());
}
