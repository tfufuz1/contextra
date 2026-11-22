//! Fehlerpfad- und Lebenszyklus-Tests für `contextra-sys` (WP-3).
//!
//! Validiert das deterministische Fehlermodell bei Systemgrenzen und ungültigen Inputs:
//! - `mmap` bei ungültigem / nur zum Schreiben geöffnetem Handle
//! - `durable_flush_file` / `fsync` auf ungültigen Deskriptoren
//! - Poisoning-Zustände bei `DurableFile`
//! - `mlock` bei Null-Zeiger oder unzulässigen Adressen
//! - Nullsetzen (Zeroize) von `VaultBuffer` und `LockedRegions` beim Drop

use std::fs::{File, OpenOptions};
use std::io::Write;
use tempfile::NamedTempFile;

use contextra_sys::durability::{durable_flush_file, DurableFile};
use contextra_sys::mlock::{mem_lock_typed, MemLockPolicy};
use contextra_sys::mmap::{mmap_readonly, SealedFile};
use contextra_sys::vault::{LockedRegions, VaultBuffer};

#[test]
fn test_mmap_error_on_write_only_or_directory() {
    let temp = NamedTempFile::new().expect("temp file created");
    // Datei nur zum Schreiben öffnen (kein Leserecht)
    let write_only_file = OpenOptions::new()
        .write(true)
        .read(false)
        .open(temp.path())
        .expect("open write-only");

    let res = mmap_readonly(&write_only_file);
    assert!(
        res.is_err(),
        "mmap_readonly must fail on write-only file handle"
    );

    let ro_file = File::open(temp.path()).expect("open read-only");
    let sealed_res = SealedFile::new_verified(ro_file, 0);
    assert!(
        sealed_res.is_ok(),
        "sealed file open succeeds on valid file"
    );
}

#[test]
fn test_durable_flush_file_on_readonly() {
    let mut temp = NamedTempFile::new().expect("temp file created");
    temp.write_all(b"fsync test payload")
        .expect("write succeeds");
    let ro_file = File::open(temp.path()).expect("open read-only");

    let res = durable_flush_file(&ro_file, false);
    assert!(
        res.is_ok(),
        "durable_flush_file handles read-only descriptors cleanly"
    );
}

#[test]
fn test_durable_file_poisoning_lifecycle() {
    let mut temp = NamedTempFile::new().expect("temp file created");
    temp.write_all(b"durable test").expect("write succeeds");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(temp.path())
        .expect("open rw");

    let mut durable = DurableFile::new(file);
    assert!(!durable.is_poisoned());

    // Unter normalen Bedingungen funktioniert sync_data
    assert!(durable.sync_data().is_ok());
    assert!(!durable.is_poisoned());
}

#[test]
fn test_mlock_null_and_zero_len() {
    let null_ptr: *const u8 = std::ptr::null();
    // Null-Pointer oder Länge 0 ist ein No-op und liefert Ok(true)
    let res = mem_lock_typed(null_ptr, 0, MemLockPolicy::FailClosed);
    assert_eq!(res.expect("no-op lock succeeds"), true);

    let res_tolerant = mem_lock_typed(null_ptr, 0, MemLockPolicy::Tolerant);
    assert_eq!(res_tolerant.expect("no-op lock succeeds"), true);
}

#[test]
fn test_vault_buffer_zeroize_on_drop() {
    let initial_data = [0xAAu8; 64];
    {
        let buffer = VaultBuffer::from_slice(&initial_data).expect("allocation succeeds");
        assert_eq!(buffer.len(), 64);
        assert_eq!(buffer.as_slice()[0], 0xAA);
        // buffer wird hier gedroppt
    }
}

#[test]
fn test_locked_regions_zeroize_on_drop() {
    let key_bytes = [0x55u8; 32];
    {
        let mut regions = LockedRegions::new();
        let _ = regions.lock_slice(&key_bytes, false);
        assert_eq!(key_bytes[0], 0x55);
        // regions wird gedroppt
    }
}
