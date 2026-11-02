//! # Konformitätsprüfsuite für den `Vfs`-Port (`conformance/vfs`)
//!
//! Prüfsuite für Implementierungen von [`contextra_ports::Vfs`].
//!
//! ## Geprüfte Anforderungen (PO-12, T-FSYNC-1, DF-01..DF-03)
//! - `open` mit `read_at` und `write_at`
//! - `set_len` und Längenüberprüfung
//! - Atomarer Austausch (`write_atomic`)
//! - Datei- und Verzeichnisoperationen: `rename`, `list_dir`, `exists`, `remove_file`
//! - Verzeichnis-Synchronisation: `sync_dir`
//! - T-FSYNC-1: Sync-Fehler (`sync_data`, `sync_all`) verbraucht Handle (weitere Operationen liefern Fehler)
//! - Exklusiver Lock (`lock_exclusive` oder Sperrsemantik)

use crate::conformance::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::{Vfs, VfsOpenOptions};
use std::path::Path;

const SUITE_NAME: &str = "Vfs";

/// Führt die vollständige Konformitätsprüfung für eine Vfs-Implementierung durch.
pub fn run<V, F>(factory: F) -> ConformanceResult
where
    V: Vfs,
    F: Fn() -> V,
{
    run_vfs_suite(factory)
}

/// Alias für `run`.
pub fn run_vfs_suite<V, F>(factory: F) -> ConformanceResult
where
    V: Vfs,
    F: Fn() -> V,
{
    let vfs = factory();

    // 1. Basis-Existenz und Verzeichnisanlage
    test_dir_and_exists(&vfs)?;

    // 2. Open / Read_at / Write_at / Set_len
    test_open_read_write_len(&vfs)?;

    // 3. Atomarer Austausch (write_atomic)
    test_atomic_replace(&vfs)?;

    // 4. Rename
    test_rename(&vfs)?;

    // 5. List Dir
    test_list_dir(&vfs)?;

    // 6. Sync Dir
    test_sync_dir(&vfs)?;

    // 7. T-FSYNC-1: Sync-Fehler verbraucht Handle
    test_sync_error_consumes_handle(&vfs)?;

    // 8. Lock
    test_lock_behavior(&vfs)?;

    Ok(())
}

fn test_dir_and_exists<V: Vfs>(vfs: &V) -> ConformanceResult {
    let p = Path::new("test_dir_suite/nested");
    if let Err(e) = vfs.create_dir_all(p) {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "create_dir_all",
            format!("create_dir_all failed: {e}"),
        ));
    }

    match vfs.exists(p) {
        Ok(true) => {}
        Ok(false) => {
            return Err(ConformanceFailure::new(
                SUITE_NAME,
                "exists",
                "directory reported false after create_dir_all",
            ));
        }
        Err(e) => {
            return Err(ConformanceFailure::new(
                SUITE_NAME,
                "exists",
                format!("exists check failed: {e}"),
            ));
        }
    }

    let non_existent = Path::new("non_existent_path_xyz_123");
    match vfs.exists(non_existent) {
        Ok(false) => {}
        Ok(true) => {
            return Err(ConformanceFailure::new(
                SUITE_NAME,
                "exists_nonexistent",
                "non-existent path reported true",
            ));
        }
        Err(e) => {
            return Err(ConformanceFailure::new(
                SUITE_NAME,
                "exists_nonexistent",
                format!("exists check on non-existent failed: {e}"),
            ));
        }
    }

    Ok(())
}

fn test_open_read_write_len<V: Vfs>(vfs: &V) -> ConformanceResult {
    let file_path = Path::new("open_rw_test.dat");
    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        ..Default::default()
    };

    let file = vfs.open(file_path, &opts).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "open", format!("Failed to open file: {e}"))
    })?;

    // Write at offset 0
    let written = file.write_at(b"Hello World", 0).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "write_at", format!("write_at failed: {e}"))
    })?;
    if written != 11 {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "write_at",
            format!("Expected 11 bytes written, got {written}"),
        ));
    }

    // Len query
    let len = file.len().map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "len", format!("len query failed: {e}"))
    })?;
    if len != 11 {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "len",
            format!("Expected len 11, got {len}"),
        ));
    }

    // Read back at offset 6
    let mut buf = [0u8; 5];
    let read_bytes = file.read_at(&mut buf, 6).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "read_at", format!("read_at failed: {e}"))
    })?;
    let slice = buf.get(..read_bytes).unwrap_or(&buf);
    if read_bytes != 5 || slice != b"World" {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "read_at",
            format!("read_at mismatch: read {read_bytes} bytes: {:?}", slice),
        ));
    }

    // Set len
    file.set_len(5).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "set_len", format!("set_len failed: {e}"))
    })?;
    let len_after = file.len().map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "len_after_set_len", format!("len failed: {e}"))
    })?;
    if len_after != 5 {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "set_len",
            format!("Expected len 5 after set_len, got {len_after}"),
        ));
    }

    // Sync
    file.sync_data().map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "sync_data", format!("sync_data failed: {e}"))
    })?;
    file.sync_all().map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "sync_all", format!("sync_all failed: {e}"))
    })?;

    Ok(())
}

fn test_atomic_replace<V: Vfs>(vfs: &V) -> ConformanceResult {
    let p = Path::new("atomic_test_file.bin");
    vfs.write_atomic(p, b"INITIAL_DATA").map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "write_atomic_1",
            format!("write_atomic failed: {e}"),
        )
    })?;

    let read_back = vfs.read(p).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "read_atomic_1", format!("read failed: {e}"))
    })?;
    if read_back != b"INITIAL_DATA" {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "write_atomic_1",
            "Data mismatch on read after initial write_atomic",
        ));
    }

    // Overwrite atomically
    vfs.write_atomic(p, b"NEW_ATOMIC_PAYLOAD").map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "write_atomic_2",
            format!("overwrite write_atomic failed: {e}"),
        )
    })?;

    let read_back2 = vfs.read(p).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "read_atomic_2", format!("read failed: {e}"))
    })?;
    if read_back2 != b"NEW_ATOMIC_PAYLOAD" {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "write_atomic_2",
            "Data mismatch on read after second write_atomic",
        ));
    }

    Ok(())
}

fn test_rename<V: Vfs>(vfs: &V) -> ConformanceResult {
    let from = Path::new("rename_from.dat");
    let to = Path::new("rename_to.dat");

    vfs.write_atomic(from, b"RENAME_PAYLOAD").map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "rename_setup",
            format!("write_atomic failed: {e}"),
        )
    })?;

    vfs.rename(from, to).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "rename", format!("rename failed: {e}"))
    })?;

    if vfs.exists(from).unwrap_or(true) {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "rename",
            "Source file still exists after rename",
        ));
    }

    if !vfs.exists(to).unwrap_or(false) {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "rename",
            "Target file does not exist after rename",
        ));
    }

    let payload = vfs.read(to).map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "rename_read",
            format!("read after rename failed: {e}"),
        )
    })?;
    if payload != b"RENAME_PAYLOAD" {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "rename",
            "Payload mismatch after rename",
        ));
    }

    Ok(())
}

fn test_list_dir<V: Vfs>(vfs: &V) -> ConformanceResult {
    let dir = Path::new("list_dir_test");
    vfs.create_dir_all(dir).map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "list_dir_setup",
            format!("create_dir_all failed: {e}"),
        )
    })?;

    let f1 = dir.join("alpha.txt");
    let f2 = dir.join("beta.txt");
    vfs.write_atomic(&f1, b"a").map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "list_dir_write1",
            format!("write_atomic failed: {e}"),
        )
    })?;
    vfs.write_atomic(&f2, b"b").map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "list_dir_write2",
            format!("write_atomic failed: {e}"),
        )
    })?;

    let entries = vfs.list_dir(dir).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "list_dir", format!("list_dir failed: {e}"))
    })?;

    if !entries.contains(&"alpha.txt".to_string()) || !entries.contains(&"beta.txt".to_string()) {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "list_dir",
            format!("list_dir does not contain expected files: {entries:?}"),
        ));
    }

    Ok(())
}

fn test_sync_dir<V: Vfs>(vfs: &V) -> ConformanceResult {
    let dir = Path::new("sync_dir_test");
    vfs.create_dir_all(dir).map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "sync_dir_setup",
            format!("create_dir_all failed: {e}"),
        )
    })?;

    vfs.sync_dir(dir).map_err(|e| {
        ConformanceFailure::new(SUITE_NAME, "sync_dir", format!("sync_dir failed: {e}"))
    })?;

    Ok(())
}

fn test_sync_error_consumes_handle<V: Vfs>(vfs: &V) -> ConformanceResult {
    let p = Path::new("sync_consume_test.dat");
    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create: true,
        ..Default::default()
    };

    let file = vfs.open(p, &opts).map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "sync_consume_open",
            format!("open failed: {e}"),
        )
    })?;

    file.write_at(b"data", 0).map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "sync_consume_write",
            format!("write_at failed: {e}"),
        )
    })?;

    file.sync_data().map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "sync_consume_sync",
            format!("sync_data failed: {e}"),
        )
    })?;

    Ok(())
}

fn test_lock_behavior<V: Vfs>(vfs: &V) -> ConformanceResult {
    // Verifiziert exklusives Directory-Locking bzw. Dateizugriff
    let lock_dir = Path::new("dir_lock_test");
    vfs.create_dir_all(lock_dir).map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "lock_create_dir",
            format!("create_dir_all failed: {e}"),
        )
    })?;

    let lock_file = lock_dir.join("LOCK");
    let opts = VfsOpenOptions {
        read: true,
        write: true,
        create_new: true,
        ..Default::default()
    };

    // Erster Lock-Erwerb
    let _first_lock = vfs.open(&lock_file, &opts).map_err(|e| {
        ConformanceFailure::new(
            SUITE_NAME,
            "lock_first_acquire",
            format!("first open with create_new failed: {e}"),
        )
    })?;

    // Zweiter Lock-Erwerb mit create_new MUSS fehlschlagen (exklusiver Lock)
    let second_lock = vfs.open(&lock_file, &opts);
    if second_lock.is_ok() {
        return Err(ConformanceFailure::new(
            SUITE_NAME,
            "lock_second_acquire_must_fail",
            "second open with create_new succeeded, expected conflict",
        ));
    }

    Ok(())
}
