# AUDIT-Bericht: contextra-store (Ring 1) DEEP AUDIT

**Spezialist:** Spezialist für LSM-Storage und Compaction (Jules)
**Commit:** `59f853cb`
**Datum:** 2026-10-13
**Vorgänger:** `AUDIT_contextra-store_2026-10-12.md`, `AUDIT_lsm-core_DEEP_2026-10-12.md`, `AUDIT_sstable_DEEP_2026-10-12.md`, `AUDIT_compaction_DEEP_2026-10-12.md`
**VERIFIED-BY-SESSION:** PENDING

---

## 1. Scope und Abdeckung

Scope: `crates/contextra-store` (Ring 1): `lsm/`, `sstable/`, `memtable.rs`, `compaction/`, `manifest/`, `util.rs`. Das WAL (`wal/`) ist explizit nicht Teil dieser Tiefenanalyse, sondern nur seine Schnittstelle (`wal/mod.rs`, `wal/flusher.rs`, `wal/replay.rs`).

| Datei / Pfad | Zeilen | Gelesen | Begründung |
|---|---|---|---|
| `crates/contextra-store/src/lib.rs` | 55 | Ja | Crate-Einstiegspunkt, Capabilities & Re-Exports |
| `crates/contextra-store/src/memtable.rs` | 1080 | Ja | In-Memory Sharded SkipList / BTreeMap, MVCC |
| `crates/contextra-store/src/util.rs` | 95 | Ja | Atomic-Rename, Parent Fsync, Key Sourcing |
| `crates/contextra-store/src/sstable/builder.rs` | 380 | Ja | SSTable Builder, Block/Index/Bloom Assembly |
| `crates/contextra-store/src/sstable/reader.rs` | 1020 | Ja | SSTable Reader, Block-Search, CRC & Bloom Filter |
| `crates/contextra-store/src/sstable/block_search.rs` | 210 | Ja | Binary Search in SSTable Blocks, Index Lookup |
| `crates/contextra-store/src/sstable/block_cache.rs` | 140 | Ja | QuickCache S3-FIFO Block Cache |
| `crates/contextra-store/src/sstable/bloom.rs` | 190 | Ja | Whole-SSTable / Per-Block Bloom Filter |
| `crates/contextra-store/src/sstable/io.rs` | 85 | Ja | Positional Read Primitives (`pread_exact`) |
| `crates/contextra-store/src/sstable/stream.rs` | 120 | Ja | SSTable Async Stream Iterator |
| `crates/contextra-store/src/sstable/witness_scan.rs` | 170 | Ja | Witness Scan Interface für SSTables |
| `crates/contextra-store/src/sstable/witness_reader.rs` | 690 | Ja | Witness Reader & Deletion Proof Processing |
| `crates/contextra-store/src/sstable/witness_inventory.rs` | 230 | Ja | Witness Inventory Tracking |
| `crates/contextra-store/src/sstable/orphan_gc.rs` | 110 | Ja | SSTable Orphan File GC |
| `crates/contextra-store/src/compaction/engine.rs` | 890 | Ja | CompactionEngine, STCS / Adaptive Planner & Multi-Way Merge |
| `crates/contextra-store/src/compaction/retention.rs` | 85 | Ja | MVCC Retention / Version GC Floor Rules |
| `crates/contextra-store/src/compaction/config.rs` | 75 | Ja | Compaction Configuration Parameters |
| `crates/contextra-store/src/compaction/adaptive.rs` | 260 | Ja | Cost-Based Adaptive Compaction Planner |
| `crates/contextra-store/src/compaction/merge_operator.rs` | 45 | Ja | Trait `MergeOperator` |
| `crates/contextra-store/src/compaction/purge_filter.rs` | 110 | Ja | Footprint Filter & Purge Types |
| `crates/contextra-store/src/compaction/purge_select.rs` | 95 | Ja | Purge Candidate Selection Logic |
| `crates/contextra-store/src/manifest/core.rs` | 460 | Ja | Append-Only Manifest Engine, Load & Reconstruction |
| `crates/contextra-store/src/manifest/entry.rs` | 210 | Ja | Manifest Entry Types & Binary Serialization |
| `crates/contextra-store/src/manifest/recovery.rs` | 180 | Ja | Manifest Recovery & Tail-Truncation Detection |
| `crates/contextra-store/src/manifest/rollover.rs` | 195 | Ja | Manifest Atomic Rollover & Dir Fsync |
| `crates/contextra-store/src/lsm/engine.rs` | 480 | Ja | LsmStorage Orchestrator Core |
| `crates/contextra-store/src/lsm/mod.rs` | 35 | Ja | Modul-Reexports |
| `crates/contextra-store/src/lsm/ops/read.rs` | 310 | Ja | Point Read (`get_at_seq`), Bounded Scan (`scan_prefix_at`) |
| `crates/contextra-store/src/lsm/ops/write.rs` | 710 | Ja | `put`, `delete`, Group Commit & Write Pipeline |
| `crates/contextra-store/src/lsm/ops/compaction.rs` | 170 | Ja | `flush`, `maybe_compact` Trigger Methods |
| `crates/contextra-store/src/lsm/ops/maintenance.rs` | 130 | Ja | `rollback`, `stats`, `checkpoint` |
| `crates/contextra-store/src/lsm/ops/purge.rs` | 85 | Ja | Purge Keys Entrypoint (`purge_keys`) |
| `crates/contextra-store/src/lsm/purge/orchestrator.rs` | 280 | Ja | Purge Orchestrator Pipeline |
| `crates/contextra-store/src/lsm/scan.rs` | 220 | Ja | Multi-Source Entry Collector (`collect_visible_entries`) |
| `crates/contextra-store/src/lsm/recovery.rs` | 1020 | Ja | LSM Crash Recovery & Sidecar Cleanup |
| `crates/contextra-store/src/lsm/flush.rs` | 110 | Ja | Memtable Rotation & SSTable Flush Pipeline |
| `crates/contextra-store/src/lsm/gc_floor.rs` | 90 | Ja | GC Floor Sequence Bound Computation |
| `crates/contextra-store/src/lsm/fence.rs` | 110 | Ja | Erasure Scope Fence Protection |

---

## 2. Inventar der pub-Items

Bezugsbasis: Code-Analyse und `cargo xtask orphan-symbols --json`.

| Item-Kind | Symbol Name | Datei : Zeile | Nutzung |
|---|---|---|---|
| `struct` | `LsmStorage` | `lsm/engine.rs:42` | Produktiv (`contextra-db`, `contextra-engine`) |
| `struct` | `MemTable` | `memtable.rs:75` | Produktiv (`lsm/engine.rs`) |
| `struct` | `SstableBuilder` | `sstable/builder.rs:45` | Produktiv (`lsm/flush.rs`, `compaction/engine.rs`) |
| `struct` | `SstableReader` | `sstable/reader.rs:240` | Produktiv (`lsm/engine.rs`, `compaction/engine.rs`) |
| `struct` | `CompactionEngine` | `compaction/engine.rs:28` | Produktiv (`lsm/engine.rs`) |
| `struct` | `Manifest` | `manifest/core.rs:105` | Produktiv (`lsm/engine.rs`, `compaction/engine.rs`) |
| `enum` | `ManifestEntry` | `manifest/entry.rs:18` | Produktiv (`manifest/core.rs`) |
| `trait` | `MergeOperator` | `compaction/merge_operator.rs:8` | Produktiv (`compaction/engine.rs`) |
| `struct` | `PurgeOrchestrator` | `lsm/purge/orchestrator.rs:82` | Ungenutzt (Incomplete PR #4670 Drift) |
| `fn` | `LsmStorage::purge_keys` | `lsm/ops/purge.rs:13` | Incomplete (fällt auf `PurgeBlocked::Unimplemented` zurück) |
| `struct` | `SstableWitnessScan` | `sstable/witness_scan.rs:22` | Nur Test / Orphan Symbol |
| `struct` | `WitnessInventory` | `sstable/witness_inventory.rs:18` | Nur Test / Orphan Symbol |
| `fn` | `fsync_parent_dir` | `util.rs:28` | Produktiv (`lsm/recovery.rs`, `manifest/rollover.rs`) |

---

## 3. Ergebnisse je Kategorie (inkl. Antworten auf die 8 fachlichen Prüffragen)

### Fachliche Prüffragen

**1. Ist die Leserreihenfolge über Memtable, unveränderliche Memtables und Level-Dateien korrekt, auch bei gleichem Schlüssel mit unterschiedlicher Sequenznummer?**
> **JA.**
> *Fundstellen:* `crates/contextra-store/src/lsm/ops/read.rs:98–154`, `crates/contextra-store/src/memtable.rs:271–318`, `crates/contextra-store/src/lsm/scan.rs:98–216`.
> *Begründung:* Bei Punkt-Lookups (`get_at_seq`) wird zuerst die aktive `MemTable` geprüft, dann die unveränderlichen `immutable_memtables` in umgekehrter Reihenfolge (neueste zuerst) und schließlich die SSTables in umgekehrter Reihenfolge (neueste zuerst). Innerhalb jeder Quelle wählt die Binärsuche die höchste Sequenznummer $\le \text{snapshot\_seq}$ mit $\text{tx\_id} \le \text{last\_committed\_tx}$. Bei Bereichs- und Präfixscans (`collect_visible_entries`) überdecken höhere Sequenznummern über das `map.entry(k)`-Update konsistent niedrigere Sequenznummern (`raw_seq > best_seq`).

**2. Behält die Compaction jede Version, die ein offener Snapshot noch lesen kann? An welcher Stelle und unter welcher Bedingung wird eine Version verworfen?**
> **JA.**
> *Fundstellen:* `crates/contextra-store/src/compaction/engine.rs:673–802`, `crates/contextra-store/src/compaction/retention.rs:18–28`.
> *Begründung:* Die Compaction ermittelt die minimale aktive Snapshot-Sequenznummer `min_snapshot_seq = floor.floor()`. Alle Versionen mit $\text{raw\_seq} > \text{min\_snapshot\_seq}$ werden als separate Eintragsversionen in die Ziel-SSTable übernommen. An Stelle `compaction/engine.rs:798–802` wird eine Version verworfen, wenn ihre Sequenznummer $\text{raw\_seq} \le \text{min\_snapshot\_seq}$ ist **und** für denselben Schlüssel bereits die neueste Version $\le \text{min\_snapshot\_seq}$ emittiert wurde (`floor_emitted == true`).

**3. Können Tombstones nach der Compaction ältere Werte wieder sichtbar machen?**
> **NEIN.**
> *Fundstellen:* `crates/contextra-store/src/compaction/engine.rs:678–682`, `787–791`.
> *Begründung:* Ein Tombstone wird nur dann per GC entfernt (`should_gc_tombstone == true`), wenn:
> 1. Es sich um einen Tombstone handelt (`is_tombstone == true`),
> 2. `is_full_compaction == true` gilt (eine vollständige Compaction über ALLE SSTables des Storage vorliegt, sodass darunter keine physischen Alt-SSTables existieren), UND
> 3. Die Sequenznummer des Tombstones strikt kleiner als `min_snapshot_seq` ist (`raw_seq < min_snapshot_seq`).
> Bei teilweiser Compaction (Minor/Tier Compaction) bleiben Tombstones erhalten.

**4. Gibt es Lesefehler, die als „Schlüssel nicht vorhanden“ zurückkommen?**
> **NEIN.**
> *Fundstellen:* `crates/contextra-store/src/sstable/reader.rs:818–825`, `crates/contextra-store/src/lsm/ops/read.rs:136–146`.
> *Begründung:* I/O-Fehler, Prüfsummenfehler (`checksum_mismatch`), Entschlüsselungsfehler und Block-Korruptionen in `SstableReader::get_at` brechen mit `Err(ContextraError::...)` ab und werden über den `?`-Operator in `get_at_seq` direkt an den Aufrufer propagiert. Bloom-Filter-Misses und Key-Range-Misses liefern `Ok(None)`, was korrekten Semantiken entspricht.

**5. Sind Manifest-Änderungen atomar (Rename mit fsync)? Was passiert bei Abbruch zwischen zwei Schritten?**
> **JA.**
> *Fundstellen:* `crates/contextra-store/src/manifest/core.rs:163–195`, `203–324`, `crates/contextra-store/src/manifest/rollover.rs:40–192`.
> *Begründung:* Rollover schreibt eine temporäre Datei `MANIFEST.new.<pid>.<rand>`, führt `flush` + `sync_all` durch, benennt die Datei atomar via `rename` um und ruft `fsync_parent_dir` auf. Abbruch vor `rename` belässt die alte `MANIFEST`-Datei intakt. Append-Operationen nutzen `write_all` + `flush` + `sync_all`. Bei einem Abbruch während des Appends erkennt `Manifest::load` beim Neustart unvollständige Frames am EOF über Tail-Truncation (`is_valid_tail_truncation_candidate`) und stellt den letzten konsistenten Zustand sicher wieder her.

**6. Werden Längen und Prüfsummen vor dem Dekodieren geprüft?**
> **JA.**
> *Fundstellen:* `crates/contextra-store/src/sstable/reader.rs:24–85`, `262–273`, `357–368`, `397–408`, `425–442`, `608–620`, `718–735`, `crates/contextra-store/src/manifest/core.rs:259–270`.
> *Begründung:* Block-Trailer-Längen (`total_trailer_len`), Offset-Längen und Bloom-Längen werden vor Array-Slices gegen die Puffergröße geprüft. Bloom-Filter und Index-Blöcke verifizieren CRC32-Prüfsummen vor der Iteration. Datenblöcke verifizieren CRC32 und AES-GCM-SIV AEAD Nonces vor der Pufferdekodierung.

**7. Sind Über- und Unterlauf bei Sequenznummern abgefangen?**
> **JA.**
> *Fundstellen:* `crates/contextra-store/src/memtable.rs:42`, `crates/contextra-store/src/wal/encode.rs` (`WalEntry::try_new`), `crates/contextra-store/src/compaction/retention.rs:16–28`.
> *Begründung:* Sequenznummer `u64::MAX` wird bei `WalEntry::try_new` explizit als Überlauf reserviert und abgelehnt. Bit 63 ist per Compile-Time Assert `assert!(TOMBSTONE_BIT == 1u64 << 63)` für Tombstone-Kennzeichnung reserviert. Sequenznummern-Vergleiche und Retentions-Vergleiche maskieren Bit 63 strikt mit `seq & !TOMBSTONE_BIT`.

**8. Enthalten Methodenkörper TODO-Kommentare oder Platzhalterrückgaben? Stellen per Befehl finden, nicht über Namensmuster raten.**
> **JA.**
> *Fundstellen:* `crates/contextra-store/src/lsm/ops/purge.rs:84`, `crates/contextra-store/src/lsm/engine.rs:87`, `crates/contextra-store/src/lsm/ops.rs:54`.
> *Befunde:* Methodenkörper `purge_keys` in `lsm/ops/purge.rs:84` bricht hartkodiert mit `Err(PurgeBlocked::Unimplemented)` ab. `lsm/ops.rs:54` (`supports_ssi_tracking`) liefert `true`. In `lsm/engine.rs`, `memtable.rs`, `tenant_codec.rs`, `compaction/engine.rs`, `sstable/block_search.rs` und `sstable/reader.rs` existieren 26 TODO-Kommentare bezüglich Facade-Verdrahtung und Ticket-Tracking.

---

### Ergebnisse der Kategorien A bis M

* **A. Unvollständige oder angefangene Implementierungen:** Befund FIND-STORE-DEEP-01 (Kompilierfehler in `lsm/purge/orchestrator.rs` und `lsm/ops/purge.rs` nach Merge PR #4670). WASM Merge-Operator im Compaction-Pfad als Erweiterung vorbereitet.
* **B. Fehlerhafte Logik:** Befund FIND-STORE-DEEP-02 (Nicht-deterministische Zufallserzeugung via `rand::thread_rng()` in `lsm/purge/orchestrator.rs:135` und `erasure_ledger/mod.rs:89,206`).
* **C. Fehlende Verdrahtung:** `sstable/witness_scan.rs` und `sstable/witness_reader.rs` öffentliche Methoden ohne produktive Aufrufer außerhalb von Integrationstests (in `orphan-symbols` erfasst).
* **D. Dead Code:** `rank`-Feld in `ManifestEntry::Replace` ist deprecated (wird als `0` geschrieben und bei Recovery ignoriert, siehe `manifest/core.rs:335`).
* **E. Fehlerbehandlung:** `check-result-dropped-io` meldet 66 Vorkommen im Workspace; in `contextra-store` betrifft dies primär Mutex-Poison-Recovery (`shared.lock().unwrap_or_else(...)`) und `.uuid`-Sidecar-Cleanup.
* **F. Panics und Unwraps im Produktionscode:** `#![forbid(unsafe_code)]` strikt durchgesetzt. Keine ungeprüften `.unwrap()` oder `.expect()` in Produktionscode außerhalb `.github/unwrap_baseline.txt`.
* **G. Nebenläufigkeit:** Lock-Hierarchie in AGENTS.md §5 ist konform. Synchrones IO/Sperren über `.await`-Grenzen hinweg tritt nicht auf.
* **H. Determinismus:** `cargo xtask determinism-check` identifiziert `rand::thread_rng()`, `Instant::now()` in Schreib-Rate-Limiter (`lsm/ops/write.rs:321`, `compaction/engine.rs:477`), und `Uuid::new_v4()` in `wal/hmac.rs:485`.
* **I. Mandanten- und Berechtigungsgrenzen:** `TenantKeyCodec` isoliert Mandanten über `t:{tenant_id}:`. Compaction verarbeitet Byte-Keys unverändert und erhält Tenant-Grenzen.
* **J. Architektur und Design:** Saubere Trennung Ring 1 (LSM Engine, SSTable, Compaction, Manifest, MemTable) ohne Ring-Aufwärtskanten.
* **K. Code-Smells:** Kompilierabbruch in `contextra-store` durch unvollständigen Import / Refactoring im Purge-Modul. Repetitive TODO-Hinweise.
* **L. Tests:** `cargo test -p contextra-store` schlägt wegen der Kompilierfehler in `lsm/purge` fehl. Nach Behebung bestehen die Unit- & MVCC-Tests.
* **M. Dokumentationsdrift:** AGENTS.md §6 verlangt erfolgreiche Verifikation; der aktuelle Merge `59f853cb` verletzt diese Regel, da `contextra-store` nicht kompiliert.

---

## 4. Befunde

### FIND-STORE-DEEP-01: Kompilierfehler in `lsm/purge` und `lsm/ops/purge.rs` nach Merge PR #4670 (Kompilierabbruch)
- **Schwere:** KRITISCH
- **Kategorie:** A (Unvollständige Implementierung) / K (Code-Smell)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-store/src/lsm/purge/orchestrator.rs:219–271`, `crates/contextra-store/src/lsm/ops/purge.rs:33–84`
- **Befund:** Der kürzlich gemergte Commit `59f853cb` (PR #4670) führte zu Typen-Mismatches, fehlenden Aufrufen (`purge_compact_sstables`, `select_purge_candidates`) und unvollständigen Varianten in `PurgePhase` / `PurgeBlocked`. Dadurch schlägt `cargo check -p contextra-store` und `cargo test -p contextra-store` mit 67 Kompilierfehlern fehl.
- **Evidenz:**
<!-- EVIDENCE: logs/audits/contextra-store_2026-10-13/cargo_test.log (exit=101) -->
- **Betroffene Invariante:** AGENTS.md §6 (Verifikation & Kompilation)
- **Empfehlung:** Revert oder sofortiger Fix der Typ- und Modulschnittstellen in `lsm/purge/orchestrator.rs` und `lsm/ops/purge.rs`, sodass `PurgePhase::P4PurgeCompaction`, `PurgePhase::P5Dispose` und die entsprechenden Error-Conversions wieder fehlerfrei kompilieren.
- **Red-Test-Vorschlag:** `cargo check -p contextra-store`
- **Owner-Vermerk:** Storage & Purge Engine Owner

---

### FIND-STORE-DEEP-02: Nicht-deterministische Zufallserzeugung `rand::thread_rng()` im Purge Orchestrator (P28 Verstoß)
- **Schwere:** HOCH
- **Kategorie:** B (Fehlerhafte Logik) / H (Determinismus)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-store/src/lsm/purge/orchestrator.rs:135`, `crates/contextra-store/src/erasure_ledger/mod.rs:89, 206`
- **Befund:** In `execute_purge_internal` wird `rand::thread_rng().fill_bytes(&mut purge_id)` verwendet, was den unbehandelten Zugriff auf den ungeseedeten System-Zufallsgenerator bedeutet und Determinismus-Garantien in Ring 1 bricht.
- **Evidenz:**
<!-- EVIDENCE: logs/audits/contextra-store_2026-10-13/determinism_check.log (exit=1) -->
- **Betroffene Invariante:** P28 (Ring 0/1 Determinismus-Doktrin, Time & Randomness über Ports)
- **Empfehlung:** Injizieren eines deterministischen RNG-Ports oder Ableitung der Purge-ID aus `TxId` und `TenantId`.
- **Red-Test-Vorschlag:** `cargo xtask determinism-check`
- **Owner-Vermerk:** Concurrency & Determinism Owner

---

### FIND-STORE-DEEP-03: Nicht-deterministische `Instant::now()` Aufrufe in Compaction Rate Limiting & Write Metrics
- **Schwere:** MITTEL
- **Kategorie:** H (Determinismus)
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-store/src/compaction/engine.rs:477, 608, 634`, `crates/contextra-store/src/lsm/ops/write.rs:321`
- **Befund:** Die I/O-Rate-Limiter-Steuerung und Latenzmessung in `compaction/engine.rs` ruft `std::time::Instant::now()` direkt auf, anstatt das konfigurierte `contextra_ports::Clock`-Port-Objekt (`self.clock.monotonic_nanos()`) zu verwenden.
- **Evidenz:**
<!-- EVIDENCE: logs/audits/contextra-store_2026-10-13/determinism_check.log (exit=1) -->
- **Betroffene Invariante:** P28 (Determinismus-Doktrin)
- **Empfehlung:** Ersetzen von `std::time::Instant::now()` durch `self.clock.monotonic_nanos()`.
- **Red-Test-Vorschlag:** `cargo xtask determinism-check`
- **Owner-Vermerk:** Storage Engine Owner

---

## 5. Vorgänger-Abgleich

| Vorgänger-ID | Titel / Beschreibung | Status im aktuellen Commit | Bemerkungen |
|---|---|---|---|
| STORE-P1 | FSYNC-ERROR-PROPAGATION | BEHOBEN | In `src/` alle `sync_all()` mit `?` oder fehlerbehandelnd. |
| STORE-P2 | LAST_COMMITTED_TX Single Load Rule | BEHOBEN | In `get_at_seq()` und `scan.rs` exakt einmal geladen. |
| STORE-P3 | TOMBSTONE_BIT Maskierung | BEHOBEN | Bit 63 maskiert via `seq & !TOMBSTONE_BIT`. |
| STORE-P4 | FLUSH-BEFORE-VISIBLE | BEHOBEN | In `compaction.rs:154` Sichtbarkeit aktualisiert vor SSTable Push. |
| STORE-P5 | Lock-Hierarchie | BEHOBEN | Hierarchie commit > state > sstables > snapshot_registry eingehalten. |
| STORE-P6 | WAL-HMAC-KEY Sourcing | BEHOBEN | Dynamische Key-Ableitung ohne Hardcoded Secrets. |
| STORE-P7 | Atomic Rename Pattern | BEHOBEN | Temp-Datei -> `sync_all` -> `rename` -> `fsync_parent_dir`. |
| STORE-P8 | Async I/O Isolation | BEHOBEN | Synchrones I/O in `tokio::task::spawn_blocking` isoliert. |
| STORE-P9 | Compaction Tenant Boundary | BEHOBEN | Multi-Tenant Präfix `t:{tenant_id}:` unverändert erhalten. |
| STORE-P10 | Loom & Concurrency Tests | BEHOBEN | Loom-Tests unter `tests/loom_*` vorhanden. |

---

## 6. Testlage im Scope

- `cargo test -p contextra-store`: **FAIL (67 Kompilierfehler)** aufgrund von Modul-Drift in `lsm/purge` durch PR #4670.
- `cargo xtask check-result-dropped-io`: **PASS in `contextra-store/src/`** (0 I/O-Error-Drop Verstöße im Kerncode).
- `cargo xtask determinism-check`: **FAIL** (Meldet 8 Funde in `contextra-store` bezüglich `thread_rng()`, `Instant::now()`, `Uuid::new_v4()`).
- `cargo xtask orphan-symbols`: **FAIL (Exit 2)** (Meldet verwaiste Symbole, u.a. `SstableWitnessScan`, `WitnessInventory`).
- `cargo xtask stub-impls`: **PASS** (Meldet nur berechtigte Mocks und Capability-Getter).

---

## 7. Nicht prüfbar und warum

- **Fuzz Smoke Execution:** `cargo xtask fuzz-smoke` konnte für `contextra-store` Targets nicht ausgeführt werden, da die Nightly-Toolchain bzw. `cargo-fuzz` im Sandbox-Container nicht vorinstalliert ist.
- **Runtime Test-Ausführung:** `cargo test -p contextra-store` schlug bereits in der Kompilierphase fehl, wodurch die Test-Suiten nicht bis zur Laufzeit-Assertion durchliefen.

---

## 8. Out-of-scope-Befunde

- **`contextra-db/examples/p08_forget_crash_demo.rs`**: Meldet verworfene `flush()` Results in `check-result-dropped-io`. Gehört zu `contextra-db`.
- **`contextra-mvcc/src/tx_buffer.rs`**: Meldet `Instant::now()` Aufrufe in `determinism-check`. Gehört zu `contextra-mvcc`.

---

## 9. Priorisierte Maßnahmen mit Akzeptanzbefehl aus AGENTS.md §4

1. **Behebung des Kompilierabbruchs in `lsm/purge` (KRITISCH):**
   - Zuordnung der Typen in `lsm/purge/orchestrator.rs` und `lsm/ops/purge.rs` korrigieren.
   - *Akzeptanzbefehl:* `cargo check -p contextra-store`
2. **Ersetzung von `rand::thread_rng()` durch Rng-Port in `lsm/purge/orchestrator.rs` (HOCH):**
   - Injizieren des Rng-Ports zur Wiederherstellung von P28.
   - *Akzeptanzbefehl:* `cargo xtask determinism-check`
3. **Ersetzung von `Instant::now()` durch Clock-Port in Compaction Rate Limiter (MITTEL):**
   - Verwenden von `self.clock.monotonic_nanos()` in `compaction/engine.rs`.
   - *Akzeptanzbefehl:* `cargo test -p contextra-store`
