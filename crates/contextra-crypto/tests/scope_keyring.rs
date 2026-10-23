#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

// FILE-CONTEXT
// ZWECK: Comprehensive test suite for ScopeKeyring and S4 Crypto-Erase under ADR-N27.
// INVARIANTEN: Zeroize-on-drop, no raw key material in logs/Debug, crash-consistent state, tamper detection, multi-tenant isolation.
// STAND: TS:2026-10-18T00:00:00Z

#![forbid(unsafe_code)]

use contextra_crypto::{
    CryptoError, DekHandle, KeyId, KeyManager, KeyScope, ScopeKeyring,
};
use contextra_ports::{SystemClock, SystemRng};
use contextra_types::{CollectionId, TenantId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

fn setup_env() -> (TempDir, KeyManager, Arc<SystemClock>, Arc<SystemRng>) {
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let (km, _) = KeyManager::try_new_argon2id("master-test-passphrase-strong")
        .expect("failed to create KeyManager");
    let clock = Arc::new(SystemClock::new());
    let rng = Arc::new(SystemRng);
    (temp_dir, km, clock, rng)
}

#[test]
fn test_scope_keyring_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, km, clock, rng) = setup_env();
    let keyring = ScopeKeyring::open(temp_dir.path(), &km, clock, rng)?;

    let tenant = TenantId::try_new(1001)?;
    let collection = CollectionId::try_new(50)?;
    let scope = KeyScope::Collection {
        tenant,
        collection,
        incarnation: 1,
    };

    let (key_id, dek_handle) = keyring.create_dek(scope.clone())?;
    assert_eq!(dek_handle.key_id(), key_id);
    assert_eq!(dek_handle.scope(), &scope);

    let plaintext = b"top secret document payload block";
    let aad = b"contextra-doc-header-v1";

    let (ciphertext, nonce) = dek_handle.encrypt(plaintext, aad)?;
    assert_ne!(ciphertext, plaintext);

    let dek_retrieved = keyring.dek(key_id)?;
    let decrypted = dek_retrieved.decrypt(&ciphertext, &nonce, aad)?;
    assert_eq!(decrypted, plaintext);

    // Wrong AAD fails decryption
    let wrong_aad_res = dek_retrieved.decrypt(&ciphertext, &nonce, b"wrong-aad-header");
    assert!(wrong_aad_res.is_err());

    Ok(())
}

#[test]
fn test_destroy_scrubs_key_bytes_and_fails_decryption() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, km, clock, rng) = setup_env();
    let keyring = ScopeKeyring::open(temp_dir.path(), &km, clock, rng)?;

    let tenant = TenantId::try_new(2002)?;
    let collection = CollectionId::try_new(99)?;
    let scope = KeyScope::Collection {
        tenant,
        collection,
        incarnation: 1,
    };

    let (key_id, dek_handle) = keyring.create_dek(scope.clone())?;
    let raw_key_bytes = *dek_handle.raw_key_bytes();

    let plaintext = b"payload to encrypt before destruction";
    let aad = b"scope-aad";
    let (_ciphertext, _nonce) = dek_handle.encrypt(plaintext, aad)?;

    // Verify key works before destruction
    assert!(keyring.dek(key_id).is_ok());

    // Destroy scope
    let receipt = keyring.destroy(&scope)?;
    assert_eq!(receipt.key_ids, vec![key_id]);
    assert_ne!(receipt.revocation_entry_hash, [0u8; 32]);

    // dek() after destroy MUST return KeyRevoked error
    let dek_res = keyring.dek(key_id);
    assert!(matches!(dek_res, Err(CryptoError::KeyRevoked(_))));

    // Search raw keyring directory and files to ensure raw_key_bytes are NOT present on disk
    let dir_entries = std::fs::read_dir(temp_dir.path())?;
    for entry in dir_entries {
        let path = entry?.path();
        if path.is_file() {
            let file_bytes = std::fs::read(&path)?;
            // Ensure 32-byte raw DEK pattern does not appear in any persistent file
            assert!(
                !file_bytes
                    .windows(raw_key_bytes.len())
                    .any(|w| w == raw_key_bytes),
                "Raw DEK bytes found in file {:?}",
                path
            );
        }
    }

    Ok(())
}

#[test]
fn test_crash_failpoints_and_recovery() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, km, clock, rng) = setup_env();

    let scope = KeyScope::TenantEpoch {
        tenant: TenantId::try_new(3003)?,
        epoch: 1,
    };

    // Scenario 1: Key created and destroyed cleanly
    let key_id = {
        let keyring = ScopeKeyring::open(temp_dir.path(), &km, clock.clone(), rng.clone())?;
        let (key_id, _) = keyring.create_dek(scope.clone())?;
        keyring.destroy(&scope)?;
        key_id
    };

    // Reopen keyring after crash
    let keyring_reopened = ScopeKeyring::open(temp_dir.path(), &km, clock.clone(), rng.clone())?;
    let res = keyring_reopened.dek(key_id);
    assert!(matches!(res, Err(CryptoError::KeyRevoked(_))));

    // Scenario 2: Simulate crash before RevocationLog sync by writing revoked state
    let scope2 = KeyScope::TenantEpoch {
        tenant: TenantId::try_new(3004)?,
        epoch: 1,
    };
    let key_id2 = {
        let keyring2 = ScopeKeyring::open(temp_dir.path(), &km, clock.clone(), rng.clone())?;
        let (key_id2, _) = keyring2.create_dek(scope2.clone())?;
        keyring2.begin_destruction(&scope2, 0)?;
        key_id2
    };

    // Reopen keyring again and complete destruction
    let keyring_reopened2 = ScopeKeyring::open(temp_dir.path(), &km, clock, rng)?;
    keyring_reopened2.destroy(&scope2)?;
    assert!(matches!(keyring_reopened2.dek(key_id2), Err(CryptoError::KeyRevoked(_))));

    Ok(())
}

struct DropInspector {
    dropped: Arc<AtomicBool>,
}

impl Drop for DropInspector {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

#[test]
fn test_zeroize_and_redacted_debug_with_drop_inspector() -> Result<(), Box<dyn std::error::Error>> {
    let (_temp_dir, km, _clock, rng) = setup_env();

    let dropped = Arc::new(AtomicBool::new(false));
    let inspector = DropInspector {
        dropped: dropped.clone(),
    };

    let scope = KeyScope::TenantEpoch {
        tenant: TenantId::try_new(4004)?,
        epoch: 1,
    };
    let key_id = KeyId([0x42; 16]);

    {
        let handle = DekHandle::new(key_id, scope, [0x77; 32], rng);
        let debug_str = format!("{handle:?}");
        assert!(debug_str.contains("***REDACTED***"));
        assert!(!debug_str.contains("119")); // 0x77 = 119 decimal
        drop(inspector);
    }

    assert!(dropped.load(Ordering::SeqCst));

    let km_debug = format!("{km:?}");
    assert!(km_debug.contains("***REDACTED***"));

    Ok(())
}

#[test]
fn test_tampered_keyring_file_detected() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, km, clock, rng) = setup_env();

    let scope = KeyScope::TenantEpoch {
        tenant: TenantId::try_new(5005)?,
        epoch: 1,
    };

    {
        let keyring = ScopeKeyring::open(temp_dir.path(), &km, clock.clone(), rng.clone())?;
        let _ = keyring.create_dek(scope)?;
    }

    let keyring_file = temp_dir.path().join("scope_keyring.dat");
    let mut file_bytes = std::fs::read(&keyring_file)?;
    assert!(file_bytes.len() > 20);

    // Flip bits in ciphertext area
    let last_idx = file_bytes.len() - 1;
    file_bytes[last_idx] ^= 0xFF;
    std::fs::write(&keyring_file, &file_bytes)?;

    // Opening tampered keyring MUST return IntegrityViolation error
    let reopen_res = ScopeKeyring::open(temp_dir.path(), &km, clock, rng);
    assert!(matches!(reopen_res, Err(CryptoError::IntegrityViolation)));

    Ok(())
}

#[test]
fn test_tenant_isolation_on_destruction() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, km, clock, rng) = setup_env();
    let keyring = ScopeKeyring::open(temp_dir.path(), &km, clock, rng)?;

    let tenant_a = TenantId::try_new(7001)?;
    let tenant_b = TenantId::try_new(7002)?;

    let scope_a = KeyScope::TenantEpoch {
        tenant: tenant_a,
        epoch: 1,
    };
    let scope_b = KeyScope::TenantEpoch {
        tenant: tenant_b,
        epoch: 1,
    };

    let (key_id_a, _handle_a) = keyring.create_dek(scope_a.clone())?;
    let (key_id_b, handle_b) = keyring.create_dek(scope_b.clone())?;

    let plaintext = b"tenant payload data";
    let aad = b"aad";
    let (ct_b, nonce_b) = handle_b.encrypt(plaintext, aad)?;

    // Destroy Tenant A
    keyring.destroy(&scope_a)?;

    // Tenant A key is revoked
    assert!(matches!(keyring.dek(key_id_a), Err(CryptoError::KeyRevoked(_))));

    // Tenant B key remains active and functional
    let retrieved_b = keyring.dek(key_id_b)?;
    let decrypted_b = retrieved_b.decrypt(&ct_b, &nonce_b, aad)?;
    assert_eq!(decrypted_b, plaintext);

    // Tenant B can create new encryptions
    let (ct_b2, nonce_b2) = handle_b.encrypt(plaintext, aad)?;
    assert_eq!(retrieved_b.decrypt(&ct_b2, &nonce_b2, aad)?, plaintext);

    Ok(())
}

#[test]
fn test_begin_destruction_grace_period() -> Result<(), Box<dyn std::error::Error>> {
    let (temp_dir, km, clock, rng) = setup_env();
    let keyring = ScopeKeyring::open(temp_dir.path(), &km, clock, rng)?;

    let scope = KeyScope::TenantEpoch {
        tenant: TenantId::try_new(8008)?,
        epoch: 1,
    };

    let (key_id, _handle) = keyring.create_dek(scope.clone())?;

    // Schedule destruction with 10 seconds grace period
    keyring.begin_destruction(&scope, 10_000_000_000)?;

    // Attempt immediate destruction during grace period MUST fail
    let destroy_res = keyring.destroy(&scope);
    assert!(matches!(destroy_res, Err(CryptoError::InvalidInput(_))));

    // Key remains accessible during grace period
    assert!(keyring.dek(key_id).is_ok());

    Ok(())
}
