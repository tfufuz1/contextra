//! # Eigenständiges Baukasten-Beispiel für `contextra-crypto`
//!
//! Dieses Beispiel demonstriert die vollständig autarke Nutzung von `contextra-crypto`
//! ohne Abhängigkeit zu höheren Contextra-Ringen:
//! 1. KDF-Passphrasenverarbeitung mit Argon2id.
//! 2. Deterministische Nonce-basierte AEAD-Verschlüsselung (AES-256-GCM-SIV).
//! 3. Ableitung dateispezifischer Teilschlüssel (HKDF-Expand).
//! 4. WAL HMAC-Integritätsprüfung.

use contextra_crypto::{derive_key_argon2id, KdfHeader, KdfParams, KeyManager, WalHmac};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== contextra-crypto Standalone Building Block Demo ===");

    // 1. Sichere Passphrasen-Ableitung via Argon2id
    let passphrase = "correct-horse-battery-staple-secure-passphrase";
    let salt = vec![0x42u8; 16];
    let header = KdfHeader::new(
        KdfParams {
            m_cost_kib: 19456, // OWASP-Mindestanforderung für schnelle Tests
            t_cost: 2,
            p_cost: 1,
        },
        salt,
    )?;
    let _derived_key = derive_key_argon2id(passphrase, &header)?;
    println!("✓ Argon2id KDF erfolgreich ausgeführt: 32 Bytes abgeleitet");

    // 2. Initialisierung des KeyManagers und AEAD-Verschlüsselung
    let km = KeyManager::try_new(passphrase, b"random_salt_1234")?;
    let plaintext = b"Autarker kryptographischer Baustein fuer lokale Vektoren und Dokumente";

    let (ciphertext, nonce) = km.encrypt_auto_nonce(plaintext)?;
    println!(
        "✓ Block verschlüsselt: {} Bytes Klartext -> {} Bytes Chiffretext (12-Byte Nonce)",
        plaintext.len(),
        ciphertext.len()
    );

    let decrypted = km.decrypt_auto_nonce(&ciphertext, &nonce)?;
    assert_eq!(plaintext, decrypted.as_slice());
    println!("✓ Entschlüsselung und Integritätsvalidierung verifiziert");

    // 3. Dateispezifische Schlüsselableitung zur Vermeidung von Nonce-Reuse
    let file_km = km.derive_file_key(b"segment_001.sst")?;
    let (file_ct, file_nonce) = file_km.encrypt_auto_nonce(b"SSTable Block 001 Payload")?;
    let file_pt = file_km.decrypt_auto_nonce(&file_ct, &file_nonce)?;
    assert_eq!(b"SSTable Block 001 Payload", file_pt.as_slice());
    println!("✓ Dateispezifischer Teilschlüssel isoliert und verifiziert");

    // 4. WAL HMAC-Integritätsnachweis
    let wal_key = km.integrity_key()?;
    let mut hmac = WalHmac::new(&wal_key)?;
    let entry_data = b"WAL_TX_COMMIT_SEQUENCE_00000042";
    hmac.update(entry_data);
    let mac = hmac.finalize();
    assert_ne!(mac, [0u8; 32]);
    println!("✓ WAL HMAC-Digest berechnet: 32 Bytes Authentifizierung");

    println!("=== Alle Baustein-Primitiven erfolgreich abgeschlossen ===");
    Ok(())
}
