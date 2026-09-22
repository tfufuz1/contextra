// FILE-CONTEXT
// ZWECK: Tests für Argon2id KDF Header, Parameter-Validierung, Parser-Robustheit und KeyManager Integration.
// INVARIANTEN: Zero Panics bei Fuzzing/ungültigen Header-Bytes. Determinismus bei gleichen Eingaben. Unveränderte HKDF-Referenzwerte.

use memfuse_crypto::crypto::KeyManager;
use memfuse_crypto::kdf::{
    derive_key_argon2id, KdfHeader, KdfParams, MIN_M_COST_KIB, MIN_P_COST, MIN_T_COST,
};
use memfuse_crypto::CryptoError;

#[test]
fn test_argon2id_determinism() {
    let params = KdfParams::new_for_test(19456, 2, 1);
    let salt = vec![0x42u8; 32];
    let header = KdfHeader::new(params, salt).expect("header creation");

    let pass = "my-secure-passphrase-123!";
    let key1 = derive_key_argon2id(pass, &header).expect("derive1");
    let key2 = derive_key_argon2id(pass, &header).expect("derive2");

    assert_eq!(
        key1.0, key2.0,
        "Gleiche Passphrase und Header müssen identische Schlüssel liefern"
    );
}

#[test]
fn test_argon2id_different_salt_or_params_yield_different_keys() {
    let params1 = KdfParams::new_for_test(19456, 2, 1);
    let params2 = KdfParams::new_for_test(19456, 3, 1);
    let salt1 = vec![0x11u8; 32];
    let salt2 = vec![0x22u8; 32];

    let header1 = KdfHeader::new(params1.clone(), salt1).expect("h1");
    let header2 = KdfHeader::new(params1, salt2).expect("h2");
    let header3 = KdfHeader::new(params2, vec![0x11u8; 32]).expect("h3");

    let pass = "my-secure-passphrase";
    let k1 = derive_key_argon2id(pass, &header1).expect("k1");
    let k2 = derive_key_argon2id(pass, &header2).expect("k2");
    let k3 = derive_key_argon2id(pass, &header3).expect("k3");

    assert_ne!(k1.0, k2.0, "Anderer Salt muss anderen Schlüssel ergeben");
    assert_ne!(
        k1.0, k3.0,
        "Andere KDF-Parameter müssen anderen Schlüssel ergeben"
    );
}

#[test]
fn test_kdf_header_binary_roundtrip() {
    let header_orig = KdfHeader::generate_default().expect("generate_default");
    let bytes = header_orig.to_bytes();

    let header_parsed = KdfHeader::from_bytes(&bytes).expect("parse_header");
    assert_eq!(header_orig, header_parsed);
}

#[test]
fn test_kdf_header_parser_fuzz_and_corruptions() {
    // 1. Leere Bytes
    assert!(matches!(
        KdfHeader::from_bytes(&[]),
        Err(CryptoError::InvalidInput(_))
    ));

    // 2. Zu kurze Bytes (< 22 Bytes)
    let short_bytes = vec![0u8; 21];
    assert!(matches!(
        KdfHeader::from_bytes(&short_bytes),
        Err(CryptoError::InvalidInput(_))
    ));

    // 3. Falsches Magic
    let mut bad_magic = KdfHeader::generate_default().unwrap().to_bytes();
    bad_magic[0..4].copy_from_slice(b"BADM");
    assert!(matches!(
        KdfHeader::from_bytes(&bad_magic),
        Err(CryptoError::InvalidInput(_))
    ));

    // 4. Unbekannte Version
    let mut bad_ver = KdfHeader::generate_default().unwrap().to_bytes();
    bad_ver[4] = 99;
    assert!(matches!(
        KdfHeader::from_bytes(&bad_ver),
        Err(CryptoError::InvalidInput(_))
    ));

    // 5. Unbekannte KdfId
    let mut bad_kdf = KdfHeader::generate_default().unwrap().to_bytes();
    bad_kdf[5] = 2; // Not Argon2id
    assert!(matches!(
        KdfHeader::from_bytes(&bad_kdf),
        Err(CryptoError::InvalidInput(_))
    ));

    // 6. Truncated Salt
    let mut trunc_salt = KdfHeader::generate_default().unwrap().to_bytes();
    trunc_salt.pop(); // Remove 1 byte from payload
    assert!(matches!(
        KdfHeader::from_bytes(&trunc_salt),
        Err(CryptoError::InvalidInput(_))
    ));
}

#[test]
fn test_kdf_header_exact_22_bytes_boundary_with_zero_salt_len() {
    // Exactly MIN_HEADER_LEN (22 bytes) with salt_len = 0.
    // If bytes.len() < MIN_HEADER_LEN is mutated to <= MIN_HEADER_LEN,
    // a 22-byte slice returns "KdfHeader-Bytes zu kurz" instead of reaching the
    // salt_len < MIN_SALT_LEN check which returns "unter Minimum".
    let mut header_22 = Vec::new();
    header_22.extend_from_slice(b"MFKD");
    header_22.push(1); // version
    header_22.push(1); // kdf_id
    header_22.extend_from_slice(&65536u32.to_be_bytes());
    header_22.extend_from_slice(&3u32.to_be_bytes());
    header_22.extend_from_slice(&1u32.to_be_bytes());
    header_22.extend_from_slice(&0u32.to_be_bytes()); // salt_len = 0
    assert_eq!(header_22.len(), 22);

    let res = KdfHeader::from_bytes(&header_22);
    match res {
        Err(CryptoError::InvalidInput(msg)) => {
            assert!(
                msg.contains("unter Minimum"),
                "Expected 'unter Minimum' from salt_len check, got: {msg}"
            );
        }
        other => panic!("Expected InvalidInput with unter Minimum, got: {:?}", other),
    }
}

#[test]
fn test_kdf_header_salt_len_boundaries() {
    let params = KdfParams::new_for_test(19456, 2, 1);

    // Exact min salt_len boundary (16 bytes)
    let salt_16 = vec![0xABu8; 16];
    let header_16 = KdfHeader::new(params.clone(), salt_16).expect("new 16B salt header");
    let bytes_16 = header_16.to_bytes();
    let parsed_16 = KdfHeader::from_bytes(&bytes_16).expect("from_bytes 16B salt header");
    assert_eq!(parsed_16.salt.len(), 16);

    // Below min salt_len (15 bytes)
    let mut bytes_15 = KdfHeader::generate_default().unwrap().to_bytes();
    bytes_15[18..22].copy_from_slice(&15u32.to_be_bytes());
    let res_15 = KdfHeader::from_bytes(&bytes_15);
    match res_15 {
        Err(CryptoError::InvalidInput(msg)) => {
            assert!(
                msg.contains("unter Minimum"),
                "Expected 'unter Minimum', got: {msg}"
            );
        }
        other => panic!("Expected InvalidInput with unter Minimum, got: {:?}", other),
    }

    // Below min salt_len (10 bytes) - Kills < with == mutant
    let mut bytes_10 = KdfHeader::generate_default().unwrap().to_bytes();
    bytes_10[18..22].copy_from_slice(&10u32.to_be_bytes());
    let res_10 = KdfHeader::from_bytes(&bytes_10);
    match res_10 {
        Err(CryptoError::InvalidInput(msg)) => {
            assert!(
                msg.contains("unter Minimum"),
                "Expected 'unter Minimum', got: {msg}"
            );
        }
        other => panic!("Expected InvalidInput with unter Minimum, got: {:?}", other),
    }

    // Exact max salt_len boundary (10,000 bytes)
    let salt_10k = vec![0xCDu8; 10_000];
    let header_10k = KdfHeader::new(params, salt_10k).expect("new 10k salt header");
    let bytes_10k = header_10k.to_bytes();
    let parsed_10k = KdfHeader::from_bytes(&bytes_10k).expect("from_bytes 10k salt header");
    assert_eq!(parsed_10k.salt.len(), 10_000);

    // Above max salt_len (10,001 bytes)
    let mut bytes_10k1 = Vec::new();
    bytes_10k1.extend_from_slice(b"MFKD");
    bytes_10k1.push(1);
    bytes_10k1.push(1);
    bytes_10k1.extend_from_slice(&65536u32.to_be_bytes());
    bytes_10k1.extend_from_slice(&3u32.to_be_bytes());
    bytes_10k1.extend_from_slice(&1u32.to_be_bytes());
    bytes_10k1.extend_from_slice(&10_001u32.to_be_bytes());
    bytes_10k1.extend_from_slice(&vec![0xEEu8; 10_001]);
    let res_10k1 = KdfHeader::from_bytes(&bytes_10k1);
    match res_10k1 {
        Err(CryptoError::InvalidInput(msg)) => {
            assert!(
                msg.contains("überschreitet Maximum"),
                "Expected 'überschreitet Maximum', got: {msg}"
            );
        }
        other => panic!(
            "Expected InvalidInput with überschreitet Maximum, got: {:?}",
            other
        ),
    }

    // Above max salt_len (10,002 bytes) - Kills > with == mutant
    let mut bytes_10k2 = Vec::new();
    bytes_10k2.extend_from_slice(b"MFKD");
    bytes_10k2.push(1);
    bytes_10k2.push(1);
    bytes_10k2.extend_from_slice(&65536u32.to_be_bytes());
    bytes_10k2.extend_from_slice(&3u32.to_be_bytes());
    bytes_10k2.extend_from_slice(&1u32.to_be_bytes());
    bytes_10k2.extend_from_slice(&10_002u32.to_be_bytes());
    bytes_10k2.extend_from_slice(&vec![0xEEu8; 10_002]);
    let res_10k2 = KdfHeader::from_bytes(&bytes_10k2);
    match res_10k2 {
        Err(CryptoError::InvalidInput(msg)) => {
            assert!(
                msg.contains("überschreitet Maximum"),
                "Expected 'überschreitet Maximum', got: {msg}"
            );
        }
        other => panic!(
            "Expected InvalidInput with überschreitet Maximum, got: {:?}",
            other
        ),
    }
}

#[test]
fn test_kdf_params_lower_bounds_validation() {
    // Untergrenzen: m_cost >= 19456, t_cost >= 2, p_cost >= 1
    assert!(KdfParams::new(MIN_M_COST_KIB - 1, MIN_T_COST, MIN_P_COST).is_err());
    assert!(KdfParams::new(MIN_M_COST_KIB, MIN_T_COST - 1, MIN_P_COST).is_err());
    assert!(KdfParams::new(MIN_M_COST_KIB, MIN_T_COST, MIN_P_COST - 1).is_err());

    let valid = KdfParams::new(MIN_M_COST_KIB, MIN_T_COST, MIN_P_COST);
    assert!(valid.is_ok());
}

#[test]
fn test_empty_passphrase_fails() {
    let params = KdfParams::new_for_test(19456, 2, 1);
    let header = KdfHeader::new(params, vec![0x42; 32]).unwrap();

    let res = derive_key_argon2id("", &header);
    assert!(matches!(res, Err(CryptoError::InvalidInput(_))));

    let km_res = KeyManager::try_new_with_kdf("", &header);
    assert!(matches!(km_res, Err(CryptoError::InvalidInput(_))));
}

#[test]
fn test_key_manager_try_new_with_kdf_encryption_roundtrip() {
    let params = KdfParams::new_for_test(19456, 2, 1);
    let header = KdfHeader::new(params, vec![0x99; 32]).unwrap();
    let pass = "passphrase-for-km-argon2id";

    let km = KeyManager::try_new_with_kdf(pass, &header).expect("km creation");
    let plaintext = b"Geheimer Datenbank-Eintrag fuer MemFuse";

    let (ciphertext, nonce) = km.encrypt_auto_nonce(plaintext).expect("encrypt");
    let decrypted = km.decrypt_auto_nonce(&ciphertext, &nonce).expect("decrypt");

    assert_eq!(plaintext, decrypted.as_slice());
}
