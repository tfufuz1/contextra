#![cfg(feature = "legacy-erasure")]
//! # Integrationstest: `crypto_vectors_extended.rs`
//!
//! Validiert die kryptographischen Invarianten und Sicherheitsanforderungen für `crypto_vectors_extended`.
//! Prüft Zero-Panic, deterministische Ausführung und Fail-Closed-Verhalten.

// FILE-CONTEXT
// ZWECK: Kryptografische Validierung gegen offizielle RFC-/NIST-Vektoren und herstellerunabhängige Standardwerte.
// INVARIANTEN: Keine selbst berechneten Soll-Werte; alle Erwartungswerte entstammen offiziellen Spezifikationen (RFC 8032, RFC 5869, RFC 4231, RFC 8452, RFC 9106, BLAKE3 specification).
// HOTSPOTS: [Ed25519, HKDF-SHA256, HMAC-SHA256, AES-256-GCM-SIV, Argon2id, BLAKE3]

#![allow(clippy::unwrap_used, clippy::expect_used)]

use aes_gcm_siv::{
    aead::{Aead, KeyInit as AeadKeyInit},
    Aes256GcmSiv, Nonce,
};
use contextra_crypto::{
    derive_key_argon2id,
    ed25519_proof::{sign_deletion_proof_v3, verify_deletion_proof_v3, DeletionProofKeyPair},
    KdfHeader, KdfParams,
};
use hkdf::Hkdf;
use hmac::{digest::KeyInit, Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn hex_decode(hex: &str) -> Vec<u8> {
    let clean = hex.replace([' ', '\n'], "");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex byte"))
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// -----------------------------------------------------------------------------
// 1. Ed25519 RFC 8032 Abschnitt 7.1 Test Cases 1, 2, 3
// Contextra-Anbindung über contextra_crypto::ed25519_proof
// -----------------------------------------------------------------------------

#[test]
fn test_rfc_8032_ed25519_vector_1() {
    // RFC 8032 Section 7.1 Test Vector 1
    // Secret key seed: 9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60
    // Public key: d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a
    // Message: (empty)
    // Signature: e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b

    let secret_seed =
        hex_decode("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60");
    let expected_pubkey =
        hex_decode("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a");
    let message = b"";
    let expected_sig = hex_decode(
        "e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155\
         5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b",
    );

    let signing_key =
        ed25519_dalek::SigningKey::from_bytes(&secret_seed.as_slice().try_into().unwrap());
    let verifying_key = signing_key.verifying_key();
    let keypair = DeletionProofKeyPair {
        signing_key,
        verifying_key,
    };

    assert_eq!(
        hex_encode(&keypair.verifying_key_bytes()),
        hex_encode(&expected_pubkey),
        "Ed25519 RFC 8032 Vector 1 public key mismatch"
    );

    let sig = sign_deletion_proof_v3(&keypair, message);
    assert_eq!(
        hex_encode(&sig),
        hex_encode(&expected_sig),
        "Ed25519 RFC 8032 Vector 1 signature mismatch"
    );

    let pubkey_arr: [u8; 32] = expected_pubkey.as_slice().try_into().unwrap();
    let sig_arr: [u8; 64] = expected_sig.as_slice().try_into().unwrap();

    // Verify OK
    assert!(verify_deletion_proof_v3(&pubkey_arr, message, &sig_arr).is_ok());

    // Tampered Message -> Fail
    assert!(verify_deletion_proof_v3(&pubkey_arr, b"tampered message", &sig_arr).is_err());

    // Tampered Signature -> Fail
    let mut tampered_sig = sig_arr;
    tampered_sig[0] ^= 0xff;
    assert!(verify_deletion_proof_v3(&pubkey_arr, message, &tampered_sig).is_err());

    // Tampered Public Key -> Fail
    let mut tampered_pk = pubkey_arr;
    tampered_pk[0] ^= 0xff;
    assert!(verify_deletion_proof_v3(&tampered_pk, message, &sig_arr).is_err());
}

#[test]
fn test_rfc_8032_ed25519_vector_2() {
    // RFC 8032 Section 7.1 Test Vector 2
    // Secret key seed: 4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb
    // Public key: 3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c
    // Message: 72 (1 byte 'r')
    // Signature: 92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00

    let secret_seed =
        hex_decode("4ccd089b28ff96da9db6c346ec114e0f5b8a319f35aba624da8cf6ed4fb8a6fb");
    let expected_pubkey =
        hex_decode("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c");
    let message = hex_decode("72");
    let expected_sig = hex_decode(
        "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da\
         085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
    );

    let signing_key =
        ed25519_dalek::SigningKey::from_bytes(&secret_seed.as_slice().try_into().unwrap());
    let verifying_key = signing_key.verifying_key();
    let keypair = DeletionProofKeyPair {
        signing_key,
        verifying_key,
    };

    assert_eq!(
        hex_encode(&keypair.verifying_key_bytes()),
        hex_encode(&expected_pubkey)
    );

    let sig = sign_deletion_proof_v3(&keypair, &message);
    assert_eq!(hex_encode(&sig), hex_encode(&expected_sig));

    let pubkey_arr: [u8; 32] = expected_pubkey.as_slice().try_into().unwrap();
    let sig_arr: [u8; 64] = expected_sig.as_slice().try_into().unwrap();
    assert!(verify_deletion_proof_v3(&pubkey_arr, &message, &sig_arr).is_ok());
}

#[test]
fn test_rfc_8032_ed25519_vector_3() {
    // RFC 8032 Section 7.1 Test Vector 3
    // Secret key seed: c1088412339a38c6211751d7253303c733f1ff7677d332616a9a9578ef2b8df1
    // Public key: 22fab83b1d5312cfd9fe6c4647fcfbb64affe431c4bd01afb84007ea8242f617
    // Message: af82 (2 bytes)
    // Signature: 0054d292a4b4dd009abc33482131da694853bd15eb163960876ddd6637783f1508033b348e5b99ab10b54e3edf33dadf27de94360fd976a113d0032d2a60df0c

    let secret_seed =
        hex_decode("c1088412339a38c6211751d7253303c733f1ff7677d332616a9a9578ef2b8df1");
    let expected_pubkey =
        hex_decode("22fab83b1d5312cfd9fe6c4647fcfbb64affe431c4bd01afb84007ea8242f617");
    let message = hex_decode("af82");
    let expected_sig = hex_decode(
        "0054d292a4b4dd009abc33482131da694853bd15eb163960876ddd6637783f15\
         08033b348e5b99ab10b54e3edf33dadf27de94360fd976a113d0032d2a60df0c",
    );

    let signing_key =
        ed25519_dalek::SigningKey::from_bytes(&secret_seed.as_slice().try_into().unwrap());
    let verifying_key = signing_key.verifying_key();
    let keypair = DeletionProofKeyPair {
        signing_key,
        verifying_key,
    };

    assert_eq!(
        hex_encode(&keypair.verifying_key_bytes()),
        hex_encode(&expected_pubkey)
    );

    let sig = sign_deletion_proof_v3(&keypair, &message);
    assert_eq!(hex_encode(&sig), hex_encode(&expected_sig));

    let pubkey_arr: [u8; 32] = expected_pubkey.as_slice().try_into().unwrap();
    let sig_arr: [u8; 64] = expected_sig.as_slice().try_into().unwrap();
    assert!(verify_deletion_proof_v3(&pubkey_arr, &message, &sig_arr).is_ok());
}

// -----------------------------------------------------------------------------
