use super::*;
use crate::wal::Wal;
use memfuse_core::TxId;
use memfuse_security::crypto::KeyManager;
use memfuse_security::wal_crypto::WalHmac;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::fs;

fn compute_v3_hmac_reference_independent(
    key: &[u8],
    prev_hmac: &[u8; 32],
    seq_no: u64,
    tx_id: u64,
    op: &WalOp,
) -> Result<[u8; 32]> {
    let mut mac = WalHmac::new(key)?;
    mac.update(prev_hmac);
    mac.update(&seq_no.to_le_bytes());
    mac.update(&tx_id.to_le_bytes());

    match op {
        WalOp::Put { key, value, .. } => {
            mac.update(&[0u8]);
            mac.update(&(key.len() as u32).to_le_bytes());
            mac.update(key);
            mac.update(&(value.len() as u32).to_le_bytes());
            mac.update(value);
        }
        WalOp::Delete { key, .. } => {
            mac.update(&[1u8]);
            mac.update(&(key.len() as u32).to_le_bytes());
            mac.update(key);
        }
    }

    Ok(mac.finalize())
}

#[tokio::test]
async fn test_wal_hash_chain_verification() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("chain_wal.log");

    {
        let wal = Wal::open(&wal_path).await.expect("open"); // expect
        let op1 = WalOp::Put {
            tx_id: TxId::new(1),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        };
        let (batch1, _) = wal.prepare_batch(vec![(op1, 1)]).await.expect("entry1");
        wal.append_batch(batch1).await.expect("append1");

        let op2 = WalOp::Put {
            tx_id: TxId::new(2),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        };
        let (batch2, _) = wal.prepare_batch(vec![(op2, 2)]).await.expect("entry2");
        wal.append_batch(batch2).await.expect("append2");
    }

    {
        let mut data = fs::read(&wal_path).await.expect("read"); // expect
                                                                 // Corrupt the payload of the first entry (offset 4 is CRC, payload starts at 8)
                                                                 // CRC itself is also part of validation. Let's flip a bit in the payload.
        if data.len() > 10 {
            data[12] ^= 0xFF;
            fs::write(&wal_path, data).await.expect("write"); // expect
        }
    }

    let result = Wal::open(&wal_path).await;
    // Should fail due to CRC mismatch or HMAC chain failure
    assert!(matches!(
        result,
        Err(MemFuseError::Serialization(_)) | Err(MemFuseError::WalCorruption { .. })
    ));
}

#[tokio::test]
async fn test_wal_random_integrity_keys_per_instance() {
    let dir1 = tempdir().expect("tempdir1"); // expect
    let dir2 = tempdir().expect("tempdir2"); // expect
    let wal_path1 = dir1.path().join("wal1.log");
    let wal_path2 = dir2.path().join("wal2.log");

    let wal1 = Wal::open(&wal_path1).await.expect("open wal1"); // expect
    let wal2 = Wal::open(&wal_path2).await.expect("open wal2"); // expect

    let key1 = wal1.get_integrity_key().expect("key1"); // expect
    let key2 = wal2.get_integrity_key().expect("key2"); // expect

    assert_ne!(
        key1, key2,
        "Two independent WAL instances must receive unique random integrity keys"
    );
}

#[tokio::test]
async fn test_integrity_key_atomic_permissions_and_race_condition() {
    let temp = tempfile::tempdir().expect("tempdir"); // expect
    let wal_path = temp.path().join("test.wal");

    // Test 1: Created key file has 0o600 permissions on Unix
    let key1 = Wal::load_or_create_integrity_key(&wal_path)
        .await
        .expect("create key"); // expect

    let key_path = temp.path().join(".wal_integrity_key");
    assert!(key_path.exists(), "Key file must exist");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&key_path).expect("metadata"); // expect
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "WAL integrity key file must have permissions 0o600 on Unix, got 0o{:o}",
            mode
        );
    }

    // Test 2: Race condition simulation with multiple concurrent callers
    let wal_path_race = temp.path().join("race.wal");
    let mut handles = Vec::new();
    for _ in 0..10 {
        let path = wal_path_race.clone();
        handles.push(tokio::spawn(async move {
            Wal::load_or_create_integrity_key(&path).await
        }));
    }

    let mut keys = Vec::new();
    for h in handles {
        let res = h.await.expect("join handle").expect("load key"); // expect
        keys.push(res);
    }

    for k in &keys {
        assert_eq!(
            k, &keys[0],
            "All concurrent tasks must receive the identical key"
        );
    }
    assert_eq!(key1.len(), 32);
}

#[cfg(windows)]
#[test]
#[allow(unsafe_code)]
fn test_windows_wal_integrity_key_acl() {
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_SUCCESS, HANDLE};
    use windows_sys::Win32::Security::Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        EqualSid, GetAce, GetTokenInformation, TokenUser, ACCESS_ALLOWED_ACE, ACL,
        DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, SECURITY_DESCRIPTOR_CONTROL,
        SE_DACL_PROTECTED, TOKEN_QUERY, TOKEN_USER,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let temp = tempfile::tempdir().expect("tempdir"); // expect
    let wal_path = temp.path().join("test_acl.wal");

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime"); // expect
    let key = rt
        .block_on(Wal::load_or_create_integrity_key(&wal_path))
        .expect("load_or_create_integrity_key"); // expect
    assert_eq!(key.len(), 32);

    let key_path = temp.path().join(".wal_integrity_key");
    assert!(key_path.exists(), "Key file must exist");

    let path_wide: Vec<u16> = key_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut p_sec_desc: PSECURITY_DESCRIPTOR = null_mut();
    let mut p_dacl: *mut ACL = null_mut();
    let mut control: SECURITY_DESCRIPTOR_CONTROL = 0;
    let mut revision = 0u32;

    // Query file's DACL and Control bits
    let status = unsafe {
        GetNamedSecurityInfoW(
            path_wide.as_ptr() as *mut _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            &mut p_dacl,
            null_mut(),
            &mut p_sec_desc,
        )
    };
    assert_eq!(
        status, ERROR_SUCCESS,
        "GetNamedSecurityInfoW failed with error {}",
        status
    );

    struct SecDescGuard(PSECURITY_DESCRIPTOR);
    impl Drop for SecDescGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    windows_sys::Win32::Foundation::LocalFree(self.0 as _);
                }
            }
        }
    }
    let _guard = SecDescGuard(p_sec_desc);

    // Verify DACL is present
    assert!(!p_dacl.is_null(), "DACL should not be null");

    // Verify control bits to check that DACL inheritance is protected/disabled
    let status = unsafe {
        windows_sys::Win32::Security::GetSecurityDescriptorControl(
            p_sec_desc,
            &mut control,
            &mut revision,
        )
    };
    assert_ne!(
        status,
        0,
        "GetSecurityDescriptorControl failed with error {}",
        unsafe { GetLastError() }
    );
    assert_ne!(
        control & SE_DACL_PROTECTED,
        0,
        "DACL inheritance must be disabled (SE_DACL_PROTECTED bit set)"
    );

    // Query process token user SID
    let mut token_handle: HANDLE = null_mut();
    let res = unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) };
    assert_ne!(res, 0, "OpenProcessToken failed");

    let mut len = 0u32;
    unsafe {
        GetTokenInformation(token_handle, TokenUser, null_mut(), 0, &mut len);
    }
    let mut buffer = vec![0u8; len as usize];
    let res = unsafe {
        GetTokenInformation(
            token_handle,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            len,
            &mut len,
        )
    };
    assert_ne!(res, 0, "GetTokenInformation failed");
    unsafe { CloseHandle(token_handle) };

    let token_user = buffer.as_ptr() as *const TOKEN_USER;
    let owner_sid = unsafe { (*token_user).User.Sid };
    assert!(!owner_sid.is_null());

    // Inspect ACE count and verify ACE matches process owner SID
    let ace_count = unsafe { (*p_dacl).AceCount };
    assert_eq!(ace_count, 1, "DACL must contain exactly 1 ACE (owner only)");

    let mut p_ace: *mut std::ffi::c_void = null_mut();
    let res = unsafe { GetAce(p_dacl, 0, &mut p_ace) };
    assert_ne!(res, 0, "GetAce failed");

    let ace = p_ace as *const ACCESS_ALLOWED_ACE;
    let ace_sid = unsafe { &(*ace).SidStart as *const u32 as *mut std::ffi::c_void };

    let same_sid = unsafe { EqualSid(owner_sid, ace_sid) };
    assert_ne!(same_sid, 0, "ACE SID must match the process owner SID");
}

#[tokio::test]
async fn test_uuid_sidecar_crash_fault_injection() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("fault_uuid.wal");
    let uuid_path = dir.path().join("fault_uuid.wal.uuid");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890")
            .expect("km"), // expect
    );

    // 1. Simulate a leftover interrupted temp file from a crashed write
    let tmp_path = dir.path().join("fault_uuid.wal.uuid.tmp.99999.12345");
    tokio::fs::write(&tmp_path, b"incomplete_uuid")
        .await
        .expect("write leftover tmp file"); // expect

    // 2. Opening WAL should cleanly recover, create valid 16-byte UUID sidecar, and ignore leftover tmp file
    let wal = Wal::open_with_key_manager(&wal_path, Some(km.clone()))
        .await
        .expect("open wal should succeed despite leftover tmp file"); // expect

    assert!(uuid_path.exists(), "UUID sidecar must exist");
    let uuid_bytes = tokio::fs::read(&uuid_path).await.expect("read uuid"); // expect
    assert_eq!(uuid_bytes.len(), 16);

    drop(wal);

    // 3. Re-opening should read the same valid UUID sidecar
    let uuid_bytes_after = Wal::load_or_create_wal_uuid(&wal_path)
        .await
        .expect("load uuid"); // expect
    assert_eq!(uuid_bytes.as_slice(), uuid_bytes_after);
}

#[tokio::test]
async fn test_prepare_batch_hmac_chain_concurrency() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("concurrency_batch.wal");

    let wal = Arc::new(Wal::open(&wal_path).await.expect("open wal")); // expect

    let wal1 = wal.clone();
    let wal2 = wal.clone();

    let handle1 = tokio::spawn(async move {
        let ops = vec![(
            WalOp::Put {
                tx_id: TxId::new(1),
                key: b"k1".to_vec(),
                value: b"v1".to_vec(),
            },
            1,
        )];
        wal1.prepare_batch(ops).await.expect("batch 1") // expect
    });

    let handle2 = tokio::spawn(async move {
        let ops = vec![(
            WalOp::Put {
                tx_id: TxId::new(2),
                key: b"k2".to_vec(),
                value: b"v2".to_vec(),
            },
            2,
        )];
        wal2.prepare_batch(ops).await.expect("batch 2") // expect
    });

    let (res1, res2) = tokio::join!(handle1, handle2);
    let (batch1, _) = res1.expect("join 1"); // expect
    let (batch2, _) = res2.expect("join 2"); // expect

    let prev1 = batch1.entries()[0].prev_hmac;
    let prev2 = batch2.entries()[0].prev_hmac;

    // One batch must have chained off the initial [0u8; 32] HMAC, and the second batch off the first's checksum.
    // Crucially, their starting prev_hmac values must NOT be identical.
    assert_ne!(
        prev1, prev2,
        "Concurrent prepare_batch calls must produce unique prev_hmac chain links"
    );

    if prev1 == [0u8; 32] {
        assert_eq!(prev2, batch1.entries()[0].checksum);
    } else {
        assert_eq!(prev1, batch2.entries()[0].checksum);
        assert_eq!(prev2, [0u8; 32]);
    }
}

#[tokio::test]
async fn test_open_with_key_manager_is_new_race_condition() {
    let dir = tempdir().expect("tempdir"); // expect
    let wal_path = dir.path().join("race_open.wal");

    let km = Arc::new(
        KeyManager::try_new("passphrase123", b"salt123456789012345678901234567890")
            .expect("km"), // expect
    );

    // Pre-create the UUID sidecar so both calls race purely on the WAL file open
    Wal::load_or_create_wal_uuid(&wal_path)
        .await
        .expect("create uuid sidecar"); // expect

    let path1 = wal_path.clone();
    let path2 = wal_path.clone();
    let km1 = km.clone();
    let km2 = km.clone();

    let h1 = tokio::spawn(async move { Wal::open_with_key_manager(path1, Some(km1)).await });
    let h2 = tokio::spawn(async move { Wal::open_with_key_manager(path2, Some(km2)).await });

    let (res1, res2) = tokio::join!(h1, h2);
    let wal1 = res1.expect("join 1").expect("open 1"); // expect
    let wal2 = res2.expect("join 2").expect("open 2"); // expect

    // Both WAL instances are opened successfully
    assert_eq!(wal1.path(), wal2.path());
}

#[test]
fn test_legacy_integrity_key_deobfuscation() {
    let key = legacy_integrity_key();
    assert_eq!(&key, b"memfuse-integrity-key-v1\0\0\0\0\0\0\0\0");
}

#[tokio::test]
async fn test_hmac_chain_intact_after_append_failure() {
    let dir = tempdir().expect("tempdir");
    let wal_path = dir.path().join("append_failure.wal");

    let wal = Wal::open(&wal_path).await.expect("open wal");

    // 1. Initial write
    let op1 = WalOp::Put {
        tx_id: TxId::new(1),
        key: b"k1".to_vec(),
        value: b"v1".to_vec(),
    };
    let (batch1, _) = wal
        .prepare_batch(vec![(op1, 1)])
        .await
        .expect("prepare batch 1");
    wal.append_batch(batch1).await.expect("append batch 1");

    let hmac_before = wal.last_hmac_snapshot().await;

    // 2. Prepare a second batch that advances last_hmac
    let op2 = WalOp::Put {
        tx_id: TxId::new(2),
        key: b"k2".to_vec(),
        value: b"v2".to_vec(),
    };
    let (batch2, prev_hmac) = wal
        .prepare_batch(vec![(op2, 2)])
        .await
        .expect("prepare batch 2");
    assert_ne!(
        wal.last_hmac_snapshot().await,
        hmac_before,
        "prepare_batch should advance in-memory last_hmac"
    );
    assert_eq!(prev_hmac, hmac_before);

    // 3. Simulate append failure by replacing file with a read-only file handle
    {
        let ro_file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(&wal_path)
            .await
            .expect("open read-only");
        let mut guard = wal.file.lock().await;
        *guard = ro_file;
    }

    let append_res = wal.append_batch(batch2).await;
    assert!(
        append_res.is_err(),
        "append_batch must fail on read-only file handle"
    );

    // Restore last_hmac as lsm commit would do upon append failure
    wal.restore_last_hmac(prev_hmac)
        .await
        .expect("restore last hmac");

    // 4. Verify last_hmac_snapshot is back to hmac_before
    let hmac_after = wal.last_hmac_snapshot().await;
    assert_eq!(
        hmac_after, hmac_before,
        "last_hmac_snapshot must match value before failed prepare_batch"
    );
}

#[tokio::test]
async fn test_hmac_chain_prev_hash_independent_reference() -> Result<()> {
    let dir = tempdir()?;
    let wal_path = dir.path().join("hmac_ref.wal");
    let wal = Wal::open(&wal_path).await?;

    let key_path = dir.path().join(".wal_integrity_key");
    let integrity_key = tokio::fs::read(&key_path).await?;

    let ops = vec![
        WalOp::Put {
            tx_id: TxId::new(10),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        },
        WalOp::Put {
            tx_id: TxId::new(11),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        },
        WalOp::Delete {
            tx_id: TxId::new(12),
            key: b"k1".to_vec(),
        },
    ];

    let mut current_chain = [0u8; 32];
    for (idx, op) in ops.into_iter().enumerate() {
        let seq_no = (idx + 1) as u64;
        let (batch, _) = wal.prepare_batch(vec![(op.clone(), seq_no)]).await?;
        let entry = &batch.entries()[0];

        let expected_hmac = compute_v3_hmac_reference_independent(
            &integrity_key,
            &current_chain,
            seq_no,
            op.tx_id().inner(),
            &op,
        )?;

        assert_eq!(
            entry.checksum, expected_hmac,
            "WAL entry checksum at seq {} must match independent HMAC-SHA256 reference calculation",
            seq_no
        );
        assert_eq!(
            entry.prev_hmac, current_chain,
            "WAL entry prev_hmac at seq {} must match previous chain digest",
            seq_no
        );

        let checksum = entry.checksum;
        wal.append_batch(batch).await?;
        current_chain = checksum;
    }

    Ok(())
}
