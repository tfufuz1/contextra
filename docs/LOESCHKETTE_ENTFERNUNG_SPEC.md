# Implementierungsspezifikation: Entfernung der Löschkette

> **Status:** Normative Umsetzungsanweisung  
> **Bezug:** [CONTEXTRA_SPEZIFIKATION_SOLL.md](CONTEXTRA_SPEZIFIKATION_SOLL.md) (Abschnitte 1.4, 2.2 [F2–F4], 6.9, 13.5, 13.6, 16.1, 16.2, Entscheidung **D-06**)  
> **Ziel:** Vollständiges, sauberes und compiler-geführtes Herausschneiden der gesamten DSGVO-Löschkette, kryptographischer Löschnachweise (`DeletionProof`, `LayerCleanupProof`, `Ed25519`), `Crypto-Shredding` (`kv_shredding`), `AuditChain`, `RevocationLog` sowie der Compliance-Generatoren (`contextra-audit-export`, `contextra-avv-generator`) über alle betroffenen Crates hinweg.

---

## 1. Architektonisches Zielbild (Ersatz für die Löschkette)

Gemäß Entscheidung **D-06** (`CONTEXTRA_SPEZIFIKATION_SOLL.md` §13.5) wird die komplexe Löschkette vollständig durch ein einfaches, versioniertes Lebenszyklus-Modell ersetzt:

| Mechanismus | Semantik | Garantien / Nicht-Garantien |
|---|---|---|
| **`retire(gid)`** (Standard) | Schreibt einen **versionierten Tombstone-Commit** (bzw. `Death`-Eintrag). Ab dem Commit-Snapshot in allen Sichten und Suchen unsichtbar; in Zeitreise-Abfragen innerhalb der Retention weiterhin sichtbar. | Kein Löschnachweis, keine Signatur, keine Kaskadenlöschung (Kanten-Sichtbarkeit folgt Endpunkten zur Lesezeit). |
| **Retention + Compaction** | Konfigurierbar je Namespace (`history_days`, `branches_days`, optional `history_versions`). Nach Ablauf entfernt die reguläre Compaction tote Zeilen physisch. | Kein Beleg, kein Layer-Abdeckungsbericht, kein Crypto-Shredding. |
| **`purge(filter)`** (Admin, Best Effort) | Setzt Tombstones auf alle passenden Records, überschreibt den Retention-Horizont auf „jetzt“ (sofern keine Pins/Tags/Branches blockieren) und triggert Compaction. | **Best Effort ohne Garantie**; blockierende Pins/Tags werden nur operativ gemeldet. |
| **Derivate-GC** | Embeddings, Summaries und Token-Counts hängen am `ContentHash` und werden bei Compaction entfernt, sobald kein lebender oder retention-pflichtiger Record mehr darauf verweist. | Referenzzählung im Compaction-Lauf statt `LayerCleanupProof`. |
| **Betriebsprotokolle** | Strukturierte `tracing`-Events (JSON-Lines) für Schreibzugriffe, Merges, Gates und Egress-Entscheidungen. | **Nicht kryptographisch verkettet** (`AuditChain` entfällt). |

---

## 2. Betroffene Crates und Schnitt-Reihenfolge

Die Entfernung erfolgt **nicht als Patch**, sondern als sauberer, compiler-gestützter Schnitt von unten nach oben (**Bottom-Up** entlang der Ring-Abhängigkeiten), damit kein Zwischenstand dauerhaft zirkuläre Altlasten mitschleppt:

```text
Welle 0: Workspace-Manifest & Feature-Flags (Cargo.toml)
   │
Welle 1: Eigenständige Compliance-Crates entfernen
   │     (contextra-audit-export, contextra-avv-generator)
   ▼
Welle 2: Fundament & Storage (Ring 0 & 1)
   │     contextra-types → contextra-crypto → contextra-store → contextra-kvcache
   ▼
Welle 3: Indizes, Engine & Privacy (Ring 2 & 3)
   │     contextra-graph → contextra-vector → contextra-engine → contextra-privacy
   ▼
Welle 4: Fassaden, APIs & Werkzeuge (Ring 4 & xtask)
   │     contextra → contextra-db → contextra-mcp → contextra-py → xtask
   ▼
Welle 5: Dokumentationsbereinigung & CI-Grep-Gate
```

> **Hinweis zur VM-Ressourcengrenze (`CLAUDE.md`):** Jeder Schritt wird pro Crate mittels `bash tools/ctx.sh check <crate>` und `bash tools/ctx.sh test <crate>` (mit `--locked`) verifiziert, nicht via `--workspace`.

---

## 3. Detaillierter Änderungsplan pro Welle & Crate

### Welle 0: Workspace (`Cargo.toml`)

- **Datei:** `Cargo.toml`
  - Aus `workspace.members` entfernen:
    - `"crates/contextra-audit-export"`
    - `"crates/contextra-avv-generator"`
  - Aus `[workspace.dependencies]` entfernen:
    - `contextra-audit-export`
    - `contextra-avv-generator`

---

### Welle 1: Compliance-Crates vollständig entfernen

1. **`crates/contextra-audit-export/`** — Verzeichnis vollständig löschen (inkl. `src/lib.rs`, `src/bsi_mapping.rs`, `src/markdown_template.rs`, `src/testkit.rs` und aller Tests unter `tests/`).
2. **`crates/contextra-avv-generator/`** — Verzeichnis vollständig löschen (inkl. `src/` und aller Tests unter `tests/`).

---

### Welle 2: Fundament & Storage (Ring 0 & Ring 1)

#### 2.1 `contextra-types` (`crates/contextra-types/`)
- **`src/error.rs` & `src/error_dto.rs`**:
  - Entfernen der Löschketten-Fehlervarianten, insbesondere:
    - `KvDeleteModeConfig(&'static str)`
    - `HnswDeletionError` / `GraphRepairFailed(#[from] HnswDeletionError)` (sofern an die Löschbeweis-/Reparatur-Kette gekoppelt)
    - alle weiteren Varianten für `DeletionProof`, `CryptoShred`, `KvDeleteMismatch`, `DurabilityVectorMismatch`.

#### 2.2 `contextra-crypto` (`crates/contextra-crypto/`)
Gemäß `CONTEXTRA_SPEZIFIKATION_SOLL.md` §16.1/§16.2 radikal auf **AEAD, KDF und Key-Provider** reduzieren:
- **Zu entfernende Quelldateien:**
  - `src/deletion_proof.rs`
  - `src/deletion_proof_typestate.rs`
  - `src/ed25519_proof.rs`
  - `src/kv_shredding.rs`
  - `src/kv_cipher.rs`
  - `src/kv_segment/` (inkl. `segment.rs`)
  - `src/revocation_log.rs`
  - `src/audit_chain.rs`
  - `src/wal_completeness.rs` und `src/anti_tamper.rs` (sofern vorhanden und Teil der HMAC-Löschbeweis-Kette)
- **Anzupassende Dateien:**
  - `src/lib.rs`: Alle `mod`-Deklarationen und `pub use`-Re-Exports der oben genannten Module entfernen.
  - `src/crypto.rs`: Legacy-HMAC-/Löschbeweis-Hilfsfunktionen entfernen; nur AEAD (`AES-256-GCM-SIV`) und KDF/Key-Management behalten.
  - `Cargo.toml`: Feature `kv-encryption` (falls ausschließlich für KV-Shredding verwendet) entfernen oder auf reine Session-/AEAD-Verschlüsselung umstellen; nicht mehr benötigte Abhängigkeiten (z. B. `ed25519-dalek`, falls nur für `ed25519_proof.rs` genutzt) prüfen und bereinigen.
- **Zu entfernende Test-/Fuzz-/Bench-Artefakte:**
  - `benches/deletion_proof_latency_bench.rs`
  - `fuzz/fuzz_targets/deletion_proof_tamper.rs`, `fuzz/fuzz_targets/deletion_proof_verify_external_fuzz.rs`
  - Tests: `audit_chain_anchor.rs`, `audit_chain_tests.rs`, `campaign_cov_deletion_proof.rs`, `campaign_cov_revocation_log.rs`, `campaign_j26_hypotheses.rs`, `deletion_proof_*.rs`, `ed25519_proof_external_verify.rs`, `guarantee_audit_chain_tamper.rs`, `guarantee_deletion_irrecoverable.rs`, `kv_revocation_wiring.rs`, `legacy_hmac_collision_property.rs`, `no_unverified_layer_proofs.rs`, `revocation_and_wal_error_paths.rs`, `revocation_log_atomic_persist.rs`, sowie `j24*`/`j25*`-Wiring-Tests für Shredding/Audit-Chain.

#### 2.3 `contextra-store` (`crates/contextra-store/`)
- **`Cargo.toml`**:
  - Features `deletion-proof` und `legacy-wal-key` sowie zugehörige `required-features = ["legacy-wal-key"]`-Test-Deklarationen entfernen.
- **Quellcode (`src/`):**
  - `src/kv/delete_mode.rs` entfernen (`KvDeleteMode`).
  - `src/kv/segment.rs` und `src/kv/mod.rs`: `Crypto-Shredding`-, `DeletionProof`-Registry- und `KvDeleteMode`-Verdrahtung entfernen.
  - `src/lsm/config.rs`: Konfigurationsfelder für `deletion_proof_active`, `KvDeleteMode` und `legacy_wal_key` entfernen.
  - WAL-Integrität auf CRC32C je Record (plus optional AEAD) umstellen; HMAC-Verkettung der Löschbeweis-Ära entfernen (`CONTEXTRA_SPEZIFIKATION_SOLL.md` §6).
- **Tests (`tests/`):**
  - Entfernen: `deletion_proof_unregistered.rs`, `kv_crypto_shred_instant_unrecoverable.rs`, `kv_delete_mode_wiring_test.rs`, `kv_deletion_proof_registry.rs`, sowie Löschketten-Anteile in `campaign_j21_hypotheses.rs`, `campaign_j42_audit.rs`, `durability_mode_feature_gate.rs`, `j07*`-Tests.

#### 2.4 `contextra-kvcache` (`crates/contextra-kvcache/`)
- **`Cargo.toml`**:
  - Feature `kv-encryption` (sofern an `ShreddableSegmentKey`/Crypto-Shredding gekoppelt) entfernen; Verweis in `contextra-infer-candle/Cargo.toml` (`kv-bridge`) prüfen und anpassen.
- **Quellcode (`src/segment.rs`, `src/lib.rs`):**
  - `ShreddableSegmentKey` und alle Crypto-Shredding-Methoden/Zeroize-Garantien zum Zweck des Löschnachweises aus `src/segment.rs` entfernen.
  - Falls Tier-2-Segmentverschlüsselung erhalten bleibt, nutzt sie den regulären `security`/AEAD-Key-Provider ohne Shredding-Nachweis.
- **Tests:** `tests/e2e_lifecycle_with_engine.rs` von Löschbeweis-/Shredding-Assertions bereinigen.

---

### Welle 3: Indizes, Engine & Privacy (Ring 2 & Ring 3)

#### 3.1 `contextra-graph` (`crates/contextra-graph/`)
- **`src/cascade.rs` & `src/lib.rs`**:
  - Kaskadenlöschung (`cascade.rs`) entfernen. Gemäß Soll-Spezifikation §10.2 erfolgt beim `retire` eines Knotens **kein** kaskadierendes Löschen von Kanten; Kanten-Sichtbarkeit wird zur Lesezeit über die Frontier-/Liveness-Bitmap beider Endpunkte bestimmt.

#### 3.2 `contextra-vector` (`crates/contextra-vector/`)
- Gemäß Soll-Spezifikation §2.2 (Befund F2) und §16.1:
  - `GraphRepairAttestation`-Bezüge sowie mutierbare Soft-Delete-/Nachbarschaftsreparatur-Pfade (`partial-index-rebuild`, `deletion.rs`-Reparatur-Guards), die für den sofortigen Löschnachweis eingeführt wurden, schrittweise durch **immutable Segmente + Liveness-Bitmap** ablösen.
  - Während des Schnitts zunächst sicherstellen, dass keine `GraphRepairAttestation`- oder `DeletionProof`-Typen mehr referenziert werden.

#### 3.3 `contextra-engine` (`crates/contextra-engine/`)
- **`src/lib.rs`**:
  - Vollständig entfernen: `DeletionLayer`, `DeletionScope`, `LayerCleanupProof`, `DeletionProof` sowie alle Hilfsfunktionen zur Beweis-Erzeugung/Signierung.
- **`src/collection/mod.rs`, `src/contextra_impl/collections.rs`, `src/contextra_impl/lifecycle.rs`**:
  - Lösch-Orchestrierung über `DeletionLayer` / `LayerCleanupProof` entfernen.
  - Stattdessen `retire` (Tombstone-Commit) und `purge` (Best-Effort-Retention-Override + Compaction-Auftrag) als einfache Lebenszyklus-Operationen bereitstellen.
- **Tests:** `tests/no_crypto_proof_stub.rs` entfernen; `invariant_tenant_isolation.rs`, `tenant_index_persistence_reload.rs`, `tenant_wrapper_coverage.rs` auf `retire`/`purge` ohne `DeletionProof` umstellen.

#### 3.4 `contextra-privacy` (`crates/contextra-privacy/`)
- **Zu entfernende Dateien:**
  - `src/avv_generator.rs`
  - `src/processing_registry.rs`
  - `src/context_edit_audit.rs` (sofern an kryptographische Audit-Chain gebunden)
- **Anzupassende Dateien:**
  - `src/lib.rs`, `src/egress_gateway.rs`: Verweise auf `avv_generator`, `processing_registry`, `DeletionProof` und `crypto_shred`-Zwang entfernen. Egress-Guard, `BulkExfiltrationDetector`, `InjectionDetector` und `GuardedPayload` bleiben erhalten.
- **Tests:** `tests/pii_vault_forces_crypto_shred.rs` und `tests/context_edit_audit_chain.rs` entfernen; `campaign_cov_privacy_egress.rs` und `campaign_j27_privacy_test.rs` bereinigen.

---

### Welle 4: Fassaden, APIs & Werkzeuge (Ring 4 & `xtask`)

#### 4.1 `contextra` (Facade, `crates/contextra/`)
- **`Cargo.toml`**:
  - Optionale Dependencies `contextra-audit-export`, `contextra-avv-generator` sowie Features `compliance`, `audit-export`, `avv-generator` entfernen.
- **`src/builder.rs`, `src/collection_profile.rs`, `src/performance_profile.rs`**:
  - Löschketten-Profile und Fehler (`KvDeleteMismatch`, `DurabilityVectorMismatch`, `DeletionProof`-Konfiguration) entfernen.
- **Tests:** `collection_profile_matrix.rs`, `builder_levers_contract.rs`, `license_gate_always_enforced.rs`, `tenant_isolation_facade.rs` entsprechend anpassen.

#### 4.2 `contextra-db` (`crates/contextra-db/`)
- Entfernen von:
  - `examples/verify_deletion_proof_external.rs`
  - `tests/deletion_proof_integration.rs`
  - `tests/drop_collection_proof_vs_files.rs`
  - `tests/e2e_compliance_scenario.rs`
- *(Hinweis: Gemäß §16.1 entfällt `contextra-db` perspektivisch vollständig zugunsten des neuen `engine`-Schnitts.)*

#### 4.3 `contextra-mcp` (`crates/contextra-mcp/`)
- **Zu entfernende Dateien:**
  - `src/proof_key.rs`
  - `src/proof_key_env.rs`
- **Anzupassende Dateien:**
  - `src/lib.rs`, `src/server.rs`, `src/server_tools.rs`, `src/server_dispatch.rs`, `src/tools_crud.rs`, `src/sandbox.rs`:
    - Alle `proof_key*`-Initialisierungen, `DeletionProof`-Rückgaben und alte `forget`/Lösch-Tools entfernen.
    - Durch deklarativ klassifiziertes `ctx_retire` (Schreiboperation, Begründung Pflicht) ersetzen (`CONTEXTRA_SPEZIFIKATION_SOLL.md` §12.3).
- **Tests:** `tests/contextra_delete_no_unverified_proof.rs` entfernen; `src/tests.rs` sowie `campaign_j42_mcp_audit.rs`, `confirm_required_semantics.rs`, `j18*`-Tests bereinigen.

#### 4.4 `contextra-py` (`crates/contextra-py/`)
- **`src/bindings/db.rs` & Python-Tests (`tests/test_bindings.py`, `tests/test_errors.py`)**:
  - Python-Methoden und Fehlertypen für `DeletionProof`, `KvDeleteMode` und `AuditChain` entfernen; durch `retire` / `purge` ersetzen.

#### 4.5 `xtask` (`xtask/`)
- **`src/check_manifest_completeness.rs` & `src/harness/gate_arch.rs`**:
  - Referenzen auf `contextra-audit-export`, `contextra-avv-generator` und Feature-Checks für `deletion-proof` / `legacy-wal-key` entfernen.
- **Neues CI-Gate (`gate_no_deletion_chain`):**
  - Verpflichtendes `grep`-Gate in `xtask` einrichten (siehe Abschnitt 4).

---

### Welle 5: Dokumentation & CI-Verifikation

1. **Dokumentationsbereinigung:**
   - Veraltete Spezifikations- und Audit-Reste (`docs/SPEC.md` Teile B.6, K.3/K.4/K.8, L.1, Veto `F-02`, `AGENTS.md` in `contextra-crypto`/`contextra-privacy`/`contextra-store`, `README.md` in `contextra-crypto`) archivieren (`docs/history/`) oder aktualisieren.
2. **Feature-Katalog neu generieren:**
   - `cargo xtask gen-feature-catalog` ausführen, damit [feature-catalog.md](reports/feature-catalog.md) keine entfernten Crates/Features mehr aufführt.

---

## 4. Verifikations- und Abnahmekriterien

Die Entfernung gilt erst als abgeschlossen, wenn **alle** folgenden Kriterien erfüllt sind:

1. **Null-Treffer-Gate im Repository** (außerhalb von `docs/history/` bzw. explizit archivierter Historie):
   ```bash
   rg -i "deletion_proof|DeletionProof|crypto_shred|kv_shred|revocation_log|audit_chain|AuditChain|GraphRepairAttestation|LayerCleanupProof|ExcludedScope|KvDeleteMode|DeletionLayer|contextra-audit-export|contextra-avv-generator|deletion-proof|legacy-wal-key" \
      Cargo.toml crates/ xtask/ benchmarks/
   ```
   **Erwartung:** `0` Treffer (Exit-Code `1`).

2. **Crate-weise Build- & Test-Verifikation** (mit `--locked`, gemäß VM-Runbook):
   - Für jedes betroffene Crate `bash tools/ctx.sh check <crate>` und `bash tools/ctx.sh test <crate>` erfolgreich durchlaufen lassen (`contextra-types`, `contextra-crypto`, `contextra-store`, `contextra-kvcache`, `contextra-graph`, `contextra-vector`, `contextra-engine`, `contextra-privacy`, `contextra`, `contextra-db`, `contextra-mcp`, `contextra-py`, `xtask`).

3. **Funktionsnachweis Ersatzmodell:**
   - Tests für `retire` (sofortige Unsichtbarkeit ab Commit-Snapshot, Sichtbarkeit bei Zeitreise innerhalb der Retention), Retention-Ablauf in Compaction sowie `purge` (Best Effort + Meldung blockierender Pins) sind grün.
