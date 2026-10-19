# Contextra Subsystem-Analyse: `contextra-vector` (VEC)

**Datum:** 2026-10-19
**Auditor:** Principal Senior Rust Architect (Adversarial Audit Mode)
**Scope-ID:** VEC
**Geprüfter Commit:** `de5000ba44b791990830bec21480d15af8480bfd`

---

## 1. Kurzfassung

- **Gesamtrisiko:** Mittel bis Hoch (S0 Befund in `search_at` MVCC isolation, S2 Goverment/ADR Governance Konflikt, S3 API/Doku-Drift).
- **Fünf schwerste Befunde:**
  1. `VEC-001` (S0, CONFIRMED): `HnswIndex::search_at` verletzt MVCC-Snapshot-Isolation nach Knoten-Löschungen (`hnsw_snapshot_delete_test` Fehlschlag).
  2. `VEC-002` (S2, STATIC): Governance-Konflikt zwischen ADR-097 (Nachbarschaftsreparatur) und `VETOES.md` (VETO-F02 Tombstone-Pruning-Verbot) sowie `AGENTS.md`.
  3. `VEC-003` (S2, STATIC): Dokumentationsdrift in `crates/contextra-vector/AGENTS.md` (veraltete flache Modulkarte vs. strukturierter Subdirectory-Dateibaum).
  4. `VEC-004` (S3, STATIC): Unverdrahtete Typen/Funktionen (`RaBitQQuantizer`, `compute_gamma_edge_budget`, `NaiveReferenceIndex`) im produktiven Pfad (`UNWIRED`).
  5. `VEC-005` (S3, CONFIRMED): Test-Kompilierungsfehler durch API-Drift in `tests/sq8_bias_calibration.rs`.
- **Was gehalten hat:**
  - `#![forbid(unsafe_code)]` strikt eingehalten (0% unsafe in `contextra-vector`, SIMD/Mmap sauber nach Ring-0 ausgelagert).
  - Deterministische HNSW-Graphkonstruktion und Seeding (`HnswConfig::rng_seed`).
  - Robuste Distanz-Validierung gegen `NaN` / `Inf` bei Insert & Search (`nan_validation_policy`).
  - Strikte Mmap-/Header-Grenzen-Validierung gegen OOB/Overflow.
- **Größte Grenzen der Analyse:** Loom-Prüfungen auf HNSW-Lock-Freiheit wurden nicht direkt im Loom-Harness ausgeführt (Stresstest via Threads durchgeführt); Performance/Latenz-Aussagen wurden ohne vollen Benchmark-Lauf gemäß `BENCHMARK_PROTOCOL.md` nicht bewertet.

---

## 2. Metadaten und Methode

- **Zeitpunkt:** Mon Oct 19 23:35:49 UTC 2026
- **Commit:** `de5000ba44b791990830bec21480d15af8480bfd`
- **Toolchain:** `rustc 1.89.0`, `cargo 1.89.0` (`nightly-2026-10-19` vorhanden)
- **Werkzeugstatus:** `cargo` (OK), `cargo +nightly` (OK), `ripgrep` (OK), `cargo-fuzz` (Nicht installiert/NOT-RUN), `cargo-mutants` (Nicht installiert/NOT-RUN), `cargo xtask` (OK).
- **Scope-Pfade:** `crates/contextra-vector/src/{hnsw/*, diskann/*, acorn/*, distance.rs, candidate_stream.rs, compute_pool.rs, partial_rebuild.rs, persistence/*, quantize/*, quantize_rabitq.rs}`
- **Gelesene Quellen:** `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`, `crates/contextra-vector/AGENTS.md`, ADR-002, ADR-013, ADR-037, ADR-061, ADR-063, ADR-070, ADR-076, ADR-083, ADR-097.
- **ADR-Ketten & Governance-Auflösung:**
  - ADR-097 hebt das bisherige Verbot von Graph-Rewiring auf und erlaubt begrenzte Nachbarschaftsreparatur bei Löschung. `VETOES.md` (VETO-F02) und `crates/contextra-vector/AGENTS.md` wurden nicht synchronisiert. Gemäß K.2 Schritt 4 gilt ADR-097 (Status Accepted), der Governance-Widerspruch wird als `VEC-002` (GOV) geführt.
  - DiskANN ist opt-in hinter Feature-Flag `experimental-diskann` (ADR-013, ADR-076).

---

## 3. Inventarmatrix

| Element | Zweck (1 Zeile) | produktive Aufrufer | Testabdeckung | zugehörige Invariante | Status |
|---|---|---|---|---|---|
| `HnswIndex` | Haupt-HNSW-Index im RAM/Mmap | `contextra-db` (`Collection`), `contextra-router` | Hoch (`tests/`) | Ring 0 VectorIndex | `OK` |
| `HnswConfig` | Konfiguration für HNSW-Parameter | `contextra-db`, `contextra-router` | Hoch | Validierung M > 0 | `OK` |
| `DiskAnnIndex` | Out-of-Core DiskANN Index | Opt-In hinter `experimental-diskann` | Hoch (`tests/diskann_*`) | Opt-In Feature | `OK` |
| `DiskAnnConfig` | Konfiguration für DiskANN | Opt-In hinter `experimental-diskann` | Hoch | Feature Gate | `OK` |
| `ScalarQuantizer` | SQ8 Skalarquantisierungs-Engine | `HnswIndex`, `DiskAnnIndex` | Hoch | Codebook Stability | `OK` |
| `RaBitQQuantizer` | 1-Bit RaBitQ Quantisierungs-Engine | Keine (`UNWIRED`) | `tests/rabitq_*` | - | `UNWIRED` |
| `MmapIndex` | Mmap-Lade- & Persistenz-Reader | `HnswIndex::load_mmap` | Hoch | Header Bounds Check | `OK` |
| `HnswHeader` | Binär-Header Format (v1/v2) | `persistence::mmap`, `persistence::node` | Hoch | Magic 0x484E5357 | `OK` |
| `compute_distance` | Hardware-Dispatch Distanzfunktion | `HnswIndex`, `DiskAnnIndex` | Hoch | Zero Unsafe in Vector | `OK` |
| `VectorCandidateStream` | Async Iterator/Stream über Treffer | `contextra-db` (Multi-Step Query) | Mittel | Bounded Max Depth | `OK` |
| `ComputePool` | Bounded Threadpool für Vektor-Compute | `DiskAnnIndex` | Hoch | Concurrency Limits | `OK` |
| `TraversalTracker` | Tracking für partielle Graph-Rebuilds | `HnswIndex` | Hoch | Ring-Buffer Size Limit | `OK` |
| `should_trigger_partial_rebuild` | Entscheidet Partielle Rebuilds | `HnswIndex::check_and_trigger_partial_rebuild` | Hoch | ADR-097 / Thresholds | `OK` |
| `compute_gamma_edge_budget` | Gamma Edge Augmentation Budget | Keine (`UNWIRED`) | `tests/acorn_*` | ACORN Filtering | `UNWIRED` |
| `NaiveReferenceIndex` | ACORN Naive Brute-Force Referenz | Keine (`UNWIRED`) | `tests/acorn_*` | Differential Ref | `TEST-ONLY` |

---

## 4. Hypothesen-Verdikte

| H-ID | Aussage | Experiment | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| H1 | Recall-Baseline stabil vor/nach Löschung | `cargo test -p contextra-vector --test recall --test recall_vs_brute_force` | `CONFIRMED` (OK) | Abs. 9 |
| H2 | Determinismus bei Einfügung und Seed | `cargo test -p contextra-vector --test determinism_test` | `CONFIRMED` (OK) | Abs. 9 |
| H3 | Concurrency & 2-Phasen-Lock deadlockfrei | `cargo test -p contextra-vector --test deleted_nodes_lock_contention` | `CONFIRMED` (OK) | Abs. 9 |
| H4 | Erreichbarkeit & Isolation nach Löschung (`search_at`) | `cargo test -p contextra-vector --test hnsw_snapshot_delete_test` | `REFUTED` (Defekt) | `VEC-001` |
| H5 | Distanz-Randfälle & NaN/Inf Handling | `cargo test -p contextra-vector --test nan_validation_policy --test distance_determinism` | `CONFIRMED` (OK) | Abs. 9 |
| H6 | Persistenz-Format, Mmap & Bounds-Check | `cargo test -p contextra-vector --test mmap_malformed_data_test --test mmap_toctou_test` | `CONFIRMED` (OK) | Abs. 9 |
| H7 | Gefilterte Suche & Unautorisierte Kandidaten | `cargo test -p contextra-vector --test filtered_selectivity_recall` | `CONFIRMED` (OK) | Abs. 9 |
| H8 | DiskANN-Pfad Konsistenz & Pending-Flush | `cargo test -p contextra-vector --features experimental-diskann --test diskann_recall_test` | `CONFIRMED` (OK) | Abs. 9 |
| H9 | Ressourcenlimits (ef, k, Dimension) | Static Analysis & Input Guard Audit | `CONFIRMED` (OK) | Abs. 9 |
| H10 | Async in Ring 0 (P26) Purity Check | `cargo xtask check-ring0-async-purity` | `CONFIRMED` (OK) | Abs. 9 |
| H11 | `partial_rebuild.rs` Aktivität & Governance | Static Trace & `tests/partial_rebuild_veto_f02_guard.rs` | `STATIC` (Gedeckt durch ADR-097) | `VEC-002` |
| H12 | SQ8 / RabiTQ Quantisierungs-Limits | `cargo test -p contextra-vector --test sq8_rerank_recall --test rabitq_calibration_recall` | `CONFIRMED` (OK) | Abs. 9 |

---

## 5. Befunde

### ID VEC-001 | `HnswIndex::search_at` verletzt Snapshot-Isolation nach Knoten-Löschung
- **Severity:** S0 (Kritisch)
- **Kategorie:** LOGIK / SICHERHEIT
- **Status:** CONFIRMED
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-vector/src/hnsw/vector_index_impl.rs` (`search_at`) und `crates/contextra-vector/src/hnsw/core_search.rs`.
- **Beobachtung:** Beim Ausführen des Regressionstests `cargo test -p contextra-vector --test hnsw_snapshot_delete_test` schlagen zwei Tests fehl (`test_search_at_includes_node_deleted_after_snapshot` und `test_search_at_self_reference_not_deleted_before_snapshot`). Knoten, die zum Zeitpunkt des Snapshots (`seq_no`) existierten, aber danach gelöscht wurden, werden bei `search_at` nicht im Ergebnis zurückgeliefert.
- **Beweis:**
  ```text
  ---- test_search_at_includes_node_deleted_after_snapshot stdout ----
  thread 'test_search_at_includes_node_deleted_after_snapshot' panicked at crates/contextra-vector/tests/hnsw_snapshot_delete_test.rs:45:5:
  search_at(seq=1) MUSS doc_id=1 enthalten
  ```
- **Gegenbeweis-Versuch:** Geprüft, ob `search_at` in der Produktion aufgerufen wird. Aufrufer existieren in `candidate_stream.rs` (`VectorCandidateStream`), wenn `seq_no` gesetzt ist. `search_at` filtert gelöschte Knoten anhand der aktuellen Tombstone-Map statt anhand des Commit-Logs zum gegebenen `seq_no`.
- **Auswirkung:** Verletzung der MVCC Snapshot-Isolation. Lese-Transaktionen gegen historische Snapshots können fälschlicherweise Daten als gelöscht betrachten, obwohl sie zum Snapshot-Zeitpunkt noch gültig waren.
- **Mindestweg zur Behebung:** In `search_at` muss die Sichtbarkeit eines Knotens anhand der Sequenznummer der Löschung (`deleted_at_seq`) gegen `seq_no` ausgewertet werden, anstatt physisch gelöschte/tombstoned Knoten universell auszublenden.
- **Policy-Check:** Betrifft MVCC Isolation (ADR-N06 / Spec §4).

---

### ID VEC-002 | Governance-Widerspruch zwischen ADR-097 und VETOES.md (VETO-F02)
- **Severity:** S2 (Mittel)
- **Kategorie:** GOV / DOKU-DRIFT
- **Status:** STATIC
- **Konfidenz:** Hoch
- **Fundstelle:** `docs/decisions/ADR-097-hnsw-nachbarschaftsreparatur-bei-loeschung-ersetzt-reines-tombstone-pruning.md`, `VETOES.md` (VETO-F02), `crates/contextra-vector/AGENTS.md`.
- **Beobachtung:** ADR-097 ("Accepted") erlaubt explizit begrenzte Nachbarschaftsreparatur beim Löschen von HNSW-Knoten und ersetzt reines Tombstone-Pruning. In `VETOES.md` wird VETO-F02 ("Graph-Rewiring bei Löschung ist verboten") weiterhin als aktives Veto aufgeführt.
- **Beweis:** `VETOES.md` verbietet Graph-Rewiring, während `partial_rebuild.rs` und `deletion.rs` Rewiring gemäß ADR-097 ausführen.
- **Gegenbeweis-Versuch:** `git log` zeigt, dass ADR-097 jünger ist und Accepted-Status hat. Nach K.2 Schritt 4 hat ADR-097 Vorrang, aber das nicht aktualisierte `VETOES.md` erzeugt Verwirrung und Veto-Validation Warnings.
- **Auswirkung:** Entwickler und automatisierte Gates erhalten widersprüchliche Vorgaben bzgl. Graph-Rewiring.
- **Mindestweg zur Behebung:** `VETOES.md` und `crates/contextra-vector/AGENTS.md` aktualisieren, um die durch ADR-097 geregelte Ausnahme zu spiegeln.
- **Policy-Check:** Governance Rules (ADR-060 / K.2 Schritt 4).

---

### ID VEC-003 | Modulkarte in `crates/contextra-vector/AGENTS.md` veraltet
- **Severity:** S3 (Niedrig)
- **Kategorie:** DOKU-DRIFT
- **Status:** STATIC
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-vector/AGENTS.md` (Abschnitt 2 Modul-Karte).
- **Beobachtung:** Die Modulkarte listet eine flache Struktur (`hnsw.rs`, `diskann.rs`, `persistence.rs`, `quantize.rs`), während der tatsächliche Dateibaum Unterverzeichnisse nutzt (`src/hnsw/`, `src/diskann/`, `src/persistence/`, `src/quantize/`, `src/acorn/`, `partial_rebuild.rs`, `quantize_rabitq.rs`).
- **Beweis:** Diff zwischen `crates/contextra-vector/AGENTS.md` und `ls -la crates/contextra-vector/src/`.
- **Gegenbeweis-Versuch:** Verifiziert, dass die Wurzelmodule alle in Unterverzeichnisse refactored wurden.
- **Auswirkung:** Fehlleitung von Audits und Entwicklern bezüglich der Modulstruktur.
- **Mindestweg zur Behebung:** Aktualisierung der Modulkarte in `crates/contextra-vector/AGENTS.md`.
- **Policy-Check:** Standard Doku-Drift Regel.

---

### ID VEC-004 | Unverdrahtete Typen und Module im Crate-Export (`UNWIRED`)
- **Severity:** S3 (Niedrig)
- **Kategorie:** UNVOLLSTÄNDIG / DEAD
- **Status:** STATIC
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-vector/src/quantize_rabitq.rs` (`RaBitQQuantizer`), `crates/contextra-vector/src/acorn/gamma_augmentation.rs` (`compute_gamma_edge_budget`), `crates/contextra-vector/src/acorn/naive_reference.rs` (`NaiveReferenceIndex`).
- **Beobachtung:** `RaBitQQuantizer` und `compute_gamma_edge_budget` sind `pub` deklariert, besitzen jedoch keinen einzigen produktiven Aufrufer in `contextra-vector`, `contextra-db`, `contextra-engine` oder `contextra` Facade. Sie werden ausschließlich in Unittests/Benchmarks verwendet.
- **Beweis:** `rg "RaBitQQuantizer"` und `rg "compute_gamma_edge_budget"` liefert Aufrufe ausschließlich in `crates/contextra-vector/tests/` und `benches/`.
- **Gegenbeweis-Versuch:** Suche über gesamten Workspace durchgeführt; kein Aufruf in produktiven Instanziierungspfaden oder Configuration Enums.
- **Auswirkung:** Toter Code im Binärartefakt / Crate-Export, der Maintenance-Overhead erzeugt.
- **Mindestweg zur Behebung:** Entweder in der Facade/Config verdrahten oder mit `#[cfg(feature = "experimental-quant")]` bzw. als `pub(crate)` markieren.
- **Policy-Check:** P26 / ADR-N16 Clean-Up.

---

### ID VEC-005 | API-Drift in Test `tests/sq8_bias_calibration.rs` führt zu Kompilierungsfehler
- **Severity:** S3 (Niedrig)
- **Kategorie:** DOKU-DRIFT / TEST-LÜCKE
- **Status:** CONFIRMED
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-vector/tests/sq8_bias_calibration.rs`.
- **Beobachtung:** Aufruf von `cargo test -p contextra-vector --test sq8_bias_calibration` schlägt mit Kompilierungsfehlern fehl. Der Test verwendet veraltete Signaturen von `ScalarQuantizer::train` (erwartet `dimension: usize`, Test übergibt `DistanceMetric`) und `Sq8Bias::calibrate_distance` (existiert nicht mehr, heißt `calibrate`).
- **Beweis:**
  ```text
  error[E0308]: arguments to this function are incorrect
  --> crates/contextra-vector/tests/sq8_bias_calibration.rs:28:21
  let quantizer = ScalarQuantizer::train(&training_data, DistanceMetric::Euclidean)
  expected `usize`, found `DistanceMetric`
  ```
- **Gegenbeweis-Versuch:** Es wurde verifiziert, dass `ScalarQuantizer` in `src/quantize/mod.rs` refactored wurde, aber dieser Test-Driver nicht angepasst wurde.
- **Auswirkung:** Ein Invarianten-Regressionstest für SQ8-Bias ist im CI-Lauf deaktiviert/defekt.
- **Mindestweg zur Behebung:** Anpassung der Methodenaufrufe in `tests/sq8_bias_calibration.rs` an die aktuelle `ScalarQuantizer`- und `Sq8Bias`-API.
- **Policy-Check:** Test Quality Rules.

---

### ID VEC-006 | DiskANN unterstützt konstruktionsbedingt kein `search_at` (Snapshot-Reads)
- **Severity:** S3 (Niedrig)
- **Kategorie:** UNVOLLSTÄNDIG / DOKU-DRIFT
- **Status:** STATIC
- **Konfidenz:** Hoch
- **Fundstelle:** `crates/contextra-vector/src/diskann/filtered.rs` (Zeile 18-20).
- **Beobachtung:** `DiskAnnIndex` implementiert das `VectorIndex`-Trait, gibt jedoch bei Aufruf von `search_at` explizit einen Fehler zurück: `Err(ContextraError::Storage("DiskANN does not support snapshot reads"))`.
- **Beweis:**
  ```rust
  async fn search_at(&self, _query: &[f32], _k: usize, _seq_no: u64) -> Result<Vec<ScoredDocument>> {
      Err(ContextraError::Storage("DiskANN does not support snapshot reads".into()))
  }
  ```
- **Gegenbeweis-Versuch:** `test_diskann_search_at_capability_unsupported` in `tests/diskann_search_filtered.rs` bestätigt dieses fail-closed Verhalten.
- **Auswirkung:** Aufrufer, die `VectorIndex::search_at` polymorph über `DiskAnnIndex` aufrufen, erhalten zur Laufzeit einen Storage-Fehler.
- **Mindestweg zur Behebung:** Dokumentation in `VectorIndex`-Trait und `DiskAnnIndex` klarstellen; Dokumentations-Hinweis für Multi-Step Queries ergänzen.
- **Policy-Check:** ADR-013 / ADR-083.

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Thema | Methode | Trefferzahl | Ergebnis |
|---|---|---|---|---|
| Q1 | Zero-Panic | `cargo run -p xtask -- panic-inventory` + `rg` | 0 in Prod | Pass |
| Q2 | Stille Fehler | `rg "let _ ="`, `unwrap_or_default` Audit | 0 kritisch | Pass |
| Q3 | Determinismus | `rg` nach `Instant::now`, `thread_rng` | Invarianten-konform | Pass (Instant::now nur für Rebuild-Timeout/Metrics) |
| Q4 | Mandantenisolation | Audit `TenantId` Nutzung | Delegiert an DB/Engine | Pass (VectorIndex speichert DocIds u64/u128) |
| Q5 | Nebenläufigkeit | Lock-Umfang Audit in `hnsw` & `diskann` | Deadlock-frei | Pass (2-Phase Lock sauber; RwLocks nie über await) |
| Q6 | Unsafe | `rg "unsafe"` in `src/` | 0 | Pass (`src/lib.rs` erzwingt `#![forbid(unsafe_code)]`) |
| Q7 | Stubs & Unvollständiges | `rg "TODO\|FIXME\|allow\(dead_code\)"` | 1 Treffer | Pass (`diskann/build.rs` `allow(dead_code)` für Helper) |
| Q8 | Verdrahtung | Trait Implementations & Trace | 2 unwired | Warnung (RaBitQ & compute_gamma_edge_budget, `VEC-004`) |
| Q9 | Ring & Abhängigkeiten | `cargo tree -p contextra-vector` | Ring 0 konform | Pass (Keine verbotenen L1/L2 High-Level Crates) |
| Q10 | Ressourcen & Missbrauch | Allocation Bounds Check (`Vec::with_capacity`) | Geschützt | Pass (`k`, `M`, `dimension` streng validiert) |
| Q11 | Dokumentationsdrift | `AGENTS.md` vs Code Drift Check | 2 Befunde | Warnung (`VEC-002`, `VEC-003`) |

---

## 7. Design- und Architektur-Optimierungen

1. **Konsolidierung des Quantisierungs-Abstraktions-Layers:**
   - **Nutzen:** `RaBitQQuantizer` und `ScalarQuantizer` unter einem gemeinsamen `Quantizer`-Trait vereinen.
   - **Risiko:** Gering.
   - **Policy-Check:** Konform mit Single-Node Architektur & Ring 0 Prinzipien.

2. **Vereinheitlichung von `MmapIndex` und `DiskAnnIndex` Mmap-Zugriffen:**
   - **Nutzen:** Vermeidung doppelter Mmap-Parsing-Logik für Header & Offsets.
   - **Risiko:** Niedrig.
   - **Policy-Check:** Konform mit ADR-017 / ADR-N03.

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `crates/contextra-vector/src/quantize_rabitq.rs`: `RaBitQQuantizer` (`UNWIRED`)
- `crates/contextra-vector/src/acorn/gamma_augmentation.rs`: `compute_gamma_edge_budget` (`UNWIRED`)
- `crates/contextra-vector/src/acorn/naive_reference.rs`: `NaiveReferenceIndex` (`TEST-ONLY`)
- `crates/contextra-vector/src/diskann/build.rs`: `#[allow(dead_code)]` bei Hilfsfunktion `recover_pending_delta_sync`

---

## 9. Was gehalten hat (Negativbefunde)

1. **Strikte `#![forbid(unsafe_code)]`-Durchsetzung (H6/Q6):**
   - **Experiment:** Quellcode-Scan nach `unsafe` Schlüsselwörtern über das gesamte `crates/contextra-vector/src/` Verzeichnis.
   - **Ergebnis:** Null Treffer. Sämtliche SIMD-Intrinsics sind in `contextra-simd` gekapselt, Mmap-Operationen in `contextra-sys`.

2. **Deterministisches Graph-Building & Seeding (H2):**
   - **Experiment:** `cargo test -p contextra-vector --test determinism_test`
   - **Ergebnis:** Identische Seeding-Konfigurationen erzeugen exakt bitgleiche Graph-Topologien (`test_hnsw_determinism_explicit_rng_same_topology` bestanden).

3. **`NaN`/`Inf` Vektor-Grenzwert-Schutz (H5):**
   - **Experiment:** `cargo test -p contextra-vector --test nan_validation_policy`
   - **Ergebnis:** Vektoren mit `NaN` oder `Inf` werden bei `insert` und `search` sofort abgefangen und als `ContextraError::InvalidInput` zurückgewiesen, ohne den Indexzustand zu korrumpieren.

4. **DiskANN Corruption & Fallback Resilience (H8):**
   - **Experiment:** `cargo test -p contextra-vector --features experimental-diskann --test diskann_corruption_fallback_test`
   - **Ergebnis:** Manipulierte oder beschädigte DiskANN Index-Dateien werden erkannt und lösen kontrollierte Fallback-Mechanismen oder fail-fast Fehler aus (`test_corrupted_diskann_use_hnsw_fallback_policy` bestanden).

---

## 10. Nicht geprüft, Grenzen, Annahmen

- **Fuzzing mit `cargo-fuzz`:** Auf der aktuellen Testumgebung nicht ausgeführt (`cargo-fuzz` Werkzeug nicht installiert/NOT-RUN).
- **Mutation-Testing mit `cargo-mutants`:** Werkzeug nicht installiert (NOT-RUN).
- **Loom Concurrency Model Checking:** HNSW feingranulars Locking wurde per Thread-Stresstest geprüft (`deleted_nodes_lock_contention`), aber nicht im formalen Loom-Model-Checker durchgerunnt.
- **Benchmark-Messungen:** Keines der Latenz- oder Durchsatz-Ergebnisse wurde als formaler Benchmark deklariert (T10), da keine Messungen gemäß `BENCHMARK_PROTOCOL.md` stattfanden.

---

## 11. Übergaben

- **Übergabe an ERA (Erasure Engine):** `HnswIndex::issue_residue_witness` prüft Residuen für gelöschte Knoten. Die Interaktion mit `ErasureLedger` und physical zeroization sollte im Subsystem ERA vertieft werden.
- **Übergabe an RNK (Ranking & Fusion):** Die Dichte- und Distanzwerte aus `candidate_stream.rs` fließen in `contextra-rank` (RRF / Score Calibration). Betrifft RNK Subsystem.

---

## 12. Vorschläge für Task-Karten

```toml
# .jules/tasks/T-2026-VEC-001.toml
[task]
id = "T-2026-VEC-001"
title = "Fix HnswIndex::search_at MVCC Snapshot Isolation for Deleted Nodes"
component = "contextra-vector"
severity = "S0"
budget_files = 3
budget_lines = 150
evidence_required = true

[acceptance]
command = "cargo test -p contextra-vector --test hnsw_snapshot_delete_test"
```

```toml
# .jules/tasks/T-2026-VEC-002.toml
[task]
id = "T-2026-VEC-002"
title = "Align VETOES.md and AGENTS.md with ADR-097 Neighborhood Repair"
component = "governance"
severity = "S2"
budget_files = 2
budget_lines = 30
evidence_required = true

[acceptance]
command = "cargo xtask check-vetoes"
```

```toml
# .jules/tasks/T-2026-VEC-005.toml
[task]
id = "T-2026-VEC-005"
title = "Fix API Drift in tests/sq8_bias_calibration.rs"
component = "contextra-vector"
severity = "S3"
budget_files = 1
budget_lines = 40
evidence_required = true

[acceptance]
command = "cargo test -p contextra-vector --test sq8_bias_calibration"
```

---

## Anhang A: Befehlslog

| Befehl | Exit-Code | Kernausgabe / Anmerkung |
|---|---|---|
| `date -u && git rev-parse HEAD && git status --porcelain` | 0 | `de5000ba44b791990830bec21480d15af8480bfd` |
| `cargo test -p contextra-vector --lib` | 0 | 68 passed; 0 failed |
| `cargo test -p contextra-vector --test recall --test determinism_test --test recall_vs_brute_force` | 0 | 8 passed; 0 failed |
| `cargo test -p contextra-vector --test hnsw_snapshot_delete_test` | 101 | 2 failed (`search_at` MVCC failure) |
| `cargo test -p contextra-vector --test nan_validation_policy --test distance_determinism --test simd_numerical_audit` | 0 | 11 passed; 0 failed |
| `cargo test -p contextra-vector --features experimental-diskann --test diskann_recall_test --test diskann_fault_injection_test --test diskann_corruption_fallback_test` | 0 | 5 passed; 0 failed |
| `cargo test -p contextra-vector --features experimental-diskann --test sq8_bias_calibration` | 101 | Kompilierungsfehler (API-Drift) |

---

## Anhang B: Quelltexte der Repro-Tests

Für `VEC-001` (Snapshot Isolation Fehlschlag):
```rust
// Auszug aus crates/contextra-vector/tests/hnsw_snapshot_delete_test.rs
#[tokio::test]
async fn test_search_at_includes_node_deleted_after_snapshot() {
    let mut config = HnswConfig::default();
    config.dimension = 4;
    let index = Arc::new(HnswIndex::try_new(config).unwrap());

    // Insert doc 1 at sequence 1
    index.insert(TxId::new(1), DocId(1), &[1.0, 0.0, 0.0, 0.0]).await.unwrap();
    index.commit(TxId::new(1)).await.unwrap();
    let seq1 = index.current_sequence_number();

    // Delete doc 1 at sequence 2
    index.delete(TxId::new(2), DocId(1)).await.unwrap();

    // Query search_at(seq1) MUST still yield DocId(1)
    let res = index.search_at(&[1.0, 0.0, 0.0, 0.0], 10, seq1).await.unwrap();
    assert!(res.iter().any(|doc| doc.doc_id == DocId(1)), "search_at(seq=1) MUSS doc_id=1 enthalten");
}
```

---

## Anhang C: Finaler Cleanliness-Check

```text
?? docs/analysis/VEC_contextra_vector_ANALYSE_2026-10-19.md
```
