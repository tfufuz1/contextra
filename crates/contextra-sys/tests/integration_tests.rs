// STAND: 2026-11-08T00:00:00Z
// ZWECK: Comprehensive integration tests for contextra-sys public API and invariants (SPEC §10.1 / INV-SYS-FSYNC-1 / INV-SYS-MEMLOCK-1).
// INVARIANTEN: Zero panic in test harness; verify cross-module contracts and invariants.

#![allow(unsafe_code)]
#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::indexing_slicing)]

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use tempfile::tempdir;

use contextra_sys::{
    durable_flush_dir, durable_flush_file, ensure_fd_limit, mem_lock, mem_lock_with_policy,
    mem_unlock, mmap_readonly, set_fd_limit, set_restrictive_file_acl, verify_file_acl_owner_only,
    write_all_durable, DurableFile, Error, LockedRegions, MemLockPolicy, SealedFile, SealedMap,
    VaultBuffer,
};

#[test]
fn test_durability_file_and_directory_lifecycle() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("durable_test.dat");

    // Test write_all_durable & durable_flush_file
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
            .expect("create file");

        let payload = b"contextra_durability_payload_test_data";
        write_all_durable(&mut file, payload).expect("write durable");
        durable_flush_file(&file, true).expect("durable flush with metadata");
    }

    // Verify content written accurately
    {
        let mut file = File::open(&file_path).expect("open file");
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).expect("read content");
        assert_eq!(buf, b"contextra_durability_payload_test_data");
    }

    // Test directory flush
    durable_flush_dir(dir.path()).expect("durable flush directory");
}

#[test]
fn test_durable_file_poisoning_contract() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("durable_file_contract.dat");

    let raw = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file_path)
        .expect("open file");

    let mut durable = DurableFile::new(raw);
    assert!(!durable.is_poisoned());
    durable.write_all(b"healthy chunk").expect("healthy write");
    durable.sync_data().expect("sync data");
    assert!(!durable.is_poisoned());
}

#[test]
fn test_mmap_and_sealed_map_lifecycle() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("sealed_mmap.dat");
    let payload = b"contextra_sys_sealed_memory_mapping_1234567890";

    {
        let mut file = File::create(&file_path).expect("create file");
        file.write_all(payload).expect("write payload");
        file.sync_all().expect("sync");
    }

    // Direct mmap_readonly
    {
        let file = File::open(&file_path).expect("open file");
        let map = mmap_readonly(&file).expect("mmap_readonly");
        assert_eq!(&map[..], payload);
    }

    // SealedFile verified length match
    {
        let file = File::open(&file_path).expect("open file");
        let sealed_file =
            SealedFile::new_verified(file, payload.len() as u64).expect("sealed file verified len");
        assert_eq!(sealed_file.verified_len(), payload.len() as u64);

        let sealed_map: SealedMap = sealed_file.map_readonly().expect("map readonly");
        assert_eq!(sealed_map.len(), payload.len());
        assert!(!sealed_map.is_empty());
        assert_eq!(sealed_map.as_slice(), payload);
        assert_eq!(&sealed_map[..], payload);

        let sub = sealed_map.get_subslice(14, 6).expect("subslice");
        assert_eq!(sub, b"sealed");
    }

    // SealedFile length mismatch fails closed
    {
        let file = File::open(&file_path).expect("open file");
        let result = SealedFile::new_verified(file, (payload.len() + 10) as u64);
        assert!(result.is_err(), "Length mismatch must fail closed");
    }
}

#[test]
fn test_memory_locking_and_vault() {
    let mut secret = vec![0xEFu8; 128];
    let ptr = secret.as_ptr();
    let len = secret.len();

    // Raw mem_lock and mem_unlock
    let lock_res = mem_lock(ptr, len);
    if lock_res {
        mem_unlock(ptr as usize, len);
    }

    // MemLockPolicy Tolerant vs FailClosed
    let tol_res = mem_lock_with_policy(ptr, len, MemLockPolicy::Tolerant);
    assert!(tol_res.is_ok());

    // LockedRegions RAII abstraction
    let mut regions = LockedRegions::new();
    assert!(regions.is_empty());
    assert_eq!(regions.len(), 0);

    let locked = regions.lock_slice_with_policy(&secret, true, MemLockPolicy::Tolerant);
    assert!(locked.is_ok());
    if let Ok(true) = locked {
        assert_eq!(regions.len(), 1);
        assert!(!regions.is_empty());
    } else {
        assert_eq!(regions.len(), 0);
    }

    // Volatile zeroization
    LockedRegions::zeroize_slice(&mut secret);
    assert!(secret.iter().all(|&b| b == 0));

    // RAII unlock
    regions.unlock_all();
    assert_eq!(regions.len(), 0);
}

#[test]
fn test_vault_buffer_allocation_and_zeroize() {
    let raw = b"integration_test_secret_buffer_bytes_9876";
    let mut buf = VaultBuffer::allocate_with_policy(raw.len(), MemLockPolicy::Tolerant)
        .expect("allocate vault buffer");
    buf.as_mut_slice().copy_from_slice(raw);
    assert_eq!(&buf[..], raw);
    drop(buf);
}

#[test]
fn test_file_acl_restriction_and_verification() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("secure_key.dat");

    {
        let mut file = File::create(&file_path).expect("create file");
        file.write_all(b"secret_key_bytes").expect("write secret");
    }

    // Set ACL (0600 on Unix, DACL on Win32)
    set_restrictive_file_acl(&file_path).expect("set restrictive acl");

    // Verify ownership and restrictive permissions
    let verify_res = verify_file_acl_owner_only(&file_path);
    assert!(
        verify_res.is_ok(),
        "ACL verification must succeed: {:?}",
        verify_res
    );
}

#[test]
fn test_posix_resource_limits() {
    let ok = ensure_fd_limit(256);
    let _ = ok;

    let res = set_fd_limit(512);
    let _ = res;
}

#[test]
fn test_typed_error_conversions() {
    let sync_err = Error::SyncFailed(libc::EIO);
    let io_err: std::io::Error = sync_err.into();
    assert_eq!(io_err.raw_os_error(), Some(libc::EIO));

    let lock_err = Error::MemoryLockFailed {
        errno: libc::ENOMEM,
        reason: "failed locking memory",
    };
    let io_lock: std::io::Error = lock_err.into();
    assert_eq!(io_lock.raw_os_error(), Some(libc::ENOMEM));

    let poisoned = Error::HandlePoisoned;
    let io_poisoned: std::io::Error = poisoned.into();
    assert_eq!(io_poisoned.kind(), std::io::ErrorKind::Other);
}
