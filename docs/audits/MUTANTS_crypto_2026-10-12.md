# Audit-Bericht: Mutation-Testing Deletion Proof (`crates/contextra-crypto/src/deletion_proof/`)

**Datum:** 2026-10-12
**Komponente:** `contextra-crypto` (`src/deletion_proof/`)
**Ziel-Mutation-Score:** ≥ 80.0 %

---

## 1. Zusammenfassung (Summary)

Ein Mutation-Testing-Audit wurde für den Löschnachweis-Kernpfad (`crates/contextra-crypto/src/deletion_proof/**`) durchgeführt. Ziel war es, überlebende Mutanten ohne Änderungen am Produktionscode durch gezielte Integrationstests in `crates/contextra-crypto/tests/mutants_deletion_proof.rs` zu töten.

* **Mutation-Score vor Audit (Initial):** **84.72 %** (61 Caught / 72 Evaluated, 11 Missed, 38 Unviable)
* **Mutation-Score nach Audit (Final):** **98.61 %** (71 Caught / 72 Evaluated, 1 Missed, 38 Unviable)
* **Zielerreichung:** **Erfüllt** (98.61 % ≥ 80.0 %)

---

## 2. Detaillierte Mutation-Ergebnisse

### 2.1 Gesamtmessung

| Metrik | Vor Audit (Initial) | Nach Audit (Final) |
| :--- | :--- | :--- |
| **Getestete Mutanten (Viable)** | 72 | 72 |
| **Getötete Mutanten (Caught)** | 61 | 71 |
| **Überlebende Mutanten (Missed)** | 11 (across 10 locations) | 1 (across 1 location) |
| **Unviable Mutanten** | 38 | 38 |
| **Mutation-Score** | **84.72 %** | **98.61 %** |

---

## 3. Getötete Mutanten (Killed Mutants)

Die folgenden 10 zuvor überlebenden Mutanten wurden durch neue Tests in `crates/contextra-crypto/tests/mutants_deletion_proof.rs` gezielt getötet:

1. **`builder.rs:134:21`** – `replace == with != in ProofBuilder::finish`
   - *Test:* `test_kill_mutant_builder_proof_id_fallback`
   - *Wirkung:* Prüft, dass bei ungesetzter `proof_id` (`[0u8; 32]`) der BLAKE3-Fallback-Hash berechnet wird.

2. **`keys.rs:13:9`** – `replace <impl std::fmt::Debug for DeletionProofKeyPair>::fmt -> std::fmt::Result with Ok(Default::default())`
   - *Test:* `test_kill_mutant_keys_debug_redaction`
   - *Wirkung:* Prüft die `Debug`-Formatierung und Sicherstellung der `***REDACTED***`-Ausgabe für den Signing Key.

3. **`keys.rs:23:9`** – `replace DeletionProofKeyPair::zeroize with ()`
   - *Test:* `test_kill_mutant_keys_zeroize_and_drop`
   - *Wirkung:* Verifiziert, dass `zeroize()` den geheimen Schlüssel im Speicher überschreibt.

4. **`keys.rs:38:9`** – `replace DeletionProofKeyPair::verifying_key_bytes -> [u8; 32] with [0; 32]`
   - *Test:* `test_kill_mutant_keys_verifying_key_bytes`
   - *Wirkung:* Prüft, dass `verifying_key_bytes()` die tatsächlichen Ed25519-Pubkey-Bytes zurückgibt und keine Null-Bytes.

5. **`keys.rs:38:9`** – `replace DeletionProofKeyPair::verifying_key_bytes -> [u8; 32] with [1; 32]`
   - *Test:* `test_kill_mutant_keys_verifying_key_bytes`
   - *Wirkung:* Prüft, dass `verifying_key_bytes()` keine Dummy-Bytes `[1u8; 32]` zurückgibt.

6. **`proof.rs:103:35`** – `replace < with == in DeletionProof::coverage`
   - *Test:* `test_kill_mutant_proof_coverage_version_threshold`
   - *Wirkung:* Überprüft die Schwellenwertlogik `signature_version < 4` für `coverage_known`.

7. **`proof.rs:103:35`** – `replace < with > in DeletionProof::coverage`
   - *Test:* `test_kill_mutant_proof_coverage_version_threshold`
   - *Wirkung:* Überprüft `signature_version < 4` Grenzfälle.

8. **`proof.rs:103:35`** – `replace < with <= in DeletionProof::coverage`
   - *Test:* `test_kill_mutant_proof_coverage_version_threshold`
   - *Wirkung:* Überprüft exakt `signature_version = 3` vs `4`.

9. **`proof_v3.rs:210:46`** – `replace < with > in DeletionProof::create_full_v3`
   - *Test:* `test_kill_mutant_proof_v3_durability_confirmed_seq_boundary`
   - *Wirkung:* Testet die Grenzwertbedingung `durability_proof.confirmed_seq_no < deleted_after_tx.0`.

10. **`proof_v3.rs:227:40`** – `replace >= with < in DeletionProof::create_full_v3`
    - *Test:* `test_kill_mutant_proof_v3_attested_at_timestamp_conversion`
    - *Wirkung:* Testet die Konvertierung von `attested_at >= 0` zu `u64` Timestamp.

---

## 4. Akzeptierter / Verbleibender Überlebender (Remaining Survivor)

Ein einziger Mutant verbleibt als akzeptierter Überlebender:

1. **`keys.rs:49:9`** – `replace <impl Drop for DeletionProofKeyPair>::drop with ()`
   - **Begründung:** Rusts `Drop`-Trait-Implementierung ruft intern `self.zeroize()` auf. Da `drop()` vom Compiler implizit am Ende des Gültigkeitsbereichs ausgeführt wird und keine Rückgabewerte oder sichtbaren Seiteneffekte im normalen Programmablauf erzeugt, kann das Nicht-Ausführen von `drop()` in einem Standard-UnitTest ohne spezialisierte Memory-Inspection nicht direkt gemessen werden. Die Methode `zeroize()` selbst wird zu 100% getestet.

---

## 5. Compliance & Besitzgrenzen

- **Arbeitsbereich-Besitz:**
  - `crates/contextra-crypto/tests/mutants_deletion_proof.rs` (neu angelegt)
  - `docs/audits/MUTANTS_crypto_2026-10-12.md` (neu angelegt)
- **Produktionscode:** Es wurden **keine** Änderungen an Dateien unter `src/` vorgenommen.
