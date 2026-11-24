//! Standalone Minimalbeispiel für `contextra-crypto` (SPEC R2/R3 Lieferung).
use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::dek::Dek;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dek = Dek::new([0x42u8; 32]);
    let doc_id = [0x11u8; 16];
    let key_meta = b"user_record_key";
    let nonce = [0x99u8; 12];
    let plaintext = b"Sensible Nutzlast";

    let envelope = seal_record_with_nonce(&dek, doc_id, key_meta, plaintext, nonce)?;
    let opened = open_record(&dek, key_meta, &envelope)?;
    assert_eq!(opened.as_slice(), plaintext);
    println!("Standalone AEAD encrypt/decrypt roundtrip successful!");
    Ok(())
}
