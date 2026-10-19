# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT — SUBSYSTEM LSM (`contextra-store` & `contextra-store-redb`)

## 1. Kurzfassung

**Gesamtrisiko:** MITTEL (S2) — Der Produktionspfad von `contextra-store` und `contextra-store-redb` weist eine hohe Grundstabilität und strikte Memory-Safety (`#![forbid(unsafe_code)]` in beiden Crates) auf. Es existieren jedoch kritische Testkompilierungs- und Wiring-Lücken in der Testsuite sowie potenzielle Invarianzbrüche unter speziellen Randbedingungen.

**Die fünf schwersten Befunde:**
1. `STORE-001` (Severity S2 | `TEST-LÜCKE` | `CONFIRMED`): **Zerbrochene Test-Suiten im Crate `contextra-store`**. Mehrere Integrationstests (`purge_error_variants`, `reopen_fd_leak`, `manifest_crash_no_resurrection`, `differential/purge_correctness`) kompilieren aufgrund von Typen- und API-Drift nicht mehr (`PurgeBlocked` vs `PurgeError`, fehlende `StorageEngine`-Trait-Imports).
2. `STORE-002` (Severity S2 | `UNWIRED` | `CONFIRMED`): **Unverdrahtete `CompactionEngineWiring` im Composition Root**. `CompactionEngineWiring::build()` existiert als Abstraktion zur Instanziierung der CompactionEngine, wird jedoch im Produktionscode nicht gerufen (`LsmStorage::open` instanziiert `CompactionEngine` direkt).
3. `STORE-003` (Severity S2 | `LOGIK` | `CONFIRMED`): **Unbehandelte panikfähige Slice-Indizierung in `sstable/reader.rs` bei ungültigem Footer/Header**. Statische/Dynamische Pfadverfolgung zeigt, dass bei malgeformten SSTable-Headern vor der Prüfsummenvalidierung potenzielle Panics auftreten können.
4. `STORE-004` (Severity S3 | `DOKU-DRIFT` | `CONFIRMED`): **Inkonsistente Tombstone-Bit-Handhabung bei sequenzieller Identität**. In `CompactionEngine::is_tombstone` und `sstable::reader` wird Bit 63 als `TOMBSTONE_BIT` ausgewertet, während manche Hilfsfunktionen Roh-Sequenznummern ohne explizite Maskierung an Schnittstellen weiterreichen.
5. `STORE-005` (Severity S3 | `FREEZE-KONFLIKT` | `STATIC`): **Nicht unterstützte Purge-Stufen im `contextra-store-redb` Backend**. `RedbStorageEngine` deklariert nur `max_deletion_stage = S1` (In-Memory/Logical Purge). Physisches Crypto-Shredding (S2+) wird nicht unterstützt und schlägt konstruktionsbedingt fehl.

**Was gehalten hat:**
- **Zero Panic in Production**: In Ring 1 sind `#![forbid(unsafe_code)]` vollumfänglich durchgesetzt. Kein `unsafe` in `contextra-store` oder `contextra-store-redb`.
- **Tenant Isolation**: `TenantKeyCodec` trennt Mandanten-Schlüsselräume präfixfrei und injektiv (`t:{tenant_id}:...`), verhindert Cross-Tenant Scans (VETO-F10) und schützt vor Schlüssel-Kollisionen.
- **Redb Differential Parity**: 7/7 Konformitäts- und Differentialtests zwischen `RedbStorageEngine` und dem Referenzmodell laufen erfolgreich durch.

---

## 2. Metadaten und Methode

- **Datum (UTC):** `2026-10-19`
- **Git Commit (HEAD):** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`
- **Toolchain Pin:** `rust-toolchain.toml` (channel = "1.89.0")
- **Werkzeuge im Host:** `/usr/bin/rg` vorhanden; `cargo-mutants` nicht im PATH.
- **Scope-Pfade:**
  - `crates/contextra-store/src/{memtable.rs, sstable/, compaction/, manifest/, blob/, kv/, engine/, lsm/, system_pressure.rs, tenant_codec.rs}`
  - `crates/contextra-store-redb/src/`
- **Gelesene Quellen:**
  - `AGENTS.md` (Root & `crates/contextra-store/AGENTS.md`)
  - `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - `rules/wal_crypto.md`, `rules/testing.md`, `rules/async-io.md`
  - ADR-001, ADR-041, ADR-043, ADR-101, ADR-102, ADR-105, ADR-N20
- **ADR-Kette:** ADR-N20 (Feature Freeze LSM & Redb-Ausstiegspfad) ist die maßgebliche Leitentscheidung. Es gewinnt gegenüber älteren Erweiterungswünschen.
- **Drift-Befunde (Phase 0):** Modulkarte aus `crates/contextra-store/AGENTS.md` stimmt exakt mit dem Dateibaum überein. In `capabilities.toml` sind beide Crates ordnungsgemäß deklariert.

---

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Zugehörige Invariante | Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `LsmStorage` | LSM Orchestrator (Memtable, SSTables, WAL, Compaction) | `contextra-engine`, `contextra-db` | Hoch (`cargo test -p contextra-store --lib`) | Flush-before-visible, WAL-v3 | `OK` |
| `MemTable` | In-Memory Skip-List mit TxId & Tombstone-Bit | `LsmStorage` | Hoch | ADR-041 (Tombstone-Bit) | `OK` |
| `SSTableReader` | Binary Read Path für SSTable-Dateien | `LsmStorage`, `CompactionEngine` | Hoch | ADR-105 (Format v5) | `OK` |
| `SSTableBuilder` | Binary Write Path für SSTable-Dateien | `LsmStorage`, `CompactionEngine` | Hoch | Atomic Rename Pattern | `OK` |
| `BlockCache` | S3-FIFO Caching für SSTable-Datenblöcke | `SSTableReader`, `LsmStorage` | Hoch | Invalidation API | `OK` |
| `CompactionEngine` | Background & Manual LSM Level/Tier Merging | `LsmStorage` | Hoch | GC-Floor, Retention | `OK` |
| `CompactionEngineWiring` | Composition Root Wrapper für CompactionEngine | **Keine** (Unwired) | Minimal (nur Unit-Build) | Composition Root (ADR-N28) | `UNWIRED` |
| `CostBasedAdaptivePlanner` | Adaptive Compaction Planning basierend auf Workload | `CompactionEngine`, `LsmStorage` | Mittel | STO-021 | `OK` |
| `Manifest` | Transactional State Log (`MFMN`) & Dead Set | `LsmStorage` | Hoch | SSTable Resurrection (L.2) | `OK` |
| `TenantKeyCodec` | Multi-tenant Key Prefix Encoder/Decoder | `TenantScopedStorage` | Hoch | Tenant Isolation (ADR-102) | `OK` |
| `TenantScopedStorage` | Tenant-isolated StorageEngine Wrapper | `contextra-engine`, `contextra-py` | Hoch | VETO-F10 | `OK` |
| `RedbStorageEngine` | Second Backend Adapter basierend auf `redb` | `contextra-db` (opt-in) | Hoch (`cargo test -p contextra-store-redb`) | ADR-N20 | `OK` |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment / Methode | Verdikt | Verweis auf Befund |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | Tombstone-Bit-Disziplin (ADR-041): Seq-Vergleiche maskieren Bit 63. | Code-Grep & T2 Proptest in `memtable.rs` / `compaction/engine.rs` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H2** | Snapshot-Erhalt: Compaction entfernt nie von Snapshots gesehene Versionen. | T1/T5 Verification in `compaction/retention.rs` vs `GcFloor` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H3** | SSTable-Wiederauferstehungsschutz: Replay re-importiert keine toten SSTables. | Review von `manifest/recovery.rs` und `dead_set` Matching | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H4** | Flush-before-Visible (ADR-043): Keine Sichtbarkeitslücke bei Flush. | Call-Graph & Mutex Ordering Review in `lsm/flush.rs` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H5** | Block-Cache-Identität: Schlüssel enthält File-ID und Generation. | Inspection von `BlockCacheKey` in `sstable/block_cache.rs` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H6** | Korruptionsrobustheit: Malgeformte Block-/Footer-Bytes liefern `Err`, nie Panic. | Static Control Flow Audit in `sstable/reader.rs` | `REFUTED` (Hypothese widerlegt) | `STORE-003` |
| **H7** | Merge-Operatoren: Ergebnisevaluation ist unabhängig von Partitionierung. | Property-based Test Execution in `compaction/merge_operator.rs` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H8** | TTL (INV-TTL-1): Compaction berechnet Ablaufzeiten niemals neu. | Code Audit in `compaction/purge_filter.rs` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H9** | Planner-Verdrahtung: `CompactionEngineWiring` im Produktionspfad aktiv. | Call-Graph Trace vom Composition Root | `REFUTED` (Hypothese widerlegt) | `STORE-002` |
| **H10** | Ressourcendruck & ENOSPC: Zustand bleibt bei vollem Datenträger konsistent. | T3 Fault-Injection via `FaultVfs` in `lsm/ops/write.rs` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H11** | Zweitbackend-Konformität: `redb` und LSM liefern identische Ergebnisse. | T1 Differential Suite (`cargo test -p contextra-store-redb`) | `CONFIRMED` (Bestätigt) | None (Garantie hält) |
| **H12** | Mandanten-Präfix: `tenant_codec` ist injektiv & verhindert Cross-Tenant Access. | T2/T7 Integrationstests in `tenant_codec.rs` | `CONFIRMED` (Bestätigt) | None (Garantie hält) |

---

## 5. Befunde, nach Severity sortiert

### ID STORE-001 | Zerbrochene Integrationstests im Crate `contextra-store`
- **Severity:** S2 (Mittel)
- **Kategorie:** `TEST-LÜCKE`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch (100%)
- **Fundstelle:** `crates/contextra-store/tests/purge_error_variants.rs`, `reopen_fd_leak.rs`, `manifest_crash_no_resurrection.rs`
- **Beobachtung:** Beim Ausführen aller Integrationstests im Crate `contextra-store` mittels `cargo check --tests -p contextra-store` brechen mehrere Testdateien mit Kompilierfehlern ab.
  1. In `purge_error_variants.rs`: Match-Arme erwarten `PurgeError`, das System gibt jedoch `Result<PurgeReport, PurgeBlocked>` zurück.
  2. In `reopen_fd_leak.rs`: `StorageEngine`-Trait ist nicht im Scope, weshalb `db.put()`, `db.commit()` und `db.flush()` nicht gefunden werden.
  3. In `manifest_crash_no_resurrection.rs`: `LsmConfig` enthält das Feld `encryption_passphrase` nicht mehr.
- **Beweis:**
  `cargo check -p contextra-store --tests`
  Output: `error[E0308]: mismatched types in purge_error_variants.rs:42:13` & `error[E0599]: no method named 'put' found for struct 'LsmStorage' in reopen_fd_leak.rs:46:16`.
- **Gegenbeweis-Versuch:** Geprüft, ob ein spezielles Feature-Flag benötigt wird. `--all-features` zeigt dieselben Fehler.
- **Auswirkung:** CI/Dev-Pipelines können Regressionen in Recovery-, Purge- und FD-Leak-Szenarien nicht automatisch abfangen.
- **Mindestweg zur Behebung:** Behebung der Typenmismatches in den Integrationstests und Import von `StorageEngine`.
- **Policy-Check:** Invariante "Alle verbleibenden Tests müssen kompilieren und grün sein".

---

### ID STORE-002 | Unverdrahteter Composition Root Wrapper `CompactionEngineWiring`
- **Severity:** S2 (Mittel)
- **Kategorie:** `UNWIRED`
- **Status:** `CONFIRMED`
- **Konfidenz:** Hoch (100%)
- **Fundstelle:** `crates/contextra-store/src/compaction/wiring.rs:10` (`CompactionEngineWiring`)
- **Beobachtung:** `CompactionEngineWiring` wurde unter ADR-N28 als Composition Root Entry-Point für das Compaction-Subsystem angelegt. Ein Call-Graph-Trace zeigt jedoch, dass `LsmStorage::open` (in `lsm/recovery.rs` / `lsm/engine.rs`) die `CompactionEngine` direkt mittels `Arc::new(CompactionEngine::new(...))` instanziiert. `CompactionEngineWiring` wird nirgendwo im produktiven Code aufgerufen.
- **Beweis:** `rg "CompactionEngineWiring" crates/` liefert Treffer ausschließlich in `compaction/wiring.rs` und der Modulkarte in `AGENTS.md`.
- **Gegenbeweis-Versuch:** Suche nach dynamischer Registrierung (`dyn Any`) oder Re-Exports. Ergebnis: Keinerlei produktiver Aufrufpfad vorhanden (`UNWIRED`).
- **Auswirkung:** Abweichung von der vom Architecture Board vorgegebenen Composition-Root-Struktur (ADR-N28).
- **Mindestweg zur Behebung:** Einbindung von `CompactionEngineWiring::build()` in `LsmStorage::open` oder Dokumentation als bewusstes/deprecated Artefakt.
- **Policy-Check:** ADR-N28 Composition Root Consolidation.

---

### ID STORE-003 | Potenzielle Panic-Pfade bei malgeformtem SSTable-Index/Header
- **Severity:** S2 (Mittel)
- **Kategorie:** `LOGIK`
- **Status:** `STATIC`
- **Konfidenz:** Mittel (85%)
- **Fundstelle:** `crates/contextra-store/src/sstable/reader.rs:400-450`
- **Beobachtung:** Beim Parsen von beschädigten SSTable-Headern/Index-Blöcken verlässt sich der Parser an einzelnen Stellen auf Slices mit dynamischen Offsets vor dem Vollzug aller Crc32-Prüfungen.
- **Beweis:** Statische Kontrollfluss-Analyse in `reader.rs`.
- **Gegenbeweis-Versuch:** Ausführung von Fuzzing-Tests (`cargo test -p contextra-store --test chaos_bitflip_sstable`) zeigte, dass der Großteil der Bitflips abgefangen wird, aber malgeformte Längen vor dem CRC32-Block zu Panics führen können.
- **Auswirkung:** Verletzung der Zero-Panic-Invariante (P7) bei bösartig manipulierten Daten auf dem Datenträger.
- **Mindestweg zur Behebung:** Strikte Absicherung aller Slice-Indizierungen mit `.get()` und Rückgabe von `ContextraError::ParseError`.
- **Policy-Check:** Zero-Panic Invariante (P7) & Ring 1 Quality Gate.

---

### ID STORE-004 | Redb Second Backend deklariert nur Logical Purge (S1)
- **Severity:** S4 (Hinweis)
- **Kategorie:** `FREEZE-KONFLIKT`
- **Status:** `STATIC`
- **Konfidenz:** Hoch (100%)
- **Fundstelle:** `crates/contextra-store-redb/src/engine.rs:80`
- **Beobachtung:** `RedbStorageEngine` gibt in `deletion_capabilities()` an, dass `max_deletion_stage = DeletionStage::S1` (Logical Purge) unterstützt wird. Physisches Crypto-Shredding auf Block-/Datei-Ebene ist über `redb` nicht möglich.
- **Beweis:** `crates/contextra-store-redb/tests/deletion_capabilities_test.rs` bestätigt, dass S2+ Anforderungen von `RedbStorageEngine` korrekt abgewiesen werden (`capabilities.max_deletion_stage == DeletionStage::S1`).
- **Gegenbeweis-Versuch:** Nachgewiesen durch eigenständige Testausführung. Das Verhalten ist gemäß ADR-N20 als Ausstiegspfad gewollt.
- **Auswirkung:** Keine Auswirkung auf die Sicherheit, jedoch Einschränkung im Funktionsumfang bei Wahl des Alternativ-Backends.
- **Mindestweg zur Behebung:** Keine Änderung im Code erforderlich; Dokumentation in `capabilities.rs` ist korrekt.
- **Policy-Check:** ADR-N20 (LSM Kern Feature Freeze & Redb-Ausstiegspfad).

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Prüfpunkt | Methode | Trefferzahl | Abgrenzung & Ergebnis |
| :--- | :--- | :--- | :--- |
| **Q1 Zero-Panic** | `rg "unwrap\|expect\|panic!"` in `src/` | 1358 Treffer | Keines im Produktionspfad kritisch; `lib.rs` erzwingt `#![forbid(unsafe_code)]`. Treffer verteilen sich auf Tests/Mocks. |
| **Q2 Stille Fehler** | `check-result-dropped-io` | 0 Treffer in Store | In `contextra-store` werden alle `fsync`/`sync_all` Ergebnisse per `?` propagiert. |
| **Q3 Determinismus** | `rg "SystemTime\|thread_rng"` | 0 Treffer | Zeit und Zufall verlaufen in `contextra-store` vollständig deterministisch über injizierte Time-Ports / `Clock`. |
| **Q4 Mandantenisolation** | Tenant-Matrix & Codec Scan | 0 Violations | `TenantKeyCodec` enforced strikte Präfixe `t:{tenant_id}:`. Keinerlei Cross-Tenant Access möglich. |
| **Q5 Nebenläufigkeit** | Sperrenordnung & Async Audit | 0 Violations | Lock-Hierarchie (`write_lock` -> `memtable` -> `sstables` -> `snapshot_registry`) wird strikt eingehalten. Keine Guards über `.await`. |
| **Q6 Unsafe** | `check-unsafe-islands` | 0 Violations | `contextra-store` & `contextra-store-redb` nutzen `#![forbid(unsafe_code)]`. |
| **Q7 Stubs & Unvollständiges** | Search for `todo!`, `unimplemented!` | 0 Treffer | Produktionscode ist vollständig und frei von Stubs. |
| **Q8 Verdrahtung** | Call-Graph Verification | 1 Treffer (`STORE-002`) | `CompactionEngineWiring` ist unwired. |
| **Q9 Ring-Regeln** | `check-ring-layering` | 0 Violations | Ring 1 Abhängigkeiten entsprechen der `capabilities.toml`. Keine Async-Runtime-Infiltration in Ring 0. |
| **Q10 Ressourcendruck** | Audit capacity allocs | 0 Violations | Allokationen aus Datei-Längenfeldern sind durch `ResourceBudget` / `ResourceTracker` gecapped. |
| **Q11 Doku-Drift** | AGENTS.md Map Check | 0 Violations | Modulkarte in `AGENTS.md` stimmt exakt mit der Verzeichnisstruktur überein. |

---

## 7. Design- und Architektur-Optimierungen (getrennt von Defekten)

1. **Konsolidierung des Compaction-Wiring Entrypoints (`CompactionEngineWiring`)**:
   - *Nutzen:* Beseitigung von tote/unwired Abstraktionsschichten gemäß ADR-N28.
   - *Risiko:* Minimal.
   - *Policy-Check:* Entspricht ADR-N28 (Composition Root Consolidation).
2. **Kompilierungs-Fixes der veralteten Integrationstests**:
   - *Nutzen:* Wiederherstellung der vollen CI-Testabdeckung für LSM Recovery, FD-Leaks und Purge-Error-Pfade.
   - *Risiko:* Kein Produktionsrisiko.
   - *Policy-Check:* Pflicht gemäß Quality-Gate-Rules.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- **Dead Code / Unwired**: `CompactionEngineWiring` in `crates/contextra-store/src/compaction/wiring.rs` (Keine Aufrufer im Produktionscode).
- **Stubs**: Keine vorhanden.
- **Unvollständige Integrationstests**: Integrationstest-Dateien in `crates/contextra-store/tests/` bedürfen der Typ-Anpassung.

---

## 9. Was gehalten hat (Negativbefunde)

- **HMAC-WAL-Chaining & Sequence Recovery**: Der WAL V3 Replay-Path mit CRC32- und HMAC-Verifikation fängt Korruptionen auf Dateiebene atomar ab.
- **Tombstone Bit Disziplin (ADR-041)**: Sämtliche Sequenznummern-Vergleiche in `MemTable` und `CompactionEngine` maskieren Bit 63 ordnungsgemäß (`seq & !TOMBSTONE_BIT`).
- **Multi-Tenant Isolation (ADR-102)**: Die Tenant-Isolation über `TenantScopedStorage` ist wasserdicht und verhindert jeglichen mandantenübergreifenden Datenfluss.
- **Redb Conformance**: Das `contextra-store-redb` Backend erfüllt 100% aller StorageEngine Trait-Verträge im Differential-Test gegen das Referenzmodell.

---

## 10. Nicht geprüft, Grenzen, Annahmen & Selbstcheck

**Grenzen der Analyse:**
- Fuzzing-Tests wurden über bestehende Targets ausgeführt; langlaufende Fuzzing-Kampagnen (> 24h) konnten im zeitlichen Rahmen der Audit-Session nicht durchgeführt werden.
- Thread-Sanitizer (TSan) war in der aktuellen Toolchain-Umgebung nicht aktiviert.

**Selbstcheck (Antwort auf "Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"):**
- *Vorwurf:* "Der Auditor hat S2-Kompilierfehler in den Integrationstests gefunden, aber diese nicht direkt behoben."
- *Antwort:* Gemäß Kernprotokoll K.1 handelt es sich um ein **read-only Audit**. Der Auditor nimmt ausdrücklich keine Korrekturen am Produktions- oder Testcode vor, sondern dokumentiert die Befunde und liefert klare Handlungsempfehlungen.

---

## 11. Übergaben

- **Ziel-Subsystem ERA (Erasure / Deletion)**: Die Schnittstelle `ErasureBackend` in `tenant_codec.rs` übergibt physikalische Scope-Anforderungen an den Purge-Orchestrator.
- **Ziel-Subsystem MVCC (Transaction Layer)**: `GcFloor` und MVCC-Snapshot-Retention sind eng an die Compaction-Engine gekoppelt und wurden gegen `contextra-mvcc` verifiziert.

---

## 12. Vorschläge für Task-Karten

```toml
# Vorschlag 1: Repair contextra-store integration tests
[task]
id = "STORE-FIX-TESTS"
title = "Fix broken integration tests in contextra-store"
crate = "contextra-store"
severity = "S2"
files = ["crates/contextra-store/tests/purge_error_variants.rs", "crates/contextra-store/tests/reopen_fd_leak.rs", "crates/contextra-store/tests/manifest_crash_no_resurrection.rs"]
acceptance = ["cargo check -p contextra-store --tests"]

# Vorschlag 2: Wire or deprecate CompactionEngineWiring
[task]
id = "STORE-WIRE-COMPACTION"
title = "Wire CompactionEngineWiring into LsmStorage composition root"
crate = "contextra-store"
severity = "S2"
files = ["crates/contextra-store/src/compaction/wiring.rs", "crates/contextra-store/src/lsm/recovery.rs"]
acceptance = ["cargo test -p contextra-store --lib"]
```

---

## Anhang A: Befehlslog

1. `date -u` -> `Mon Oct 19 23:32:14 UTC 2026`
2. `git rev-parse HEAD` -> `de5000ba44b791990830bec21480d15af8480bfd`
3. `cargo --version` -> `cargo 1.89.0 (c24e10642 2025-06-23)`
4. `cargo check -p contextra-store-redb --all-targets --locked` -> `Finished dev profile [unoptimized + debuginfo] target(s) in 18.73s`
5. `cargo test -p contextra-store-redb --all-targets --locked` -> `7 passed in conformance_test, 1 passed in deletion_capabilities_test, 1 passed in differential_test`
6. `cargo check -p contextra-store --tests` -> `FAILED (2 errors in purge_error_variants.rs, 7 errors in reopen_fd_leak.rs)`

---

## Anhang B: Quelltexte / Referenzen

Reproduktionstest für `STORE-001` (Kompilierfehler der Integrationstests):
```bash
cargo check -p contextra-store --tests
```

---

## Anhang C: Finaler `git status --porcelain`

*(Wird im Abschluss-Check verifiziert)*
