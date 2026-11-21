# contextra-store — Implementierungsspezifikation

**Ebene:** E1 (Kern, R2)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
LSM-Speicher: Memtable, SST, Manifest, WAL (`lsm/`), KV-Familien (`kv`, `KvStore`), Segmente (`SegmentStore`, ohne eigene Verschlüsselung), Kompaktion mit Löschfilter für Records toter Seals (REQ-SCR-3), `fsync`, `budget`, `stats` (SPEC 15, 3.1, Kapitel 5). Ziel-Umfang: 14.000 Zeilen (SPEC 3.1). Ist: 35.383 Zeilen laut Zielspalte (Altbestand), das größte Crate des Workspace.
Abhängigkeiten nach Anhang E: `ports`, `ids`, `env`, `sys`, `durable-fs`.

## 2. Vertrag
Implementiert (SPEC 3.4): `KvStore` und `SegmentStore`. Ist: `KvStore` ist als Trait in `contextra-ports` (Datei kv.rs) unter dem Namen `KvPrefixStore` definiert (siehe PLAN von `contextra-ports`, O-1). `SegmentStore` fehlt im Ports-Crate.
Scrub-Ziel: `ScrubTarget` für den Store (SPEC 4.4, 3.4). Ist: nicht implementiert.

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`, `env`, `sys`, `durable-fs`.
<<<<<<< HEAD
**Ist (Cargo.toml):** `contextra-ids`, `contextra-env`, `contextra-sys`, `contextra-durable-fs` (erlaubt), dazu **`contextra-erasure`, `contextra-crypto`, `contextra-version`** (**verboten**, R2 und ADR-32, REM-27), sowie `tokio`, `quick_cache`, `lru`, `memmap2`, `parking_lot` u. a.
**Ist-Abweichung:** `contextra-crypto`, `contextra-erasure`, `contextra-version` müssen entfernt werden (REM-27, SPEC 3.4: Schlüsselvernichtung über `KeyDestroyer`, Scrub über `ScrubTarget`). `contextra-ports` fehlt in Ist, obwohl Anhang E es verlangt.
=======
**Ist (Cargo.toml):** `contextra-ids`, `contextra-env`, `contextra-sys`, `contextra-durable-fs` (erlaubt), dazu **`contextra-erasure`, `contextra-crypto`, `contextra-mvcc`** (**verboten**, R2 und ADR-32, REM-27), sowie `tokio`, `quick_cache`, `lru`, `memmap2`, `parking_lot` u. a.
**Ist-Abweichung:** `contextra-crypto`, `contextra-erasure`, `contextra-mvcc` müssen entfernt werden (REM-27, SPEC 3.4: Schlüsselvernichtung über `KeyDestroyer`, Scrub über `ScrubTarget`). `contextra-ports` fehlt in Ist, obwohl Anhang E es verlangt.
>>>>>>> a9b569ed6 (Shell-Commit)
**Features:** Ist `workspace`, `test-hooks`, `inspect`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`, `legacy-wal-key`, `docid-128`. Soll (SPEC 15): keine Features außer den in Kapitel 15 genannten. `workspace` verletzt CMP-10 und G-11. `fault-injection` widerspricht REM-13 (Fault-Injection gehört nach testkit).

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/lsm/*` | LSM-Kern: Commit, Engine, Flush, Group-Commit, Fence, Locks, Erasure-Modul | Soll `lsm/` (SPEC 15). `src/lsm/erasure.rs` verstößt gegen R2: prüfen und lösen |
| `src/compaction/*` | Kompaktion, adaptive Planung, Retention, Merge-Operator | Soll `compaction` (SPEC 15). Die adaptiven Teile sind nicht in SPEC 15 genannt, offen |
| `src/io_seam/*` | E/A-Naht, `real_fs` | zu prüfen |
| `src/key_source.rs` | Schlüsselquelle | **Ist-Abweichung**: Schlüssel sind Sache von `crypto` (R4) |
| `src/budget.rs`, `src/fsync.rs`, `src/error.rs`, `src/lib.rs` | Budget, Fsync, Fehler | Soll `budget`, `fsync` |
| `src/compaction/tests*`, `src/compaction/tests.rs` | Tests im Produktcode | Verbleib offen |

Soll-Module ohne Ist-Entsprechung: `kv` (KvStore-Familie), `segments` (`SegmentStore` mit Kopf nach 5.8). Ist: `segments` fehlt als eigenes Modul.

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-02 | in SPEC 15 genannt, Inhalt in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| REM-05 | dito | siehe SPEC 12 |
| REM-08 | dito | siehe SPEC 12 |
| REM-26 | dito | siehe SPEC 12 |
<<<<<<< HEAD
| REM-27 | Abhängigkeiten zu `crypto`, `erasure`, `mvcc` | `grep -rln "contextra_crypto\|contextra_erasure\|contextra_version" crates/contextra-store` leer |
=======
| REM-27 | Abhängigkeiten zu `crypto`, `erasure`, `mvcc` | `grep -rln "contextra_crypto\|contextra_erasure\|contextra_mvcc" crates/contextra-store` leer |
>>>>>>> a9b569ed6 (Shell-Commit)
| CMP-4 | Feature und Schlüssel-Kompatibilität `legacy-wal-key`, `workspace` (SPEC 12.5 prüfen) | `grep -rn "compat\|workspace" crates/contextra-store/Cargo.toml` leer |
| REM-14 | ANCHOR-Dateien (51) | `grep -rl "ANCHOR\[" crates/contextra-store` leer |

## 6. Umbau
1. Abhängigkeiten zu `crypto`, `erasure`, `mvcc` lösen (REM-27). Schlüsselvernichtung über den Port `KeyDestroyer`, Scrub über `ScrubTarget` (ADR-32).
2. `key_source.rs` und `src/lsm/erasure.rs` nach `crypto` bzw. `erasure` verlagern oder über Ports anbinden.
3. Feature `workspace` entfernen (CMP-10), `fault-injection` nach `testkit` (REM-13).
4. `unwrap`/`expect`/`panic!` (1.247 Treffer) nach R6 abbauen, schrittweise je Modul.
5. Benchmark G-STORE als Gate vor und nach dem Umbau.

**Offene Punkte (Entscheidung nötig):**
- **O-1** Zuordnung der adaptiven Kompaktion (`compaction/adaptive/*`), nicht in SPEC 15.
- **O-2** `quick_cache`, `lru`, `tokio` (Abhängigkeiten): nicht in SPEC 15. `tokio` ist Teil von Punkt 10 der CLAUDE.md, `deny.toml` führt `contextra-store` als Wrapper.
- **O-3** Inhalt von REM-02, REM-05, REM-08, REM-26 aus SPEC 12 bestätigen.
- **O-4** Reihenfolge und Zuschnitt des Umbaus (größtes Crate, eigene Sitzung je Schritt, CLAUDE.md §8).

## 7. Tests
- G-STORE-Benchmark, Wiederanlauf, Kompaktionsfilter (SPEC 15).
- Ist-Befund: Tests in `src/compaction/tests/*`, Zählung nicht erfolgt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | kein `unsafe` (erlaubt wäre nur in `sys`/`simd`), `forbid` in 2 Dateien | eingehalten (Ausnahme laut tests/unsafe_islands.rs prüfen) |
| G-03 | verbotene Kanten zu `crypto`, `erasure`, `mvcc` | verletzt (REM-27) |
| G-07 | Import `contextra_types` (1 Datei) | verletzt (REM-30) |
| G-09 | 51 Dateien mit ANCHOR/FILE-CONTEXT | verletzt (REM-14) |
| G-11 | Feature `workspace` | verletzt (CMP-10) |
| G-12 | 7 Treffer `Instant::now`/`SystemTime::now` | verletzt, Fundstellen auf `Clock` umstellen |
| G-13 | 1.247 Treffer | Abbau nach R6 |
