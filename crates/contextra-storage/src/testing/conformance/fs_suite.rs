//! # Dateisystem-Konformitätssuite
//!
//! Generische Konformitätstests für Implementierungen des `Fs`-Ports nach SPEC §4 und §18.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use crate::env::{write_atomic, RealFs};
use contextra_storage_ports::{Fs, LockMode, OpenMode, Perm, ReadReq};
use std::path::Path;

/// Führt die vollständige Konformitätssuite für eine `Fs`-Instanz aus.
pub fn run_fs_conformance_suite<F: Fs>(fs: &F, root: &Path) {
    test_dir_and_file_creation(fs, root);
    test_file_read_write(fs, root);
    test_read_at_short_read(fs, root);
    test_read_batch(fs, root);
    test_rename_and_sync_dir(fs, root);
    test_directory_operations(fs, root);
    test_atomic_write(fs, root);
    test_lock_exclusivity(fs, root);
}

fn test_dir_and_file_creation<F: Fs>(fs: &F, root: &Path) {
    let sub = root.join("test_dir_creation");
    fs.create_dir_all(&sub, Perm::Private)
        .expect("create_dir_all succeeds");

    #[cfg(unix)]
    if fs.caps().private_perm {
        if let Ok(mode) = RealFs::get_mode(&sub) {
            assert_eq!(
                mode & 0o777,
                0o700,
                "directory must have 0700 permissions"
            );
        }
    }

    let file_path = sub.join("test_file.dat");
    let file = fs
        .open(&file_path, OpenMode::CreateNew(Perm::Private))
        .expect("create file succeeds");
    drop(file);

    #[cfg(unix)]
    if fs.caps().private_perm {
        if let Ok(mode) = RealFs::get_mode(&file_path) {
            assert_eq!(
                mode & 0o777,
                0o600,
                "file must have 0600 permissions"
            );
        }
    }
}

fn test_file_read_write<F: Fs>(fs: &F, root: &Path) {
    let file_path = root.join("rw_test.dat");
    let file = fs
        .open(&file_path, OpenMode::CreateNew(Perm::Private))
        .expect("create succeeds");

    let payload = b"Hello, Contextra Storage Engine!";
    file.write_all_at(payload, 100)
        .expect("write_all_at succeeds");
    file.sync_data().expect("sync_data succeeds");
    file.sync_all().expect("sync_all succeeds");

    let len = file.len().expect("len succeeds");
    assert_eq!(len, 100 + payload.len() as u64);

    let mut buf = vec![0u8; payload.len()];
    let read_bytes = file.read_at(&mut buf, 100).expect("read_at succeeds");
    assert_eq!(read_bytes, payload.len());
    assert_eq!(&buf, payload);

    file.set_len(50).expect("set_len succeeds");
    assert_eq!(file.len().expect("len succeeds"), 50);
}

fn test_read_at_short_read<F: Fs>(fs: &F, root: &Path) {
    let file_path = root.join("short_read_test.dat");
    let file = fs
        .open(&file_path, OpenMode::CreateNew(Perm::Private))
        .expect("create succeeds");

    let payload = b"0123456789abcdefghijklmnopqrst"; // 30 bytes
    file.write_all_at(payload, 0).expect("write succeeds");
    file.sync_all().expect("sync_all succeeds");

    // Puffer ist 50 Bytes groß, ab Offset 10 verbleiben aber nur 20 Bytes in der Datei
    let mut large_buf = vec![0u8; 50];
    let n = file.read_at(&mut large_buf, 10).expect("short read succeeds");
    assert_eq!(n, 20, "read_at must return remaining bytes as short read");
    assert_eq!(&large_buf[..20], &payload[10..]);

    // Lesen genau an EOF liefert 0 Bytes
    let mut eof_buf = [0u8; 10];
    let n_eof = file.read_at(&mut eof_buf, 30).expect("read at EOF succeeds");
    assert_eq!(n_eof, 0, "read_at at EOF must return 0 bytes");

    // Lesen jenseits von EOF liefert 0 Bytes
    let n_beyond = file.read_at(&mut eof_buf, 100).expect("read beyond EOF succeeds");
    assert_eq!(n_beyond, 0, "read_at beyond EOF must return 0 bytes");
}

fn test_read_batch<F: Fs>(fs: &F, root: &Path) {
    let file_path = root.join("batch_test.dat");
    let file = fs
        .open(&file_path, OpenMode::CreateNew(Perm::Private))
        .expect("create succeeds");

    file.write_all_at(b"AAAABBBBCCCCDDDD", 0)
        .expect("write succeeds");

    let mut buf1 = [0u8; 4];
    let mut buf2 = [0u8; 4];
    let mut buf3 = [0u8; 4];

    let mut reqs = [
        ReadReq {
            offset: 0,
            buf: &mut buf1,
        },
        ReadReq {
            offset: 8,
            buf: &mut buf2,
        },
        ReadReq {
            offset: 12,
            buf: &mut buf3,
        },
    ];

    file.read_batch(&mut reqs).expect("read_batch succeeds");
    assert_eq!(&buf1, b"AAAA");
    assert_eq!(&buf2, b"CCCC");
    assert_eq!(&buf3, b"DDDD");
}

fn test_rename_and_sync_dir<F: Fs>(fs: &F, root: &Path) {
    let from = root.join("rename_from.dat");
    let to = root.join("rename_to.dat");

    let file = fs
        .open(&from, OpenMode::CreateNew(Perm::Private))
        .expect("create succeeds");
    file.write_all_at(b"rename payload", 0)
        .expect("write succeeds");
    drop(file);

    fs.rename(&from, &to).expect("rename succeeds");
    fs.sync_dir(root).expect("sync_dir succeeds");

    let entries = fs.list_dir(root).expect("list_dir succeeds");
    assert!(entries.iter().any(|e| e.name == "rename_to.dat"));
    assert!(!entries.iter().any(|e| e.name == "rename_from.dat"));

    fs.remove_file(&to).expect("remove_file succeeds");
    fs.sync_dir(root).expect("sync_dir succeeds after remove");

    let entries_after = fs.list_dir(root).expect("list_dir succeeds");
    assert!(!entries_after.iter().any(|e| e.name == "rename_to.dat"));
}

fn test_directory_operations<F: Fs>(fs: &F, root: &Path) {
    let sub = root.join("dir_ops_parent").join("child");
    fs.create_dir_all(&sub, Perm::Private).expect("create_dir_all nested");

    let file_path = sub.join("item.dat");
    let file = fs
        .open(&file_path, OpenMode::CreateNew(Perm::Private))
        .expect("create file");
    drop(file);

    let entries = fs.list_dir(&sub).expect("list_dir succeeds");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "item.dat");

    fs.remove_file(&file_path).expect("remove file in dir");
    fs.sync_dir(&sub).expect("sync_dir succeeds");
    assert!(fs.list_dir(&sub).expect("list_dir succeeds").is_empty());

    fs.remove_dir(&sub).expect("remove_dir succeeds");
}

fn test_atomic_write<F: Fs>(fs: &F, root: &Path) {
    let target = root.join("atomic_target.dat");
    let data = b"Atomare Daten fuer SPEC Invariante";

    write_atomic(fs, &target, data).expect("write_atomic succeeds");

    let file = fs
        .open(&target, OpenMode::Read)
        .expect("open target succeeds");
    let mut buf = vec![0u8; data.len()];
    file.read_at(&mut buf, 0).expect("read_at succeeds");
    assert_eq!(&buf, data);

    // Erneutes atomic_write überschreibt atomar
    let new_data = b"Ueberschriebene Daten";
    write_atomic(fs, &target, new_data).expect("second write_atomic succeeds");

    let file2 = fs
        .open(&target, OpenMode::Read)
        .expect("open second succeeds");
    let mut buf2 = vec![0u8; new_data.len()];
    file2.read_at(&mut buf2, 0).expect("read_at succeeds");
    assert_eq!(&buf2, new_data);
    assert_eq!(file2.len().expect("len succeeds"), new_data.len() as u64);
}

fn test_lock_exclusivity<F: Fs>(fs: &F, root: &Path) {
    let lock_file = root.join("test.lock");
    let guard1 = fs
        .lock(&lock_file, LockMode::Exclusive)
        .expect("first lock succeeds");

    let lock2_res = fs.lock(&lock_file, LockMode::Exclusive);
    assert!(
        lock2_res.is_err(),
        "second exclusive lock must fail while guard is held"
    );

    drop(guard1);

    let guard3 = fs
        .lock(&lock_file, LockMode::Exclusive)
        .expect("lock succeeds after guard dropped");
    drop(guard3);
}
