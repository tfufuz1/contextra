# Spezifikation: `contextra-crypto` (Ring 0 / Klasse A)

## 1. Zweck und Baukasten-Profil
`contextra-crypto` ist die fundamentale, autarke Kryptographie-Bibliothek des Systems Contextra (Ring 0). Sie implementiert standardkonforme kryptographische Primitive und Chiffren:
- **AEAD:** AES-256-GCM-SIV (RFC 8452) mit deterministischer Nonce-Isolation.
- **KDF:** Argon2id (RFC 9106) für Passphrasen-Ableitung und HKDF-Expand (RFC 5869 / HMAC-SHA256).
- **WAL-Integrität & Anti-Tamper:** HMAC-SHA256 Integritätsketten, Sequenznummern-Validierung und Anti-Mirroring.
- **Schlüsselspeicher & Memory-Hygiene:** `Zeroize` und `ZeroizeOnDrop` für flüchtige Schlüsselmaterialien (`SecretKey`, `VolatileEncryptionKey`, `UnitKey`, `Dek`).
- **Audit-Kette:** Kryptographisch verknüpfte Audit-Log-Einträge mit Salted BLAKE3-Commitments.
- **KV-Segment-Verschlüsselung:** Neutrale Segment-Chiffre (`KvSegmentCipher`, `EncryptedKvLayer`) zur Speicherung segmentierter Tensoren und Vektoren.

### Nicht-Ziele
- Keine Löschbeweise, Schichtenscanner, Zeugen-Aggregation oder DSGVO/GDPR-Fachlogik. Diese Komponenten gehören in `contextra-erasure` (SPEC §6.2) und verbleiben hier nur übergangsweise hinter dem Feature `legacy-erasure` mit `#[deprecated]`-Markierung.
- Keine direkte Dateisystem- oder Netzwerk-I/O in reinen Krypto-Routinen (I/O- und Lock-Freiheit in `src/crypto`).

---

## 2. Invarianten und Verträge
1. **INV-CRYPTO-NO-CUSTOM-PRIMITIVES:** Keine Eigenentwicklungen kryptographischer Algorithmen. Nur standardisierte, peer-reviewte Bibliotheken (`aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3`, `ed25519-dalek`).
2. **INV-CRYPTO-ZEROIZE-ON-DROP:** Sämtliches Schlüsselmaterial (`VolatileEncryptionKey`, `SecretKey`, `UnitKey`, `Dek`, `KvSegment`) überschreibt seinen Heap- und Stack-Speicher bei Freigabe (`Drop`) deterministisch mit Nullen.
3. **INV-CRYPTO-CONSTANT-TIME:** Sämtliche kryptographischen MAC-, Hash- und Prüfsummen-Vergleiche werden in konstanter Zeit mittels `subtle::ConstantTimeEq` (`ct_eq`) ausgeführt.
4. **INV-CRYPTO-KDF-BOUNDS:** Argon2id wird mit sicherem, zufälligem Salt (mindestens 16 Bytes), Mindestspeicher $m \ge 64\text{ MiB}$ und Iterationen $t \ge 3$ ausgeführt.
5. **INV-CRYPTO-NONCE-ISOLATION:** Nonces für AES-256-GCM-SIV werden per Call zufällig über einen CSPRNG generiert; Counter-basierte Ableitungen schließen Zähler-Überlauf (fail-closed bei `u64::MAX`) aus.
6. **INV-CRYPTO-ZERO-PANIC:** Produktionscode enthält kein `unwrap()`, `expect()`, `panic!()` oder ungeprüfte Index-Zugriffe (P6).

---

## 3. Architektur-Entscheidung (ADR): Zuordnung von `kv_segment/` und `kv_cipher.rs`
- **Kontext:** `kv_cipher.rs` und `kv_segment/` stellen Segment-Verschlüsselung (`EncryptedKvLayer`) und Sharding-Stores (`TenantIsolatedKvStore`) für KV-Cache-Schichten bereit.
- **Entscheidung:** Die Segment-Verschlüsselung verbleibt als neutrale Chiffre-Komponente (`SegmentCipher`) in `contextra-crypto`.
- **Begründung:** Die Chiffrierung von Speicher- und Cache-Segmenten unter Einbindung von Mandanten- und Modell-Fingerprints ist eine hardware- und speicherbezogene Primitiv-Aufgabe (Ring 0) ohne DSGVO-Löschlogik.
- **Status:** Offene Architekturentscheidung für Welle 4/5: Reines Envelope-Crypto verbleibt in `contextra-crypto`; das Crypto-Shredding (`KeyRegistry`) ist übergangsweise hinter `legacy-erasure` gebunden.

---

## 4. Fehlermodell
Alle kryptographischen Fehler werden über den crate-lokalen Fehlertyp `CryptoError` typisiert zurückgegeben:
- `CryptoError::Aead`: Authentifizierungs- oder Entschlüsselungsfehler.
- `CryptoError::Kdf`: Unzureichende KDF-Parameter oder KDF-Berechnungsfehler.
- `CryptoError::HmacMismatch`: Erkannte Integritätsverletzung oder fehlerhafter HMAC-Tag.
- `CryptoError::ZeroizedKeyAccess`: Zugriff auf bereits gelöschtes Schlüsselmaterial.
- `CryptoError::NonceCounterExhausted`: Fail-closed Schutz bei Nonce-Zählerüberlauf.
