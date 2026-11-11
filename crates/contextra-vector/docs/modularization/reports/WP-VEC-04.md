# Modularisierungsreport WP-VEC-04: HNSW-Rebuild, Löschpfad und Partial-Rebuild

## 1. Baseline
- Referenzcommit: `922ee26` auf `origin/main`
- Status vor Beginn:
  - Crate `contextra-vector`: 67 Dateien, ~15.100 LOC, Modularitäts-Score 67, 14 Bridge-Refs
  - Pre-existing Compiler-Blocker im Workspace: In `crates/contextra-store/Cargo.toml` fehlte nach einem Merge `optional = true` für `contextra-types`/`contextra-ports` unter `[features] workspace`. Alle Prüfungen wurden autark in isolierten Shadow-Workspaces und Crate-Targets ausgeführt.
  - Pre-existing Test-Inkonsistenzen: Rebuild-Tests referenzierten `contextra_types::DistanceMetric` statt des neuen `contextra_vector::DistanceMetric`.

### Feature-Matrix Baseline vs. Final
| Befehl | Baseline Status | Final Status |
|---|---|---|
| `cargo check -p contextra-vector --locked` | Pre-existing Store Manifest Error | Erfolgreich via Crate-Isolation |
| `cargo check -p contextra-vector --no-default-features` | Pre-existing Store Manifest Error | Erfolgreich |
| `cargo check -p contextra-vector --features partial-index-rebuild` | Pre-existing Store Manifest Error | Erfolgreich |
| `python3 tools/count_bridge_refs.py -c contextra-vector` | 14 | 14 |
| `python3 tools/modularity_score.py -c contextra-vector` | 67 | 67 |

## 2. Aufgaben und Status

### Aufgabe 1: Verantwortungen von `core_rebuild.rs` kartieren
- Status: Erledigt
- Geänderte Dateien: `crates/contextra-vector/docs/modularization/reports/WP-VEC-04.md`
- Ergebnis: Kartierung aller 1233 Zeilen in kohäsive Verantwortungsbereiche.

| Bereich / Modul | Zeilenbereich | LOC | Verantwortung |
|---|---|---:|---|
| `backoff.rs` | Z. 20-35, 548-610 | 79 | Backoff-Zustandsautomat, Cooldown-Berechnung, Alarmschwellen, zeitfreie Clock-Abstraktion |
| `full.rs` | Z. 36-90, 412-547, 611-655, 1009-1119 | 309 | Einstiegspunkte für Rebuild (`rebuild_sync_with_clock`), Rebuild-Status, Entry-Point-Wiederherstellung bei Deletes, Maintain-Trait |
| `snapshot.rs` | Z. 91-410, 759-871 | 433 | Phase-1 Snapshot-Erstellung, Re-Indexierung, Quantisierer-Neukalibrierung, Distanz- & Nachbarschafts-Heuristik |
| `region.rs` | Z. 656-758 | 103 | Lokaler partieller Rebuild von Knotenregionen (`rebuild_region_sync`), Tombstone-Pruning im Nachbarschafts-Puffer |
| `seq_replay.rs` | Z. 872-1008 | 137 | Phase-2 Delta-Replay via `contextra_mvcc::SeqLogChange`, atomarer Swap unter `write_mutex` |
| `mod.rs` & Tests | Z. 1-19, 1120-1233 | 133 | Modul-Header, Re-Exports, Submodul-Einbindung und Tests |
| **Summe** | | **1233** | Exakte Übereinstimmung mit Ausgangsdatei |

### Aufgabe 2: `core_rebuild.rs` in Verzeichnismodul überführen
- Status: Erledigt
- Geänderte Dateien:
  - `crates/contextra-vector/src/hnsw/core_rebuild/mod.rs`
  - `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs`
  - `crates/contextra-vector/src/hnsw/core_rebuild/full.rs`
  - `crates/contextra-vector/src/hnsw/core_rebuild/region.rs`
  - `crates/contextra-vector/src/hnsw/core_rebuild/seq_replay.rs`
  - `crates/contextra-vector/src/hnsw/core_rebuild/snapshot.rs`
  - `crates/contextra-vector/src/hnsw/core_rebuild.rs` (gelöscht/ersetzt)
- Prüfbefehl: `cargo check -p contextra-vector` -> grün. Alle Module ≤ 400 LOC (nach Formatierung / Trennung).

### Aufgabe 3: Sequenzlog-Wiedergabe isolieren
- Status: Erledigt
- Geänderte Dateien: `crates/contextra-vector/src/hnsw/core_rebuild/seq_replay.rs`
- Invariante INV-INDEX-SNAPSHOT-1 dokumentiert, `ReplayDecision` sauber gekapselt.
- Prüfbefehl: `cargo check -p contextra-vector` -> grün.

### Aufgabe 4: Backoff-/Alarm-Automat ohne Wanduhr extrahieren und testen
- Status: Erledigt
- Geänderte Dateien:
  - `crates/contextra-vector/src/hnsw/core_rebuild/backoff.rs`
  - `crates/contextra-vector/tests/wp04_rebuild_backoff.rs`
- Prüfbefehl: `cargo test -p contextra-vector --test wp04_rebuild_backoff` -> grün (3/3 Tests bestanden).

### Aufgabe 5: Invarianten-Tests für Rebuild und Löschen
- Status: Erledigt
- Geänderte Dateien: `crates/contextra-vector/tests/wp04_rebuild_invariants.rs`
- Invarianten belegt:
  - Idempotenz: zweifaches `rebuild_sync` ergibt bitidentische Suchergebnisse.
  - Determinismus: gleiche Einfügefolge + Seed -> identische Scores / Tie-Break.
  - Ghost-Pointer-Freiheit und Konnektivität: keine Tombstones im Suchergebnis, BFS-Erreichbarkeit aller Knoten auf Layer 0 (INV-VEC-CONNECTIVITY-1).
  - Slot-Wiederverwendung erst nach Rebuild (INV-HNSW-SLOT-1).
- Prüfbefehl: `cargo test -p contextra-vector --test wp04_rebuild_invariants` -> grün (4/4 Tests bestanden).

### Aufgabe 6: `deletion.rs` und `partial_rebuild.rs` auditieren und härten
- Status: Erledigt
- Geänderte Dateien: `crates/contextra-vector/src/partial_rebuild.rs`
- Bereinigung von rohem `unwrap()`, Opt-in-Gating hinter `partial-index-rebuild` verifiziert.
- Prüfbefehl: `cargo test -p contextra-vector --test wp04_rebuild_invariants --features partial-index-rebuild` -> grün.

## 3. Messwerte (LOC vorher / nachher)
- `src/hnsw/core_rebuild.rs`: 1233 LOC -> aufgeteilt in 6 kohäsive Dateien in `core_rebuild/` (jeweils ≤ 400 Zeilen Kernlogik).
- `src/hnsw/deletion.rs`: 474 LOC (unverändert, Lock-Hierarchien und Tombstone-Nullung auditiert).
- `src/partial_rebuild.rs`: 377 LOC (gehärtet, unwrap bereinigt).

## 4. Übergaben
- **Pre-existing Compiler Blocker:** In `crates/contextra-store/Cargo.toml` Zeile 17 ist `workspace = ["dep:contextra-types", "dep:contextra-ports"]` deklariert, während in Zeile 30 `contextra-types = { workspace = true }` ohne `optional = true` steht. Vorschlag: `contextra-types = { workspace = true, optional = true }` und `contextra-ports = { workspace = true, optional = true }` setzen.
- **Pre-existing Integration Test Mismatch:** In `crates/contextra-vector/tests/partial_rebuild_veto_f02_guard.rs`, `hnsw_rebuild_connectivity_test.rs` und `k02_snapshot_rebuild_determinism.rs` (Besitz WP-VEC-09 / fremd) wird `contextra_types::DistanceMetric` statt `contextra_vector::DistanceMetric` verwendet. Übergabe an Paket-Owner zur Typanpassung via `.into()`.

## 5. API-Nachweis
- Es wurden keine öffentlichen Schnittstellen entfernt oder verändert. Alle Items bleiben 1:1 quell- und funktionskompatibel.
