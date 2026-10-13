# AUDIT-Bericht: MVCC & Commit-Pfad (`contextra-mvcc`, `contextra-engine`)

**Spezialist:** Spezialist für MVCC, Serializable Snapshot Isolation und Transaktions-Commit
**Commit:** `59f853cb`
**Datum:** 2026-10-13
**Vorgänger:** `docs/audits/AUDIT_contextra-mvcc_2026-10-12.md` (Git HEAD `cf65dbcba86807c380a9dcbfd66523cb41528d18`)
`VERIFIED-BY-SESSION: PENDING`

<!-- EVIDENCE: logs/audits/mvcc_2026-10-13/cargo_test_mvcc.log (exit=0) -->
<!-- EVIDENCE: logs/audits/mvcc_2026-10-13/check_result_dropped_io.log (exit=1) -->

---

## 1 Scope und Abdeckung

| Datei / Komponente | Zeilen (LOC) | Gelesen (Ja/Nein) | Begründung / Bemerkung |
| :--- | :---: | :---: | :--- |
| `crates/contextra-mvcc/src/lib.rs` | 32 | Ja | Crate-Root, Exports |
| `crates/contextra-mvcc/src/seq_log.rs` | 425 | Ja | SequenceLog In-Memory Monotonic Sequence Tracking |
| `crates/contextra-mvcc/src/snapshot.rs` | 550 | Ja | SnapshotRegistry, Active Snapshot Tracking |
| `crates/contextra-mvcc/src/ssi.rs` | 910 | Ja | SequenceLogSsiValidator & ReadSet SSI Logic |
| `crates/contextra-mvcc/src/tx_buffer.rs` | 1420 | Ja | TxBuffer Sharded Staging |
| `crates/contextra-mvcc/src/floor.rs` | 65 | Ja | GcFloor Tombstone Watermark Calculation |
| `crates/contextra-mvcc/src/lease.rs` | 35 | Ja | RAII SnapshotLease Guard |
| `crates/contextra-engine/src/transaction/db_transaction_commit.rs` | 358 | Ja | 2-Phasen-Commit & CommitLedger Staging/Persistenz |
| `crates/contextra-engine/src/transaction/db_transaction_rollback.rs` | 288 | Ja | Rollback & Kompensationspfade |
| `crates/contextra-engine/src/collection/relate.rs` | 275 | Ja | Schreib- & Lesepfad Relationen (`relate`) |
| `crates/contextra-engine/src/collection/crud/insert.rs` | 420 | Ja | Document Insert / Upsert / Insert_Op |
| `crates/contextra-engine/src/collection/crud/update.rs` | 135 | Ja | Document Update / Update_Op |
| `crates/contextra-engine/src/collection/crud/delete.rs" | 380 | Ja | Document Erase / Delete_Op |
| `crates/contextra-engine/src/collection/crud/read.rs` | 260 | Ja | Document Read / Get_At_Snapshot |

---

## 2 Inventar der pub-Items

| Art | Symbol | Crate / Pfad | Nutzung |
| :--- | :--- | :--- | :--- |
| `struct` | `SnapshotRegistry` | `contextra_mvcc::SnapshotRegistry` | Produktiv (Storage, Engine) |
| `struct` | `SnapshotGuard` | `contextra_mvcc::SnapshotGuard` | Deprecated Wrapper / Produktiv |
| `struct` | `SnapshotLease` | `contextra_mvcc::SnapshotLease` | Produktiv (Ring 0 / LSM Engine) |
| `struct` | `GcFloor` | `contextra_mvcc::GcFloor` | Produktiv (GC Watermark Computation) |
| `struct` | `SequenceLog` | `contextra_mvcc::SequenceLog` | Produktiv (In-memory MVCC) |
| `struct` | `TxBuffer` | `contextra_mvcc::TxBuffer` | Produktiv (Staging Writes) |
| `struct` | `ReadSet` | `contextra_mvcc::ReadSet` | Produktiv (SSI Read-Tracking) |
| `struct` | `SequenceLogSsiValidator` | `contextra_mvcc::SequenceLogSsiValidator` | Produktiv (SSI Validation) |
| `struct` | `DbTransaction` | `contextra_engine::transaction::DbTransaction` | Produktiv (Engine 2PC) |

---

## 3 Fachliche Antworten auf die Prüffragen 1–7

### Frage 1: Lesezeitpunkte & `u64::MAX`-Analyse
- **Fundstellen-Klassifikation:**
  - `snapshot.rs` / `floor.rs`: `unwrap_or(u64::MAX)` beim Auslesen von Active Snapshot Sequences ist **KORREKT**, da ohne aktive Snapshots keine GC-Einschränkung gilt.
  - `insert.rs` (`insert_op`), `update.rs` (`update_op`), `delete.rs` (`delete_op` & `erase_document`), `relate.rs` (`relate`): `get_at_seq_tracked(tx, key, u64::MAX)`
    - **KLASSIFIKATION: SNAPSHOT-VERLETZUNG (HOCH).**
    - **Begründung:** Bei Schreib-Aktionen liest die Engine alte Werte zwecks Kompensation und Read-Tracking mit der expliziten Sequenz `u64::MAX` anstelle der `tx.read_snapshot_seq`. Dadurch liest eine lange laufende Transaktion uncommitted oder neuere Commits aus parallelen Transaktionen ("Dirty Read" / "Read Uncommitted" bezüglich des Read Snapshots), was die Snapshot-Isolation (ein `max_seq` je Transaktion) verletzt.

### Frage 2: SSI Read-Tracking Vollständigkeit
- **Ungetrackte Pfade:**
  - `relate.rs`: `relate()` liest Quell- und Zieldokument-Existenz über `get_at_seq_tracked(db_tx.tx_id, &key, u64::MAX)`. Das Read-Set zeichnet zwar den Schlüssel auf, aber mit Sequenz `u64::MAX` statt `read_snapshot_seq`.
  - `crud/links.rs:203`: `self.storage.get_at_seq(&doc_key, u64::MAX).await?` liest ungetrackt ohne Transaktionskontext.
  - `query_builder/builder_exec.rs`: Fällt bei fehlender TxId auf `u64::MAX` zurück.
- **Ergebnis:** Ungetrackte bzw. mit `u64::MAX` getrackte Lesezugriffe führen zu falschen Snapshot-Sequenzen im SSI `ReadSet`.

### Frage 3: SSI-Validator nach Cahill & Fekete / Fail-Open vs. Fail-Closed
- **Cahill/Fekete Abdeckung:** `SequenceLogSsiValidator` prüft Anti-Dependencies ($rw$-Antikanten) über direkte Key-Commits und Range-Prefix-Überlappungen.
- **Gefahrenstelle / Fail-Open Risiko:**
  - In `ssi.rs` (`check_and_coarsen_if_needed`): Wenn die Kapazitätsgrenze (`max_tracked_keys`, z. B. 1.000.000) zu 80% erreicht ist, skaliert der Validator alte Key-Buckets in Coarsened-Prefixes um. Das garantiert **0% False Negatives** (Fail-Closed für Echte Konflikte).
  - **ABER:** `validate_internal` wirft bei Erreichen von `max_tracked_keys` einen `ContextraError::Conflict`. Wenn der Aufrufer diesen Fehler abfängt oder als retriable ignoriert, entsteht Fail-Open Potenzial. Im Produktionscode von `db_transaction_commit.rs` wird `validate` des SSI-Validators **GAR NICHT AUFGERUFEN** vor Phase (b) LSM-Commit!

### Frage 4: Mandanten-ID Kontext & Fallbacks
- **Ableitung vs. Hardcoding:**
  - In `db_transaction_commit.rs:324`: `let tenant_id = TenantId::try_new(1).unwrap_or_default();` (Mandant 1 hartkodiert!).
  - In `db_transaction_rollback.rs:13`: `let tenant = contextra_types::TenantId::try_new(1).unwrap_or_default();` (Mandant 1 hartkodiert!).
- **Verstoß gegen Invariante:**
  - **INV-TENANT-1** besagt: Mandant 0 ist SYSTEM (`TenantId(0)` ist SYSTEM-reserved, `try_new(0)` -> Err). `unwrap_or_default()` liefert `TenantId(0)`.
  - Wenn `TenantId::try_new(1)` (oder bei Anpassungen) fehlschlägt, greift der Fallback auf `TenantId(0)` (SYSTEM-Mandant), was zu **Cross-Tenant Leakage / System-Key Isolation Violations** führt!

### Frage 5: Rollback-Vollständigkeit & Kompensations-Idempotenz
- **Vollständigkeit:** `db_transaction_rollback.rs` kompensiert LSM, HNSW, BM25 Text und CSR Graph.
- **Idempotenz:** Kompensierende Actions in `compensating_actions.rs` überschreiben Werte mit `old_val` oder löschen neu angelegte Schlüssel. Bei Wiederholung schreiben sie atomar denselben Zustand. Die Kompensation ist **IDEMPOTENT**.
- **Lücke:** Bei Fehlschlag von Kompensationsversuchen (z. B. 3 Versuche in `compensate_lsm`) bricht die Transaktion ab und loggt `split_brain` -> Split-Brain Zustand zwischen RAM-Indizes und LSM Store.

### Frage 6: Doppelte Snapshot-Typen
- **Analyse:**
  - `SnapshotGuard` (`snapshot.rs`): Deprecated RAII-Guard um `SnapshotRegistry`.
  - `SnapshotLease` (`lease.rs`): Kanonischer, neuer RAII Move-Only Guard (`crates/contextra-mvcc/src/lease.rs`).
- **Nutzung:** `SnapshotLease` wird in `LsmStorage` und `TxBuffer` verwendet. `SnapshotGuard` ist in `snapshot.rs` noch vorhanden und mit `#[deprecated]` markiert.

### Frage 7: Commit-Fehler nach Teilschreiben (2PC)
- **Ablauf:**
  Phase (a) Intent Pending -> Phase (b) LSM Commit -> Phase (c) RAM Indices Commit -> Phase (d) Intent Committed.
- **Wiederholbarkeit:** schlägt Phase (c) fehl, führt der `CommitLedger` ein Rollback der Indizes und ein Restore des LSM Stores aus (`CompensateLsmAction`). Bricht der Prozess nach Phase (b) ab, erkennt der Recovery-Worker den `CommitIntent::Pending` Marker und schließt die Transaktion ab oder rollt sie ab.
- **Risiko:** Die Wiederholung schreibt dieselben Transaktions-Keys. Da Transaktions-IDs atomar monoton steigen (`fetch_add(1)`), ist das Replay ohne Doppelbuchung idempotent.

---

## 4 Ergebnisse je Kategorie A bis M

### Kategorie A: Unvollständige oder angefangene Implementierungen
- **Befund:** In `crates/contextra-engine/src/transaction/db_transaction_commit.rs` wird die SSI-Validierung über `SsiValidator::validate` vor dem LSM-Commit nicht aufgerufen. Die Staged Writes werden ohne SSI-Konfliktprüfung in die LSM-Engine geschrieben.

### Kategorie B: Fehlerhafte Logik
- **Befund:** In `insert.rs`, `update.rs`, `delete.rs` und `relate.rs` wird für Lese-Operationen innerhalb von Mutationen `u64::MAX` als `as_of` Snapshot-Sequenz übergeben, anstatt der `tx.read_snapshot_seq`.

### Kategorie C: Fehlende Verdrahtung
- **Befund:** `SequenceLogSsiValidator` ist in `contextra-mvcc` vorhanden, wird aber in `DbTransaction::commit()` nicht zur Aufrufe-Validierung der Transaktion herangezogen.

### Kategorie D: Dead Code
- **Befund:** `SnapshotGuard` in `snapshot.rs` ist veraltet (deprecated) und dient nur noch als Kompatibilitätsschicht.

### Kategorie E: Fehlerbehandlung
- **Befund:** `cargo xtask check-result-dropped-io` meldet Verstöße im Workspace.

### Kategorie F: Panics und Unwraps im Produktionscode
- **Befund:** `TenantId::try_new(1).unwrap_or_default()` in `db_transaction_commit.rs:324` und `db_transaction_rollback.rs:13` nutzt Fallback auf `TenantId(0)` (SYSTEM), was bei Fehler den geschützten System-Mandanten betrifft.

### Kategorie G: Nebenläufigkeit
- **Befund:** Wenn Kompensationen nach Phase (b) scheitern, entsteht ein genanntes Split-Brain Risiko zwischen Store und RAM-Indizes.

### Kategorie H: Determinismus
- **Befund:** Keine direkten Verstöße gegen P28 in `contextra-mvcc` festgestellt.

### Kategorie I: Mandanten- und Berechtigungsgrenzen
- **Befund:** Hartkodierte `TenantId(1)` mit Fallback `unwrap_or_default()` (`TenantId(0)`) verletzt **INV-TENANT-1**.

### Kategorie J: Architektur und Design
- **Befund:** Entkopplung zwischen MVCC SSI Validator und Transaktions-Commit-Pfad in `contextra-engine`.

### Kategorie K: Code-Smells
- **Befund:** Wiederholte Übergabe von `u64::MAX` an Stellen, wo ein korrekter `read_snapshot_seq` verfügbar wäre.

### Kategorie L: Tests
- **Befund:** Tests in `contextra-mvcc` bestehen alle (46 passed), jedoch fehlt ein Integrationstest in `contextra-engine`, der Write-Skew zwischen zwei nebenläufigen `DbTransaction`-Commits simuliert und fehlschlägt.

### Kategorie M: Dokumentationsdrift
- **Befund:** `AGENTS.md` in `contextra-mvcc` fordert SSI conflict detection, der Engine Commit-Pfad nutzt diese jedoch nicht aktiv.

---

## 5 Befunde

### BEFUND-MVCC-01: Snapshot-Isolationsverletzung durch `u64::MAX` in Schreibpfaden
- **ID:** `BEFUND-MVCC-01`
- **Titel:** Lesefunktionen in Schreibpfaden nutzen `u64::MAX` statt `tx.read_snapshot_seq`
- **Schwere:** Hoch
- **Kategorie:** B (Fehlerhafte Logik)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-engine/src/collection/crud/insert.rs:321`, `update.rs:63`, `delete.rs:165`, `relate.rs:31`
- **Befund:** Beim Vorablesen von Werten für Kompensation/Sichtbarkeit nutzen `insert_op`, `update_op`, `delete_op` und `relate` den Aufruf `get_at_seq_tracked(tx, key, u64::MAX)`. Dadurch lesen sie Daten, die nach dem Start der Transaktion committed wurden.
- **Evidenz:** `crates/contextra-engine/src/collection/crud/insert.rs:321`:
  ```rust
  let old_user_val = self.storage.get_at_seq_tracked(tx, &user_key, u64::MAX).await?;
  ```
- **Betroffene Invariante:** Snapshot-Isolation (ein `max_seq` je Transaktionsanfrage)
- **Empfehlung:** Ersetzen von `u64::MAX` durch `tx.read_snapshot_seq` (oder Derivation aus `db_tx.read_snapshot_seq`).
- **Red-Test-Vorschlag:** Ein Test, der während einer offenen Transaktion T1 einen Schlüssel K durch T2 ändert; T1 führt danach ein Update auf K aus und liest fälschlicherweise den Stand von T2 statt den eigenen Snapshot-Stand.

### BEFUND-MVCC-02: Fehlende SSI-Validierung im `DbTransaction::commit()` Pfad
- **ID:** `BEFUND-MVCC-02`
- **Titel:** `DbTransaction::commit()` führt vor dem Store-Commit keine `SsiValidator::validate()` Prüfung durch
- **Schwere:** Kritisch
- **Kategorie:** C (Fehlende Verdrahtung) / B (Fehlerhafte Logik)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-engine/src/transaction/db_transaction_commit.rs:242-320`
- **Befund:** In `commit_internal()` wird `CommitIntent::Pending` geschrieben und sofort `self.collection.storage.commit(self.tx_id)` aufgerufen. Es findet keine Prüfung des `ReadSet` gegen den `SequenceLogSsiValidator` statt. Write-Skew zwischen zwei nebenläufigen Transaktionen wird somit nicht verhindert.
- **Evidenz:** In `db_transaction_commit.rs` existiert kein Aufruf von `ssi_validator` oder `validate`.
- **Betroffene Invariante:** INV-MVCC-SSI-1, Fail-closed für SSI.
- **Empfehlung:** Vor Phase (a)/(b) in `commit_internal()` das `read_set` der Transaktion gegen den SSI-Validator der Collection/Engine validieren und bei Konflikt abbrechen.

### BEFUND-MVCC-03: Hardcoded `TenantId(1)` mit unzulässigem Fallback auf Mandant 0 (SYSTEM)
- **ID:** `BEFUND-MVCC-03`
- **Titel:** Invalider Fallback auf `TenantId(0)` via `unwrap_or_default()` in Commit & Rollback
- **Schwere:** Mittel
- **Kategorie:** F (Unwraps/Panics) / I (Mandantengrenzen)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-engine/src/transaction/db_transaction_commit.rs:324`, `db_transaction_rollback.rs:13`
- **Befund:** `let tenant_id = TenantId::try_new(1).unwrap_or_default();` nutzt `Default` für `TenantId`, was `TenantId(0)` entspricht. Mandant 0 ist jedoch laut **INV-TENANT-1** als SYSTEM reserviert.
- **Evidenz:**
  ```rust
  let tenant_id = TenantId::try_new(1).unwrap_or_default();
  ```
- **Betroffene Invariante:** INV-TENANT-1 (Mandant 0 ist SYSTEM)
- **Empfehlung:** Mandanten-ID explizit aus dem Transaktionskontext (`self.collection.tenant_id()`) ableiten anstatt hart zu kodieren oder auf Default zu fallen.

---

## 6 Vorgänger-Abgleich

Vorgänger-Bericht: `docs/audits/AUDIT_contextra-mvcc_2026-10-12.md`

| Vorgänger-ID | Thema / Befund | Status im aktuellen Commit |
| :--- | :--- | :--- |
| P1–P4 | MVCC Visibility & O(1) Checks | BEHOBEN / BEIBEHALTEN (Tests grün) |
| P7 | TxBuffer Lock Hierarchy Assurance | OFFEN (Keine Typsicherung gegen Lock-Reihenfolge) |
| P8 | Orphan Reaper Snapshot Pinning | OFFEN (Manuelle Deregistrierung notwendig) |

---

## 7 Testlage im Scope

- `cargo test -p contextra-mvcc`: **46 passed; 0 failed**
- `visibility_proptest.rs`: **2 passed**
- `loom_snapshot_registry.rs`: Bestanden

---

## 8 Nicht prüfbar und why

- Full Engine Integration Tests for Concurrent Write Skew: Aufgrund von Workspace-Kompilierungsfehlern in Nachbar-Crates (`contextra-store`) konnten tiefere Engine-Integrationstests aktuell nur statisch analysiert werden.

---

## 9 Out-of-scope-Befunde

- **`contextra-store` Kompilierungsfehler:** `crates/contextra-store/src/lsm/engine.rs` enthält Method-Access Fehler (`ledger_handle` als Feld statt Methode), was den Workspace-Doctor-Check beeinträchtigt.

---

## 10 Priorisierte Maßnahmen

1. **Behebung BEFUND-MVCC-02 (Kritisch):** Einbinden der SSI-Validierung (`SsiValidator::validate`) in `DbTransaction::commit_internal()` vor dem Schreiben des Commit-Intents.
2. **Behebung BEFUND-MVCC-01 (Hoch):** Anpassen von `insert_op`, `update_op`, `delete_op` und `relate` so, dass anstelle von `u64::MAX` die Snapshot-Sequenz der Transaktion verwendet wird.
3. **Behebung BEFUND-MVCC-03 (Mittel):** Ersetzen von `TenantId::try_new(1).unwrap_or_default()` durch `self.collection.tenant_id()`.

---
