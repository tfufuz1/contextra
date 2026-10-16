# Audit-Bericht: WAL-Kryptografie & Integrität (contextra-crypto)

**Datum:** 2026-10-02
**Modul:** `crates/contextra-crypto`
**Prüfer:** Jules (Security & Cryptography Audit)
**Status:** Bestanden mit behebbaren Lücken (alle Korrekturen umgesetzt und verifiziert)

---

## 1. Zusammenfassung & Scope

Gegenstand des Audits war die WAL-relevante Kryptografie im Ring-0/Ring-1 Baustein `contextra-crypto`.
Folgende Kernkomponenten wurden in den zugewiesenen Dateien auditiert:

- `crates/contextra-crypto/src/wal_crypto.rs`: `IntegrityVerifier`, `WalHmac`, `WalEntrySnapshot`, `EncryptedWal`
- `crates/contextra-crypto/src/wal_completeness.rs`: `verify_wal_chain_completeness`
- `crates/contextra-crypto/src/crypto.rs`: `KeyManager` (Nonce-Politik, Key Derivation, Zeroization, `encrypt_auto_nonce`, `decrypt_auto_nonce`)
- `crates/contextra-crypto/src/kdf.rs`: Argon2id KDF-Header und Parametersicherheit
- `crates/contextra-crypto/tests/wal_integrity_audit.rs`: Neue Audit-Regressionstests für Kettensicherheit und Replays

---

## 2. Detaillierte Befunde & Behebungen

### Befund 1: Fehlende Sequence-Number-Überprüfung in `IntegrityVerifier` (Behoben)
- **Datei:** `crates/contextra-crypto/src/wal_crypto.rs:191-385`
- **Schweregrad:** Hoch
- **Beschreibung:** `IntegrityVerifier::verify_and_update_v3` (und `v2`) überprüfte zwar die mathematische HMAC-Integrität (`prev_hmac == last_hmac` und `computed_checksum == entry.checksum`), jedoch wurde `entry.seq_no` nicht auf strikte Monotonie und Lückenlosigkeit in der Zustandsmaschine geprüft.
- **Risiko:** Ein Angreifer, der valide signierte WAL-Einträge mit Lücken injizieren oder doppelt senden konnte (sofern HMACs angepasst wurden oder genesis-Zustände vorlagen), wurde nicht durch eine explizite Sequence-Prüfung gestoppt.
- **Fix:** `IntegrityVerifier` speichert nun `last_seq_no: Option<u64>`. Bei jedem Aufruf von `verify_and_update_v3`/`v2` wird erzwungen:
  1. `entry.seq_no > last_seq` (Verhinderung von Replay-/Duplikat-/Rückwärts-Injektionen)
  2. `entry.seq_no == last_seq + 1` (Verhinderung von Lücken)
  3. `entry.op_type <= 2` (Verhinderung von ungültigen Operationstypen)
  Bei Verstößen wird sofort ein unterscheidbarer `CryptoError::WalCorruption` ausgelöst.

### Befund 2: Nonce-Counter-Überlauf-Invariante in `KeyManager` (Behoben)
- **Datei:** `crates/contextra-crypto/src/crypto.rs:346-353`
- **Schweregrad:** Mittel
- **Beschreibung:** In `KeyManager::encrypt_auto_nonce` wurde `nonce_counter.fetch_add(1, Ordering::Relaxed)` aufgerufen. Bei einem theoretischen Überlauf von `u64::MAX` würde der Zähler auf `0` überlaufen, was bei fortgesetzter Instanz-Nutzung zu Nonce-Reuse führen könnte.
- **Fix:** Es wurde eine explizite Überprüfung `if counter_val == u64::MAX` hinzugefügt, die bei Erreichen des Maximums kontrolliert `CryptoError::Encryption("Nonce counter exhausted")` zurückgibt.
- **Kollisionsgrenzen:** Bei $2^{64}$ Verschlüsselungen je `KeyManager`-Instanz schlägt die Verschlüsselung fehl. Durch per-file Subkey Derivation via `derive_file_key` hat jede WAL-Datei einen eigenen 32-Byte Subkey, wodurch instanzübergreifende Kollisionen ausgeschlossen sind.

### Befund 3: Dokumentationskorrektur der Nonce-Suffix-Erzeugung (Behoben)
- **Datei:** `crates/contextra-crypto/src/crypto.rs:16`
- **Schweregrad:** Niedrig (Dokumentation)
- **Beschreibung:** Der Modul-Dokkommentar gab irrtümlich an, dass der 8-Byte Nonce-Suffix per `OsRng` generiert wird, obwohl die Implementierung einen atomaren 8-Byte Big-Endian Sequence Counter nutzt.
- **Fix:** Der Modulkommentar wurde präzisiert: "4 bytes: random nonce_prefix generated once per KeyManager instance via OsRng; 8 bytes: big-endian u64 sequence counter suffix incremented atomically per encryption call."

---

## 3. Gegenüberstellung der V3-Checksummen-Implementierungen

Im Zuge des Audits wurde die V3-Checksummenberechnung zwischen `contextra-crypto` (`wal_crypto.rs`) und `contextra-store` (`wal/encode.rs`) verglichen:

| Feld / Eigenschaft | `contextra-crypto` (`wal_crypto.rs`) | `contextra-store` (`wal/encode.rs`) | Übereinstimmung |
| :--- | :--- | :--- | :--- |
| **Domain Separation** | `b"contextra-wal-v1"` | `b"contextra-wal-v1"` (via `WalHmac`) | ✅ Exakt |
| **Prev HMAC & SeqNo** | `prev_hmac` (32B) + `seq_no` (8B LE) | `prev_hmac` (32B) + `seq_no` (8B LE) | ✅ Exakt |
| **Tx ID Position** | `tx_id` (8B LE) vor `op_type` | `tx_id` (8B LE) vor `op_type` | ✅ Exakt |
| **Put Op (0)** | `0u8` + `key_len` (4B LE) + `key` + `val_len` (4B LE) + `value` | `0u8` + `key_len` (4B LE) + `key` + `val_len` (4B LE) + `value` | ✅ Exakt |
| **Delete Op (1)** | `1u8` + `key_len` (4B LE) + `key` | `1u8` + `key_len` (4B LE) + `key` | ✅ Exakt |
| **TxEnd Op (2)** | `2u8` + `committed` (1B) | `2u8` + `committed` (1B) | ✅ Exakt |

**Ergebnis:** Es liegt **keine Divergenz** vor. Beide Implementierungen sind 100% kompatibel und verwenden identische Längenpräfixe und Feldreihenfolgen.

---

## 4. Semantik & Bedrohungsmodell: `verify_wal_chain_completeness`

- `verify_wal_chain_completeness` in `wal_completeness.rs` führt einen Konstantzeitvergleich (`subtle::ConstantTimeEq`) zwischen dem letzten HMAC-Tail des WAL und der im Checkpoint-Manifest abgelegten High-Water-Mark durch.
- **Schutzwirkung:** Schützt zuverlässig gegen unbeabsichtigte oder bösartige partielle Dateitrunkierungen (WAL-Tail-Cut).
- **Grenzen:** Verhindert keine synchronen Rollback-Angriffe, bei denen ein Angreifer mit vollem Dateisystemzugriff sowohl den WAL als auch die Manifest-Datei auf einen früheren gemeinsamen Checkpoint zurücksetzt. Diese Sicherheitsgarantie wird erst durch den Audit-Chain-Anker (ADR-042 / Spec §15.6) hergestellt.

---

## 5. Verfassungs-Invarianten & Code-Qualität

1. **Zero Panics:** Kein `unwrap`, `expect`, `panic!` oder unprüfbares Indexing in Nicht-Test-Code. Alle Fehlerpfade geben explizit `CryptoError` zurück.
2. **Zero Unsafe:** `#![forbid(unsafe_code)]` ist strikt in allen auditierten Dateien deklariert.
3. **Konstantzeitvergleiche:** Alle HMAC- und Hash-Vergleiche nutzen `subtle::ConstantTimeEq` zur Unterdrückung von Timing-Seitenkanälen.
4. **Zeroization:** `IntegrityVerifier`, `KeyManager` und Schlüsselstrukturen löschen kritisches Schlüsselmaterial zuverlässig aus dem Speicher (`zeroize::Zeroize`, `ZeroizeOnDrop`, `VolatileEncryptionKey`).
5. **Redaktion in Debug/Display:** `KeyManager` gibt in `Debug` niemals Schlüsselmaterial aus (`"***REDACTED***"`).

---

## 6. Testergebnisse

Alle Regressionstests in `tests/wal_integrity_audit.rs` sowie die bestehenden Testsuiten wurden ausgeführt und verifiziert:

- `cargo test -p contextra-crypto --test wal_integrity_audit` (8/8 green)
- `cargo test -p contextra-crypto --all-features` (132/132 green)
- `cargo clippy -p contextra-crypto --all-targets --all-features -- -D warnings` (0 warnings)
- `cargo fmt --all -- --check` (pass)
