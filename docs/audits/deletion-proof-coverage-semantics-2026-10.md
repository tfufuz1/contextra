# DeletionProof Semantik, Mindestabdeckung und Optionen für signierte Nicht-Abdeckung

**Auftrag A-07 / Task T-2026-0235**
**Datum:** Oktober 2026
**Crate:** `contextra-crypto`
**Ziel:** Analyse der aktuellen DeletionProof-Abdeckungssemantik als Entscheidungsgrundlage für ADR-4.

---

## 1. Inventar der DeletionProof- und LayerCleanupProof-Aufrufer

Die folgende Aufstellung katalogisiert alle Aufrufer der Methoden zur Erzeugung von `DeletionProof` und `LayerCleanupProof` im gesamten Repository, getrennt nach **Produktion**, **Tests**, **Examples**, **Benches** und **Fuzzing**.

### 1.1 `DeletionProof::create`
- **Produktion (1 Aufruf):**
  - `crates/contextra-engine/src/contextra_impl/collections.rs:397` (in `Contextra::drop_collection`)
  - *(Hinweis: `crates/contextra-engine/src/lib.rs:206` enthält eine no-crypto Stub-Implementierung für `cfg(not(feature = "crypto"))`)*
- **Tests (27 Aufrufe):**
  - `crates/contextra-crypto/src/deletion_proof.rs:388`, `431`, `450`, `482`, `505`, `533`, `549`, `586`, `622`, `658`, `694`, `707`
  - `crates/contextra-crypto/tests/deletion_proof_external_verify_rejects_tamper.rs:142`
  - `crates/contextra-crypto/tests/deletion_proof_hash_collision.rs:18`, `28`, `54`, `64`, `90`
  - `crates/contextra-crypto/tests/deletion_proof_signature_version_serde.rs:96`
  - `crates/contextra-crypto/tests/deletion_proof_v3_validation.rs:159`
  - `crates/contextra-crypto/tests/p04b_deletion_proof_hardening.rs:18`, `45`, `72`, `281`
  - `crates/contextra-db/tests/deletion_proof_integration.rs:243`, `264`
  - `crates/contextra-engine/tests/no_crypto_proof_stub.rs:22`
- **Examples / Benches / Fuzzing:** 0 Aufrufe

### 1.2 `DeletionProof::create_with_wal_receipt`
- **Produktion:** 0 Aufrufe
- **Tests (1 Aufruf):** `crates/contextra-crypto/src/deletion_proof.rs:817`
- **Benches (1 Aufruf):** `crates/contextra-crypto/benches/deletion_proof_latency_bench.rs:81`
- **Examples / Fuzzing:** 0 Aufrufe

### 1.3 `DeletionProof::create_v3`
- **Produktion:** 0 Aufrufe
- **Examples (1 Aufruf):** `crates/contextra-db/examples/verify_deletion_proof_external.rs:123`
- **Tests (16 Aufrufe):**
  - `crates/contextra-crypto/src/deletion_proof.rs:98`, `124`, `143`, `355`, `369`, `673`, `852`, `898`
  - `crates/contextra-crypto/tests/deletion_proof_external_verify_rejects_tamper.rs:113`
  - `crates/contextra-crypto/tests/deletion_proof_signature_version_serde.rs:121`
  - `crates/contextra-crypto/tests/deletion_proof_v3_validation.rs:24`, `71`, `107`
  - `crates/contextra-crypto/tests/guarantee_deletion_irrecoverable.rs:486`
  - `crates/contextra-crypto/tests/j25-contextra-crypto-crypto-shredding-audit-_wiring_test.rs:70`
  - `crates/contextra-crypto/tests/p04b_deletion_proof_hardening.rs:210`
- **Benches / Fuzzing:** 0 Aufrufe

### 1.4 `DeletionProof::create_v3_with_audit_position`
- **Produktion:** 0 Aufrufe
- **Tests (2 Aufrufe):**
  - `crates/contextra-crypto/tests/j25-contextra-crypto-crypto-shredding-audit-_wiring_test.rs:85`
  - `crates/contextra-crypto/tests/j25closure-contextra-crypto_symbols_test.rs:64`
- **Examples / Benches / Fuzzing:** 0 Aufrufe

### 1.5 `DeletionProof::create_with_wal_receipt_v3`
- **Produktion:** 0 Aufrufe
- **Tests (3 Aufrufe):**
  - `crates/contextra-crypto/src/deletion_proof.rs:163`, `909`
  - `crates/contextra-crypto/tests/j25-contextra-crypto-crypto-shredding-audit-_wiring_test.rs:109`
- **Examples / Benches / Fuzzing:** 0 Aufrufe

### 1.6 `DeletionProof::create_full_v3`
- **Produktion:** 0 Aufrufe
- **Tests (4 Aufrufe):**
  - `crates/contextra-crypto/src/deletion_proof_typestate.rs:335`
  - `crates/contextra-crypto/tests/guarantee_deletion_irrecoverable.rs:258`, `355`, `402`
- **Examples / Benches / Fuzzing:** 0 Aufrufe

### 1.7 `KeyManager::create_deletion_proof(_v3)`
- **Produktion:** 0 Aufrufe
- **Tests (2 Aufrufe):**
  - `crates/contextra-crypto/tests/j24closure-contextra-crypto_closure_tests.rs:48` (`km.create_deletion_proof`)
  - `crates/contextra-crypto/tests/p04b_deletion_proof_hardening.rs:177` (`KeyManager::create_deletion_proof_v3`)
- **Examples / Benches / Fuzzing:** 0 Aufrufe

### 1.8 `DeletionProofBuilder` (`finish`)
- **Produktion:** 0 Aufrufe
- **Tests (1 Aufruf):** `crates/contextra-crypto/src/deletion_proof_typestate.rs:343` *(doc-comment Beispiel in `deletion_proof_typestate.rs:23-50` existiert ebenfalls)*
- **Examples / Benches / Fuzzing:** 0 Aufrufe

### 1.9 `LayerCleanupProof::new_after_verified_empty`
- **Produktion (1 Aufruf):**
  - `crates/contextra-engine/src/contextra_impl/collections.rs:384`
- **Examples (3 Aufrufe):**
  - `crates/contextra-db/examples/verify_deletion_proof_external.rs:112`, `113`, `114`
- **Fuzzing (1 Aufruf):**
  - `crates/contextra-crypto/fuzz/fuzz_targets/deletion_proof_tamper.rs:50`
- **Tests (29 Aufrufe):**
  - `crates/contextra-crypto/src/deletion_proof.rs:105`, `170`, `400`, `443`, `492`, `665`, `798`, `811`, `860`, `903`
  - `crates/contextra-crypto/src/deletion_proof_typestate.rs:309`, `311`, `313`, `315`, `318`, `320`, `322`
  - `crates/contextra-crypto/tests/j25-contextra-crypto-crypto-shredding-audit-_wiring_test.rs:68`
  - `crates/contextra-crypto/tests/j25closure-contextra-crypto_symbols_test.rs:60`
  - `crates/contextra-crypto/tests/no_unverified_layer_proofs.rs:265`, `278`, `292`, `303`, `337`
  - `crates/contextra-crypto/tests/p04b_deletion_proof_hardening.rs:22`, `49`, `76`, `182`, `214`
  - `crates/contextra-crypto/tests/p06_persistent_shredding_durability_test.rs:152`, `153`
  - `crates/contextra-engine/tests/embedding_backend_candle_variant.rs:25`
  - `crates/contextra-engine/tests/no_crypto_proof_stub.rs:21`, `137`, `143`

### 1.10 `LayerCleanupProof::verify_and_create`
- **Produktion:** 0 Aufrufe
- **Tests (11 Aufrufe):**
  - `crates/contextra-crypto/src/deletion_proof.rs:835`, `841`, `847`
  - `crates/contextra-db/tests/deletion_proof_integration.rs:232`, `237`, `253`, `258`
  - `crates/contextra-engine/tests/embedding_backend_candle_variant.rs:31`
  - `crates/contextra-engine/tests/no_crypto_proof_stub.rs:112`, `118`, `126`

### Zusammenfassung Question 1
Anzahl der Produktionsaufrufer zur Erzeugung eines `DeletionProof` außerhalb von `collections.rs`: **EXAKT 0**.
Einziger Produktionsaufrufer im gesamten Workspace ist `crates/contextra-engine/src/contextra_impl/collections.rs:397`.

---

## 2. Prüfung in `verify.rs` (v1/v2/v3) & Mindestabdeckungsprüfung

### Befund
Die Verifikationslogik in `crates/contextra-crypto/src/deletion_proof/verify.rs` (Zeilen 20–122 für `verify` sowie Zeilen 160–186 für `verify_external`) führt **ausschließlich eine kryptographische Signaturprüfung** durch.

- **v1 (`SignatureVersion::V1`, Zeilen 33–48):** Rechnet HMAC-SHA256 über `scope_bytes`, `deleted_keys_hash` und `tx_bytes` neu aus und vergleicht diese via `ConstantTimeEq` mit `self.signature`.
- **v2 (`SignatureVersion::V2`, Zeilen 49–93):** Rechnet HMAC-SHA256 über die serialisierte v2-Payload (`construct_v2_full_payload`) bzw. über die Legacy-Sequenz aus.
- **v3 (`SignatureVersion::V3`, Zeilen 94–121):** Prüft die Ed25519-Signatur über das durch `construct_v3_payload()` erzeugte Byte-Array.

### Bestätigung der Vermutung
**Ja, die Vermutung wird vollumfänglich bestätigt.**
Es gibt in `verify.rs` **keinerlei Mindestabdeckungsprüfung**. `verify.rs` prüft weder:
1. Ob `covered_layers` mindestens einen bestimmten Layer (z. B. `LsmMemtable` oder `SsTableAllLevels`) enthält,
2. Ob `covered_layers` leer ist (`covered_layers.is_empty()`),
3. Noch ob Nicht-Abdeckungen korrespondierende Markierungen besitzen.

Ein `DeletionProof`, bei dem `covered_layers` eine leere Liste `vec![]` oder nur einen einzigen Layer `vec![LsmMemtable]` enthält, wird von `verify(&key)` und `verify_external(&vk)` als **vollständig gültig (`Ok(true)` / `Ok(())`)** akzeptiert, solange die kryptographische Signatur über die im Proof enthaltenen Felder korrekt ist.

---

## 3. Felder der signierten v3-Payload & Eignung von `ExcludedScope`

### 3.1 Felder in der signierten v3-Payload
In `crates/contextra-crypto/src/deletion_proof/proof_v3.rs:125–171` stellt `construct_v3_payload(&self)` die Bytes zusammen, über die Ed25519-Signatur erzeugt und verifiziert wird. Die Payload besteht aus genau 9 konsekutiven Komponenten:

1. `scope_bytes` (`bincode::serialize(&self.scope)`, Zeile 126)
2. `deleted_keys_hash` (`32 Bytes` Blake3, Zeile 156)
3. `tx_bytes` (`self.deleted_after_tx.0.to_le_bytes()`, Zeile 128)
4. `timestamp_bytes` (`self.timestamp.to_le_bytes()`, Zeile 129)
5. `covered_layers_bytes` (`bincode::serialize(&self.covered_layers)`, Zeile 130)
6. `excluded_scopes_bytes` (`bincode::serialize(&self.excluded_scopes)`, Zeile 132)
7. `graph_repair_bytes` (`bincode::serialize(&self.graph_repair)`, Zeile 134)
8. `receipt_part` (`wal_chain_receipt` mit 32 Bytes oder leer, Zeilen 136–141)
9. `audit_pos_part` (`audit_chain_position` als 8 Bytes LE oder leer, Zeilen 143–148)

### 3.2 Analyse von `ExcludedScope`
`ExcludedScope` ist in `crates/contextra-crypto/src/deletion_proof/types.rs:56–62` wie folgt definiert:

```rust
pub enum ExcludedScope {
    /// Wissen in Zusammenfassungen die als LLM-Fine-Tuning-Input dienten.
    ConsolidatedAndDistilled,
    /// LLM-Modellparameter (arXiv:2505.16831 — Unlearning Isn't Deletion).
    LlmParameterMemory,
}
```

**Ergebnis:**
`ExcludedScope` drückt **funktionale/KI-bezogene Daten-Scopes** aus, bei denen eine mathematisch garantierte Löschung prinzipbedingt unmöglich oder unentscheidbar ist (LLM-Gewichte, Fine-Tuning-Derivate).
`ExcludedScope` ist **nicht geeignet**, die Nicht-Abdeckung eines konkreten Storage-Layers (z. B. "SSTable-Ebene nicht bereinigt/geprüft") auszudrücken, da es weder eine entsprechende Enum-Variante enthält noch für Storage-Layer-Semantik konzipiert ist.

---

## 4. Verhalten von Konsumenten bei partieller Abdeckung (`[LsmMemtable]`)

### 4.1 `contextra-db/examples/verify_deletion_proof_external.rs`
- **Ablauf (Zeilen 40–51 & 180–190):** Liest Proof-JSON von Disk und ruft `verify_external(&proof, &verifying_key)` auf.
- **Verhalten bei `covered_layers = [LsmMemtable]`:** Gibt `Ok(())` zurück und gibt auf `stdout` die Felder `covered_layers` und `excluded_scopes` rein informativ aus (Zeilen 187–188). Es erfolgt keine Validierung der Layer-Menge.

### 4.2 `crates/contextra-audit-export`
- **Ablauf (`lib.rs:32–41` & `audit_export.rs:10–24`):** Erzeugt aus einem `DeletionProof` per `export_for_audit()` eine JSON-Repräsentation für den DSGVO Art. 30 Audit-Export.
- **Verhalten bei `covered_layers = [LsmMemtable]`:** Exportiert das JSON unverändert. Für v1-Legacy-Proofs wird eine Warnung `integrity_warning` eingefügt. Die Abdeckung selbst wird weder gefiltert noch als unzureichend markiert.

### 4.3 `crates/contextra-mcp`
- **Ablauf (`tools_crud.rs:133` & `server_tools.rs:458`):** `contextra_drop_collection` ruft `col.drop_collection()` auf, nimmt den zurückgegebenen `DeletionProof`, exportiert diesen via `export_for_audit()` und reicht die JSON-Zeichenkette an den MCP-Client weiter.
- **Verhalten bei `covered_layers = [LsmMemtable]`:** Der MCP-Server leitet den Proof transparent als JSON an den Anrufer weiter. Es gibt keine Prüfung der Layer-Vollständigkeit. Single-Document-Löschungen stellen per Design keinen Proof aus (`proof: null`, `sandbox.rs:231`).

### 4.4 `crates/contextra-engine`
- **Ablauf (`collections.rs:384–397`):** `Contextra::drop_collection` erzeugt explizit einen Proof mit nur einem Layer:
  `layer_proofs = vec![LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, ...)]`.
- **Begründung im Code (`collections.rs:373–381`):**
  > *"LSM deletion via delete_prefix is tombstone-only. ... Therefore, drop_collection claims ONLY DeletionLayer::LsmMemtable. SsTableAllLevels or WalAllSegments may only be re-added together with a real physical verifier that inspects disk storage."*

**Zusammenfassung:**
Sämtliche Konsumenten im Workspace stufen einen `DeletionProof` mit `covered_layers = [LsmMemtable]` als **100 % valide und vollständig** ein.

---

## 5. Die 7-Layer Typestate-Mechanik (`deletion_proof_typestate.rs`)

### 5.1 Funktionsweise
`DeletionProofBuilder` in `crates/contextra-crypto/src/deletion_proof_typestate.rs` nutzt compile-zeitliches Typestate mit 7 phantom-typisierten Schichten (`Lsm`, `Sst`, `Hnsw`, `Wal`, `Csr`, `Kv`, `Emb`), die initial alle den Marker `Missing` tragen.

- Für jeden Layer existiert eine Setter-Methode (z. B. `with_lsm_memtable_cleanup`, `deletion_proof_typestate.rs:141`), die den Phantom-Typ von `Missing` auf `Cleaned` überführt.
- Die Methode `.finish()` (`deletion_proof_typestate.rs:301–326`) ist **ausschließlich** für die voll-attestierte Variante `DeletionProofBuilder<Cleaned, Cleaned, Cleaned, Cleaned, Cleaned, Cleaned, Cleaned>` implementiert.
- Fehlt auch nur ein einziger Layer, bricht der Rust-Compiler beim Aufruf von `.finish()` ab (`method finish not found`).

### 5.2 Nutzung im Produktionscode
- **Anzahl der Produktionsaufrufer:** **EXAKT 0**.
- **Beleg:** Eine Workspace-weite Suche zeigt, dass `DeletionProofBuilder` im gesamten Produktionscode von `crates/**` an keiner Stelle aufgerufen wird.
- **Einziges Vorkommen:** Der interne Unit-Test `test_typestate_builder_complete_chain_matches_direct_creation` in `crates/contextra-crypto/src/deletion_proof_typestate.rs:343`.
- Production (`collections.rs:397`) umgeht den Typestate-Builder vollständig durch direkten Aufruf von `DeletionProof::create(...)` mit einem 1-elementigen Vektor.

---

## 6. Optionen für eine signierte Nicht-Abdeckung (Entscheidungsgrundlage ADR-4)

Soll in einem `DeletionProof` explizit signiert und prüfbar dokumentiert werden, dass bestimmte Storage-Layer **nicht geprüft** oder **nicht bereinigt** wurden, bestehen zwei primäre Architekturoptionen.

---

### Option (a): Erhöhung der Ausdrucksstärke im bestehenden Datenmodell (ohne v4-Payload-Formatänderung)

#### Konzept
Nutzung/Erweiterung der bestehenden, bereits signierten Felder `covered_layers` oder `excluded_scopes`:
- **Variante a.1:** Erweiterung von `DeletionLayer` um Unverifiziert-Statusvarianten, z. B.:
  `enum DeletionLayerStatus { Cleaned(DeletionLayer), Unverified(DeletionLayer) }` oder direkte Varianten in `DeletionLayer` wie `DeletionLayer::UnverifiedSsTable`.
- **Variante a.2:** Erweiterung von `ExcludedScope` um ungeprüfte Storage-Layer:
  `ExcludedScope::UnverifiedStorageLayer(DeletionLayer)`.

#### Kompatibilität mit bestehenden v3-Proofs
- **Vollständig abwärtskompatibel:** `construct_v3_payload` serialisiert `excluded_scopes` und `covered_layers` via `bincode::serialize`.
- Da Ed25519 die Signatur über den Bincode-Byte-Stream der Enums bildet, können v3-Verifizierer (`verify()` und `verify_external()`) den Proof ohne Ändern des Verifikations-Codes erfolgreich verifizieren.
- Ältere Deserialisierer würden bei unbekannten Enum-Varianten jedoch Deserialisierungsfehler werfen.

#### Betroffene Dateien
- `crates/contextra-crypto/src/deletion_proof/types.rs` (Erweiterung Enum `ExcludedScope` oder `DeletionLayer`)
- `crates/contextra-crypto/src/deletion_proof/audit_export.rs` (Anpassung JSON-Export)
- `crates/contextra-engine/src/contextra_impl/collections.rs` (Befüllen des Feldes bei `drop_collection`)
- Tests in `contextra-crypto`, `contextra-db`, `contextra-mcp`

#### Geschätzter Umfang
- **3–5 Dateien**, ca. **60–100 Zeilen**.

---

### Option (b): Neues explizites Payload-Feld → Neue Proof-Version v4

#### Konzept
Einführung eines dedizierten Feldes `pub unverified_layers: Vec<DeletionLayer>` in `DeletionProof` sowie Erhöhung der Signaturversion auf `signature_version = 4`.

#### Ablauf & Konstruktion
1. `SignatureVersion::V4 = 4` in `crates/contextra-crypto/src/ed25519_proof.rs` ergänzen.
2. `construct_v4_payload()` in `proof_v4.rs` nimmt `unverified_layers_bytes` explizit in die Ed25519-Signaturkette auf.
3. In `verify.rs` wird ein Match-Arm für `SignatureVersion::V4` hinzugefügt.

#### Kompatibilität mit bestehenden v3-Proofs
- **100 % abwärtskompatibel zu v1, v2 und v3:** Bestehende v1/v2/v3-Proofs verifizieren unverändert über ihre jeweiligen Match-Arme in `verify.rs`.
- Verifikatoren, die v4 noch nicht unterstützen, lehnen v4-Proofs mit `CryptoError::UnsupportedProofVersion(4)` ab (fail-closed).

#### Betroffene Dateien
- `crates/contextra-crypto/src/ed25519_proof.rs` (Typ `SignatureVersion`, `TryFrom<u8>`)
- `crates/contextra-crypto/src/deletion_proof/proof.rs` & `proof_v4.rs` (neues Modul/Erweiterung `DeletionProof`-Struct)
- `crates/contextra-crypto/src/deletion_proof/verify.rs` (Match-Arm `V4` in `verify` und `verify_external`)
- `crates/contextra-crypto/src/deletion_proof/audit_export.rs` (v4 JSON-Export)
- `crates/contextra-engine/src/contextra_impl/collections.rs` (Umstellung auf `create_v4` mit expliziten unverified_layers)
- `crates/contextra-db/examples/verify_deletion_proof_external.rs` (Aktualisierung Verifikations-Beispiel)
- Integrationstests in `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-mcp`

#### Geschätzter Umfang
- **8–12 Dateien**, ca. **250–400 Zeilen**.

---

## 7. Zusammenfassende Matrix

| Kriterium | Aktueller Zustand (v3) | Option (a) Erweiterung `ExcludedScope` | Option (b) Neue Version v4 |
|---|---|---|---|
| **Storage-Layer Nicht-Abdeckung ausdrückbar?** | Nein (nur `covered_layers`) | Ja (über neue Enum-Varianten) | Ja (über eigenes Feld `unverified_layers`) |
| **Prüfung in `verify.rs`?** | Nur Signatur | Nur Signatur | Signatur über explizite Nicht-Abdeckung |
| **Abwärtskompatibel mit v3-Proofs?** | Ja | Ja | Ja |
| **Breaking Change für Verifikatoren?** | Nein | Nur bei Enum-Deserialisierung | Ja (erfordert v4-Parser) |
| **Geschätzter Aufwand** | 0 Zeilen | ~60–100 Zeilen (3–5 Dateien) | ~250–400 Zeilen (8–12 Dateien) |

---
*Ende des Berichts — `docs/audits/deletion-proof-coverage-semantics-2026-10.md`*
