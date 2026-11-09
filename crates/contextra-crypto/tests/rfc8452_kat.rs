//! RFC 8452 Known-Answer-Tests (KAT) for AES-256-GCM-SIV.

use aes_gcm_siv::{
    aead::{Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};

fn hex_decode(hex: &str) -> Vec<u8> {
    let clean = hex.replace(' ', "");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex byte"))
        .collect()
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn test_rfc_8452_aes_256_gcm_siv_kat_vector_1() {
    let key_bytes = hex_decode("0100000000000000000000000000000000000000000000000000000000000000");
    let nonce_bytes = hex_decode("030000000000000000000000");
    let plaintext = b"";
    let expected_ct_tag = hex_decode("07f5f4169bbf55a8400cd47ea6fd400f");

    let cipher = Aes256GcmSiv::new_from_slice(&key_bytes).expect("key init");
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).expect("encrypt");

    assert_eq!(
        hex_encode(&ciphertext),
        hex_encode(&expected_ct_tag),
        "AES-256-GCM-SIV RFC 8452 Vector 1 mismatch!"
    );

    let decrypted = cipher.decrypt(nonce, ciphertext.as_ref()).expect("decrypt");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_rfc_8452_aes_256_gcm_siv_kat_vector_2() {
    let key_bytes = hex_decode("0100000000000000000000000000000000000000000000000000000000000000");
    let nonce_bytes = hex_decode("030000000000000000000000");
    let plaintext = hex_decode("0100000000000000");
    let expected_ct_tag = hex_decode("c2ef328e5c71c83b843122130f7364b761e0b97427e3df28");

    let cipher = Aes256GcmSiv::new_from_slice(&key_bytes).expect("key init");
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_ref()).expect("encrypt");

    assert_eq!(
        hex_encode(&ciphertext),
        hex_encode(&expected_ct_tag),
        "AES-256-GCM-SIV RFC 8452 Vector 2 mismatch!"
    );

    let decrypted = cipher.decrypt(nonce, ciphertext.as_ref()).expect("decrypt");
    assert_eq!(decrypted, plaintext);
}
