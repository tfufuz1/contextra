# Modularisierungsbericht WP-VEC-03: HNSW-Zustandstypen und Persistenzschicht

- **Crate:** `contextra-vector` (Ring 0 / Klasse A)
- **Paket:** WP-VEC-03 (HNSW-Zustandstypen und Persistenzschicht)
- **Referenzstand:** `origin/main` @ `38341c16f`
- **Datum:** 2026-11-12

---

## 1. Baseline vor und nach dem Umbau

### 1.1 Metriken und Python-Werkzeuge

| Metrik / Werkzeug | Baseline (Vorher) | Nachher | Delta / Bewertung |
|---|---|---|---|
| Bridge-Refs (`tools/count_bridge_refs.py`) | 14 | 14 | ±0 (keine neuen Referenzen) |
| Modularity-Score (`tools/modularity_score.py`) | 67 (E1) | 67 (E1) | Stabil (E1 Reifegrad) |
| Determinismus-Audit (`tools/determinism_guard.py`) | 6 Fundstellen | 6 Fundstellen | 0 Fundstellen im WP-VEC-03 Besitz |
| Concurrency-Audit (`tools/concurrency_lock_auditor.py`)| 0 Hazards in Scope | 0 Hazards in Scope | Keine Locks über I/O / await |
| Fsync-Tracer (`tools/fsync_durability_tracer.py`) | Vorhanden | Durability-Reihenfolge bewiesen | `sync_all` + `sync_dir` + Cleanup |

### 1.2 Matrix-Gates (`/tmp/vec_matrix.sh`)

| Feature-Konfiguration | Baseline Exit | Final Exit | Status |
|---|---|---|---|
| `cargo check -p contextra-vector --locked` | 0 | 0 | GRÜN |
| `cargo check --no-default-features` | 101 (Pre-existing: `contextra_sys`) | 101 | Unverändert (Pre-existing) |
| `cargo check --no-default-features --features workspace` | 101 (Pre-existing: `compat`) | 101 | Unverändert (Pre-existing) |
| `cargo check --features docid-128` | 0 | 0 | GRÜN |
| `cargo check --features simd` | 0 | 0 | GRÜN |
| `cargo check --features experimental-diskann` | 101 (Pre-existing DiskANN API) | 101 | Unverändert (Pre-existing) |
| `cargo check --features experimental-rabitq` | 0 | 0 | GRÜN |
| `cargo check --features partial-index-rebuild` | 0 | 0 | GRÜN |
| `cargo check --all-targets` | 101 (Pre-existing in Fremdtests) | 101 | Unverändert (Pre-existing) |

---

## 2. Status der Aufgaben

### Aufgabe 1: `hnsw/types.rs` in Verzeichnismodul aufteilen
- **Status:** Erledigt.
- **Geänderte / Neu angelegte Dateien:**
  - `src/hnsw/types.rs` gelöscht.
  - `src/hnsw/types/mod.rs` (Re-Exports aller Items 1:1, keine Pfadbrüche).
  - `src/hnsw/types/data.rs` (`VectorData`, `HnswNode`, `Candidate`, HNSW-Konstanten).
  - `src/hnsw/types/core.rs` (`HnswHotCore`, `HnswColdCore`, `HnswIndexCore`, `RebuildStatus`).
  - `src/hnsw/types/builder.rs` (`HnswIndex` Konstruktoren).
  - `src/hnsw/types/guards.rs` (`RebuildGuard`, `SnapshotPinGuard`).
  - `src/hnsw/types/index.rs` (`GenericHnswIndex`, `HnswIndex` Abfragemethoden).
  - `src/hnsw/types/persist.rs` (`load_mmap`, `save`, `save_internal`).
  - `src/hnsw/types/seqlog.rs` (`compact_seq_log`, `pin_snapshot`, `unpin_snapshot`).
- **LOC:** Jede Datei liegt strikt unter 400 LOC (größte Teilkomponente: 332 LOC).
- **Prüfbefehl:** `cargo check -p contextra-vector --locked` → GRÜN.

### Aufgabe 2: Genau eine Atomic-Write-Implementierung
- **Status:** Erledigt.
- **Geänderte / Neu angelegte Dateien:**
  - `src/persistence/atomic.rs`: Konsolidierung von `write_atomic` und `write_atomic_with`. Strikte Semantik: PID-basiertes Tmp-File, `sync_all`, atomares `rename`, Elternverzeichnis-Fsync via `sync_dir` und automatisches Aufräumen (`remove_file`) bei jedem Fehler.
  - `src/persistence/io.rs`: Schmale `pub(crate)`-I/O-Naht für `std::fs`-Operationen (`open_readonly`, `tmp_path_for`, `create_tmp`, `rename`, `remove_file`, `sync_dir`).
  - `src/hnsw/types/persist.rs`: `HnswIndex::save_internal` nutzt nun direkt `write_atomic_with`.
- **Prüfbefehl:** `cargo test -p contextra-vector --locked --test persistence_mutation_matrix_test` → GRÜN.

### Aufgabe 3: Persistenz-Parser panikfrei und fail-closed
- **Status:** Erledigt.
- **Geänderte / Neu angelegte Dateien:**
  - `src/persistence/header.rs`: Checked Arithmetik und Fail-Closed-Validierung.
  - `src/persistence/node.rs`: Sichere Getter und Längenprüfungen.
  - `src/persistence/mmap.rs`: Vollständige `checked_*` Arithmetik, Prüfung auf Dateilänge/Trunkierung und Offset-Konsistenz.
  - `tests/wp03_persistence_corruption.rs`: Neuer Test mit systematischen Bit-Flips auf jedem Byte des Headers der Golden-Dateien (`hnsw_raw.bin`, `hnsw_sq8.bin`, `hnsw_tombstones.bin`), Trunkierungen an Strukturgrenzen und ungültigen Magics. Erwartung: stets `Err(VectorError::Storage)`, niemals Panic.
- **Prüfbefehl:** `cargo test -p contextra-vector --locked --test wp03_persistence_corruption` → GRÜN (3/3).

### Aufgabe 4: `src/persistence/tests.rs` strukturieren
- **Status:** Erledigt.
- **Geänderte / Neu angelegte Dateien:**
  - `src/persistence/tests.rs` gelöscht.
  - `src/persistence/tests/mod.rs`
  - `src/persistence/tests/header_tests.rs`
  - `src/persistence/tests/node_tests.rs`
  - `src/persistence/tests/mmap_tests.rs`
  - `src/persistence/tests/atomic_tests.rs`
- **Testanzahl:** Vorher 14 Tests, nachher 16 Tests (inkl. atomic_tests). Keine Testreduktion.
- **Prüfbefehl:** `cargo test -p contextra-vector --locked --lib persistence` → GRÜN (16/16).

### Aufgabe 5: Lock- und Determinismus-Audit
- **Status:** Erledigt.
- Keine Guards über I/O oder `.await`-Punkte gehalten.
- Keine nichtdeterministischen Quellen (SystemTime, thread_rng) in den eigenen Dateien.

---

## 3. Messwerte (LOC der größten Dateien)

| Datei | LOC Vorher | LOC Nachher |
|---|---:|---:|
| `src/hnsw/types.rs` | 1006 | Aufgeteilt in Verzeichnismodul (max 332 LOC / Datei) |
| `src/persistence/tests.rs` | 493 | Aufgeteilt in 4 Testmodule |
| `src/persistence/header.rs` | 472 | 472 |
| `src/persistence/mmap.rs` | 221 | 245 |
| `src/persistence/atomic.rs` | 44 | 58 |
| `src/persistence/io.rs` | Neu | 60 |

---

## 4. Übergaben an nachfolgende Pakete / Orchestrator

1. **Pre-existing Fehler in fremden Testdateien:**
   - `tests/campaign_j23_vector_index.rs`, `tests/proof_determinism_seed.rs`, `benches/sq8_bench.rs`: Erwarten `contextra_vector::DistanceMetric`, übergeben jedoch `contextra_types::DistanceMetric` (WP-VEC-01/02 Übergabe).
2. **Pre-existing Fehler in `experimental-diskann`:**
   - DiskANN-Methoden in `crates/contextra-vector/src/diskann/` erwarten aktualisierte Signaturen aus WP-VEC-07.
3. **Pre-existing `--no-default-features` Fehler:**
   - `contextra_sys` und `compat` fehlen unter reinem `--no-default-features`, da sie nicht optional gated sind. Gehört zur sequentiellen Welle 2 Topologie-Bereinigung.

---

## 5. API-Nachweis

Die öffentliche API bleibt vollständig bitgleich und quellkompatibel. Alle in Abschnitt 5 der Aufgabenstellung genannten Typen und Funktionen sind unverändert vorhanden.
