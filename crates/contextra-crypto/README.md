# contextra-crypto

Autarker Rust-Baustein für symmetrische und asymmetrische kryptographische Primitive, AEAD, KDF, Schlüsselverwaltung und Integritätsketten.

Normativitätsklasse: **Klasse A** (Baukasten-Baustein, Ring 0).

## Rolle & Zweck
`contextra-crypto` stellt auditierbare, I/O-freie kryptographische Primitiven, Schlüsselverwaltung mit `ZeroizeOnDrop`, und Anti-Tamper Integritätsketten für LLM-/RAG-Software bereit.

## Sicherheitsmodell
- **Bedrohungsmodell:** Schutz von Daten-at-Rest und In-Memory-Schlüsseln vor unbefugtem Auslesen, Kaltstart-Angriffen (Memory Dumps), Nonce-Misuse-Angriffen, Replay/Tampering in Logs und Timing-Seitenkanalangriffen.
- **Nicht-Ziele:** Keine juristischen Löschnachweise oder DSGVO-Sagas (diese gehören nach `contextra-erasure`). Keine Authentifizierungs-Token-Infrastruktur oder Netzwerk-Protokollierung.

## Garantien & Invarianten
- **Keine Eigenbauten:** Ausschließlich geprüfte RustCrypto-Crates (`aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3`).
- **`INV-CRYPTO-ZEROIZE-1`:** Alle Schlüsselstrukturen (`SecretKey`, `UnitKey`, `Dek`, `VolatileEncryptionKey`) werden beim Verlassen des Gültigkeitsbereichs (`Drop`) deterministisch im RAM genullt.
- **`INV-CRYPTO-CONSTANT-TIME-1`:** Alle Vergleiche von MACs, Tags und Schlüsselbytes erfolgen in konstanter Zeit mittels `subtle::ConstantTimeEq`.
- **Zero-Panic:** 0 Panics (`unwrap`, `expect`, etc.) im Produktionscode.

## Copy-Paste-Anleitung & Autarker Export
`contextra-crypto` besitzt keine internen Workspace-Abhängigkeiten und kann direkt exportiert werden:
```bash
python3 tools/export_crate.py contextra-crypto --out /tmp/export-contextra-crypto
```

## Minimalbeispiel

```rust
use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::dek::Dek;
use contextra_crypto::secret::SecretKey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Schlüsselcontainer mit Zeroize-on-Drop
    let secret = SecretKey::new([0x42u8; 32]);
    let dek = Dek::new(secret);

    // 2. Authentifizierte AEAD-Verschlüsselung (RFC 8452)
    let doc_tag = [0xAAu8; 16];
    let key_bytes = b"sample_record_key";
    let plaintext = b"Sensibler Kontext für RAG-Pipeline";
    let nonce = [0x01u8; 12];

    let envelope = seal_record_with_nonce(&dek, doc_tag, key_bytes, plaintext, nonce);

    // 3. Entschlüsselung mit AAD-Authentifizierung
    let decrypted = open_record(&dek, key_bytes, &envelope)?;
    assert_eq!(decrypted.as_slice(), plaintext);

    println!("Erfolgreich verschlüsselt und entschlüsselt.");
    Ok(())
}
```
