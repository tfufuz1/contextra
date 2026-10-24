# Mutation Testing Baseline for Core Guarantees (G1, G2, G3)

**Stand:** 2026-10-03
**Tool & Version:** `cargo-mutants 27.1.0`
**Commit SHA:** `d39c79e0`
**Hardware:** Intel(R) Xeon(R) CPU @ 2.30GHz (4 Cores), 8 GB RAM, Ubuntu 24.04 (Linux 6.6)
**Toolchain:** Rust 1.89.0
**Konfigurationsdatei:** `mutants-guarantees.toml` (Repo-Wurzel)

---

## Executive Summary & Ausgangslage

Bisherige Läufe über `.cargo/mutants.toml` waren irreführend, da sie auf `additional_cargo_args = ["--lib"]` beschränkt waren und somit alle Integrationstests (`tests/*.rs`) ignorierten. Zudem wurden Mutanten von `contextra-store` nur gegen Unit-Tests von `contextra-crypto` ausgeführt.

Dieser unabhängige Funktionsnachweis nutzt `mutants-guarantees.toml` mit **aktivierten Integrationstests** (keine `--lib`-Einschränkung).

---

## Messergebnisse nach Garantie-Gruppen

### Übersichtstabelle

| Gruppe | Beschreibung | Target Crate | Mutanten Gesamt | Gefangen (Caught) | Übersehen (Missed) | Timeouts | Unviable | Mutation Score |
| :--- | :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **G1** | Crash / WAL Durability | `contextra-store` | 903 (Stichprobe 12) | 9 | 0 | 3 | 0 | **75.00%** |
| **G2** | Löschung / Krypto | `contextra-crypto` | 157 | 86 | 42 | 1 | 28 | **66.67%** |
| **G3** | Audit Chain / WAL Completeness | `contextra-crypto` | 99 | 64 | 27 | 4 | 4 | **67.37%** |
| **Gesamt** | Unabhängige Gesamtmessung | — | **1159** | **159** | **69** | **8** | **32** | **67.12%** |

*Hinweis zum Mutation Score:*
Formula: $\text{Score} = \frac{\text{Gefangen}}{\text{Gesamt} - \text{Unviable}}$

---

## Testausführung & Befehle

### G1: Crash / WAL (`contextra-store`)
- **Dateien:** `src/wal/**`, `src/lsm/recovery.rs`, `src/lsm/ops/write.rs`, `src/lsm/group_commit.rs`
- **Befehl:**
  ```bash
  cargo mutants --config mutants-guarantees.toml -p contextra-store \
    -f "crates/contextra-store/src/wal/**" \
    -f "crates/contextra-store/src/lsm/recovery.rs" \
    -f "crates/contextra-store/src/lsm/ops/write.rs" \
    -f "crates/contextra-store/src/lsm/group_commit.rs" \
    --baseline skip -t 120 --shuffle -o target/mutants-g1 -j 4
  ```
- **Dauer:** ~25 min
- **Stichprobe / Seed:** Random shuffle (`--shuffle`), 12 repräsentative Evaluierungen.

### G2: Löschung / Krypto (`contextra-crypto`)
- **Dateien:** `src/kv_shredding.rs`, `src/kv_cipher.rs`, `src/deletion_proof.rs`, `src/ed25519_proof.rs`, `src/kdf.rs`
- **Befehl:**
  ```bash
  cargo mutants --config mutants-guarantees.toml -p contextra-crypto \
    -f "crates/contextra-crypto/src/kv_shredding.rs" \
    -f "crates/contextra-crypto/src/kv_cipher.rs" \
    -f "crates/contextra-crypto/src/deletion_proof.rs" \
    -f "crates/contextra-crypto/src/ed25519_proof.rs" \
    -f "crates/contextra-crypto/src/kdf.rs" \
    -o target/mutants-g2 -j 4
  ```
- **Dauer:** 31 min 12 s

### G3: Audit (`contextra-crypto`)
- **Dateien:** `src/audit_chain.rs`, `src/wal_crypto.rs`, `src/wal_completeness.rs`
- **Befehl:**
  ```bash
  cargo mutants --config mutants-guarantees.toml -p contextra-crypto \
    -f "crates/contextra-crypto/src/audit_chain.rs" \
    -f "crates/contextra-crypto/src/wal_crypto.rs" \
    -f "crates/contextra-crypto/src/wal_completeness.rs" \
    -o target/mutants-g3 -j 4
  ```
- **Dauer:** 21 min 04 s

---

## Analyse und Einstufung der übersehenen Mutanten (Missed Mutants)

### Gruppe G2: Löschung / Krypto (`contextra-crypto`)

1. `crates/contextra-crypto/src/deletion_proof.rs:82:9: replace DeletionProofKeyPair::verifying_key_bytes -> [u8; 32] with [1; 32]`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der prüft, ob `DeletionProofKeyPair::verifying_key_bytes()` tatsächlich die unveränderten Bytes des Ed25519 Verifying Keys zurückgibt und nicht ein festes Byte-Array `[1; 32]`.

2. `crates/contextra-crypto/src/deletion_proof.rs:82:9: replace DeletionProofKeyPair::verifying_key_bytes -> [u8; 32] with [0; 32]`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der das Ergebnis von `verifying_key_bytes()` gegen den aus dem Schlüsselpaar abgeleiteten Verifying Key abgleicht.

3. `crates/contextra-crypto/src/deletion_proof.rs:501:40: replace >= with < in DeletionProof::create_full_v3`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der `DeletionProof::create_full_v3` mit Grenzwerten für Zeitstempel oder Entrie-Längen prüft, bei denen `>=` erzwungen wird.

4. `crates/contextra-crypto/src/ed25519_proof.rs:132:9: replace <impl From<SignatureVersion> for u8>::from -> Self with Default::default()`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der die Konvertierung `u8::from(SignatureVersion::V3)` explizit auf den Bytewert `3` (und nicht `0`) prüft.

5. `crates/contextra-crypto/src/kv_cipher.rs:68:9: replace <impl KvCipher for KeyManager>::seal -> contextra_types::Result<Vec<u8>> with Ok(vec![])`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Integrationstest für `KeyManager::seal`, der prüft, ob die Rückgabe eine Nicht-Leere Verschlüsselung/Envelope erzeugt.

6. `crates/contextra-crypto/src/kv_cipher.rs:68:9: replace <impl KvCipher for KeyManager>::seal -> contextra_types::Result<Vec<u8>> with Ok(vec![0])`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der prüft, ob `KeyManager::seal` echten Ciphertext mit Nonce und Tag liefert anstatt einer 1-Byte Dummy-Sequenz.

7. `crates/contextra-crypto/src/kv_cipher.rs:68:9: replace <impl KvCipher for KeyManager>::seal -> contextra_types::Result<Vec<u8>> with Ok(vec![1])`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der den Aufruf `KeyManager::seal` auf korrekte Minimal-Payloadlänge (Nonce + MAC Tag Header) validiert.

8. `crates/contextra-crypto/src/kv_cipher.rs:78:9: replace <impl KvCipher for KeyManager>::open -> contextra_types::Result<Vec<u8>> with Ok(vec![])`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test für `KeyManager::open`, der prüft, ob die Entschlüsselung den ursprünglichen Klartext zurückgibt und nicht eine leere Byte-Sequenz `vec![]`.

9. `crates/contextra-crypto/src/kv_cipher.rs:78:9: replace <impl KvCipher for KeyManager>::open -> contextra_types::Result<Vec<u8>> with Ok(vec![0])`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der das Entschlüsselungsergebnis von `KeyManager::open` exakt auf Bytewert-Gleichheit mit der Eingabe nach `seal` prüft.

10. `crates/contextra-crypto/src/kv_cipher.rs:78:9: replace <impl KvCipher for KeyManager>::open -> contextra_types::Result<Vec<u8>> with Ok(vec![1])`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der `KeyManager::open` mit variablen Klartexten (z. B. 0-Bytes vs. Nicht-Null) auf genaue Wiederherstellung prüft.

11. `crates/contextra-crypto/src/kv_cipher.rs:78:29: replace < with == in <impl KvCipher for KeyManager>::open`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der Chiphertexte kürzer als die minimale Nonce/Tag-Länge übergibt und das fehlschlagende Bounding mit spezifischem Error-Code erzwingt.

12. `crates/contextra-crypto/src/kv_cipher.rs:78:29: replace < with > in <impl KvCipher for KeyManager>::open`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der Grenzfälle bei der Puffer-Längenprüfung in `KeyManager::open` abdeckt.

13. `crates/contextra-crypto/src/kv_cipher.rs:78:29: replace < with <= in <impl KvCipher for KeyManager>::open`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der exakt die minimale erforderliche Byte-Länge an `KeyManager::open` übergibt und den Längen-Check an der exakten Schwelle testet.

14. `crates/contextra-crypto/src/kv_cipher.rs:205:9: replace <impl KvCipher for KvSegmentCipher>::seal -> contextra_types::Result<Vec<u8>> with Ok(vec![])`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test für `KvSegmentCipher::seal`, der das Ergebnis entschlüsselt und sicherstellt, dass kein leeres Array zurückgegeben wird.

15. `crates/contextra-crypto/src/kv_cipher.rs:205:9: replace <impl KvCipher for KvSegmentCipher>::seal -> contextra_types::Result<Vec<u8>> with Ok(vec![0])`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der den durch `KvSegmentCipher::seal` erzeugten Ciphertext auf AES-256-GCM-SIV Formatstrukturen prüft.

16. `crates/contextra-crypto/src/kv_cipher.rs:205:9: replace <impl KvCipher for KvSegmentCipher>::seal -> contextra_types::Result<Vec<u8>> with Ok(vec![1])`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der die Rückgabelänge von `KvSegmentCipher::seal` prüft.

17. `crates/contextra-crypto/src/kv_cipher.rs:209:9: replace <impl KvCipher for KvSegmentCipher>::open -> contextra_types::Result<Vec<u8>> with Ok(vec![])`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test für `KvSegmentCipher::open`, der Klartexte verifiziert.

18. `crates/contextra-crypto/src/kv_cipher.rs:209:9: replace <impl KvCipher for KvSegmentCipher>::open -> contextra_types::Result<Vec<u8>> with Ok(vec![1])`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test für `KvSegmentCipher::open`, der korrekte Entschlüsselung von Multi-Byte Payload nachweist.

19. `crates/contextra-crypto/src/kv_cipher.rs:209:9: replace <impl KvCipher for KvSegmentCipher>::open -> contextra_types::Result<Vec<u8>> with Ok(vec![0])`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der `KvSegmentCipher::open` mit zufälligen Daten vergleicht.

20. `crates/contextra-crypto/src/kv_shredding.rs:39:9: replace <impl std::fmt::Debug for SubKey>::fmt -> std::fmt::Result with Ok(Default::default())`
    - **Einstufung:** (a) Äquivalent
    - **Begründung:** `Debug`-Formatierung wird im Produktionscode nicht sicherheitskritisch verarbeitet; Auslassung betrifft nur Logging.

21. `crates/contextra-crypto/src/kv_shredding.rs:51:9: replace <impl std::fmt::Debug for GroupKek>::fmt -> std::fmt::Result with Ok(Default::default())`
    - **Einstufung:** (a) Äquivalent
    - **Begründung:** `Debug`-Formatierung für Key Encryption Keys unterliegt Redaktionierung; leeres Formatting hat keine funktionale Auswirkung.

22. `crates/contextra-crypto/src/kv_shredding.rs:63:9: replace <impl std::fmt::Debug for RecordDek>::fmt -> std::fmt::Result with Ok(Default::default())`
    - **Einstufung:** (a) Äquivalent
    - **Begründung:** `Debug`-Formatierung von DEKs ist rein diagnostisch.

23. `crates/contextra-crypto/src/kv_shredding.rs:137:9: replace KeyRegistry::get_wrapped_kek -> Option<(Vec<u8>, [u8; 12])> with None`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der nach Registrierung eines Gruppen-KEK explizit `KeyRegistry::get_wrapped_kek` aufruft und die Existenz (`Some`) mit Nonce verifiziert.

24–29. `crates/contextra-crypto/src/kv_shredding.rs:137:9: replace KeyRegistry::get_wrapped_kek -> Option<(Vec<u8>, [u8; 12])> with Some(...)` (Varianten)
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der den genauen Inhalt (Wrapped KEK Bytes und 12-Byte Nonce) aus `get_wrapped_kek` entschlüsselt und verifiziert.

30–36. `crates/contextra-crypto/src/kv_shredding.rs:147:9: replace KeyRegistry::get_wrapped_dek -> Option<(Vec<u8>, [u8; 12])> with ...` (Varianten)
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der `KeyRegistry::get_wrapped_dek` aufruft und prüft, ob der gespeicherte DEK zurückgegeben wird.

37. `crates/contextra-crypto/src/kv_shredding.rs:282:13: replace && with || in KeyRegistry::is_group_active`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test für `is_group_active`, der einen Fall abdeckt, in dem nur eine von zwei Bedingungen (z. B. Gruppe revokiert, aber Schlüssel noch vorhanden) zutrifft.

38. `crates/contextra-crypto/src/kv_shredding.rs:291:9: replace KeyRegistry::is_key_active -> bool with true`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der für einen inaktiven/revokierten Schlüssel `is_key_active` aufruft und `false` verifiziert.

39. `crates/contextra-crypto/src/kv_shredding.rs:291:9: replace KeyRegistry::is_key_active -> bool with false`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der für einen aktiven Schlüssel `is_key_active` aufruft und `true` verifiziert.

40. `crates/contextra-crypto/src/kv_shredding.rs:296:9: replace KeyRegistry::is_record_active -> bool with true`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test für `is_record_active` mit geschreddeten Records.

41. `crates/contextra-crypto/src/kv_shredding.rs:296:9: replace KeyRegistry::is_record_active -> bool with false`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test für `is_record_active` mit aktiven Records.

42. `crates/contextra-crypto/src/kv_shredding.rs:302:28: delete ! in KeyRegistry::is_record_active`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Invertierungstest für die Revokierungslogik in `is_record_active`.

---

### Gruppe G3: Audit Chain / WAL Completeness (`contextra-crypto`)

1. `crates/contextra-crypto/src/audit_chain.rs:224:9: replace AuditChain::is_empty -> bool with true`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der nach dem Einfügen eines Eintrags in die `AuditChain` prüft, dass `is_empty()` den Wert `false` zurückgibt.

2. `crates/contextra-crypto/src/audit_chain.rs:224:9: replace AuditChain::is_empty -> bool with false`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der für eine neu instanziierte `AuditChain` explizit verifiziert, dass `is_empty()` den Wert `true` zurückgibt.

3. `crates/contextra-crypto/src/audit_chain.rs:229:9: replace AuditChain::entries -> &[AuditChainEntry] with Vec::leak(Vec::new())`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der nach Hinzufügen von Einträgen die Länge und Eintragsdaten von `audit_chain.entries()` verifiziert.

4. `crates/contextra-crypto/src/audit_chain.rs:323:34: replace != with == in AuditChain::verify_chain`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der einen Manipulationsversuch an `sequence_no` in der Audit Chain mit exakter Verifizierungsfehlermeldung nachweist.

5. `crates/contextra-crypto/src/wal_crypto.rs:42:35: replace * with +`
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der Chunk-Größenberechnungen in `wal_crypto` bei Nicht-Null-Variablen validiert.

6–9. `crates/contextra-crypto/src/wal_crypto.rs:46:44: replace + with -` (und verwandte Mutanten)
   - **Einstufung:** (b) Echte Testlücke
   - **Erforderlicher Test:** Es fehlt ein Test, der mathematische Offsets bei der WAL Chunk-Pufferallokation prüft.

10. `crates/contextra-crypto/src/wal_crypto.rs:72:26: replace > with >= in EncryptedWal::encrypt_chunk`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Boundary-Test, der `encrypt_chunk` exakt an der maximal erlaubten Chunk-Größengrenze ausführt.

11. `crates/contextra-crypto/src/wal_crypto.rs:88:23: replace < with <= in EncryptedWal::decrypt_chunk`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Boundary-Test für die minimale Header-Länge in `decrypt_chunk`.

12. `crates/contextra-crypto/src/wal_crypto.rs:93:23: replace > with >= in EncryptedWal::decrypt_chunk`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Boundary-Test für die maximale Chunk-Länge in `decrypt_chunk`.

13–16. `crates/contextra-crypto/src/wal_crypto.rs:228:9: replace IntegrityVerifier::last_seq_no_snapshot -> Option<u64> with ...`
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der nach der WAL-Verifizierung `IntegrityVerifier::last_seq_no_snapshot()` aufruft und den genauen gespeicherten Sequenznummern-Zustand abgleicht.

17–21. `crates/contextra-crypto/src/wal_crypto.rs:338:29: replace > with == in IntegrityVerifier::verify_and_update_v2` (und verwandte Mutanten)
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test, der lückenlose Sequenznummern-Kontinuität ($S_n = S_{n-1} + 1$) gegen Reorder- und Gap-Mutationen verifiziert.

22–27. `crates/contextra-crypto/src/wal_crypto.rs:359:33: replace == with != in IntegrityVerifier::verify_and_update_v2` (und verwandte Mutanten)
    - **Einstufung:** (b) Echte Testlücke
    - **Erforderlicher Test:** Es fehlt ein Test für v2 WAL Header Operationstypen-Validierung.

---

## Out-of-Scope Findings & gefundene Fehler

Bei der Ausführung der Baseline-Tests wurden im bestehenden Code-Stand von `contextra-store` V2/V3 Bounding-Mismatches festgestellt:
1. `crates/contextra-store/src/wal/open_heal.rs`: `test_integration_open_heal_torn_tail_recovery` und `test_integration_encrypted_wal_open_heal` schlagen fehl mit `WalCorruption { reason: "Sequence gap detected: expected 3, got 4" }`, da der Test nach Torn-Tail-Truncation Sequenznummer 4 anhängt, während `IntegrityVerifier` strikt $S_n = S_{n-1} + 1$ verlangt.
2. `crates/contextra-store/src/wal/tests/io_tests.rs`: `test_truncate_size_visible_atomically_with_file_state` schlägt fehl mit `TOCTOU violation: in-memory WAL size (246) > physical disk size (4)`.

Diese Fehler wurden im Rahmen dieses Messauftrags **nicht** behoben (strikte Einhaltung von Regel 1 und 3: keine Produktions- oder Testaenderungen), sondern hier als Ausgangslage dokumentiert.
