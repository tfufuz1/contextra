# Implementierungsplan: contextra-crypto

**Ebene:** L2 · **Ziel-Zustand:** Adapter der Kernel-Krypto-Ports (XChaCha20-Poly1305, HMAC-SHA256, HKDF, CRC32C, Argon2id-Passphrase, Ed25519-Signatur), ohne Standard-Features, die Löschung oder Verschlüsselung umgehen.
**Basis:** SPEC R14 §2.7, §8.4, §8.9, DEC-04, DEC-05, DEC-11, FIX-09, FIX-21, §13.3; R15 B-04, B-05, DEC-44, DEC-45.

## 1. Ist-Befund (gemessen)

- 28 Dateien, 8 797 Zeilen. Module: `aead`, `anti_tamper`, `audit_chain`, `crypto` (948), `dek`, `deletion_proof/` (Verzeichnis), `ed25519_proof`, `error`, `kdf`, `kv_cipher`, `kv_segment/` (Verzeichnis), `kv_shredding`, `lib`, `revocation_log`, `secret`, `wal_completeness`, `wal_crypto` (1163).
- **270 Treffer verbotener Muster im Produktionscode**, größter Block `kv_segment`.
- `Cargo.toml`:
  - `aes-gcm-siv` (SPEC: verboten, DEC-04).
  - `blake3`, `bincode`, `tokio` (über Workspace), `regex`, `lru`, `rand`, `ahash`, `serde`, `serde_json`, `parking_lot` (SPEC §2.7: verboten im Kern; `crypto` ist L2, daher verboten).
  - `argon2` ist vorhanden, SPEC erlaubt es nur für den Passphrase-Provider.
  - **`chacha20poly1305` fehlt**, obwohl SPEC es als Pflicht-Cipher nennt. `aead.rs` nutzt stattdessen `aes_gcm_siv`.
  - `ed25519-dalek` vorhanden, mit `rand_core`-Feature (Entropie aus `rand`, SPEC verlangt `getrandom` nur im `Entropy`-Adapter).
- **Standard-Features:** `default = ["legacy-erasure", "kv-encryption", "test-utils"]`. `kv-encryption` aktiviert die verbotenen KV-Module im Standardbuild.
- `contextra-ports` ist Abhängigkeit, aber die Ports-Traits `Cipher`, `Mac`, `Kdf`, `Checksum`, `KeyProvider` existieren dort nicht (siehe Plan `contextra-ports`). Adapter können also heute nicht implementiert werden.

## 2. Entfernen

| Ziel | Aktion |
|---|---|
| `kv_segment/` (Verzeichnis) | löschen (R14 §13.3, P0-B) |
| `kv_cipher.rs`, `kv_shredding.rs` | löschen |
| `aes-gcm-siv` in `Cargo.toml` und `aead.rs` | ersetzen durch `chacha20poly1305` |
| `blake3`, `bincode`, `tokio`, `lru`, `regex`, `parking_lot`, `ahash`, `serde`, `serde_json`, `rand` | entfernen, soweit nicht im Passphrase-Provider oder Entropie-Adapter nötig |
| Standard-Features `kv-encryption`, `legacy-erasure` | aus `default` entfernen. Mit `cargo tree -e features` prüfen, dass kein KV-Pfad übrig bleibt |
| `wal_crypto.rs`-Teile, die nur KV-Segmente bedienen | nach Analyse entfernen |
| `cloud-egress-guard` (leeres Feature) | entfernen oder nach `policy` verschieben |

**Vor jeder Löschung:** `grep -rn '<Symbol>' crates/`. Nur Symbole ohne Verwender entfernen.

## 3. Ändern

- **`aead.rs`:** Siegel XChaCha20-Poly1305 mit 24-Byte-Zufallsnonce. Nonce kommt vom `Entropy`-Port, nicht aus einem Zähler (DEC-04). Rückgabe: `Ciphertext` mit angehängtem Tag, AAD-Pflicht.
- **`crypto.rs` (948 Zeilen):** aufteilen in
  - `crypto/cipher.rs`: `Cipher`-Adapter.
  - `crypto/mac.rs`: `Mac`-Adapter (HMAC-SHA256, 16 B und 32 B Ausgabe).
  - `crypto/kdf.rs`: HKDF-SHA256 mit Domain-Strings (z. B. `"ctxs erase request id v1"`, SPEC §8.9).
  - `crypto/checksum.rs`: CRC32C über `crc32c`-Crate (FIX-21).
- **`wal_crypto.rs` (1163 Zeilen):** Redo-Siegel auf neuen `Cipher`/`Mac`-Port umstellen. Frame-AAD mit `op_uid`, `block_uid`, `record_uid` (AT-3). Ketten-MAC für Redo-Frames.
- **`dek.rs`, `secret.rs`:** `Zeroize` auf allen Schlüsseltypen. `Debug` durch `Redacted` ersetzen (§2.6).
- **`deletion_proof/` und `ed25519_proof.rs`:** Umbau auf `ErasureReceipt`-Eingang (R14 §13.3). Signatur über kanonische Bytes mit Domain-Präfix `"ctxs receipt sig v1"`. Verifikation mit `verify_strict`, Signaturbytes handkodiert.
- **`audit_chain.rs`, `revocation_log.rs`:** Prüfen, ob sie Teil des Ledgers (§8.9) oder eigenständige Altlogik sind. Im ersten Fall in `storage::erase::ledger` integrieren, im zweiten löschen.
- **`anti_tamper.rs`:** `VolatileEncryptionKey` mit `Zeroizing<[u8; 32]>` ist brauchbar. Aufgabe prüfen; wenn nicht Teil der Spec (§2.7 nennt es zur Behaltung), dokumentieren oder entfernen.
- **`lib.rs`:** Modulliste auf die Ziel-Struktur bringen.

## 4. Implementieren

1. **Adapter-Liste** (alle implementieren `contextra_ports::kernel`-Traits):
   - `XChaCha20Poly1305: Cipher`
   - `HmacSha256: Mac`
   - `HkdfSha256: Kdf`
   - `Crc32c: Checksum`
   - `FilePassphrase: KeyProvider` (Argon2id, Parameter als Konfiguration, `cohort` optional)
   - `OsEntropy: Entropy` (`getrandom`, nur hier erlaubt)
2. **Ledger-Schlüssel aus RootKey** (DEC-45, FIX-09): `ledger_key = Kdf(root, "ctxs ledger key v1" ‖ instance_id)`. `LedgerScopeKey` analog. Kein Zufallserzeugen je Öffnung.
3. **Schlüssel-Lebenszyklus:**
   - `create_cluster_keys`: Batch, idempotent je `BatchId`, durabel vor Rückgabe.
   - `destroy_keys`: nach Übergabe an KeyProvider und Ledger-Event.
   - Schlüsselmaterial nie in `Debug`, nie in Fehlertexten.
4. **Passphrase-Provider** (`FilePassphrase`): Argon2id mit festen Mindestparametern; Schlüsselwrap im `KEYS`-Container.
5. **Ed25519-Signatur:** Schlüsselverwaltung nicht im Kern (NZ-15). Die Funktion nimmt einen Signierschlüssel als Parameter.
6. **`verify_receipt`:** nutzt nur den `Mac`-Port, keine Signatur im Kern (FIX-21).

## 5. Tests

- **Testvektoren:** XChaCha20-Poly1305, HMAC-SHA256, HKDF-SHA256, CRC-32C gegen veröffentlichte RFC- und Referenzvektoren. Ohne sie kein Merge.
- **AAD-Test:** Änderung eines AAD-Feldes muss Entsiegelung verhindern.
- **Nonce-Test:** 24-Byte-Nonce, zwei Siegelungen desselben Klartexts ergeben verschiedene Ciphertexte.
- **Zeroize-Test:** Schlüsseltypen werden beim Drop auf null gesetzt (mit `zeroize`-Probe).
- **Kein-Klartext-Test:** Ein gesiegelter Cluster enthält im Dateiabbild keinen Klartext-Marker.
- **Krypto-Review** vor G-E (§13.1), extern.

## 6. Abhängigkeiten nach Umbau

Erlaubt: `chacha20poly1305 ^0.10`, `hmac ^0.12`, `sha2 ^0.10`, `hkdf ^0.12`, `argon2 ^0.5` (nur Passphrase-Provider), `crc32c ^0.6`, `subtle ^2`, `getrandom ^0.3` (nur `OsEntropy`), `ed25519-dalek ^2`, `zeroize`, `contextra-ports`, `contextra-types`, `thiserror`.
Verboten: `aes-gcm-siv`, `blake3`, `bincode`, `tokio`, `rand`, `serde`, `regex`, `lru`, `parking_lot`, `ahash`.

## 7. Gate und Reihenfolge

1. Standard-Features reduzieren, `kv_*` entfernen, `aes-gcm-siv` entfernen. Build muss ohne KV-Module grün sein.
2. Adapter `XChaCha20Poly1305`, `HmacSha256`, `HkdfSha256`, `Crc32c` mit Testvektoren.
3. `FilePassphrase` und `OsEntropy`.
4. Ledger-Schlüssel-Ableitung (DEC-45).
5. Signatur und `deletion_proof`-Umbau.
6. **G-0:** Konformanz `Cipher`/`KeyProvider` grün. **G-E:** Krypto-Review.

**Kritisches Risiko:** Ohne diese Adapter gibt es im Workspace **keine** echte Verschlüsselung. DEC-44 verbietet Sealed ohne Adapter; bis P0-B fertig ist, bleibt Sealed-Cluster-Erzeugung verweigert.
