# contextra-crypto

Kryptographische Primitive, AEAD, Schlüsselableitung, Integritätsketten und geheime Container (Klasse A, Ring 0).

## Rolle in einem Satz
`contextra-crypto` stellt I/O-freie kryptographische Primitive, symmetrische/asymmetrische Schlüsselcontainer, Integritätsprüfketten und Nonce-sichere AEAD-Verschlüsselung nach RFC 8452 bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Kryptographie & Kernprimitive, I/O-frei im Kern)
- **Klasse:** Klasse A (Baukasten-Baustein, vollständig autark)
- **Reifegrad:** `stable`
- **Abhängigkeiten:** Keine internen `contextra-*` Abhängigkeiten; exportierbar und autark kompilierbar.
- **Lints:** `#![forbid(unsafe_code)]`, Zero-Panic in Produktionscode (`zero_panic_analyzer`).

## Sicherheitsmodell & Bedrohungsmodell
- **Vertraulichkeit & Authentizität:** AES-256-GCM-SIV (RFC 8452) bietet Nonce-Misuse-Resistance gegen versehentliche Nonce-Wiederverwendung und authentifiziert Header/Metadaten vollständig über Associated Data (AAD).
- **Timing-Orakel-Schutz (INV-CRYPTO-CONSTANT-TIME-1):** Sämtliche kryptographischen Hash-, Schlüssel- und Tag-Vergleiche werden ausnahmslos in konstanter Zeit mittels `subtle::ConstantTimeEq` ausgeführt.
- **Speicherhygiene (INV-CRYPTO-ZEROIZE-1):** Alle Schlüsselstrukturen (`SecretKey`, `Secret`, `VolatileEncryptionKey`, `UnitKey`, `Dek`) implementieren `ZeroizeOnDrop` und werden bei Freigabe deterministisch genullt. Sensible Daten werden in `Debug` und `Display` als `[REDACTED]` maskiert.
- **Nicht-Ziele:** Keine juristische Nachweiserstellung oder DSGVO-Löschketten (diese liegen in `contextra-erasure`).

## Minimalbeispiel (Autarker Einsatz)

```rust
use contextra_crypto::aead::{seal_record_with_nonce, open_record};
use contextra_crypto::dek::Dek;
use contextra_crypto::secret::SecretKey;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dek = Dek::new([0x42u8; 32]);
    let doc_id = [0x11u8; 16];
    let key_meta = b"user_record_key";
    let nonce = [0x99u8; 12];
    let plaintext = b"Sensible Nutzlast";

    // 1. Versiegeln mit AES-256-GCM-SIV und AAD-Bindung
    let envelope = seal_record_with_nonce(&dek, doc_id, key_meta, plaintext, nonce)?;

    // 2. Öffnen und Authentifizieren
    let opened = open_record(&dek, key_meta, &envelope)?;
    assert_eq!(opened.as_slice(), plaintext);

    Ok(())
}
```
