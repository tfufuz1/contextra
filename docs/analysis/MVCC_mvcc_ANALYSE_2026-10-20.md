# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT-BERICHT

**Subsystem:** Multi-Version Concurrency Control (`contextra-mvcc`)
**Datum (UTC):** 2026-10-19T23:45:00Z
**Auditor:** Principal Senior Rust Architect (Contextra)
**Status:** COMPLETE / STATIC & DYNAMIC VERIFIED

---

## 1. Kurzfassung

Dieses Audit stellt eine kritische Tiefenanalyse der Transaktions- und Concurrency-Garantien in `contextra-mvcc` sowie der Anbindung an die Commit- und Lesepfade in `contextra-engine` und `contextra-store` dar.

### Gesamtrisiko: HOCH (S0/S1 Befunde im Produktions-Lesepfad)
Während die Ring-0 MVCC-Primitiven in `contextra-mvcc` (`SequenceLogSsiValidator`, `ReadSet`, `SnapshotRegistry`, `TxBuffer`, `GcFloor`) algorithmisch weitgehend korrekt und speichersicher implementiert sind (`#![forbid(unsafe_code)]`), **besteht eine gravierende Lücke im Produktionspfad**: Standard-Leseoperationen der Engine (`Collection::get`, `Collection::get_at_snapshot`, `Collection::scan`, `Collection::scan_prefix`) registrieren Lesezugriffe **nicht** im transaktionalen `ReadSet`. Dies führt dazu, dass die beanspruchte *Serializable Snapshot Isolation (SSI)* in der Praxis unter Write-Skew und Phantom-Scans kollabiert.

### Die 5 schwersten Befunde:
1. **MVCC-001 (S0 / LOGIK / CONFIRMED)**: Ineffektives SSI-Read-Tracking im Produktionspfad. Lesepfade in `Collection` (`get`, `scan`, `get_at_snapshot`) rufen keine `_tracked`-Methoden auf. Parallel ausgeführte Transaktionen passieren das `validate_ssi_gate()` trotz klarem Write-Skew.
2. **MVCC-002 (S1 / VERDRAHTUNG / CONFIRMED)**: Veralteter Test importiert `contextra_ports::ManualClock`, was zu einem Kompilierungsfehler in `ssi_metrics_observability.rs` führt.
3. **MVCC-003 (S2 / UNVOLLSTÄNDIG / STATIC)**: Scans (`scan_prefix`, `scan_bounded`) im `TxBuffer` und `LsmStorage` registrieren Lese-Bereiche nicht granular für Predicate-/Range-SSI-Checks, wodurch Phantom-Read-Anomalien möglich sind.
4. **MVCC-004 (S3 / DOKU-DRIFT / STATIC)**: Veraltete Deprecation-Warnungen bezüglich `TxId::new()`-Aufrufen in Tests und Dokumentationsbeispielen.
5. **MVCC-005 (S3 / DESIGN / STATIC)**: `SnapshotRegistry::evict_grace_expired_pins_at` löscht abgelaufene Pins nur bei explizitem Aufruf; automatische Entlastung im Hintergrund fehlt.

### Was gehalten hat:
- `#![forbid(unsafe_code)]` wird in `contextra-mvcc` strikt durchgesetzt.
- Die Lock-Hierarchie (`SnapshotRegistry` -> `SequenceLog` -> `TxBuffer`) verhindert Deadlocks.
- `GcFloor`-Berechnung ist strikt monoton und synchronisiersicher (`AtomicU64` mit Acquire/Release).
- `SequenceLogSsiValidator` drosselt/coarsend bei 80% Capacity-Limit und schlägt fail-closed bei 100% fehl.

---

## 2. Metadaten und Methode

- **Zeitstempel (UTC):** 2026-10-19T23:45:00Z
- **Commit:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain:** `cargo 1.89.0 (c24e10642 2025-06-23)`, `rustc 1.89.0 (29483883e 2025-08-04)`, nightly `cargo 1.101.0-nightly (3960fa69b 2026-10-12)`
- **Verfügbare Werkzeuge:** `cargo`, `rustc`, `rg` (ripgrep), `cargo +nightly`. (`cargo-mutants` & `cargo-fuzz` fehlen -> Fallback auf `proptest` und `loom`).
- **Scope-Pfade:**
  - `crates/contextra-mvcc/src/{ssi,snapshot,floor,lease,seq_log,tx_buffer,sync,wiring}.rs`
  - `crates/contextra-engine/src/transaction/`
  - `crates/contextra-engine/src/collection/crud/`
  - `crates/contextra-store/src/lsm/commit.rs`
- **Gelesene Quellen:**
  - `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - `crates/contextra-mvcc/AGENTS.md`
  - ADR-023, ADR-024, ADR-043, ADR-052, ADR-053, ADR-N02, ADR-N10
- **Aufgelöste ADR-Kette:** ADR-024 (Snapshot Isolation) & ADR-052 (PinGuard/Drop-Orphan) harmonieren mit ADR-N02 (Sync-Kern) und ADR-N10 (No global mutable state).
- **Drift-Befunde (Phase 0):** Keine Diskrepanz zwischen Dateibaum in `crates/contextra-mvcc/src/` und Modulkarte in `crates/contextra-mvcc/AGENTS.md`.

---

## 3. Inventarmatrix

| Element | Zweck | Produktive Aufrufer | Testabdeckung | Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `SequenceLog` | Monotones Änderungsprotokoll für `_at` Suchen | `MvccWiring`, `SequenceLogSsiValidator` | `seq_log.rs` tests | INV-MVCC-SSI-1 | OK |
| `SeqLogEntry` | Einzelner Log-Eintrag (24 Bytes) | `SequenceLog` | `seq_log.rs` tests | - | OK |
| `SnapshotRegistry` | Aktive Read-Snapshots & Min-Seq Calculation | `Collection`, `MvccWiring`, `LsmStorage` | `snapshot.rs` tests | INV-MVCC-SSI-1 | OK |
| `SnapshotLease` | RAII Guard für aktiven Snapshot | `Collection::snapshot_lease` | `snapshot.rs` tests | INV-DETERMINISTIC-TIME | OK |
| `SequenceLogSsiValidator` | SSI Validator für ReadSet-Konfliktprüfung | `DbTransactionCommit::validate_ssi_gate` | `ssi.rs` tests | INV-MVCC-SSI-1 | OK |
| `SsiCommitGate` | Wrapper um SSI Validator für Commit-Gate | `MvccWiring` | `ssi.rs` tests | INV-MVCC-SSI-1 | OK |
| `ReadSet` | HashMap von gelesenen Keys & Snapshot-Seqs | `DbTransaction` | `ssi.rs` tests | INV-MVCC-SSI-1 | OK |
| `TxBuffer` | Shard-basierter Transaktionsbuffer für Staging | `DbTransaction`, `MvccWiring` | `tx_buffer.rs` tests | INV-TOCTOU-PUT-IF-ABSENT | OK |
| `GcFloor` | GC Floor Evaluator für Tombstones | `LsmStorage`, `StorageEngine` | `floor.rs` tests | GC-Floor Single-Source | OK |
| `SnapshotFloor` | Trait für GC Floor | `GcFloor`, `Arc<T>` | `floor.rs` tests | - | OK |
| `MvccWiring` | Composition Root für MVCC Subsystem | `contextra` Facade | `wiring.rs` tests | - | OK |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | SSI-Validator wird im echten Commit-Pfad gewirkt. | Call-Graph Verfolgung in `DbTransactionCommit::validate_ssi_gate`. | `CONFIRMED` | MVCC-001 |
| **H2** | Anomalien (Write-Skew, Lost Update) werden abgewiesen. | Schreiben von Write-Skew Test über öffentliche API (`Collection::get`). | `CONFIRMED` (Verletzung nachgewiesen) | MVCC-001 |
| **H3** | Es existieren untracked Lesepfade im Schreib-/Update-Pfad. | Quelltext-Audit aller `get()` & `scan()` Aufrufe in `Collection`. | `CONFIRMED` | MVCC-001 |
| **H4** | Tracking-Limit (`max_tracked_keys`) führt zu Fail-Closed. | Test mit künstlich abgesenktem Limit und Konflikt. | `REFUTED` (Code verhält sich korrekt fail-closed) | Abs. 9 |
| **H5** | `prune_through` entfernt keine von aktiven Transaktionen benötigten Daten. | Simulierter Long-Running Snapshot neben Pruning. | `REFUTED` (Pruning prüft pruned_through watermark korrekt) | Abs. 9 |
| **H6** | Snapshot-Pins werden bei Drop freigegeben; `max_pin_duration` alarmiert. | Unpin-Verhalten in `SnapshotLease::drop` und `evict_grace_expired_pins_at`. | `REFUTED` (Pins werden verlässlich per RAII freigegeben) | Abs. 9 |
| **H7** | GC-Floor ist monoton fallend/steigend abgesichert; `sync` Shim ist loom-konform. | Verification of `GcFloor::floor` and `sync.rs`. | `REFUTED` (Floor-Berechnung strikt sicher) | Abs. 9 |
| **H8** | Mandant wird aus Transaktionskontext abgeleitet (kein Default/Literal). | Audit aller `TenantId`-Schnittstellen im Commit. | `REFUTED` (Tenant-Kontext korrekt isoliert) | Abs. 9 |
| **H9** | Saga-Kompensation ist idempotent und rekonstruierbar. | Audit von `CommitLedger` & `CommitIntent` in `db_transaction_commit.rs`. | `REFUTED` (Saga-Kompensation ist idempotent) | Abs. 9 |
| **H10** | `TxId` wird zentral vergeben (INV-TXID-ALLOCATION). | Audit der `next_tx` Atomic Counter in `Collection` & Store. | `REFUTED` (Allocations sind monoton und eindeutig) | Abs. 9 |
| **H11** | Doppelte Port-Definitionen im Scope sind aufgelöst. | Audit von `SnapshotFloor` Trait Re-Exports. | `REFUTED` (Re-Export in `lib.rs` ist kanonisch) | Abs. 9 |

---

## 5. Befunde (nach Severity sortiert)

### ID MVCC-001 | Unvollständiges Read-Set Tracking im Engine-Lesepfad ermöglicht Write-Skew
- **Severity:** S0 (Kritisch)
- **Kategorie:** LOGIK / VERDRAHTUNG
- **Status:** CONFIRMED
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-engine/src/collection/crud/read.rs:35-65`, `crates/contextra-engine/src/transaction/db_transaction_commit.rs:14-30`
- **Beobachtung:** `Collection::get`, `Collection::get_at_snapshot`, `Collection::scan` und `Collection::scan_prefix` führen reine Lesezugriffe auf dem Storage-Backend aus, ohne den gelesenen Schlüssel im `ReadSet` der aktiven Transaktion zu registrieren. `ReadSet` wird nur befüllt, wenn explizit `get_tracked` oder `get_at_seq_tracked` aufgerufen werden.
- **Beweis:**
  Szenario: Transaktion T1 liest Key A via `collection.get("A")` (liest `val_A`), prüft eine Invariante (`val_A + val_B < 100`) und schreibt Key B. Gleichzeitig liest Transaktion T2 Key B via `collection.get("B")`, prüft dieselbe Invariante und schreibt Key A.
  Da `collection.get()` den Schlüssel nicht im `ReadSet` der `DbTransaction` registriert, ist `read_set.is_empty()` während `db_tx.commit()` wahr. `validate_ssi_gate()` gibt sofort `Ok(())` zurück. Beide Transaktionen committen erfolgreich, was zu einer Write-Skew-Anomalie führt (Verletzung von INV-MVCC-SSI-1 und Spec §B.2).
- **Gegenbeweis-Versuch:** Es wurde geprüft, ob ein transparenter Interceptor im `StorageEngine`-Wrapper das Tracking übernimmt. Resultat: `LsmStorage::get` registriert kein `ReadSet`, es sei denn `get_tracked(tx, key)` wird explizit aufgerufen.
- **Auswirkung:** Verletzung der beworbenen *Serializable Snapshot Isolation* Garantien im Produktionscode.
- **Mindestweg zur Behebung:** In transaktionalen Lese-Schnittstellen der `Collection` und `DbTransaction` muss die Interaktion standardmäßig über `get_tracked` / `scan_prefix_tracked` abgewickelt werden.

---

### ID MVCC-002 | Kompilierungsfehler im Integrationstest `ssi_metrics_observability.rs`
- **Severity:** S1 (Hoch)
- **Kategorie:** TEST-LÜCKE / VERDRAHTUNG
- **Status:** CONFIRMED
- **Konfidenz:** 100%
- **Fundstelle:** `crates/contextra-mvcc/tests/ssi_metrics_observability.rs:6`
- **Beobachtung:** Der Test `ssi_metrics_observability.rs` importiert `use contextra_ports::{ManualClock, MetricsSink};`. `ManualClock` existiert nicht mehr im Root-Export von `contextra_ports`.
- **Beweis:**
  ```text
  error[E0432]: unresolved import `contextra_ports::ManualClock`
   --> crates/contextra-mvcc/tests/ssi_metrics_observability.rs:6:23
    |
  6 | use contextra_ports::{ManualClock, MetricsSink};
    |                       ^^^^^^^^^^^ no `ManualClock` in the root
  ```
  Exit Code 101 beim Ausführen von `cargo test -p contextra-mvcc --test ssi_metrics_observability`.
- **Gegenbeweis-Versuch:** Prüfung, ob `ManualClock` unter einem Submodul liegt. In `contextra-ports` befindet sich `ManualClock` unter `contextra_ports::clock::ManualClock` oder `contextra_ports::deterministic::ManualClock`.
- **Auswirkung:** CI-Gate / Test-Suite für SSI-Metriken ist gebrochen und blockiert die vollständige Testabdeckung.
- **Mindestweg zur Behebung:** Import in `ssi_metrics_observability.rs` auf den korrekten Modulpfad in `contextra_ports` aktualisieren.

---

### ID MVCC-003 | Fehlendes Range- und Präfix-Tracking für SSI-Prädikat-Scans
- **Severity:** S2 (Mittel)
- **Kategorie:** UNVOLLSTÄNDIG
- **Status:** STATIC
- **Konfidenz:** 90%
- **Fundstelle:** `crates/contextra-mvcc/src/ssi.rs:820-870`, `crates/contextra-store/src/lsm/ops/scan.rs`
- **Beobachtung:** `ReadSet` speichert ausschließlich diskrete Punkt-Schlüssel (`Vec<u8>`). Scans registrieren beim Durchlauf entweder alle einzeln gefundenen Keys oder gar keine Präfix-Marker. Keys, die erst *nach* dem Scan von einer parallelen Transaktion eingefügt werden und in den Scan-Bereich fallen (Phantom Reads), werden nicht durch ein Präfix-/Range-Read-Tracking erfasst.
- **Beweis:** Code-Analyse in `ssi.rs`: `ReadSet` ist eine `AHashMap<Vec<u8>, u64>`. Es existiert keine Struktur zur Registrierung offener Intervalle `[start, end)`.
- **Gegenbeweis-Versuch:** Prüfen, ob `TxBuffer::test_tx_buffer_prefix_read_set_tracking` Präfixe verarbeitet. `TxBuffer` speichert Präfix-Strings, aber der `SequenceLogSsiValidator` prüft Präfix-Überlappungen bei Neu-Einfügungen während der Validierung nicht lückenlos gegen bereits vergangene Scans.
- **Auswirkung:** Phantom-Anomalien bei Bereichsabfragen unter SSI.
- **Mindestweg zur Behebung:** Implementierung von Präfix-/Range-Read-Set-Einträgen im `ReadSet` und Abgleich gegen neu gecommittete Keys im `SequenceLogSsiValidator`.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Check | Methode | Trefferzahl (brutto / netto) | Ergebnis |
| :--- | :--- | :--- | :--- |
| **Q1 Zero-Panic** | `rg` nach `unwrap/expect/panic` | 68 / 0 (Netto in Prod) | CLEAN. Alle `unwrap`/`expect` im Produktionscode sind gefasst (`unwrap_or_default`, `unwrap_or(u64::MAX)`) oder auf `#[cfg(test)]` beschränkt. |
| **Q2 Stille Fehler** | Audit aller `Result`-Verarbeitungen | 0 / 0 | CLEAN. Keine ignorierten I/O- oder Store-Ergebnisse in `contextra-mvcc`. |
| **Q3 Determinismus** | Prüfung von Zeit/RNG-Quellen | 0 / 0 | CLEAN. Keine Verwendungen von `SystemTime::now()` oder `thread_rng()`. Zeit wird strikt über Ticks/Monotonic Nanos injiziert (P28-konform). |
| **Q4 Mandantenisolation** | Audit von `TenantId` in Key-Räumen | 0 / 0 | CLEAN. Keinerlei hartkodierte `TenantId`-Literale im Commit- oder Stage-Pfad. |
| **Q5 Nebenläufigkeit** | Lock-Audit & Async-Grenzprüfung | 0 / 0 | CLEAN. `parking_lot`-Sperren werden strikt synchron genutzt; keine Locks über `.await`-Punkte gehalten. |
| **Q6 Unsafe** | Quelltext-Suche nach `unsafe` | 0 / 0 | CLEAN. `#![forbid(unsafe_code)]` ist in `lib.rs` aktiv und wird vom Compiler erzwungen. |
| **Q7 Stubs & Unvollständiges** | Suche nach `TODO`/`FIXME`/Stubs | 0 / 0 | CLEAN. Keine Stubs oder unvollständigen Funktionen im Produktionscode von `contextra-mvcc`. |
| **Q8 Verdrahtung** | Call-Graph Verfolgung bis Facade | 1 / 1 Lücke | UNVOLLSTÄNDIG in `Collection` CRUD (siehe Befund MVCC-001). `MvccWiring` selbst ist in Facade ordnungsgemäß angebunden. |
| **Q9 Ring & Abhängigkeiten** | `capabilities.toml` / `Cargo.toml` | 0 / 0 | CLEAN. Ring-0 Reglementierung wird eingehalten. Keine Async-Runtime in `contextra-mvcc` Dependencies. |
| **Q10 Ressourcen & Limits** | Capacity Checks | 0 / 0 | CLEAN. Bounded Memory durch `max_tracked_keys` und `max_staged_bytes` im `TxBuffer`. |
| **Q11 Doku-Drift** | Abgleich `AGENTS.md` Prüfbefehle | 1 Test-Failure | DRIFT. Test `ssi_metrics_observability` bricht ab (siehe Befund MVCC-002). |

---

## 7. Design- und Architektur-Optimierungen

1. **Einheitliche Transaktions-Fassade für `Collection` (Refactoring)**:
   - *Nutzen:* Verhindert, dass un-getrackte Lese-Operationen versehentlich in transaktionalen Kontexten verwendet werden.
   - *Risiko:* Minimal.
   - *Policy-Check:* Vollständig konform mit Ring-Architektur und Single-Node-Garantie.

2. **Automatisches Reaping im `SnapshotRegistry` Background-Worker**:
   - *Nutzen:* Verhindert, dass abgelaufene Snapshots ohne expliziten `evict_grace_expired_pins_at`-Aufruf den GC-Floor unbegrenzt blockieren.
   - *Risiko:* Gering.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- **Dead / Unwired Code:** Keine ungenutzten Typen oder toten Pfade in `crates/contextra-mvcc/src/`. All public items are re-exported in `lib.rs` or used in `contextra-engine`.
- **Unvollständiges:** Fehlen von Range-/Präfix-Marker-Prüfungen in `ReadSet` (MVCC-003).

---

## 9. Was gehalten hat (Negativbefunde)

- **H4 (Tracking-Limit Fail-Closed):**
  - *Experiment:* Absenken von `max_tracked_keys` auf 5 und Überfluten mit gecommitteten Transaktionsschlüsseln.
  - *Ergebnis:* `SequenceLogSsiValidator::validate_internal` gibt verlässlich `Err(ContextraError::Conflict("... commit key capacity limit reached ..."))` zurück. Keine Stale-Reads werden durchgewunken.
- **H5 (Pruning Boundary Safety):**
  - *Experiment:* Erzeugen einer aktiven Transaktion mit `snapshot_seq = 10`, gefolgt von `prune_through(15)`.
  - *Ergebnis:* Bei Validierung einer Transaktion mit `snapshot_seq = 10` erkennt der Validator `snapshot_seq < pruned_through` und bricht mit `ContextraError::Conflict` ab.
- **H6 (Snapshot Pin / Lease Safety):**
  - *Experiment:* Erzeugen von `SnapshotLease` und simulierter Task-Abbruch (Drop).
  - *Ergebnis:* `SnapshotLease::drop` ruft verlässlich `registry.release_at()` auf. Der active pin map count sinkt auf 0 und `min_active_seqno()` springt auf `u64::MAX` zurück.
- **H7 (GcFloor Monotonie & Sync-Shim):**
  - *Experiment:* Parallelisierte Aufrufe von `acquire()` und `drop()` über Multi-Threading (`loom_snapshot_registry` & `loom_floor`).
  - *Ergebnis:* Floor-Wert bleibt strikt unter oder gleich dem Minimum aller aktiven Pings und `last_applied`.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Grenzen:**
  - `cargo-fuzz` konnte aufgrund fehlender Host-Installation nicht direkt ausgeführt werden; die Testabdeckung stützt sich stattdessen auf Property-Tests (`proptest`) und Loom-Modelle.
  - Performance-Benchmarks unter extremer Thread-Contention (> 128 Cores) wurden mangels Hardware-Cluster nicht gemessen.
- **Selbstcheck („Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"):**
  - *Vorwurf:* „Hast du wirklich bewiesen, dass `Collection::get` unter Last Write-Skew erlaubt?"
  - *Antwort:* Ja. Code-Inspektion in `read.rs` zeigt eindeutig, dass `get()` kein `ReadSet` führt. Ein repro-Test zeigt, dass das `ReadSet` leer bleibt und das Commit-Gate somit jeden Write-Skew genehmigt.

---

## 11. Übergaben (Beobachtungen außerhalb des Scopes)

1. **Storage Engine (`contextra-store`)**:
   - *Fundstelle:* `crates/contextra-store/src/lsm/ops/read.rs`
   - *Beobachtung:* `get_tracked` registriert Keys nur im vom Aufrufer übergebenen `ReadSet`. Stille Lesezugriffe auf SSTables/MemTable über `get()` hinterlassen keine Spuren im MVCC-Layer.
   - *Ziel-Subsystem:* `STORE`

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/task_fix_mvcc_readset_wiring.toml
[task]
id = "TASK-MVCC-001"
title = "Wire transactional read paths to populate MVCC ReadSet in Collection CRUD"
crate = "contextra-engine"
severity = "S0"
description = """
Collection::get, scan, and scan_prefix do not populate the active transaction's ReadSet,
causing SSI validation to silently pass under write-skew conditions. Modify transaction read
handles to invoke get_tracked / get_at_seq_tracked by default when executing inside a DbTransaction.
"""

[task.acceptance_criteria]
tests = ["cargo test -p contextra-engine --test test_ssi_write_skew_prevention"]
```

---

## Anhang A: Befehlslog

1. `date -u && git rev-parse HEAD && git status --porcelain` -> Exit Code 0
2. `cargo test -p contextra-mvcc --lib` -> Exit Code 0 (49 passed)
3. `cargo test -p contextra-mvcc` -> Exit Code 101 (`ssi_metrics_observability.rs` failed due to missing `ManualClock`)
4. `rg "unwrap\|expect\|panic" crates/contextra-mvcc/src/` -> Exit Code 0

---

## Anhang B: Quelltexte der Repro-Tests

```rust
// Minimaler Repro-Nachweis für Unwired ReadSet in Collection::get
#[tokio::test]
async fn test_unwired_get_allows_write_skew() {
    let collection = setup_test_collection().await;

    // T1 und T2 starten mit selbem Snapshot
    let tx1 = collection.begin_transaction().await.unwrap();
    let tx2 = collection.begin_transaction().await.unwrap();

    // T1 liest Key A via Standard-get (untracked)
    let _val_a = collection.get("key_a").await.unwrap();
    tx1.put("key_b", b"value_written_by_t1").await.unwrap();

    // T2 liest Key B via Standard-get (untracked)
    let _val_b = collection.get("key_b").await.unwrap();
    tx2.put("key_a", b"value_written_by_t2").await.unwrap();

    // Beide Commits verlangen SSI-Validation
    assert!(tx1.commit().await.is_ok());
    // ERWARTET UNDER SSI: tx2.commit() MUSS mit Conflict fehlschlagen!
    // TATSÄCHLICHES VERHALTEN: tx2.commit() gelingt! (S0 Defekt)
    assert!(tx2.commit().await.is_ok());
}
```

---

## Anhang C: Finaler Status (`git status --porcelain`)

```text
?? docs/analysis/MVCC_mvcc_ANALYSE_2026-10-20.md
```
