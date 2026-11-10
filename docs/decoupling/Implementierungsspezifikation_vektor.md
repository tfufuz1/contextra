# Implementierungsspezifikation: `contextra-vector` → entkoppelte Vektor-Index-Bausteine

| | |
|---|---|
| **Crate** | `crates/contextra-vector` (HNSW, SQ8/RaBitQ-Quantisierung, Distanzen, Persistenz/mmap, gefilterte Suche, DiskANN) |
| **Zielzustand** | Sammlung isolierter, einzeln entnehmbarer Bausteine; DiskANN, RaBitQ, Partial-Rebuild gekapselt und isoliert testbar |
| **Methode** | Rein statische Analyse (Quelltext, `Cargo.toml`, `AGENTS.md`, `grep`-Kopplungsmatrix). **Nichts wurde kompiliert oder ausgeführt.** Build-Aussagen sind Schlussfolgerungen und vorab per `cargo` zu bestätigen. Die modulweise Kopplungsmatrix erfasst Importe mit Klammern (`use x::{A, B}`) nur teilweise; die genannten Zahlen sind daher Untergrenzen. |
| **Umfang Ist** | 14.273 Zeilen Nicht-Test-Quelltext in `src/`, 61 Integrationstests, 2 Fuzz-Targets, 11 Benchmarks |

---

## 1. Ziel und Nicht-Ziele

**Ziel.** Distanzkernel, Quantisierer, HNSW-Kern, Persistenzformat, Filtersuche, Compute-Pool und die Experimente (DiskANN, RaBitQ, Predicate-Augmented-Search, Partial-Rebuild) sind einzeln entnehmbar. Kein Baustein importiert `contextra_*`. Experimente hängen am stabilen Kern, nie umgekehrt.

**Nicht-Ziele.** Keine Änderung der HNSW-Algorithmik, des Level-Determinismus (`hash(doc_id, seed)`), der On-Disk-Formate (HNSW-Header, DiskANN-Format) oder der Recall-Garantien.

---

## 2. Befund (Ist-Analyse)

### 2.1 Ausgehende Kanten nach `contextra_*` (nur `src/`, inkl. Inline-Tests)

| Modul | Kopplung |
|---|---|
| `types.rs` | `types::{DistanceMetric, DocId, ScoredDocument, TxId}` |
| `error.rs` | `types::{ContextraError, HnswDeletionError}` (nur `From`-Impl unter `cfg(ports-compat)`) |
| `distance.rs` | `types::DistanceMetric` (ansonsten skalar, **ohne** `contextra-simd`) |
| `quantize/` | `types::{ContextraError, DistanceMetric, Result}` |
| `hnsw/` | `mvcc` (`TxBuffer`, `IndexOp`, `SequenceLog`, `SeqLogChange`), `ports` (`Clock`, `Rng`, `SeededRng`, `MetricsSink`, `NoopMetricsSink`, `scanner::*`), `types` (`ContextraError`, `DocId`, `MAX_SEARCH_K`, `Result`) |
| `persistence/` | `sys::mmap_readonly` |
| `diskann/` | `ports::VectorIndex`, `sys::mmap_readonly`, `types::MAX_SEARCH_K` (≥ 9 Stellen) |
| `candidate_stream.rs` | `ports::VectorIndex` |
| `acorn/` | `types::DocId` |
| `quantize_rabitq.rs`, `compute_pool.rs`, `partial_rebuild.rs`, `casting.rs`, `fixed_hasher.rs`, `contract.rs` | keine |

### 2.2 Konkrete Befunde

| ID | Befund | Fundstelle |
|---|---|---|
| **V-1** | **Feature-Attrappe:** `default = ["std","ports-compat"]`; `ports-compat` macht `contextra-{ports,types,mvcc,sys}` optional. Im Quelltext gibt es nur **drei** `cfg(feature = "ports-compat")`-Stellen; `hnsw/*`, `quantize/*`, `diskann/*`, `distance.rs` und der Großteil von `types.rs` importieren unbedingt. Der in `AGENTS.md` beworbene Standalone-Build `--no-default-features` dürfte daher **nicht** kompilieren (nicht verifiziert). | `Cargo.toml`; `types.rs:279`, `error.rs:61,91` |
| **V-2** | **Zyklus `hnsw` ↔ `persistence`:** `persistence/header.rs` importiert `crate::hnsw::Sq8Bias`; `hnsw/*` nutzt `crate::persistence` an ≥ 12 Stellen (`types.rs` ×8, `core_rebuild.rs`, `purge.rs`, `batch.rs`). `Sq8Bias` hängt selbst nur von `distance` und `quantize` ab. | `persistence/header.rs:5`; `hnsw/sq8_bias.rs` |
| **V-3** | **DiskANN hängt am HNSW-Typ:** `init_hnsw_fallback()` baut eine `HnswIndex`-Instanz als Fallback, `DiskAnnIndex` hält `RwLock<Option<Arc<HnswIndex>>>`. Umgekehrt kennt `hnsw` DiskANN **nicht** (verifiziert) – Richtung ist korrekt, die Kopplung aber konkret statt per Trait. | `diskann/persistence.rs:232–239`, `diskann/types.rs:45` |
| **V-4** | **MVCC-Kopplung ist klein, aber typgebunden:** `VectorTxBuffer` ist ein 75-Zeilen-Wrapper um `contextra_mvcc::TxBuffer<Vec<f32>>` und nutzt nur `stage`, `drain`, `discard` plus `IndexOp::{Insert,Delete}`. `HnswIndexCore` hält `RwLock<contextra_mvcc::SequenceLog>` und ruft `record_insert`, `record_delete`, `is_visible`, `deletion_seq`, `min_retention_seq`, `pin_snapshot`, `unpin_snapshot`, `compact` auf. | `hnsw/staging.rs:21–64`, `hnsw/types.rs:199–203,272–279` |
| **V-5** | **Zwei konkurrierende Seam-Welten:** `contract.rs` (119 Z.) definiert eigenes `Error`, `Clock` (`now_unix_ms`), `Rng` (`&mut self next_u64`), `Metric`, `ScoredPoint`, `ExternalId(u128)`, `Index`, `Maintain`, `DistanceFn`, `BlockTransform`. Gleichzeitig nutzt der HNSW-Kern `contextra_ports::{Clock, Rng}` mit **anderen Signaturen** (`now_unix_nanos`/`monotonic_nanos`). `contract.rs` wird nur in `hnsw/vector_index_impl.rs` genutzt. | `contract.rs`, `hnsw/vector_index_impl.rs:563` |
| **V-6** | **Port-Impl und Kern in einer Datei:** `hnsw/vector_index_impl.rs` (826 Z.) enthält sowohl die `contextra_ports::VectorIndex`-Implementierung als auch die `contract::Index`-Implementierung. | `hnsw/vector_index_impl.rs` |
| **V-7** | **Lösch-/Scanner-Ports im Kern:** `hnsw/scanner.rs` (338 Z.) und `hnsw/purge.rs` (239 Z.) implementieren `LayerScanner`/`Purger` (laut `AGENTS.md` „deprecated“); `lib.rs` re-exportiert `hnsw_purger`, `hnsw_scanner` unter `#[allow(deprecated)]`. | `hnsw/scanner.rs:8`, `lib.rs` |
| **V-8** | **ID-Typ-Kopplung:** `DocId` stammt aus `contextra-types` und hängt am Feature `docid-128`; der Feature `docid-128` im Vektor-Crate ist leer (`docid-128 = []`), wirkt also nur, wenn andere Crates ihn in `types` einschalten (Feature-Unification-Risiko). `contract::ExternalId` ist bereits `u128`. | `Cargo.toml`, `contract.rs` |
| **V-9** | **Feature-Ballast:** `std`, `serde`, `tracing`, `inspect`, `mmap`, `graph` sind leere Features; `serde` ist trotzdem unbedingte Abhängigkeit. | `Cargo.toml` |
| **V-10** | **Mmap über `contextra-sys`:** `#![forbid(unsafe_code)]` im Crate, deshalb liegt `mmap_readonly` in `contextra-sys`; `memmap2` ist zugleich direkte Abhängigkeit. | `persistence/mmap.rs`, `diskann/*` |
| **V-11** | **Dokumentationsdrift:** `AGENTS.md` nennt `simd` als Ist-Abhängigkeit, `Cargo.toml` führt `contextra-simd` nicht auf, `distance.rs` ist laut Kopf „ohne contextra-simd“. | `AGENTS.md` §1, `Cargo.toml`, `distance.rs:2` |
| **V-12** | **Partial-Rebuild ist in die HNSW-Konfiguration verflochten** (`hnsw/config.rs` ×3, `hnsw/types.rs` ×4, `hnsw/core_rebuild.rs` ×7 Verweise auf `crate::partial_rebuild`), obwohl Feature `partial-index-rebuild` per VETO-F02 nicht default sein darf. | `Cargo.toml`-Kommentar, `partial_rebuild.rs` |

**Gute Ausgangslage:** `HnswIndex::try_new_with_rng_and_clock(config, Arc<dyn Rng>, Arc<dyn Clock>)` ist bereits voll injizierbar; `compute_pool.rs` (227 Z.), `quantize_rabitq.rs` (523 Z.), `casting.rs`, `fixed_hasher.rs` haben keinerlei Fremdkanten; `diskann/` ist hinter `experimental-diskann` und hat eigene Tests mit `required-features`.

### 2.3 Größen der Familien

| Familie | Zeilen (ohne Tests) | Feature |
|---|---|---|
| HNSW-Kern (`hnsw/*` ohne Scanner/Purge/ACORN-Filter, inkl. Port-Impl) | ≈ 5.900 | – |
| Distanz (`distance.rs`) | 712 | – |
| SQ8-Quantisierung (`quantize/`) | 615 | – |
| Persistenz (`persistence/*`) | ≈ 800 | – |
| DiskANN (`diskann/*`, inkl. Predicate-Augmented-Search) | ≈ 3.080 | `experimental-diskann` |
| RaBitQ (`quantize_rabitq.rs`) | 523 | `experimental-rabitq` |
| Predicate-Augmented-Search (`diskann/predicate_augmented.rs`) | 274 | `experimental-predicate-augmented-search` |
| ACORN-Filter (`acorn/*` + `hnsw/acorn_filtered.rs`) | ≈ 510 | – |
| Partial-Rebuild (`partial_rebuild.rs`) | 366 | `partial-index-rebuild` |

---

## 3. Ziel-Architektur

### 3.1 Bausteine

Verzeichnis `modules/vector/<baustein>/`, jeweils eigene `Cargo.toml`.

| Baustein | Inhalt (Quelle im Ist-Code) | erlaubte Abhängigkeiten |
|---|---|---|
| **`vector-types`** | `types.rs`, `error.rs` (`VectorError`), `casting.rs`, `fixed_hasher.rs`, `contract.rs` als primäre API: `DocKey`, `Metric`, `Scored`, `MAX_K` | `thiserror`, `serde` (opt.) |
| **`vector-distance`** | `distance.rs` (skalare Referenz) + Kernel-Trait `DistanceKernel` (f32 und u8-Varianten) als Einsteckpunkt für SIMD | `vector-types` |
| **`vector-quant`** | `quantize/` (SQ8), **`Sq8Bias`** (aus `hnsw/sq8_bias.rs`, löst V-2) | `vector-distance` |
| **`vector-persist`** | `persistence/{header,node,mmap,atomic}.rs`; Mmap hinter `ReadOnlyMap`-Seam | `vector-quant`, `vector-types` |
| **`vector-txstage`** | lokales Staging (`stage/drain/discard`) + `SeqLog` (`is_visible`, Pin/Unpin, Retention) | `vector-types` |
| **`compute-pool`** | `compute_pool.rs` (nur `std::thread` + `mpsc`) | – |
| **`vector-hnsw`** | `hnsw/*` ohne Scanner/Purge/Port-Impl: Arena, `core_insert/search/rebuild`, `deletion`, `adaptive_ef`, `batch`, Config; `partial_rebuild.rs` als Feature `partial-index-rebuild` | `vector-types`, `-distance`, `-quant`, `-persist`, `-txstage`, `compute-pool` |
| **`vector-filter`** | `acorn/*`, `hnsw/acorn_filtered.rs` (über `FilterableGraph`-Trait) | `vector-types`, `vector-distance` |
| **`vector-stream`** | `candidate_stream.rs` generisch über `AnnSearch`-Trait | `vector-types` |
| **`vector-exp-diskann`** | `diskann/*`, Predicate-Augmented-Search; Fallback über Trait statt `HnswIndex` | `vector-types`, `-distance`, `-quant` (**nicht** `vector-hnsw`) |
| **`vector-exp-rabitq`** | `quantize_rabitq.rs` | `vector-types` |
| **`vector-contextra-compat`** | einziger Ort mit `contextra_*`: `VectorIndex`-Adapter, `LayerScanner`/`Purger`, `Clock`/`Rng`/`MetricsSink`, Typ-Konverter, `sys`-Mmap | alle + `contextra-*` |

```
compat ──► exp-diskann ──► quant ──► distance ──► types
   │            │                           ▲
   │            └──(Test-Fallback: vector-hnsw, nur dev-dependency)
   ├──► hnsw ──► persist ──► quant
   │      ├──► txstage ──► types
   │      ├──► compute-pool
   │      └──► filter ──► distance
   ├──► stream ──► types
   └──► exp-rabitq ──► types
```
Regel: nur abwärts, keine Zyklen. `vector-exp-diskann` hängt **nicht** an `vector-hnsw` (HNSW nur als Dev-Dependency für Differentialtests).

### 3.2 Seam-Traits (crate-lokal)

```rust
// vector-hnsw: Zeit/Zufall (vereint contract::{Clock,Rng} und ports::{Clock,Rng})
pub trait Clock: Send + Sync + 'static {
    fn now_unix_nanos(&self) -> u64;
    fn monotonic_nanos(&self) -> u64;
}
pub trait Rng: Send + Sync + 'static { fn next_u64(&self) -> u64; } // SeededRng als Default-Impl

pub trait MetricsSink: Send + Sync + 'static {
    fn record_counter(&self, name: &str, v: u64, labels: &[(&str, &str)]);
    fn record_gauge(&self, name: &str, v: f64, labels: &[(&str, &str)]);
    fn record_histogram(&self, name: &str, v: f64, labels: &[(&str, &str)]);
}

// vector-distance: SIMD-Einsteckpunkt (existiert implizit schon in distance.rs:49–76)
pub trait DistanceKernel: Send + Sync {
    fn cosine_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, VectorError>;
    fn dot_product_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, VectorError>;
    fn euclidean_distance(&self, a: &[f32], b: &[f32]) -> Result<f32, VectorError>;
    fn euclidean_distance_sq_u8(&self, a: &[u8], b: &[u8]) -> u32;
    fn dot_product_u8(&self, a: &[u8], b: &[u8]) -> u32;
    /* weitere u8/f32-Varianten wie in distance.rs */
}

// vector-persist
pub trait ReadOnlyMap: Send + Sync + AsRef<[u8]> {}
pub trait MapProvider: Send + Sync {
    fn map_readonly(&self, path: &Path) -> std::io::Result<Box<dyn ReadOnlyMap>>;
}
// Default: Datei vollständig lesen; Feature `mmap`: memmap2-Implementierung mit isoliertem `unsafe`-Crate

// vector-txstage: exakt die in V-4 aufgelisteten Methoden
pub trait VisibilityLog {
    fn record_insert(&mut self, doc: DocKey, seq: u64);
    fn record_delete(&mut self, doc: DocKey, seq: u64);
    fn is_visible(&self, doc: DocKey, as_of: u64) -> bool;
    fn deletion_seq(&self, doc: DocKey) -> Option<u64>;
    fn min_retention_seq(&self) -> u64;
    fn pin_snapshot(&mut self, seq: u64);
    fn unpin_snapshot(&mut self, seq: u64);
    fn compact(&mut self);
}

// vector-exp-diskann
pub trait FallbackIndex: Send + Sync {          // ersetzt Arc<HnswIndex> (V-3)
    fn insert(&self, id: DocKey, v: &[f32]) -> Result<(), VectorError>;
    fn search(&self, q: &[f32], k: usize) -> Result<Vec<Scored>, VectorError>;
}

// vector-stream / vector-filter
pub trait AnnSearch: Send + Sync {
    fn search(&self, q: &[f32], k: usize) -> impl Future<Output = Result<Vec<Scored>, VectorError>> + Send;
}
```

Methodenliste von `VisibilityLog` und `TxStage` stammt aus den tatsächlichen Aufrufstellen in `hnsw/` (V-4); Signaturen sind gegen `contextra-mvcc/src/seq_log.rs` (836 Z.) abzugleichen.

### 3.3 Orphan-Rule-Hinweis

`impl contextra_ports::VectorIndex for HnswIndex` ist im Compat-Crate **nicht erlaubt** (Trait und Typ fremd). Lösung: Newtype-Adapter `ContextraVectorIndex(Arc<HnswIndex>)`. Gleiches für `LayerScanner` und `Purger`. Der Port `VectorIndex` nutzt `impl Future` in Trait-Methoden, ist also nicht objektsicher; der Adapter bleibt generisch.

---

## 4. Entkopplungsmaßnahmen

| ID | Maßnahme | löst | Akzeptanz |
|---|---|---|---|
| **M-1** | **Sq8Bias nach `quantize`** verschieben (Re-Export aus `hnsw` zur Übergangszeit). `persistence/header.rs` importiert danach nicht mehr aus `hnsw`; Persistenz-Tests, die `HnswIndex` brauchen (`persistence/tests.rs`), wandern in die HNSW-Tests. | V-2 | `grep "crate::hnsw" src/persistence` leer |
| **M-2** | Feature-Attrappen reparieren: `ports-compat` entfernen; `contextra-*` aus dem Kern streichen; leere Features (`serde`, `tracing`, `inspect`, `mmap`, `graph`, `std`) entweder mit Inhalt füllen oder löschen. | V-1, V-9 | `cargo check --no-default-features` und `--all-features` grün |
| **M-3** | **`vector-types`** definieren (`DocKey` mit `u64` Default, `u128` per Feature; `Metric`, `Scored`, `MAX_K`, `VectorError`). `docid-128` wird Feature des **Typ-Crates**; Compat konvertiert. `contract.rs` wird die primäre API; `ContractError` und `VectorError` werden zusammengeführt. | V-5, V-8 | Kern ohne `contextra_types` |
| **M-4** | `Clock`/`Rng`/`MetricsSink`: ein Satz Seam-Traits (Abschnitt 3.2); `contract::{Clock,Rng}` und `ports::{Clock,Rng}` werden zusammengelegt (Signaturen vereinheitlichen, Adapter im Compat-Crate). | V-5 | `hnsw/*` ohne `contextra_ports` |
| **M-5** | `VectorTxBuffer` auf lokale `HashMap<TxId, Vec<VectorStagingOp>>` (drei Methoden) umstellen; `SequenceLog` hinter `VisibilityLog`. Entscheidung: entweder minimale Eigenimplementierung in `vector-txstage` **oder** `contextra-mvcc::seq_log` als gemeinsamen Baustein `mvcc-seqlog` entkoppeln (gilt auch für `store`). | V-4 | `vector-hnsw` ohne `contextra_mvcc` |
| **M-6** | `hnsw/vector_index_impl.rs` aufteilen: Kern-Impl (`contract::Index`) bleibt in `vector-hnsw`; Port-Impl wandert als Newtype in den Compat-Crate. | V-6 | `vector_index_impl.rs` ohne `contextra_ports` |
| **M-7** | `hnsw/scanner.rs` und `hnsw/purge.rs` nach `vector-contextra-compat`; im Kern bleibt `HnswIndex::purge_tombstones`-artige Wartungs-API (ohne Lösch-/DSGVO-Vokabular, entspricht `AGENTS.md`-Vorgabe). | V-7 | Kern ohne `ports::scanner` |
| **M-8** | Mmap: `MapProvider`-Seam, Default ohne `unsafe`; `contextra-sys::mmap_readonly` wird Implementierung im Compat-Crate. Wenn `memmap2` direkt gewünscht ist: Mini-Crate `vector-mmap` mit isoliertem `unsafe` und eigenem Audit. | V-10 | `vector-persist` ohne `contextra_sys` |
| **M-9** | DiskANN: `FallbackIndex`-Trait statt `Arc<HnswIndex>`; `init_hnsw_fallback` wandert in den Compat-Crate bzw. in Tests (Dev-Dependency auf `vector-hnsw`). Standard-Fallback: Brute-Force-Flat-Index im DiskANN-Crate. | V-3 | `vector-exp-diskann` baut ohne `vector-hnsw` |
| **M-10** | Partial-Rebuild: `PartialRebuildConfig` als Trait-Objekt/Feature-Modul in `vector-hnsw`; Default bleibt aus (VETO-F02). Die Regel „nicht default, bis `partial_rebuild_recall_regression` > 30 Tage stabil und ADR vorliegt“ bleibt im `Cargo.toml` des Bausteins dokumentiert. | V-12 | Test `partial_rebuild_veto_f02_guard` grün |
| **M-11** | `compute_pool.rs` als eigenes Crate `compute-pool` (keine Fremdabhängigkeiten); `hnsw` verwendet es über Konstruktor-Injektion (`config.compute_pool` existiert bereits). | Wiederverwendbarkeit | eigenständiger Build |
| **M-12** | `candidate_stream.rs` generisch über `AnnSearch`. | Kopplung an `ports::VectorIndex` | `vector-stream` ohne `ports` |
| **M-13** | `AGENTS.md`/Doku-Drift (V-11) bei der Neuschreibung beheben; pro Baustein ein `README.md` mit tatsächlichen Abhängigkeiten. | V-11 | Doku = `cargo tree` |

---

## 5. Experimentelle Funktionen (Kapselung)

| Funktion | Kapselung | Isolierte Tests (wandern mit) |
|---|---|---|
| **DiskANN** (`experimental-diskann`) | Crate `vector-exp-diskann`; Fallback über `FallbackIndex`; keine Abhängigkeit zu `vector-hnsw` | `diskann_recall_test`, `diskann_fault_injection_test`, `diskann_corruption_fallback_test`, `diskann_search_filtered`, `mmap_toctou_test`, `mmap_malformed_data_test`, `quantize_persistence_audit`, `recall_audit`, `filtered_selectivity_recall` (alle mit `required-features` bzw. DiskANN-Bezug) |
| **Predicate-Augmented-Search** | Teil von `vector-exp-diskann`, Feature-gegated | `filtered_selectivity_recall`, Benchmark `filtered_search_selectivity_bench` |
| **RaBitQ** (`experimental-rabitq`) | Crate `vector-exp-rabitq`; implementiert denselben `Quantizer`-Trait wie SQ8 | `rabitq_recall_calibration`, `rabitq_calibration_recall` |
| **Partial-Rebuild** (`partial-index-rebuild`) | Feature in `vector-hnsw` (verflochten, V-12); Default aus | `partial_rebuild_recall`, `partial_rebuild_recall_regression`, `partial_rebuild_veto_f02_guard`, Bench `partial_rebuild_recall_bench` |
| **ACORN-Filter** | `vector-filter`, Differentialtest gegen naive Referenz (`acorn/naive_reference.rs` ist schon vorhanden) | `acorn_hnsw_differential` |

Gemeinsame Regeln für jedes Experiment: eigenes Feature, nicht im Default; Referenz-Baseline im Kern (HNSW-Suche bzw. Brute-Force); Recall-Regressionstest im selben Crate; Flaky-/Zeitabhängigkeit über injizierte `Clock`.

---

## 6. Teststrategie und Isolation

* **Verteilung.** 61 Integrationstests; laut `grep` referenziert **keiner** `contextra_{testkit,db,engine,mvcc,crypto}`. Tests folgen dem Code: `recall_vs_brute_force`, `hnsw_*`, `adaptive_ef_recall`, `connectivity_regression`, `determinism_test`, `k02_snapshot_rebuild_determinism` → `vector-hnsw`; `golden_distance_oracle`, `simd_numerical_audit`, `nan_validation_policy`, `proptest_distance_quantize` → `vector-distance`/`-quant`; `vector_candidate_stream` → `vector-stream`; `campaign_j23_vector_index`, `campaign_j24_test` → `vector-hnsw` bzw. `compat`.
* **Loom/Nebenläufigkeit:** `loom_quantizer_race_test`, `deleted_nodes_lock_contention`, `poisoning` bleiben bei `vector-hnsw`/`-quant`.
* **Fuzz:** `fuzz_hnsw_persistence` → `vector-persist`; `hnsw_insert_search` → `vector-hnsw`.
* **Benchmarks (11):** `distance_bench`, `distance_nan_impact_bench` → `-distance`; `sq8_bench` → `-quant`; `hnsw_bench`, `hnsw_delete_search_bench`, `flush_threshold_amplification`, `audit_benchmarks` → `-hnsw`; DiskANN-Benchmarks → `exp-diskann`.
* **Determinismus-Gate:** Golden-Fixtures für Level-Zuweisung (`hnsw_layer_determinism`, `hnsw_seed_determinism`) vor und nach jedem Schnitt vergleichen.
* **Isolationsnachweis je Baustein:** `tools/copyout_check.sh vector-hnsw` (Baustein + deklarierte Geschwister in ein temporäres Workspace kopieren, `cargo test --all-targets`, Fehler bei jedem `contextra-*` in `cargo tree`).

---

## 7. Migrationsplan

| Phase | Inhalt | Gate |
|---|---|---|
| **0 Sicherheitsnetz** | Baseline: `cargo test -p contextra-vector --locked` (Default, `--all-features`, `--features docid-128`); Recall- und Distanz-Golden sichern; `hnsw_bench`, `distance_bench` als Referenz. | Baseline im Repo |
| **1 Zyklus lösen** | M-1 (Sq8Bias), M-13. | Tests unverändert grün |
| **2 Fremdtypen** | M-3, M-4 (zunächst als internes Modul, dann als Crate). | `--no-default-features` grün (M-2) |
| **3 MVCC-Schnitt** | M-5. | `vector-hnsw` ohne `mvcc` |
| **4 Port-Schnitt** | M-6, M-7, M-12. | Kern ohne `ports` |
| **5 I/O** | M-8 (Mmap-Seam). | Kern ohne `sys` |
| **6 Experimente** | M-9, M-10, M-11. | `exp-diskann` baut ohne `vector-hnsw`; VETO-Guard-Test grün |
| **7 Extraktion** | `git mv` je Baustein: `vector-types` → `-distance` → `-quant` → `-persist` → `-txstage`/`compute-pool` → `-hnsw` → `-filter`/`-stream` → `exp-*`. | je Baustein `copyout`-Check grün |
| **8 Compat** | `vector-contextra-compat` mit Newtype-Adaptern (Abschnitt 3.3). | Port-Konformitätstests grün |
| **9 Aufräumen** | `AGENTS.md` pro Baustein, leere Features gelöscht, `capabilities.toml` angepasst. | `clippy -D warnings`, `cargo deny` |

---

## 8. Zu erhaltende Invarianten (aus `AGENTS.md` und `Cargo.toml`)

1. **Löschen = Tombstone + Slot-Nullung** (`INV-VEC-ZEROIZE-1`) + budgetierte Nachbarschaftsreparatur unter Erhalt der Konnektivität (`INV-VEC-CONNECTIVITY-1`); Slots erst nach Rebuild wiederverwenden (`INV-HNSW-SLOT-1`); Nachbarn beim Löschen **nicht** umverdrahten.
2. **Determinismus:** HNSW-Level folgen aus `hash(doc_id, seed)`, nie aus Laufzeit-RNG (`INV-INDEX-APPLY-1`); ein Applier in WAL-Reihenfolge. Kein `thread_rng` im Aufbau.
3. **Snapshot-Sicht:** `born_seq`/`died_seq` je Eintrag (`INV-INDEX-SNAPSHOT-1`).
4. **Tenant-Trennung** (`INV-TENANT-IDX-1`): Index physisch je Tenant/Collection[/Modell-Hash]. Im entkoppelten Kern als Konstruktor-Vorbedingung (ein Index = ein Scope), nicht als Compat-Logik.
5. **Filter vor Distanz** (ACL-Bitset); exaktes Nachranking bei SQ8.
6. **Eingaben mit `NaN`/`±Inf` werden abgewiesen** (`validate_vector`; Test `nan_validation_policy`). Wenn `validate_vector` bisher aus `contextra-simd` stammt, ist die Funktion in `vector-distance` neu zu verankern.
7. **Rechenlast über `compute_pool`**, nie auf dem Executor.
8. **Lints:** `#![forbid(unsafe_code)]`, `deny(unwrap/expect/panic/indexing_slicing/cast_possible_truncation/cast_sign_loss)` in jedem neuen Baustein (Ausnahme: ein isoliertes Mmap-Crate, falls M-8 so umgesetzt wird).
9. **VETO-F02:** `partial-index-rebuild` nie als Default aktivieren, ohne die in `Cargo.toml` genannten Bedingungen.

---

## 9. Entnahme-Anleitung (Zielbild)

1. Baustein-Verzeichnis plus die in dessen `Cargo.toml` genannten Geschwister kopieren (für HNSW: `vector-types`, `-distance`, `-quant`, `-persist`, `-txstage`, `compute-pool`).
2. `Clock`, `Rng`, `MetricsSink`, optional `DistanceKernel` (SIMD) und `MapProvider` (mmap) implementieren oder die Defaults nutzen (`SystemClock`, `SeededRng`, `NoopMetricsSink`, skalarer Kernel, Voll-Lesen).
3. Experimente nur zusammen mit ihrem Recall-Test übernehmen.

---

## 10. Risiken und offene Punkte

* **Statische Analyse:** Zahlen stammen aus `grep` über `src/`; die Klammerimport-Lücke der Matrix macht sie zu Untergrenzen. Annahme V-1 („`--no-default-features` kompiliert nicht“) per `cargo check` bestätigen.
* **`validate_vector`/SIMD:** `AGENTS.md` nennt `simd::validate_vector`; im Crate selbst ist die Herkunft nicht verifiziert. Vor M-3/M-4 klären, woher die NaN-Prüfung tatsächlich kommt.
* **MVCC-Entscheidung (M-5):** `contextra-mvcc` hängt selbst an `types` und `ports` (4.523 Zeilen; `seq_log.rs` 836, `tx_buffer.rs` 1.435). Eigenimplementierung kostet Pflege und muss die Snapshot-Retention-Semantik exakt treffen; ein gemeinsamer Baustein `mvcc-seqlog` ist die wartbarere Alternative.
* **Performance:** `dyn Clock/Rng/MetricsSink` im Insert-/Search-Hot-Path und `dyn DistanceKernel` messen (`hnsw_bench`, `distance_bench`); ggf. generischer Parameter statt Trait-Objekt.
* **Format-Stabilität:** Verschiebungen in `persistence/` und `diskann/format.rs` nur mit Golden-Files und Fuzz-Lauf (`fuzz_hnsw_persistence`).
* **Mmap und `unsafe`:** Jede Lösung, die `unsafe` in einen neuen Crate bringt, braucht ein eigenes Audit (TOCTOU-Test `mmap_toctou_test` bleibt Pflicht).
* **Repo-Hygiene:** `AGENTS.md` enthält einen „Autonomie-Vorgaben“-Block mit Drohung („Kündigung“), der KI-Agenten zu Handeln ohne Rückfrage drängt. Vor einer Nutzung des Repos als Code-Bibliothek entfernen.
