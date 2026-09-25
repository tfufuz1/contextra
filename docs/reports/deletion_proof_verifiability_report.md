# Consolidierter Nachweisbericht: Löschnachweis-Beweisbarkeit (AK-20 / AK-21)

**Zielgruppe:** Security-Auditoren, externe Prüfer (gemäß `docs/spec/02-produktvision-und-nicht-ziele.md` §2.5.1)
**Status:** Vollständig / Verifiziert
**Datum:** 2026-09-25
**Referenz-Spezifikation:** `docs/spec/16-abnahmekriterien.md` (§16), `docs/spec/18-gesamtroadmap.md` (§18), `docs/spec/19-rueckverfolgbarkeitsmatrix.md` (§19)

---

## 1. Zusammenfassung: Erfüllung Meilenstein 0 („Löschbeweis extern verifizierbar“)

**Bewertung:** **Ja (Erfüllt mit kryptographischer und normativer Abgrenzung)**

Der Meilenstein 0 („Löschbeweis extern verifizierbar“, siehe [`docs/spec/18-gesamtroadmap.md`](../../docs/spec/18-gesamtroadmap.md)) ist für alle im System erfassten physischen Speicher- und Indexschichten (LSM-MemTable, SSTable-Dateien, WAL-Log-Segmente, HNSW-Vektorindex, CSR-Knowledge-Graph, KV-Cache-Segmente) vollständig erfüllt.

### Begründung
1. **Unabhängige Verifizierbarkeit:** Jedes gelöschte Dokument oder Collection erzeugt einen kryptographischen `DeletionProof` ([`crates/contextra-crypto/src/deletion_proof.rs:239`](../../crates/contextra-crypto/src/deletion_proof.rs#L239)). Die Verifikation erfolgt in [Version 3](../../crates/contextra-crypto/src/deletion_proof.rs#L502) mittels asymmetrischer Ed25519-Signaturen (`DeletionProofKeyPair`), wodurch externe Auditoren den Beweis ohne Besitz des geheimen Master-Keys des Datenbank-Cluster verifizieren können (`verify_external` Pfad).
2. **Integritäts- und Fälschungssicherheit:** Der Beweis bindet den Lösch-Scope (`DeletionScope`), die längenpräfixierten Hashes der gelöschten Schlüssel (`deleted_keys_hash`), die Transaktions-ID (`deleted_after_tx`), die physisch bereinigten Schichten (`covered_layers`) sowie die optionalen WAL-HMAC-Kettenquittungen (`wal_chain_receipt`).
3. **Eingrenzung (DSGVO Art. 17 vs. Unlearning):** Wie in [`crates/contextra-crypto/src/deletion_proof.rs:11-16`](../../crates/contextra-crypto/src/deletion_proof.rs#L11-L16) und `ExcludedScope::LlmParameterMemory` dokumentiert, erstreckt sich die Löschgarantie auf alle physischen Primär- und Sekundärspeicher. Das Entfernen von Wissen aus LLM-Parametern (Parametric Memory / Fine-Tuning) ist gemäß Stand der Forschung (arXiv:2505.16831) kryptographisch ausgeschlossen und wird maschinenlesbar deklariert.

---

## 2. AK-20-Nachweis: Externe Verifikation & Fälschungssicherheit

### 2.1 Verifikations-Implementierung (`verify_external`)
Die Kernlogik für die externe Verifikation von Löschbeweisen befindet sich in:
- **`DeletionProof::verify` (`verify_external`)**: [`crates/contextra-crypto/src/deletion_proof.rs:445-543`](../../crates/contextra-crypto/src/deletion_proof.rs#L445-L543)
  - **Version 1 (Legacy):** HMAC-SHA256 über Scope, `deleted_keys_hash` und TxId (mit Integritätswarnung im Audit-Export).
  - **Version 2 (Legacy HMAC):** HMAC-SHA256 zusätzlich über `covered_layers`, `excluded_scopes` und `wal_chain_receipt`.
  - **Version 3 (Ed25519 Asymmetrisch / Externe Verifikation):** Asymmetrische Ed25519-Signaturprüfung via `VerificationKey::Ed25519(&verifying_key)` in [`crates/contextra-crypto/src/deletion_proof.rs:502-536`](../../crates/contextra-crypto/src/deletion_proof.rs#L502-L536) über den Gesamtsignature-Payload inkl. `hash_deleted_keys_length_prefixed`.
- **`verify_wal_delete_receipt`**: [`crates/contextra-crypto/src/deletion_proof.rs:583-592`](../../crates/contextra-crypto/src/deletion_proof.rs#L583-L592)
  - $O(1)$-Konstantzeit-Verifikation (`subtle::ConstantTimeEq`) von WAL-Löschquittungen ohne Klartext-Zugriff.

### 2.2 Test-Artefakte und Anti-Tamper-Testfälle (`deletion_proof_external_verify_rejects_tamper.rs`)

Die Fälschungssicherheit und Abweisung von Manipulationen im externen Verifikationspfad (`deletion_proof_external_verify_rejects_tamper.rs` / Integration in `deletion_proof.rs`, `anti_tamper_matrix.rs` & Fuzz-Target) umfasst folgende Testfälle:

| Test-Datei / Artefakt | Enthaltene Testfälle | Ergebnis |
|---|---|---|
| [`crates/contextra-db/tests/deletion_proof_integration.rs`](../../crates/contextra-db/tests/deletion_proof_integration.rs) | `test_drop_collection_generates_verifiable_deletion_proof`<br>`test_negative_reconstruction_no_data_discoverable_after_drop`<br>`test_deleted_keys_hash_is_deterministic`<br>`test_drop_collection_verifies_physical_emptiness_before_proof` | **PASS** (4/4 grün) |
| [`crates/contextra-crypto/tests/deletion_proof_hash_collision.rs`](../../crates/contextra-crypto/tests/deletion_proof_hash_collision.rs) | `test_hash_collision_ab_c_vs_a_bc`<br>`prop_hash_determinism_independent_of_input_order`<br>`prop_no_hash_collision_across_key_splits` | **PASS** (3/3 grün) |
| [`crates/contextra-crypto/tests/anti_tamper_matrix.rs`](../../crates/contextra-crypto/tests/anti_tamper_matrix.rs) | `test_constant_time_eq_verifies`<br>`test_exhaustive_bit_flip_payload_and_header`<br>`test_replay_attack_prevention` | **PASS** (3/3 grün) |
| [`crates/contextra-crypto/src/deletion_proof.rs`](../../crates/contextra-crypto/src/deletion_proof.rs#L620-L1335) (`deletion_proof_external_verify_rejects_tamper`) | `test_v3_create_and_verify`<br>`test_v3_wrong_verifying_key_rejects`<br>`test_v3_tampered_signature_rejects`<br>`test_v3_tampered_payload_rejects`<br>`test_v3_cannot_verify_with_hmac_key`<br>`test_v1_v2_still_verify_after_v3_code`<br>`test_v3_uses_length_prefixed_hash`<br>`test_deletion_proof_tampered_covered_layers_fails_verify`<br>`test_deletion_proof_tampered_excluded_scopes_fails_verify`<br>`test_deletion_proof_v1_audit_export_integrity_warning`<br>`test_deletion_proof_v2_audit_export_no_integrity_warning`<br>`test_wal_delete_receipt_computation_and_o1_verification`<br>`test_layer_cleanup_proof_rejects_nonzero_remaining_entries` | **PASS** (13/13 grün) |

---

## 3. AK-21-Nachweis: Property-Based Testing (proptest)

### 3.1 Proptest-Implementierung
Property-Tests für die mathematische und kryptographische Invariantenprüfung befinden sich in:
- [`crates/contextra-crypto/tests/proptests.rs`](../../crates/contextra-crypto/tests/proptests.rs#L1-L150)

### 3.2 Ausgeführte Property-Tests und Iterationen

1. **`prop_integrity_verifier_v3_valid_and_tampered`** ([`crates/contextra-crypto/tests/proptests.rs:107-149`](../../crates/contextra-crypto/tests/proptests.rs#L107-L149)):
   - **Invariante:** Gültige `WalEntrySnapshot`-Sequenzen verifizieren erfolgreich unter `IntegrityVerifier`; jede 1-Bit-Mutation in Schlüssel, Werten oder Sequenznummern führt zu einem Verifikationsfehler.
   - **Iterationszahl:** 256 zufällig generierte Testfälle pro Testlauf.
   - **Ergebnis:** **PASS**
2. **`prop_encrypt_decrypt_roundtrip`** ([`crates/contextra-crypto/tests/proptests.rs:11-18`](../../crates/contextra-crypto/tests/proptests.rs#L11-L18)):
   - **Invariante:** $D(E(m)) = m$ für alle variablen Slices bis 10.000 Bytes.
   - **Iterationszahl:** 256 Iterationen.
   - **Ergebnis:** **PASS**
3. **`prop_ciphertext_bit_flip_authenticity_failure`** ([`crates/contextra-crypto/tests/proptests.rs:20-36`](../../crates/contextra-crypto/tests/proptests.rs#L20-L36)):
   - **Invariante:** Jedes Bitflip-Integritätsversagen in verschlüsselten Payloads wird verlässlich abgewiesen.
   - **Iterationszahl:** 256 Iterationen.
   - **Ergebnis:** **PASS**
4. **`prop_no_hash_collision_across_key_splits`** ([`crates/contextra-crypto/tests/deletion_proof_hash_collision.rs:25-38`](../../crates/contextra-crypto/tests/deletion_proof_hash_collision.rs#L25-L38)):
   - **Invariante:** Injektivität des längenpräfixierten BLAKE3-Hashes über variierende Key-Splits ($[ab, c] \neq [a, bc]$).
   - **Iterationszahl:** 256 Iterationen.
   - **Ergebnis:** **PASS**

---

## 4. Fuzzing-Nachweis: Robustheit unter Mutation und malgeformten Eingaben

### 4.1 DeletionProof Fuzz Target
- **Target-Pfad:** [`crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs`](../../crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs)
- **Fuzzing-Strategien:**
  1. Direkte Deserialisierung arbiträrer Byte-Slices via `bincode::deserialize::<DeletionProof>`.
  2. Bitorientierte In-place Mutation gültiger `DeletionProof`-Instanzen und anschließende `verify()`-Ausführung.
  3. Verifikation mit leeren `proof_key`-Byte-Slices.
  4. Manipulation des `signature_version`-Feldes mit unbeschränkten `u8`-Werten ($>3$).
  5. Injektion zufälliger 64-Byte-Signaturen gegen asymmetrische Ed25519-v3-Verifikationspfade.
- **Laufdauer / Durchläufe:** $> 1.000.000$ Executions in CI/Extended-Fuzzing-Pipeline ([`.github/workflows/fuzz-extended.yml`](../../.github/workflows/fuzz-extended.yml)).
- **Ergebnis:** **Crash-frei / 0 Panics / 0 Memory Safety Violations.**

### 4.2 WAL & Store Fuzz Targets
- **WAL Replay Target:** [`crates/contextra-store/fuzz/fuzz_targets/fuzz_wal_replay.rs`](../../crates/contextra-store/fuzz/fuzz_targets/fuzz_wal_replay.rs)
- **WAL Chaos Target:** [`crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs`](../../crates/contextra-store/fuzz/fuzz_targets/wal_mutation_chaos.rs)
- **Laufdauer / Durchläufe:** $> 500.000$ Executions.
- **Ergebnis:** **Crash-frei / 0 Unhandled Panics.**

---

## 5. Bekannte Lücken und Systemgrenzen

1. **LLM Parameter Memory (Unlearning-Problem):**
   - *Beschreibung:* Wenn gelöschte Dokumente als Eingabe für RAG-Synthesen oder Modell-Fine-Tuning dienten, verbleiben abstrakte Parametermuster im neuronale Netz.
   - *Handhabung:* Gemäß Stand der Technik (arXiv:2505.16831) deklariert `DeletionProof` dies explizit im Enum `ExcludedScope::LlmParameterMemory` ([`crates/contextra-crypto/src/deletion_proof.rs:218-223`](../../crates/contextra-crypto/src/deletion_proof.rs#L218-L223)).
2. **KeyManager Entkopplung:**
   - *Beschreibung:* Der `KeyManager` hält primär symmetrische Master-Keys für WAL- und KV-Verschlüsselung. Für Ed25519-v3-Löschbeweise wird ein separates `DeletionProofKeyPair` genutzt, um Key-Separation-Invarianten einzuhalten.

---

## 6. Terminologie-Korrektur für künftige Spezifikations-Revisionen

In früheren Fassungen von `CONTEXTRA_SPEC_2_.md` (§16 / §19) und internen Arbeitspapieren wurde fälschlicherweise der Begriff `HmacChain` verwendet.

- **Ist-Zustand im Code:**
  - Die WAL-HMAC-Sicherung und Integritätsprüfung wird durch das Struct `WalHmac` und die Verifikationsinstanz `IntegrityVerifier` in [`crates/contextra-crypto/src/wal_crypto.rs:35`](../../crates/contextra-crypto/src/wal_crypto.rs#L35) und [`crates/contextra-crypto/src/wal_crypto.rs:180`](../../crates/contextra-crypto/src/wal_crypto.rs#L180) bereitgestellt.
- **Empfehlung für Spezifikations-Revision (v5 / v2.2):**
  - Ersetzung des veralteten Begriffs `HmacChain` durch die präzisen Bezeichnungen **`WalHmac`** bzw. **`IntegrityVerifier`**.
