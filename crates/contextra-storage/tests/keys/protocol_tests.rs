//! # KEYS Protokoll Tests
//!
//! SPEC §9.3:
//! Verifiziert Serialisierung, Deserialisierung, Verschlüsselung und Integrität der KEYS-Datei.

use contextra_crypto_ref::{Blake3Kdf, ChaChaCipher};
use contextra_storage::env::RealEntropy;
use contextra_storage::keys::protocol::*;
use contextra_storage::keys::secret::Secret32;

#[test]
fn test_keys_plain_encode_decode_roundtrip() {
    let root = Secret32::new([0x11u8; 32]);
    let dek1 = Secret32::new([0x22u8; 32]);
    let dek2 = Secret32::new([0x33u8; 32]);

    let plain = KeysPlain {
        root_key: root,
        space_deks: vec![
            SpaceDekRecord {
                space: 1,
                dek_gen: 10,
                dek: dek1,
            },
            SpaceDekRecord {
                space: 2,
                dek_gen: 20,
                dek: dek2,
            },
        ],
    };

    let encoded = plain.encode();
    assert_eq!(encoded.len(), 34 + 2 * 38);

    let decoded = KeysPlain::decode(&encoded).expect("Decode plain must succeed");
    assert_eq!(decoded.root_key.expose(), &[0x11u8; 32]);
    assert_eq!(decoded.space_deks.len(), 2);
    assert_eq!(decoded.space_deks[0].space, 1);
    assert_eq!(decoded.space_deks[0].dek_gen, 10);
    assert_eq!(decoded.space_deks[0].dek.expose(), &[0x22u8; 32]);
    assert_eq!(decoded.space_deks[1].space, 2);
    assert_eq!(decoded.space_deks[1].dek_gen, 20);
    assert_eq!(decoded.space_deks[1].dek.expose(), &[0x33u8; 32]);
}

#[test]
fn test_keys_file_seal_open_roundtrip() {
    let kdf = Blake3Kdf;
    let cipher = ChaChaCipher;
    let entropy = RealEntropy;

    let kek = Secret32::new([0xAAu8; 32]);
    let instance_id = [0x77u8; 16];
    let keys_generation = 42;

    let root = Secret32::new([0xBBu8; 32]);
    let dek = Secret32::new([0xCCu8; 32]);
    let plain = KeysPlain {
        root_key: root,
        space_deks: vec![SpaceDekRecord {
            space: 0,
            dek_gen: 1,
            dek,
        }],
    };

    let keys_file = KeysFile::seal(
        &plain,
        &kek,
        &instance_id,
        keys_generation,
        &kdf,
        &cipher,
        &entropy,
    )
    .expect("Seal must succeed");

    // Serialisiere zu Datei-Bytes
    let file_bytes = keys_file.encode();
    assert!(file_bytes.starts_with(KEYS_MAGIC));

    // Deserialisiere aus Datei-Bytes
    let loaded_file = KeysFile::decode(&file_bytes).expect("Decode keys file must succeed");
    assert_eq!(loaded_file.version, KEYS_VERSION_1);
    assert_eq!(loaded_file.keys_generation, keys_generation);

    // Entschlüssele
    let opened_plain = loaded_file
        .open(&kek, &instance_id, &kdf, &cipher)
        .expect("Open must succeed");

    assert_eq!(opened_plain.root_key.expose(), &[0xBBu8; 32]);
    assert_eq!(opened_plain.space_deks.len(), 1);
    assert_eq!(opened_plain.space_deks[0].dek.expose(), &[0xCCu8; 32]);
}

#[test]
fn test_keys_file_tampering_fails() {
    let kdf = Blake3Kdf;
    let cipher = ChaChaCipher;
    let entropy = RealEntropy;

    let kek = Secret32::new([0xAAu8; 32]);
    let instance_id = [0x77u8; 16];

    let plain = KeysPlain {
        root_key: Secret32::new([0x12u8; 32]),
        space_deks: Vec::new(),
    };

    let mut keys_file =
        KeysFile::seal(&plain, &kek, &instance_id, 1, &kdf, &cipher, &entropy).unwrap();

    // Manipulation der Payload
    if let Some(first) = keys_file.sealed_payload.first_mut() {
        *first ^= 0xFF;
    }

    let open_res = keys_file.open(&kek, &instance_id, &kdf, &cipher);
    assert!(
        matches!(open_res, Err(KeysError::Cipher(_))),
        "Manipulierte Payload muss mit KeysError::Cipher scheitern"
    );
}
