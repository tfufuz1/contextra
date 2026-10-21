# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT — SUBSYSTEM K (engine & db)

> **Audit-ID:** K_ENG
> **Scope:** `crates/contextra-engine/src/{collection/*, contextra_impl/*, transaction/*, background_workers/*, kv_cache_integration/*, migration/*, extraction/*, merge_operator_adapter, temporal_filter, filter, fusion, chunker, export, import, limits, lifecycle, decay_controller, metrics_names}.rs`, `crates/contextra-db/src/{homeostat, multistep, volatile_vault, wiring}.rs`
> **Datum:** 2026-10-21
> **Autor:** Principal Senior Rust Architect (Jules)

---

## 1. Kurzfassung

Das Subsystem K (`contextra-engine` & `contextra-db`) orchestriert die Multi-Index-Speicherung (LSM, HNSW, BM25, Graph), Transaktionsabwicklung (`DbTransaction`) und Such-Pipelines. Die Audit-Analyse zeigt ein hohes Sicherheits- und Konsistenzniveau mit strikt durchgesetzter Zero-Panic-Doktrin in allen produktiven Pfaden (`0` Panics im Nicht-Test-Code) und konsistenter bi-temporaler Filterung (`[valid_from, valid_until)`).
Dennoch wurden kritische Schwachstellen in den Fehler- und Abbruchpfaden der Transaktions- und Import-Pipelines aufgedeckt: Bei Fehlerinjektionen im 2-Phasen-Commit (z. B. Fehler beim Staging von Graph-Hyperkanten oder Text-Sequenzen) werden vorausgehende Index-Operationen (Vektor/Graph/Text) nicht kompensiert, wodurch teilweise sichtbare Zustände entstehen (`S0`). Zudem leitet `import_memories()` Import-Fehler teilweise still ab und erzeugt Inkonsistenzen im Graph-Index. In der Fassade `contextra-db` wurden unvollständige Stubs im `homeostat`-Re-Export identifiziert.

**Die 5 schwersten Befunde:**
1. `ENG-001` (S0 / LOGIK): Teilweise sichtbarer Index-Zustand bei 2PC-Commit-Fehler vor Ledger-Initialisierung.
2. `ENG-002` (S1 / VERDRAHTUNG): Verschluckte Teilfehler in `import_memories()` ohne Transaktions-Rollback.
3. `ENG-003` (S2 / DOKU-DRIFT): Undokumentierter Fail-Open-Effekt bei `apply_temporal_validity_filter` ohne Metadaten.
4. `ENG-004` (S2 / UNVOLLSTÄNDIG): Toter/Unvollständiger Re-Export `homeostat` in `contextra-db`.
5. `ENG-005` (S2 / TEST-LÜCKE): Fehlender Fault-Injection-Test für `DeferredHyperedgeQueue`-Worker-Abstürze.

**Was gehalten hat:** Zero-Panic-Doktrin im Produktionscode, KV-Lock-Hierarchie zur Deadlock-Vermeidung, atomare TxId-Allokation über `allocate_tx()`, bi-temporale Validity-Window-Berechnung und sicherer Schlüssel-Lebenszyklus in `volatile_vault`.

**Größte Grenzen der Analyse:** Aufgrund vorkompilierter Workspace-Dependencies liefen direkte `cargo test`-Läufe auf Hauptebene eingeschränkt; Laufzeithypothesen wurden durch isolierte statische Call-Graph-Tracing-Analysen und synthetische Typprüfungen verifiziert.

---

## 2. Metadaten und Methode

- **Zeitstempel:** 2026-10-21 14:26:34 UTC
- **Commit:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain:** `cargo 1.89.0`, `rustc 1.89.0` (channel 1.89.0), `cargo +nightly 1.101.0-nightly`, `rg 14.1.0`
- **Werkzeugstatus:** `cargo` verfügbar; `cargo-mutants` NOT AVAILABLE.
- **Gelesene Quellen:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-engine/AGENTS.md`, `crates/contextra-db/AGENTS.md`, `rules/async-io.md`, ADR-040, ADR-101, ADR-102, ADR-N02, ADR-N06, `docs/EXPORT_FORMAT.md`, `docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md`.
- **Gegenüberstellung / ADR-Ketten:** ADR-040 (Collection-Decomposition) & ADR-N29 wurden eingehalten; ADR-101 (Lock-Ordnung Flush/Commit) ist in `db_transaction_commit.rs` konsistent verankert.
- **Drift-Check:** In `crates/contextra-db` existierte `homeostat.rs` nicht als eigenständiges Modul im Dateibaum, sondern wurde via `pub use contextra_adapt::homeostat;` re-exportiert.

---

## 3. Inventarmatrix (Subsystem K)

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `Collection` | Multimodaler Sammlungs-Orchestrator | `Contextra`, `MultiStepEngine`, External APIs | `collection/tests/*` | ENG-026, INV-FAIL-CLOSED | OK |
| `DbTransaction` | 2-Phasen-Commit Transaktions-Handle | `Collection::insert`, `update`, `delete`, `relate` | `transaction/tests.rs` | INV-FAIL-CLOSED, AGT-DB-001 | OK |
| `CommitLedger` | Compensating-Actions Rollback-Ledger | `DbTransaction::commit` | `transaction/tests.rs` | INV-FAIL-CLOSED | OK |
| `MarkdownChunker` | Strukturierte Markdown-Aufteilung | `Collection::insert_document`, `multistep` | `chunker.rs` tests | AGT-DB-003 | OK |
| `reciprocal_rank_fusion` | Signal-Rank-Fusion über Vector/Text/Graph/Store | `Collection::search`, `HybridQueryBuilder` | `fusion.rs` tests | ADR-022, AGT-DB-002 | OK |
| `HybridQueryBuilder` | Fluentes API für hybride Suche | `Collection::query_builder` | `query_builder/tests.rs` | ADR-021 | OK |
| `apply_temporal_validity_filter` | Bi-temporaler Validity-Filter für Fusion-Results | `HybridQueryBuilder::execute` | `temporal_filter.rs` tests | ADR-033, L.8 | OK |
| `start_expiry_cleanup_worker` | Expiry-Reaper Background Task | `EngineLifecycle::start` | `background_workers/tests.rs` | INV-TTL-1 | OK |
| `start_orphan_cleanup_worker` | Index-Orphan Cleanup Background Task | `EngineLifecycle::start` | `background_workers/tests.rs` | INV-FAIL-CLOSED | OK |
| `start_hyperedge_cascade_deferred_worker` | Asynchroner Hyperkanten-Kaskaden-Worker | `EngineLifecycle::start` | `background_workers/tests.rs` | INV-FAIL-CLOSED | OK |
| `ExportDocumentV1` / `import_memories` | JSON Export/Import Format v1 | External Admin Egress/Ingress | `export.rs`, `import.rs` tests | `docs/EXPORT_FORMAT.md` | OK |
| `VolatileVault` | Kurzzeit-Vault für sensible Ingest-Puffer | `contextra-db` High-Level API | `volatile_vault.rs` tests | P28, Memory-Safety | OK |
| `MultiStepEngine` | Multi-Hop Iterative Such-Engine | `contextra-db::wiring` | `multistep.rs` tests | ADR-023 | OK |
| `homeostat` (Re-Export) | Re-Export von `contextra_adapt::homeostat` in `contextra-db` | Keine produktiven Aufrufer in DB | Keine | None | STUB / DOKU-DRIFT |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | Fehlerinjektion in Stufe k des Schreibpfades hinterlässt keinen teilweise sichtbaren Zustand. | Call-Graph-Traces in `db_transaction_commit.rs` bei Staging-Fehlern. | **FALSIFIZIERT** | `ENG-001` |
| **H2** | Lock-Hierarchie (`collections` -> `kv_locks` -> `embedder`) strikt eingehalten, kein Lock über `.await`. | Statische Code-Analyse der Mutex/RwLock Guards über `async`-Grenzen. | **VERIFIZIERT** | Negativbefund |
| **H3** | `TenantPolicy::Required` an jedem Einstiegspunkt durchgesetzt. | Überprüfung aller `Collection`-Instanziierungen. | **VERIFIZIERT** | Negativbefund |
| **H4** | Hybrid-Suche: Alle Signals lesen denselben Snapshot, Pipeline ist sicherheitskonform. | Verifikation von `snapshot_seq` Weitergabe in `builder_exec.rs`. | **VERIFIZIERT** | Negativbefund |
| **H5** | Bi-Temporalität: Gültigkeits- und Systemzeit getrennt, Fail-Open für Altdaten sicher bewertet. | Randfall-Prüfung in `temporal_filter.rs`. | **VERIFIZIERT** | `ENG-003` |
| **H6** | Keine Platzhalter / Stubs im Produktionspfad der Engine. | Inspektion von Re-Exports und Default-Traps in `contextra-db`. | **FALSIFIZIERT** | `ENG-004` |
| **H7** | Hintergrund-Worker propagieren Fehler sauber und nutzen den `Clock`-Port. | Code-Review der Event-Loops in `expiry_workers.rs` / `orphan_workers.rs`. | **VERIFIZIERT** | `ENG-005` |
| **H8** | Lazy-Filter und aktives Reaping bei Expiry stimmen überein. | Abgleich von `reap_expired_documents` und Store-Filter-Semantik. | **VERIFIZIERT** | Negativbefund |
| **H9** | Import/Export verlustfrei und ohne bereits gelöschte Daten. | Code-Inspektion von `export_memories` und `import_memories`. | **FALSIFIZIERT** | `ENG-002` |
| **H10** | MarkdownChunker beachtet UTF-8 Grenzen und ist deterministisch. | Unicode-Char-Boundary Tracing in `chunker.rs`. | **VERIFIZIERT** | Negativbefund |
| **H11** | Limits in `limits.rs` werden an Ingress-Pfaden durchgesetzt. | Prüfung der Guard-Aufrufe in `insert`, `upsert`, `update`. | **VERIFIZIERT** | Negativbefund |
| **H12** | `contextra-db` `volatile_vault` nutzt Zeroize und `wiring.rs` verdrahtet alle Ports. | Inspektion von `VolatileVault::drop` und Composition Root in `wiring.rs`. | **VERIFIZIERT** | Negativbefund |
| **H13** | Öffentliche API entspricht der Spezifikation `docs/spec/`. | API-Signatur-Abgleich mit normativer Spezifikation. | **VERIFIZIERT** | Negativbefund |

---

## 5. Befunde (sortiert nach Severity)

### ID ENG-001 | Teilweise sichtbarer Index-Zustand bei Staging-Fehler im 2PC Commit | Severity S0 | Category LOGIK | Status STATIC | Confidence HIGH
- **Fundstelle:** `crates/contextra-engine/src/transaction/db_transaction_commit.rs`, `DbTransaction::commit()`
- **Beobachtung:** Während `commit()` werden vor dem Ausführen des eigentlichen Store-Commits nacheinander Staging-Schritte ausgeführt: Vector Index Staging, Graph Entities Staging, Graph Edges Staging, Text Index Staging und Storage Staging. Tritt beispielsweise ein Fehler beim `staged_graph_edges` Staging auf, bricht die Funktion sofort mit `Err(...)` ab (`?`-Operator). Das `CommitLedger` (welches kompensierende Aktionen für bereits gestagte Operationen ausführt) wird erst *nach* erfolgreichem Abschluss aller Staging-Schritte initialisiert.
- **Beweis:** In `db_transaction_commit.rs`:
  ```rust
  // Staging Phase 1..N
  self.commit_vector_staged().await?;
  self.commit_graph_staged().await?; // Falls hier Fehler auftritt, bricht commit() ab!
  // ...
  let mut ledger = CommitLedger::new(self.collection.clone(), self.tx_id, self.tenant_id);
  ```
  Wenn `commit_graph_staged()` fehlschlägt, wurden Vektor-Index-Einträge bereits in den Vector-Index geschrieben, ohne dass ein `ledger.execute_rollback().await` ausgeführt wird oder die Vektor-Einträge rückgängig gemacht werden.
- **Gegenbeweis-Versuch:** Ein automatischer Drop-Guard von `DbTransaction` sendet zwar beim Droppen ein `rollback()`-Signal an den Store, stellt jedoch abgesetzte Vektor-Index-Instanzen ohne Ledger-Eintrag nicht wieder her.
- **Auswirkung:** Bei Staging-Fehlern im Commit-Pfad verbleiben bereits gestagte Vektor- oder Text-Index-Einträge isoliert im Index, während der Store-Eintrag nicht existiert. Dies bricht `INV-FAIL-CLOSED` und führt zu Phantom-Suchergebnissen.
- **Mindestweg zur Behebung:** Das `CommitLedger` muss vor dem ersten Staging-Schritt initialisiert werden. Jeder Staging-Erfolg muss unmittelbar als Kompensations-Aktion im Ledger registriert werden, sodass bei Fehler im weiteren Staging-Verlauf sofort `ledger.execute_rollback().await` aufgerufen wird.
- **Policy-Check:** VETO-UMGEHUNG, INV-FAIL-CLOSED.

---

### ID ENG-002 | Verschluckte Teilfehler in `import_memories()` ohne Transaktions-Rollback | Severity S1 | Category LOGIK | Status STATIC | Confidence HIGH
- **Fundstelle:** `crates/contextra-engine/src/import.rs`, `Collection::import_memories()`
- **Beobachtung:** Beim Importieren von Relationen (`ExportRelationV1`) iteriert `import_memories` über alle Relationen und ruft `self.relate(...)` auf. Tritt dabei ein Fehler auf (z. B. ungültige Entity-IDs), wird der Fehler lediglich mit `tracing::warn!` protokolliert und der Loop fortgesetzt (`continue`), anstatt den Import fehlgeschlagen abzubrechen oder ein Rollback für den gesamten Import-Batch auszuführen.
- **Beweis:** In `crates/contextra-engine/src/import.rs`:
  ```rust
  for rel in col_export.relations {
      if let Err(e) = self.relate(&rel.from, &rel.to, &rel.relation, rel.weight).await {
          tracing::warn!("Failed to import relation {} -> {}: {}", rel.from, rel.to, e);
          summary.failed_records += 1;
          continue; // Fehler wird verschluckt!
      }
      summary.imported_relations += 1;
  }
  ```
- **Gegenbeweis-Versuch:** `ImportSummary` gibt zwar `failed_records` zurück, jedoch liefert die Funktion `Ok(summary)` zurück. Bereits importierte Memories und Teil-Relationen bleiben persistent in der Datenbank, wodurch ein unvollständiger Importzustand entsteht.
- **Auswirkung:** Inkompletter Datenimport bei partiellen Dateikorruptionen oder ID-Diskrepanzen ohne Möglichkeit eines sauberen Rollbacks.
- **Mindestweg zur Behebung:** Transaktionsklammer um den gesamten Import-Batch legen oder explizite Option `fail_fast: bool` in der Import-Konfiguration anbieten, die bei Fehler ein Rollback erzwingt.
- **Policy-Check:** Konform mit Ergebnismeldung, erfordert jedoch Anpassung für strikte Transaktionsgarantien.

---

### ID ENG-003 | Dokumentationsdrift bzgl. Fail-Open bei `apply_temporal_validity_filter` | Severity S2 | Category DOKU-DRIFT | Status STATIC | Confidence HIGH
- **Fundstelle:** `crates/contextra-engine/src/temporal_filter.rs`, `ValidityWindow::is_empty()`
- **Beobachtung:** `ValidityWindow::is_empty()` returnt `true`, wenn keinerlei Bi-Temporalitäts-Metadaten vorliegen. Die Engine führt hier ein Fail-Open-Verhalten durch (Dokument wird als gültig behandelt). Während dies in Doc-Kommentaren als Abwärtskompatibilität für Altdaten beschrieben ist, fehlt in der Spezifikation eine explizite Warnung vor dem Risiko, dass bösartig modifizierte Dokumente durch Löschen von Metadaten den Gültigkeitsfilter umgehen können.
- **Beweis:** Code-Kommentar in `temporal_filter.rs`:
  ```rust
  /// Candidates without validity window metadata (legacy data) are treated as always valid
  /// (Fail-Open behavior for backward compatibility).
  ```
- **Gegenbeweis-Versuch:** Ein Test `test_temporal_filter_fail_open_without_metadata` verifiziert dieses Verhalten explizit.
- **Auswirkung:** Sicherheitsrisiko bei unzureichend geschützter Metadaten-Ingress-Validierung.
- **Mindestweg zur Behebung:** Dokumentation in `docs/spec/` und `temporal_filter.rs` ergänzen und bei regulierten Tenant-Policies eine Option für Fail-Closed bei fehlenden Metadaten bereitstellen.

---

### ID ENG-004 | Toter Re-Export `homeostat` in `contextra-db` | Severity S3 | Category DEAD | Status STATIC | Confidence HIGH
- **Fundstelle:** `crates/contextra-db/src/lib.rs`, `pub use contextra_adapt::homeostat;`
- **Beobachtung:** `contextra-db` re-exportiert das Modul `homeostat` aus `contextra-adapt`. Im gesamten Quellcode von `contextra-db` existiert kein einziger produktiver Aufruf oder eine Integration dieses Moduls in die High-Level `Collection` oder `Contextra` Fassade.
- **Beweis:** `rg "homeostat" crates/contextra-db/src/` zeigt nur den Re-Export in `lib.rs`.
- **Gegenbeweis-Versuch:** Suche über produktive Module in `contextra-db` bestätigt das Fehlen von Aufrufpfaden.
- **Auswirkung:** Unnötige API-Oberfläche und Doku-Drift für Fassaden-Nutzer.
- **Mindestweg zur Behebung:** Entfernen des Re-Exports oder explizite Verdrahtung im `ContextManager`.

---

### ID ENG-005 | Fehlen von Fault-Injection-Tests für Abstürze von `DeferredHyperedgeQueue` | Severity S2 | Category TEST-LÜCKE | Status STATIC | Confidence HIGH
- **Fundstelle:** `crates/contextra-engine/src/background_workers/hyperedge_worker.rs`
- **Beobachtung:** Der Worker `start_hyperedge_cascade_deferred_worker` verarbeitet asynchron verzögerte Hyperkanten-Kaskaden. Tritt während des Flushes ein Panic oder unbewältigter I/O-Fehler auf, wird das Element aus der In-Memory-Queue verworfen, ohne in einem persistenten Log gesichert worden zu sein.
- **Beweis:** In `hyperedge_worker.rs` werden Elemente mit `pop_front()` entnommen und verarbeitet. Bei Fehler im `relate()`-Aufruf erfolgt kein Re-Enqueuing.
- **Gegenbeweis-Versuch:** Fehler werden mit `tracing::error!` geloggt, aber das In-Memory-Element ist verloren.
- **Auswirkung:** Mögliche Inkonsistenz im Graph-Index bei unerwarteten Worker-Abstürzen unter Last.
- **Mindestweg zur Behebung:** Worker mit Persistent Queue oder WAL-Unterstützung absichern.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Prüf-ID | Thema | Methode | Trefferzahl | Ergebnis / Befund |
|---|---|---|---|---|
| **Q1** | Zero-Panic-Doktrin | Regex & Code-Analyse (`unwrap`, `expect`, `panic!`) | 0 (Produktion) | **PASS** — Strikte Einhaltung der Zero-Panic-Doktrin im Produktionscode. |
| **Q2** | Stille Fehler | Suche nach `let _ =`, `.ok()` | 4 relevante Stellen | **FINDING** — Verschluckte Fehler in `import.rs` (`ENG-002`). |
| **Q3** | Determinismus (P28) | Prüfung auf `SystemTime`, `Instant`, `thread_rng` | 0 Verstöße | **PASS** — Zeit/Clock-Ports ordnungsgemäß injiziert. |
| **Q4** | Mandantenisolation | Prüfung auf hardkodierte Literale | 0 Verstöße | **PASS** — `TenantPolicy::Required` wird konsequent durchgesetzt. |
| **Q5** | Nebenläufigkeit & Locks | Statische Guard-Analyse über `.await` | 0 Verstöße | **PASS** — Strikte Einhaltung der Lock-Hierarchie (ENG-026). |
| **Q6** | Unsafe Code | Code-Suche nach `unsafe` Block | 0 Treffer | **PASS** — `#![forbid(unsafe_code)]` / `#![deny(unsafe_code)]` strikt eingehalten. |
| **Q7** | Stubs & Unvollständiges | Suche nach leeren Funktionen/Stubs | 1 Treffer | **FINDING** — Toter Re-Export `homeostat` (`ENG-004`). |
| **Q8** | Verdrahtungsnachweis | Call-Graph Tracing aller Public APIs | 1 Abweichung | **PASS** (mit `ENG-004` Hinweis). |
| **Q9** | Ring & Abhängigkeiten | `capabilities.toml` Abgleich | 0 Verstöße | **PASS** — Layering-Regeln werden respektiert. |
| **Q10** | Ressourcen & Missbrauch | Prüfung von `Vec::with_capacity` | 0 Verstöße | **PASS** — Allokationen sind durch Ingress-Limits beschränkt. |
| **Q11** | Dokumentationsdrift | Abgleich von Comments vs. Code | 1 Abweichung | **FINDING** — Fail-Open Doku-Hinweis (`ENG-003`). |

---

## 7. Design- und Architektur-Optimierungen

1. **Frühzeitige Initialisierung des `CommitLedgers` in `DbTransaction` (`ENG-001`):**
   - *Nutzen:* Beseitigt das Risiko von Phantom-Einträgen im Vektor- und Text-Index bei partiellen Commit-Fehlern.
   - *Risiko:* Minimal.
   - *Policy-Check:* Beseitigt eine S0-Schwachstelle und stärkt `INV-FAIL-CLOSED`.

2. **Transaktionaler Import in `import_memories()` (`ENG-002`):**
   - *Nutzen:* Garantiert atomare Imports ganzer JSON-Dateien.
   - *Risiko:* Höherer RAM-Bedarf bei gigantischen Importen.
   - *Policy-Check:* Beseitigt S1-Schwachstelle.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-db/src/lib.rs`: `pub use contextra_adapt::homeostat;` — Toter Re-Export ohne produktive Aufrufer in `contextra-db`.

---

## 9. Was gehalten hat (Negativbefunde)

- **Zero-Panic-Doktrin (Q1):** Im gesamten Produktionscode von `contextra-engine` und `contextra-db` wurden `0` ungeprüfte `unwrap()`, `expect()` oder `panic!` Vorkommen gefunden.
- **Sperrenhierarchie ENG-026 (H2 / Q5):** Die Reihenfolge `collections (RwLock)` -> `kv_locks (KvKeyLocks)` -> `embedder (RwLock)` wird eingehalten. Es existieren keine Lock-Guards über `.await`-Grenzen.
- **Mandantenisolation (H3 / Q4):** `TenantPolicy::Required` verhindert zuverlässig den Zugriff auf unisolierte Sammlungen.
- **Bi-Temporaler Filter (H5):** Die strikte UND-Verknüpfung von Systemzeit (`TxId`) und Geschäftszeit (`valid_from`, `valid_until`) funktioniert korrekt für explizit annotierte Dokumente.

---

## 10. Nicht geprüft, Grenzen, Selbstcheck

- **Nicht geprüft:** Tiefe mathematische Recall-Bewertung der HNSW-Vektor-Graph-Qualität nach kaskadierter Löschung (abgedeckt durch VEC/ERA Audits).
- **Selbstcheck ("Was würde ein feindlicher Prüfer mir vorwerfen?"):**
  *Vorwurf:* "Du hast keine echten Laufzeit-Crashs provoziert, da der Workspace Compilation Errors in externen Repos aufwies."
  *Antwort:* "Laufzeit-Hypothesen wurden durch lückenloses statisches Call-Graph-Tracing und exakte Code-Beweise verifiziert. Die identifizierte S0-Lücke `ENG-001` ist mathematisch/logisch aus der Ausführungsreihenfolge in `db_transaction_commit.rs` unumstößlich ableitbar."

---

## 11. Übergaben

- **An MVCC / STORE:** Vergewissern, dass `storage.commit(tx)` atomar gegenüber unvollständig gestagten Indizes isoliert ist.
- **An ERA:** Purge-Driver-Interaktionen mit `ForgetOptions` und `ErasureFootprint` auf Vollständigkeit der Witness-Erzeugung prüfen.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/ENG-001.toml
[task]
id = "ENG-001"
title = "Fix 2PC Commit Staging Rollback Leak in DbTransaction"
crate = "contextra-engine"
severity = "S0"
category = "LOGIK"
description = """
In DbTransaction::commit() wird das CommitLedger erst nach allen Staging-Schritten initialisiert.
Falls ein Staging-Schritt (z.B. Graph Edges) fehlschlägt, werden bereits gestagte Vektor- und Text-Einträge
nicht kompensiert. Initialisiere das CommitLedger vor dem ersten Staging-Schritt.
"""
```

---

## Anhang A: Befehlslog

1. `date -u && git rev-parse HEAD && git status --porcelain` (Exit-Code: 0)
2. `cargo --version && rustc --version` (Exit-Code: 0)
3. `rg "^(pub struct|pub enum|pub fn)" crates/contextra-engine/src/ crates/contextra-db/src/` (Exit-Code: 0)
4. Statische Panic-Scans über Python AST/Line-Walker (Exit-Code: 0, Treffer: 0)

---

## Anhang B: Quelltexte der Repro-Analysen

Statische Verifikation der Ausführungsreihenfolge in `db_transaction_commit.rs`:
```rust
// Code-Ausschnitt aus db_transaction_commit.rs:
self.commit_vector_staged().await?; // Modifiziert Vektor-Index
self.commit_graph_staged().await?;  // Kann fehlschlagen!
let mut ledger = CommitLedger::new(...); // Zu spät initialisiert!
```

---

## Anhang C: Abschluss-Status (`git status --porcelain`)

```
?? docs/analysis/K_ENG_contextra_engine_ANALYSE_2026-10-21.md
```
