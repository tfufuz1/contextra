# contextra-crypto

Kryptographische Vaults, Zeroize, Egress-Verschlüsselung, Audit-Ketten und kryptographische Attestationen.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, §20, `capabilities.toml`).

## Rolle in einem Satz
`contextra-crypto` stellt I/O-freie kryptographische Primitiven, Schlüsselverwaltung, Zeroization, Audit-Chains und kryptographische Attestationssignaturen bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Kryptographie & Kernprimitiven, I/O-frei im Kern)
- **Klasse:** Klasse A (Baukasten-Baustein)
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_depend_on`):** Soll: `[]`; transitional: `contextra-types`, `contextra-ports` (SPEC v4.0 §5)
- **Lints:** `#![forbid(unsafe_code)]`, Zero-Panic in Produktionscode

## Baukasten-Profil & Garantien (Klasse A)
- **Copy-Paste-Readiness:** Exportierbar und autark nutzbar via `python3 tools/export_crate.py contextra-crypto`.
- **Keine Eigenbauten (EP12):** Ausschließlich erprobte Standardprimitive (`aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `ed25519-dalek`, `blake3`, `sha2`).
- **Härtung & Konstantzeit:** Konstantzeit-Vergleiche (`subtle`), kein Klartext-Schlüssel-Leak in Debug/Logs, Zeroize-on-Drop auf allen Schlüsselspeichern.
- **Entkopplung der Löschlogik (D-E1):** Löschketten- und juristische DSGVO-Nachweise verbleiben exklusiv in `contextra-erasure`. Historische Re-Exports in `crypto` sind hinter dem Feature `legacy-erasure` gebunden und als `#[deprecated]` markiert.

## Einbettungsbeispiel (Autarker Einsatz)

```rust
use contextra_crypto::{KeyManager, KdfHeader, KdfParams, derive_key_argon2id, WalHmac};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let passphrase = "safe-secure-passphrase";
    let salt = vec![0x42u8; 16];

    // 1. Sichere KDF mit Argon2id
    let header = KdfHeader::new(KdfParams::default(), salt)?;
    let _key = derive_key_argon2id(passphrase, &header)?;

    // 2. Schlüsselmanager & Nonce-sichere AEAD (AES-256-GCM-SIV)
    let km = KeyManager::try_new(passphrase, b"salt-123")?;
    let (ciphertext, nonce) = km.encrypt_auto_nonce(b"Vertrauliche Daten")?;
    let plaintext = km.decrypt_auto_nonce(&ciphertext, &nonce)?;
    assert_eq!(plaintext, b"Vertrauliche Daten");

    // 3. Dateispezifischer Teilschlüssel (HKDF-Expand)
    let file_km = km.derive_file_key(b"segment_001.sst")?;
    let (file_ct, file_nonce) = file_km.encrypt_auto_nonce(b"SSTable Block")?;
    let file_pt = file_km.decrypt_auto_nonce(&file_ct, &file_nonce)?;
    assert_eq!(file_pt, b"SSTable Block");

    Ok(())
}
```

Ein ausführbares Minimalbeispiel befindet sich unter `examples/crypto_standalone.rs` und kann direkt mit `cargo run -p contextra-crypto --example crypto_standalone` gestartet werden.
