# contextra-crypto

Kryptographische Primitive, AEAD, Schlüsselableitung, WAL-Integritätsketten und sichere Schlüsselcontainer.

## Rolle in einem Satz
`contextra-crypto` stellt I/O-freie kryptographische Primitive, symmetrische Schlüsselcontainer, Integritätsketten für WAL und KV-Segmentverschlüsselung bereit.

## Profil
- **Zweck:** Authentifizierte Blockverschlüsselung (AES-256-GCM-SIV), KDF (Argon2id, HKDF), WAL-HMAC-Integritätsprüfung, KV-Segment-Verschlüsselung.
- **Abhängigkeiten:** `contextra-ports`, `contextra-types`.
- **Lints:** `#![forbid(unsafe_code)]`, Zero-Panic in Produktionscode.
- **Features:** `kv-encryption` (Standard), `test-utils`, `cloud-egress-guard`.
- **Schlanke Defaults:** `default = ["kv-encryption"]`. `ed25519-dalek` und Beweis-/Nachweismodule wurden nach `archive/contextra-crypto-proofs/` ausgelagert.

## Sicherheitsmodell
- **Vertraulichkeit & Authentizität:** AES-256-GCM-SIV (RFC 8452) bietet Nonce-Misuse-Resistance gegen versehentliche Nonce-Wiederverwendung und authentifiziert Header/Metadaten vollständig über Associated Data (AAD).
- **Timing-Schutz:** Sämtliche Hash-, Schlüssel- und Tag-Vergleiche werden in konstanter Zeit mittels `subtle::ConstantTimeEq` ausgeführt.
- **Speicherhygiene:** Alle Schlüsselstrukturen (`SecretKey`, `Secret`, `VolatileEncryptionKey`, `UnitKey`, `Dek`) implementieren `ZeroizeOnDrop` und werden bei Freigabe deterministisch genullt. Sensible Daten werden in `Debug` und `Display` als `[REDACTED]` maskiert.
- **Nicht-Ziele:** Keine Nachweiserstellung für Löschungen (diese liegen in `contextra-erasure`).

## Minimalbeispiel

```rust
use contextra_crypto::aead::{seal_record_with_nonce, open_record};
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

    Ok(())
}
```
