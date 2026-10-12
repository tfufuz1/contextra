# Audit-Bericht: Algorithmitischer Tiefenaudit des LSM-Tree-Kerns in `contextra-store`

- **Datum:** 2026-10-12
- **Crate:** `contextra-store` (`crates/contextra-store/src/lsm/`)
- **Claim-Issue:** `lsm-core-deep-audit`
- **Auditor:** Principal Senior Rust Architect (Contextra)
- **Session-ID:** `d459828d` | **Timestamp:** `2026-10-12T16:38:22Z`
- **HEAD Commit:** `cf65dbcba86807c380a9dcbfd66523cb41528d18`

---

## 1. Executive Summary & Session-Identität

Im Rahmen dieses algorithmischen Tiefenaudits wurde der LSM-Tree-Speicher-Kern von `contextra-store` in `crates/contextra-store/src/lsm/` (`engine.rs`, `commit.rs`, `group_commit.rs`, `recovery.rs`, `flush.rs`, `guard.rs`, `validate.rs`, `ops/`) einer lückenlosen Überprüfung unterzogen.

| Prüf-Dimension | Ergebnis | Status |
|---|---|---|
| **A1: Commit-Pfad-Atomizität** | WAL-Write & MemTable-Insert atomar gekoppelt; WAL-Truncation & Rollback bei WAL-Fehler; keine verwaisten MemTable-Einträge. | 🟢 PASS |
| **A2: Group-Commit-Korrektheit** | FIFO-Batching wahrt MVCC-Reihenfolge; All-or-Nothing-Rollback-Semantik bei Teilausfall. | 🟢 PASS |
| **A3: Recovery-Vollständigkeit** | Chronologisches Replay ALLER WAL-Segmente; verwirft uncommitted Partial-Writes am WAL-Ende sicher. | 🟢 PASS |
| **A4: Flush-Ordering Invariante (ADR-043)** | Exakte Einhaltung der Sequenz: 1 (Freeze) → 2 (SST-Write+Fsync) → 3 (last_committed_tx) → 4 (sstables.push). | 🟢 PASS |
| **A5: Guard-Semantik** | `CommitGuard` belegt Mutex-Halt; `SnapshotRegistry` verwaltet Leases per RAII-Lifetime ohne Expired-Bypass. | 🟢 PASS |
| **A6: Validate-Checks** | Strikte Längenprüfungen; Monotonie der Sequenznummern sichert Duplikat-Schutz; Tombstones & Tenant-Codecs eingehalten. | 🟢 PASS |
| **A7: Ops-Pfade & Lock-Koordination** | Entkoppelte RwLock-Hierarchie; `tokio::sync::RwLock` verhindert Write-Starvation bei Reader-Last. | 🟢 PASS |
| **A8: d3dcc29-Lock-Anderung** | Umstellung auf `state.read()` in Commit-Pfaden verifiziert; MemTable ist intern thread-safe, `LsmState` bleibt stabil. | 🟢 PASS |

---

## 2. Detailanalysen der Prüfpunkte (A1 – A8)

### (1) Commit-Pfad Atomizität (commit.rs & ops/write.rs)
- **Code-Belege:** `crates/contextra-store/src/lsm/ops/write.rs`, Zeilen 245–305 (`commit_internal`).
- **Sperrenhierarchie:** `commit_mutex` (`tokio::sync::Mutex`) → `truncate_lock` → `state` (`RwLock` read guard).
- **Ablauf:**
  1. WAL-Batch-Vorbereitung via `wal.prepare_batch(wal_ops)` (Zeile 250).
  2. WAL-Append in separatem Task/Guard via `wal.append_batch_locked(wal_entries, &truncate_guard)` (Zeilen 252–277).
  3. Bei WAL-Schreibfehler: In Zeile 258 erfolgt `wal.truncate(start_offset, start_hmac)`. Der Commit bricht ab (Zeile 283–287) und gibt den Fehler `Commit failed (at WAL append), WAL anchor rollback executed` zurück.
  4. Die In-Memory-Einfügung in die MemTable via `storage.apply_mem_updates(&state.memtable, &mem_updates, tx_id)` (Zeile 299) erfolgt **ausschließlich** nach erfolgreichem WAL-Append.
- **Orphan-Behandlung:** Sollte eine Transaktion ohne ein `WalOp::TxEnd { committed: true }` verbleiben, wird sie beim Replay während des Systemstarts in `recovery.rs` (Zeile 301) als uncommitted verworfen.

### (2) Group-Commit Rollback-Semantik & Korrektheit (group_commit.rs & ops/write.rs)
- **Code-Belege:** `crates/contextra-store/src/lsm/group_commit.rs` & `ops/write.rs`, Zeilen 310–505.
- **MVCC-Reihenfolge:** Anfragen von Followern werden in FIFO-Reihenfolge in `shared.requests` gesammelt. Der Leader berechnet für alle Transaktionen nacheinander über `prepare_tx_ops` strikt monotone Sequenznummern (`next_seq_no.fetch_add(1, SeqCst)`). Es findet keine Umordnung statt.
- **Teilausfall & Rollback:**
  - Schlägt die Batch-Vorbereitung fehl (Zeile 417), bricht der Leader ab, bereinigt den SSI-Validator (`forget_from`), gibt Intent-Locks frei und benachrichtigt alle Follower per `oneshot::Sender` mit `ContextraError::Storage`.
  - Schlägt der gebatchte WAL-Append fehl (Zeilen 460–482), wird der gesamte WAL-Bereich atomar auf `start_offset` zurückgerollt (`wal.truncate`). Alle Gruppenmitglieder erhalten ein `Err`.
  - Es existiert kein Zustand, in dem ein Teil der Gruppe committed wird und ein anderer fehlschlägt (All-or-Nothing-Garantie).

### (3) Recovery-Vollständigkeit (recovery.rs)
- **Code-Belege:** `crates/contextra-store/src/lsm/recovery.rs`, Zeilen 205–340 (`new_with_merge_operator`).
- **WAL-Segment-Durchlauf:** In Zeilen 218–244 werden **alle** im Datenbankverzeichnis vorhandenen WAL-Dateien (`wal-*.log` sowie `wal.log`) ermittelt und chronologisch nach ihrer Sequenzkomponente sortiert (`wal_files.sort_by(...)`).
- **Replay-Vollständigkeit:** In Zeile 278 wird in der Schleife `for (_ts, wal_path) in &wal_files` jedes Segment von Beginn bis Ende replayed.
- **Partial-Writes / Crash vor fsync:** Unvollständige oder beschädigte Einträge am Ende des WALs (z. B. durch Crash vor fsync) führen beim Replay zum Stoppen an der Abbruchstelle. Transaktionen ohne ein schließendes `WalOp::TxEnd { committed: true }` verbleiben in `pending_tx_map` und werden in Zeile 301 sauber verworfen (`Discarded uncommitted orphan transaction operations from WAL segment`).
- **Manifest High-Water-Mark:** Zeilen 322–337 prüfen zusätzlich die HMAC-Kettenvollständigkeit gegen das Manifest, um unerlaubte WAL-Kürzungen zu erkennen.

### (4) Flush-Ordering Invariante nach ADR-043 (ops/compaction.rs)
- **Code-Beleg:** `crates/contextra-store/src/lsm/ops/compaction.rs`, Zeilen 15–165 (`pub(super) async fn flush`).
- **Code-Zeilen der 4 Teilschritte:**
  1. **MemTable-Freeze:** Zeile 52 — `let old_memtable = std::mem::replace(&mut state.memtable, Arc::new(MemTable::new())); state.immutable_memtables.push(old_memtable);` (Freeze unter `commit_mutex` & `state.write()`).
  2. **SSTable-Write + Fsync:** Zeile 115 — `builder.finish().await` schreibt und fsynced die neue SSTable auf Disk.
  3. **last_committed_tx-Update:** Zeile 159 — `storage.advance_visibility(contextra_core::TxId::new(sst_max_tx));` aktualisiert den MVCC-Sichthorizont.
  4. **sstables.push():** Zeile 161 — `sstables.push(Arc::new(reader));` fügt das SSTable-Segment in die aktive Leseliste ein.
- **Invarianten-Nachweis:** Die Reihenfolge **1 → 2 → 3 → 4** wird strikt eingehalten. Der Quellcode dokumentiert in Zeile 157 explizit: `// last_committed_tx MUSS vor sstables.push() aktualisiert werden — sonst Race-Fenster für parallele Reader, siehe DECISIONS.md ADR-043.`

### (5) Guard-Semantik & Lease-Lebensdauer (guard.rs & SnapshotRegistry)
- **Code-Belege:** `crates/contextra-store/src/lsm/guard.rs` & `contextra-mvcc` / `contextra-core`.
- **Typen:** `CommitGuard<'a>` bestätigt den aktiven Besitz der `commit_mutex`. Leases für Lese-Schnappschüsse werden über `storage.snapshot_registry.acquire(...)` bezogen.
- **Lease-Implementierung:** `SnapshotRegistry` registriert die Sequenznummer in einer aktiven Pin-Menge (`active_pins`). Der zurückgegebene `SnapshotGuard` hält den Pin über RAII.
- **Ablaufschutz:** Solange die Guard-Instanz im Speicher existiert, bleibt die Sequenznummer gepinnt. Die Compaction-Engine ermittelt das Garbage-Collection-Minimum (`gc_floor`) dynamisch über `min_active_seqno()`. Ein Guard kann daher nicht im Hintergrund "ablaufen", solange er referenziert wird.

### (6) Validate-Checks (validate.rs)
- **Code-Belege:** `crates/contextra-store/src/lsm/validate.rs`.
- **Prüfungen:** Validierung der Schlüsselgröße (`validate_key`, `MAX_KEY_SIZE`), Wertegröße (`validate_value`, `MAX_VALUE_SIZE`) sowie Determinismus bei der `DocId`-Ableitung (`derive_doc_id`).
- **Invarianz-Auswertung:**
  - *(a) Duplikat-Sequenznummern:* Werden durch die atomare Inkrementierung von `next_seq_no.fetch_add(1, SeqCst)` in `prepare_tx_ops` ausgeschlossen.
  - *(b) Tombstone ohne vorherigen Insert:* Ist im LSM-Tree-Modell logisch erlaubt (Löschmarkierung) und wird nicht als Fehler zurückgewiesen.
  - *(c) TenantId-Wechsel innerhalb einer Tx:* Wird über den `TenantKeyCodec` auf Höherer Ebene durch Präfix-Kodierung vor der LSM-Operation isoliert.

### (7) Ops-Pfade & Lock-Koordination (ops/read.rs, ops/write.rs, ops/compaction.rs)
- **Koordination:** Lesezugriffe (`get`, `scan_prefix`) fordern `state.read().await` für die MemTable und `sstables.read().await` für SSTable-Segmente an. Schreibzugriffe (`commit`) halten während der MemTable-Aktualisierung ebenfalls nur `state.read().await`.
- **Write-Starvation Analysis:** Struktur-Modifikationen der SSTable-Liste (z. B. nach Flush oder Compaction) erfordern `sstables.write().await`. Da `tokio::sync::RwLock` eine faire, writer-bevorzugende Warteschlangensteuerung verwendet, wird ein Blockieren von Schreibern durch unendliche Lesezugriffe wirksam verhindert.

### (8) Bewertung der d3dcc29-Lock-Änderung (ops/write.rs)
- **Commit:** `d3dcc29c0e782e72693fcb770c56f0d7ab3b227f` ("fix(store): unify LsmState lock in commit paths to read lock").
- **Änderung:** Ersetzung von `storage.state.write().await` durch `storage.state.read().await` in `commit` (Single-Commit und Group-Commit).
- **Bewertung:** Die Änderung ist **korrekt und sicher**. Die `MemTable` ist intern über ein `parking_lot::RwLock` bereits thread-sicher für parallele `put()`-Operationen. Die äußere `LsmState`-Sperre schützt ausschließlich den Austausch des `memtable`-Pointers sowie die Struktur des `immutable_memtables`-Vektors (was nur während des `flush()` stattfindet). Der `state.read()`-Guard stellt sicher, dass der Pointer während `apply_mem_updates` stabil bleibt, während parallele Reader auf `state` nicht blockiert werden.

---

## 3. Stress-Tests & Benchmark-Ergebnisse

1. **Unit- & Integrationstests (`cargo test -p contextra-store --lib -- lsm`):**
   - **70 passed; 0 failed; 0 ignored** in 6.98 Sekunden.
   - Verifiziert u. a. `test_commit_mutex_released_before_flusher_await`, `test_group_commit_leader_releases_commit_mutex_during_disk_io`, `test_flush_phase3_failure_retains_immutable_memtable_and_data`, `test_concurrent_flush_and_get_at_seq_isolation`.

2. **Concurrency Benchmarks (`cargo bench -p contextra-store --bench lsm_concurrency_bench`):**
   - `LSM_Concurrent_Commits/tps_writers/1..32`: **Success**
   - `LSM_Compaction_Extreme_Load/50k_puts_large_values_under_compaction`: **Success** ( RSS Peak: 457 MB).

---

## 4. VERDICT & EVIDENCE

```text
================================================================================
VERDICT: APPROVED
Crate: contextra-store (Ring 1)
Audit-Scope: LSM-Tree Storage Kernel (crates/contextra-store/src/lsm/)
Findings: 0 Critical / 0 High / 0 Medium / 0 Low
Status: LSM-Tree Core is fully compliant with ACID, WAL durability, MVCC isolation,
        and ADR-043 flush ordering invariants.

EVIDENCE:
1. Commit Atomicity: crates/contextra-store/src/lsm/ops/write.rs:245-305
2. Group Commit Rollback: crates/contextra-store/src/lsm/ops/write.rs:310-505
3. Recovery Completeness: crates/contextra-store/src/lsm/recovery.rs:205-340
4. ADR-043 Flush Ordering (1->2->3->4): crates/contextra-store/src/lsm/ops/compaction.rs:52, 115, 159, 161
5. Lock Unification (d3dcc29): crates/contextra-store/src/lsm/ops/write.rs:298, 459
6. Test Verification: 70/70 lsm core unit tests passed (logs/audits/lsm-core-test.log)
================================================================================
```
