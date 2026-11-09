# Spezifikation: `contextra-crypto` (Ring 0 / Klasse A)

## 1. Zweck und Baukasten-Profil
`contextra-crypto` ist ein autarker, kleiner und auditierbarer Rust-Baustein symmetrischer und asymmetrischer Primitive für LLM-/RAG-Software.
Sie implementiert ausschließlich geprüfte RustCrypto-Bibliotheken und bietet:
- **AEAD:** AES-256-GCM-SIV (RFC 8452) mit Associated Data und Nonce-Disziplin (injizierter `Rng` oder explizite Nonce).
- **DEK:** Data-Encryption-Key-Verwaltung mit `ZeroizeOnDrop` (`INV-CRYPTO-ZEROIZE-1`).
- **Secret:** Typisierte Geheimnis-Container (`SecretKey`) ohne Klartext-Debug/Display, Zugriff via `expose_secret()`.
- **KDF:** Argon2id (RFC 9106) für Passphrasen-Ableitung und HKDF-Expand (RFC 5869 / HMAC-SHA256) mit Mindestkosten-Validierung.
- **Anti-Tamper & Integritätsketten:** BLAKE3- und HMAC-basierte Integritätsketten mit Konstantzeit-Vergleichen (`INV-CRYPTO-CONSTANT-TIME-1`).
- **WAL- & KV-Cipher:** Versionierte Record- und Block-Verschlüsselung mit Bindung von Kontext und Namespace in die AAD.

### Nicht-Ziele
- Keine Löschnachweise, Schichtenscanner, Zeugen-Aggregation oder DSGVO/GDPR-Fachlogik. Diese Komponenten gehören in `contextra-erasure` (SPEC §6.2).
- Keine Mandanten- oder Transaktionskonzepte im Kern (Klasse-A Isolation).
- Keine Eigenbau-Primitive; Zufall ausschließlich über injizierten `Rng` bzw. dokumentierte OS-Entropie (`OsRng`).

---

## 2. Invarianten und Verifikations-Zuordnung

| Invariante | Beschreibung | Testnachweis |
|---|---|---|
| `INV-CRYPTO-NO-CUSTOM-PRIMITIVES` | Nur erprobte RustCrypto-Bibliotheken (`aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3`). | Audit & `Cargo.toml` |
| `INV-CRYPTO-ZEROIZE-1` | Sämtliche Schlüsselstrukturen (`SecretKey`, `UnitKey`, `Dek`, `VolatileEncryptionKey`) und Zwischenpuffer werden beim Drop genullt. | `tests/invariants_and_proptest.rs::test_inv_crypto_zeroize_1_*` |
| `INV-CRYPTO-CONSTANT-TIME-1` | Alle Schlüssel-, Tag- und Prüfsummenvergleiche erfolgen in konstanter Zeit mittels `subtle::ConstantTimeEq`. | `tests/invariants_and_proptest.rs::test_inv_crypto_constant_time_1_tag_comparison` |
| `INV-CRYPTO-NONCE-DISCIPLINE` | Nonce-Disziplin für AEAD; Nonce wird vom Aufrufer injiziert oder zählerbasiert erzeugt. | `src/aead.rs`, `tests/invariants_and_proptest.rs` |
| `INV-CRYPTO-RFC-8452-COMPLIANCE` | Korrekte Implementierung der AES-256-GCM-SIV Testvektoren gemäß RFC 8452 Anhang C.2. | `tests/rfc8452_kat.rs` |
| `INV-CRYPTO-ZERO-PANIC` | 0 Panics im Produktionscode (`unwrap`, `expect`, `panic!`, etc.). | `tools/zero_panic_analyzer.py` |
