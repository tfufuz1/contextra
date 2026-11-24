# Spezifikation: `contextra-crypto` (Ring 0 / Klasse A)

## 1. Zweck und Baukasten-Profil
`contextra-crypto` ist die fundamentale, autarke Kryptographie-Bibliothek des Systems Contextra (Ring 0). Sie implementiert standardkonforme kryptographische Primitive und Chiffren:
- **AEAD:** AES-256-GCM-SIV (RFC 8452) mit deterministischer Nonce-Isolation.
- **KDF:** Argon2id (RFC 9106) für Passphrasen-Ableitung und HKDF-Expand (RFC 5869 / HMAC-SHA256).
- **WAL-Integrität & Anti-Tamper:** HMAC-SHA256 Integritätsketten, Sequenznummern-Validierung und Anti-Mirroring.
- **Schlüsselspeicher & Memory-Hygiene:** `Zeroize` und `ZeroizeOnDrop` für flüchtige Schlüsselmaterialien (`SecretKey`, `Secret`, `VolatileEncryptionKey`, `UnitKey`, `Dek`).
- **Audit-Kette:** Kryptographisch verknüpfte Audit-Log-Einträge mit Salted BLAKE3-Commitments.
- **KV-Segment-Verschlüsselung:** Neutrale Segment-Chiffre (`KvSegmentCipher`, `EncryptedKvLayer`) zur Speicherung segmentierter Tensoren und Vektoren.

### Nicht-Ziele
- Keine Löschbeweise, Schichtenscanner, Zeugen-Aggregation oder DSGVO/GDPR-Fachlogik. Diese Komponenten gehören in `contextra-erasure` (SPEC §6.2) und verbleiben hier nur übergangsweise hinter dem Feature `legacy-erasure` mit `#[deprecated]`-Markierung.
- Keine direkte Dateisystem- oder Netzwerk-I/O in reinen Krypto-Routinen (I/O- und Lock-Freiheit in `src/crypto`).

---

## 2. Invarianten und Verträge
1. **INV-CRYPTO-NO-CUSTOM-PRIMITIVES:** Keine Eigenentwicklungen kryptographischer Algorithmen. Nur standardisierte, peer-reviewte Bibliotheken (`aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3`, `ed25519-dalek`).
   - *Test:* `tests/rfc_vectors.rs` (RFC 8452, RFC 5869, RFC 4231).
2. **INV-CRYPTO-ZEROIZE-1:** Sämtliches Schlüsselmaterial (`VolatileEncryptionKey`, `SecretKey`, `Secret`, `UnitKey`, `Dek`) überschreibt seinen Speicher bei Freigabe (`Drop`) deterministisch mit Nullen.
   - *Test:* `tests/crypto_property_and_invariants.rs::test_inv_crypto_zeroize_1_contract`.
3. **INV-CRYPTO-CONSTANT-TIME-1:** Sämtliche kryptographischen MAC-, Hash-, Schlüssel- und Prüfsummen-Vergleiche werden in konstanter Zeit mittels `subtle::ConstantTimeEq` (`ct_eq`) ausgeführt.
   - *Test:* `tests/crypto_property_and_invariants.rs::test_inv_crypto_constant_time_1_contract`.
4. **INV-CRYPTO-KDF-BOUNDS:** Argon2id wird mit sicherem, zufälligem Salt (mindestens 16 Bytes), Mindestspeicher $m \ge 19456\text{ KiB}$ und Iterationen $t \ge 2$ ausgeführt.
   - *Test:* `tests/kdf_argon2id.rs`.
5. **INV-CRYPTO-NONCE-ISOLATION:** Nonces für AES-256-GCM-SIV werden per Call zufällig über einen CSPRNG generiert oder explizit injiziert; Counter-basierte Ableitungen schließen Zähler-Überlauf (fail-closed bei `u64::MAX`) aus.
   - *Test:* `tests/nonce_reuse.rs`, `tests/nonce_stress.rs`.
6. **INV-CRYPTO-ZERO-PANIC:** Produktionscode enthält kein `unwrap()`, `expect()`, `panic!()` oder ungeprüfte Index-Zugriffe (P6).
   - *Audit:* `tools/zero_panic_analyzer.py --crate contextra-crypto --fail-on-prod`.

---

## 3. Fehlermodell
Alle kryptographischen Fehler werden über den crate-lokalen Fehlertyp `CryptoError` typisiert zurückgegeben:
- `CryptoError::Aead`: Authentifizierungs- oder Entschlüsselungsfehler.
- `CryptoError::KeyDerivation`: Unzureichende KDF-Parameter oder KDF-Berechnungsfehler.
- `CryptoError::IntegrityViolation`: Erkannte Integritätsverletzung oder fehlerhafter MAC-Tag.
- `CryptoError::WalCorruption`: WAL-Korruptionserkennung mit Byte-Offset.
- `CryptoError::InvalidLength`: Ungültige Schlüssel- oder Nonce-Länge.
