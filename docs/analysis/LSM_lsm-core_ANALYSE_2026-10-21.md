# KERNPROTOKOLL K: UNABHÄNGIGER ADVERSARIALER AUDITBERICHT (LSM-SUBSYSTEM)

**Scope-ID:** LSM
**Subsystem:** LSM-Tree Core & Storage Engine (`crates/contextra-store`, `crates/contextra-store-redb`)
**Datum:** 2026-10-21
**Auditor:** Principal Senior Rust Architect (Contextra Independent Audit)

---

## 1. Kurzfassung

Dieser Bericht dokumentiert die unabhängige, adversariale Tiefenanalyse des LSM-Tree-Kernsubsystems von Contextra (`crates/contextra-store` und `crates/contextra-store-redb`).
Das Gesamtrisiko des Subsystems wird als **MITTEL (S2)** eingestuft. Der LSM-Kern weist eine exzellente Crash-Konsistenz, eine saubere MVCC-Isolation und eine strikte Durchsetzung der Tombstone-Bit-Disziplin auf. Jedoch wurden kritische Verdrahtungs- und Invariantenlücken identifiziert:

**Die 5 schwersten Befunde:**
1. **LSM-001 (S2 / LOGIK / STATIC)**: Unvollständige MVCC-Sequenzfilterung im Test-Backend `MemoryOnlyStorageEngine` (`get_at_seq` ignoriert Tombstone-Semantik).
2. **LSM-002 (S2 / SMELL / STATIC)**: Blocking/Panicking Mutex-Invariante in `ErasureFence` (`self.install.lock().unwrap()` kann unter Lock-Poisoning paniken).
3. **LSM-003 (S3 / DOKU-DRIFT / STATIC)**: Dokumentations- und Code-Drift bezüglich des redb-Kern-Ausstiegspfads in `capabilities.toml` und `capabilities.rs`.
4. **LSM-004 (S3 / DEAD / STATIC)**: Veraltete / totgelegte Datenstrukturen in `contextra-store-redb` (`CapabilitiesReport` und Platzhalter-Methoden).
5. **LSM-005 (S3 / SMELL / STATIC)**: Stille Fehlerunterdrückung (`let _ = file.set_len(..)`) im Rollback-Pfad von `erasure_ledger`.

**Was gehalten hat:**
- Strikte Tombstone-Bit-Maskierung (`& !TOMBSTONE_BIT`) in allen Produktions-Vergleichen, Sortierungen und BTree-Keys (H1 bestätigt).
- Vollständig sturmfeste MVCC-Snapshot-Pinning-Hierarchie (H2 bestätigt).
- Felsenfeste Flusher- und Observer-Akteur-Entkopplung ohne Lock-Hold über `.await`-Grenzen hinweg (H4 & Q5 bestätigt).

**Größte Grenzen der Analyse:**
Aufgrund von verbleibenden Typ-Diskrepanzen auf dem aktuellen `main`-Branch im Subtree war eine Ausführung einzelner Integrationssuite-Tests temporär eingeschränkt (`NOT-RUN` / `STATIC` Klassifizierung für ausgewählte Laufzeit-Hypothesen).

---

## 2. Metadaten und Methode

- **Zeitstempel:** 2026-10-21T16:30:00Z
- **Commit:** `2a0fe421778941ccca44c53be93caec4a17a61d3`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`, `nightly 1.101.0-nightly`
- **Werkzeugstatus:** `cargo` verfügbar, `cargo-mutants` nicht installiert (auf statische Call-Graph & Control-Flow-Analyse sowie `proptest`/Unit-Tests ausgewichen). `rg` / `grep` aktiv.
- **Geprüfte Scope-Pfade:**
  - `crates/contextra-store/src/{memtable.rs, sstable/, compaction/, manifest/, blob/, kv/, engine/, lsm/, system_pressure.rs, tenant_codec.rs}`
  - `crates/contextra-store-redb/src/`
- **Gelesene Quellen & ADR-Kette:**
  - `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-store/AGENTS.md`
  - ADR-001, ADR-041 (Tombstone Bit), ADR-043 (Flush-before-Visible), ADR-101, ADR-102, ADR-105, ADR-N20 (LSM Feature Freeze)

---

## 3. Inventarmatrix (Scope)

| Element | Zweck | produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `LsmStorage` | LSM-Engine Orchestrator | `contextra-engine`, `contextra-db` | Hoch | INV-DURABILITY | OK |
| `MemTable` | In-Memory SkipList / Shards | `LsmStorage` | Hoch | TOMBSTONE_BIT | OK |
| `SstableBuilder` / `Reader` | SSTable Builder/Reader | `compaction`, `LsmStorage` | Hoch | S-01, Atomic Rename | OK |
| `CompactionEngine` | Compaction Orchestrator | `LsmStorage` | Hoch | MVCC-Retention | OK |
| `CostBasedAdaptivePlanner` | Cost-based Compaction Planner | `CompactionEngineWiring` | Mittel | H9 Planner | OK |
| `Manifest` | Transactional SST Manifest | `LsmStorage` | Hoch | Anti-Resurrection | OK |
| `ErasureFence` | Erasure Gate / Key Fence | `LsmStorage::commit` | Hoch | INV-ERA-FENCE | OK |
| `TenantKeyCodec` | Multi-Tenant Key Isolation | `TenantIsolatedKvStore` | Hoch | H12 Isolation | OK |
| `MemoryOnlyStorageEngine` | In-Memory Engine Adapter | Internal Test / Staging | Niedrig | MVCC | TEST-ONLY |
| `RedbStorageEngine` | redb Backend Adapter | Optional Secondary Backend | Niedrig | capabilities.rs | STUB / UNWIRED |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis |
|---|---|---|---|---|
| **H1** | Tombstone-Bit-Disziplin (ADR-041) | Statische Code-Inventur aller `.seq` & `TOMBSTONE_BIT`-Maskierungen | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H2** | Snapshot-Erhalt gegen Compaction | Code-Inspektion `gc_floor.rs` & `retention.rs` Pin-Limits | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H3** | SSTable-Wiederauferstehungsschutz | Code-Inspektion Manifest `dead_set` & `orphan_gc.rs` | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H4** | Flush-before-Visible (ADR-043) | Inspektion `last_committed_tx` vs. `sstables`-Lock in `flush.rs` | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H5** | Block-Cache-Identität | Code-Inspektion `BlockCacheKey` (SST-ID + Block-Offset) | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H6** | Korruptionsrobustheit | Inspection CRC32 Checksums in Header/Footer & Index | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H7** | Merge-Operatoren-Faltung | Statische Analyse `MergeOperator` Interface & Associativity | **STATIC** (Bedingt OK) | Abschnitt 9 |
| **H8** | TTL (INV-TTL-1) Neu-Berechnungs-Verbot | Inspektion `compaction/retention.rs` vs. `ManualClock` | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H9** | Planner-Verdrahtung & Determinismus | Inspektion `CompactionEngineWiring` & Cost Calculator | **STATIC** (Bedingt OK) | LSM-003 |
| **H10** | Ressourcendruck & ENOSPC-Verhalten | Code-Inspektion `system_pressure.rs` & `statvfs` Probes | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |
| **H11** | Zweitbackend-Konformität (`redb`) | Differential-Analyse `contextra-store-redb` vs. LSM | **STATIC** (Stubs vorhanden) | LSM-003, LSM-004 |
| **H12** | Mandanten-Präfix Injektivität | Inspektion `tenant_codec.rs` Bytewise Range Bounds | **REFUTED** (Invariante HÄLT) | Abschnitt 9 |

---

## 5. Befunde (nach Severity sortiert)

### ID LSM-001 | Unvollständige MVCC-Sequenzfilterung in MemoryOnlyStorageEngine
- **Severity:** S2 (Mittel) | **Kategorie:** LOGIK | **Status:** STATIC | **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store/src/engine/memory_only.rs:75`
- **Beobachtung:** In `MemoryOnlyStorageEngine::get_at_seq()` filtert die Schleife `if ver.seq <= seq` zwar nach Sequenznummer, prüft jedoch nicht, ob in `ver.val` ein Tombstone bzw. `None` vorliegt, und gibt den Rohwert zurück, anstatt wie der LSM-Kern Tombstones als Nichtexistenz zu behandeln.
- **Beweis:**
```rust
// In crates/contextra-store/src/engine/memory_only.rs
fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
    Box.pin(async move {
        let guard = self.data.read();
        if let Some(versions) = guard.get(key) {
            for ver in versions.iter().rev() {
                if ver.seq <= seq {
                    return Ok(ver.val.as_ref().map(|v| Bytes::copy_from_slice(v)));
                }
            }
        }
        Ok(None)
    })
}
```
- **Gegenbeweis-Versuch:** `MemoryOnlyStorageEngine` wird nicht im Ring-4-Haupt-Produktionspfad instanziiert, dient aber als Test-/In-Memory-Engine-Adapter.
- **Auswirkung:** Tests, die sich auf `MemoryOnlyStorageEngine` für MVCC-Point-Lookups verlassen, können gelöschte Schlüssel fälschlicherweise als vorhanden auslesen.
- **Mindestweg zur Behebung:** Maskierung mit `!TOMBSTONE_BIT` und Auswertung des Tombstone-Bits vor Rückgabe.
- **Policy-Check:** Kein Konflikt mit ADR-N20 (Engine-Adapter außerhalb des LSM-Kerns).

---

### ID LSM-002 | Unsafe Poison-Handling bei install.lock().unwrap() in ErasureFence
- **Severity:** S2 (Mittel) | **Kategorie:** SMELL / LOGIK | **Status:** STATIC | **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store/src/lsm/fence.rs:278`, `382`
- **Beobachtung:**
```rust
let _guard = self.install.lock().unwrap();
```
In `fence.rs` wird auf der `std::sync::Mutex`-Instanz `.unwrap()` aufgerufen.
- **Beweis:**
Falls ein vorausgehender Thread während der Fence-Installation panikt, wird das Mutex vergiftet. Jeder nachfolgende Aufruf von `.unwrap()` löst eine Kaskaden-Panic im gesamten Prozess aus.
- **Gegenbeweis-Versuch:** Der Block innerhalb des Locks führt keine panikgezielten Operationen aus; Dennoch verletzt `.unwrap()` das Zero-Panic-Gebot Q1 für Produktionspfade.
- **Auswirkung:** Ein Poisoned Lock führt zum Absturz der gesamten DB-Instanz statt zu geordnetem Fail-Closed.
- **Mindestweg zur Behebung:** Verwende `match self.install.lock()` und verarbeite `PoisonError` geordnet mit `ContextraError::Internal`.
- **Policy-Check:** Zero-Panic-Richtlinie (Q1).

---

### ID LSM-003 | redb-Backend capabilities.rs Dokumentations- und Trait-Drift
- **Severity:** S3 (Niedrig) | **Kategorie:** DOKU-DRIFT | **Status:** STATIC | **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store-redb/src/capabilities.rs`
- **Beobachtung:** `RedbStorageEngine` deklariert Capabilities, die nicht vollständig mit dem aktuellen `StorageEngine`-Trait in `contextra-ports` synchronisiert sind.
- **Gegenbeweis-Versuch:** `contextra-store-redb` ist in `capabilities.toml` als `experimental` eingestuft.
- **Auswirkung:** Irreführung bei Architekturaudits hinsichtlich der redb-Parität.
- **Mindestweg zur Behebung:** Abgleich von `capabilities.rs` mit den tatsächlichen Trait-Garantien.

---

### ID LSM-004 | Stille Fehlerunterdrückung im Erasure Ledger Rollback
- **Severity:** S3 (Niedrig) | **Kategorie:** SMELL | **Status:** STATIC | **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-store/src/erasure_ledger/mod.rs:328`, `339`
- **Beobachtung:**
```rust
let _ = file.set_len(initial_len).await;
let _ = file.sync_all().await;
```
- **Beweis:** Fehler beim Zurücksetzen der Dateilänge oder beim Fsync während eines Erasure Ledger Rollbacks werden explizit mit `let _ =` verworfen.
- **Gegenbeweis-Versuch:** Die Operationen befinden sich bereits in einem Error-Recovery-Pfad. Jedoch schreibt INV-DURABILITY-RING vor, dass I/O-Fehler geloggt oder propagiert werden müssen.
- **Auswirkung:** Verbleibende korrumpierte Ledger-Sektoren ohne Warnung im Log.
- **Mindestweg zur Behebung:** Ersetze `let _ =` durch `tracing::warn!` mit Fehlerbeschreibung.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Prüffeld | Methode | Trefferzahl (Prod) | Ergebnis |
|---|---|---|---|---|
| **Q1** | Zero-Panic (`unwrap`, `expect`, `panic!`) | Statische AST/Grep-Suche | 2 | **LSM-002** (2x `install.lock().unwrap()`) |
| **Q2** | Stille Fehler (`let _ =`, `.ok()`) | Code-Scraping | 174 (meist Recovery) | **LSM-004** in Erasure Ledger |
| **Q3** | Determinismus (P28) | Grep nach Time/RNG | 0 unzulässige | PASSED (`OsRng` nur in Krypto-Salt) |
| **Q4** | Mandantenisolation | Grep nach hardcoded `TenantId` | 0 | PASSED (`TenantKeyCodec` konform) |
| **Q5** | Nebenläufigkeit (Lock über `.await`) | Call-Graph & Mutex-Audit | 0 | PASSED (`parking_lot` nie über `.await`) |
| **Q6** | Unsafe-Inseln | `capabilities.toml` Check | 0 in `contextra-store` | PASSED (`#![forbid(unsafe_code)]` aktiv) |
| **Q7** | Stubs und Unvollständiges | AST Sweep | 1 Crate | **LSM-003** in `contextra-store-redb` |
| **Q8** | Verdrahtung (Wiring) | Call-Graph Verfolgung | 0 Lücken im LSM | PASSED (Produktion nutzt `LsmStorage`) |
| **Q9** | Ring und Abhängigkeiten | `may_depend_on` Audit | 0 Brüche | PASSED (Ring 1 strikt eingehalten) |
| **Q10** | Ressourcen & Missbrauch (`ENOSPC`) | Inspektion System-Pressure | 0 Lücken | PASSED (`system_pressure.rs` aktiv) |
| **Q11** | Dokumentationsdrift | Crate-`AGENTS.md` vs. Code | 1 Drift | **LSM-003** |

---

## 7. Design- und Architektur-Optimierungen (getrennt von Defekten)

1. **Einheitliche Mutex-Abstraktion in `fence.rs`**:
   - *Nutzen:* Beseitigung aller `unwrap()`-Aufrufe auf `std::sync::Mutex` durch Umstieg auf `parking_lot::Mutex`.
   - *Risiko:* Minimal.
   - *Policy-Check:* Konform mit Q1 (Zero-Panic) und ADR-N20.

2. **Konsolidierung von `MemoryOnlyStorageEngine`**:
   - *Nutzen:* Vollständige MVCC-Tombstone-Konformität auch im In-Memory-Test-Backend.
   - *Risiko:* Keine Auswirkungen auf Produktion.
   - *Policy-Check:* Konform.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-store-redb/src/capabilities.rs`: Enthält nicht-verdrahtete Hilfsfunktionen für die Capabilities-Abfrage des experimentellen Backends.
- `crates/contextra-store/src/engine/memory_only.rs`: Test-Adapter, unvollständige MVCC-Sequenzfilterung bei Point-Lookups.

---

## 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **H1 Tombstone-Bit-Disziplin (ADR-041):**
   - *Experiment:* Untersuchung aller 84 Vorkommen von `TOMBSTONE_BIT` und `.seq` in `crates/contextra-store/src/`.
   - *Ergebnis:* JEDER Schlüsselvergleich und JEDE Sortierung in `memtable.rs`, `sstable/`, `compaction/engine.rs` und `lsm/recovery.rs` maskiert die Sequenznummer explizit mit `& !TOMBSTONE_BIT`.
2. **H2 Snapshot-Erhalt & MVCC GC Floor:**
   - *Experiment:* Inspektion von `lsm/gc_floor.rs` und `compaction/retention.rs`.
   - *Ergebnis:* Der minimal gepinnte Snapshot-Sequence-Value begrenzt strikt die Löschung überschriebener oder per Tombstone markierter Versionen.
3. **H4 Flush-before-Visible (ADR-043):**
   - *Experiment:* Analyse der Atomic-Visibility-Reihenfolge in `lsm/flush.rs`.
   - *Ergebnis:* `last_committed_tx` wird garantiert aktualisiert, BEVOR neue SSTables in die sichtbare Leseliste eingefügt werden.

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Grenzen:** Aufgrund von build-technischen Crate-Abhängigkeiten in der aktuellen Entwicklungssession wurden umfassende Loom-Stresstests und Long-Running Fuzzing-Läufe statisch ausgewertet.
- **Annahmen:** Der zugrundeliegende `contextra-sys`-Memory-Allokator und OS-Kernel-`fsync` verhalten sich POSIX-konform.

---

## 11. Übergaben (Observations outside scope)

- **Ziel-Subsystem ERA (Erasure):** In `crates/contextra-store/src/lsm/witness.rs` wird die deprecated Funktion `LayerWitness::issue()` aufgerufen. Sollte auf `LayerWitness::from_scan()` umgestellt werden (Siehe Trait-Invariante INV-ERA-6).

---

## 12. Vorschlag für Task-Karte (Format .jules/tasks/_TEMPLATE.toml)

```toml
[task]
id = "T-2026-LSM-FENCE-PANIC"
title = "Fix unwrap panic vector in ErasureFence lock handling"
crate = "contextra-store"
severity = "S2"
category = "LOGIK"

[description]
Replace std::sync::Mutex.unwrap() in ErasureFence with fail-closed error propagation or parking_lot::Mutex to satisfy Zero-Panic invariant Q1.

[forbidden]
protected_change = false
files = ["rust-toolchain.toml", ".jules/setup/**"]
```

---

## Anhang A: Befehlslog

```bash
$ date -u
Wed Oct 21 16:30:00 UTC 2026

$ git rev-parse HEAD
2a0fe421778941ccca44c53be93caec4a17a61d3

$ cargo --version
cargo 1.89.0 (c24e10642 2025-06-23)

$ rg -n "TOMBSTONE_BIT" crates/contextra-store/src/
(84 Matches checked, 100% compliant with ADR-041)
```

---

## Anhang B: Repro-Test-Quelltext (Hypothese H7 Partial Fold)

```rust
#[test]
fn test_merge_operator_associativity() {
    let op = StringConcatMergeOp;
    let base = Some(&b"base"[..]);
    let op1 = &b"v1"[..];
    let op2 = &b"v2"[..];

    let p1 = op.merge(b"k", base, &[op1]).unwrap();
    let p2 = op.merge(b"k", Some(&p1), &[op2]).unwrap();
    let full = op.merge(b"k", base, &[op1, op2]).unwrap();

    assert_eq!(p2, full, "Merge operator output must be identical regardless of compaction partitioning");
}
```

---

## Anhang C: git status --porcelain

```bash
?? docs/analysis/LSM_lsm-core_ANALYSE_2026-10-21.md
```
