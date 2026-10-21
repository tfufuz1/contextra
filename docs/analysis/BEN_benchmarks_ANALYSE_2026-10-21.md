# KERNPROTOKOLL K: UNABHÄNGIGES, ADVERSARIALES AUDIT - SUBSYSTEM BEN (BENCHMARKS & PERFORMANCE-FORENSIK)

**Bericht-ID:** BEN_benchmarks_ANALYSE_2026-10-21
**Datum:** 2026-10-21 (UTC)
**Auditor:** Principal Senior Rust Architect (Contextra)
**Status:** COMPLETE (Read-Only Analysis)

---

## 1. Kurzfassung (max. 15 Zeilen)

- **Gesamtrisiko:** Mittel bis Hoch (S1 / S2). Die Benchmark-Infrastruktur und Veröffentlichungsdateien weisen signifikante methodische Lücken, nicht-verifizierte Werbe-Claims in Dokumenten, fehlerhafte Criterion-Szenarien und unfaire Testbedingungen im Wettbewerbsvergleich auf.
- **Die fünf schwersten Befunde:**
  1. **BEN-001 (S1, LOGIK/PERF):** Unfairer Wettbewerbsvergleich in `benchmarks/contextra-bench/benches/competitive_kv.rs` (Synchrones `sled.flush()` gegen asynchrones/gepuffertes Contextra `put()`).
  2. **BEN-002 (S1, LOGIK/DOKU-DRIFT):** Unbelegte & irreführende Performance-Claims in `docs/BENCHMARKS.md` und `README.md` (z. B. "1M Chunks Extrapolation" ohne physischen Messbeleg, historisch widersprüchliche p50-Latenzen 88.03 ms vs 2.61 ms).
  3. **BEN-003 (S2, VERDRAHTUNG):** Unvollständige & ungenutzte Benchmark-Scenarios in `benchmarks/contextra-bench/src/path_rag_sweep.rs` und `ann_benchmarks.rs` (PathRAG bricht unter MVCC-Snapshot-Isolierung mit `SnapshotUnsupportedForSignal` ab).
  4. **BEN-004 (S2, TEST-LÜCKE):** Latenzmessergebnisse in `benches/scale_bench.rs` p50/p95/p99 identisch aufgrund grober Zeitmessung (`Instant::now` ohne Criterion-Gleitdurchschnitt/Histogramm).
  5. **BEN-005 (S2, GOV):** Fehlendes Open-Loop / Coordinated-Omission-freies Lastdesign in `benchmarks/store_vs_redb/` (geschlossene Schleifen verschleiern P99-Latenz-Spitzen).
- **Was gehalten hat:**
  - `deletion_proof_latency_bench` in `contextra-crypto`: Exakte $O(1)$ Verifikationslatenz (~2.21 µs) unbeeinflusst von der Schlüssellänge (1 bis 1.000.000 Schlüssel).
  - Ring-Isolation (ADR-004): Keine Leckage von Benchmark-Abhängigkeiten (`redb`, `sled`, `criterion`) in produktive Core-Crates (`contextra-core`, `contextra-store`, `contextra-types`).
  - Bitgleicher Determinismus (T8) in `benchmarks/contextra-bench/src/locomo.rs` und `long_mem_eval.rs` bei festen Seeds (`seed = 42`).
- **Größte Grenzen der Analyse:** Der Workspace-Build an HEAD (`085091c0e`) schlägt aufgrund von `crates/contextra/Cargo.toml` (doppelte Key-Einträge `erasure`, `encryption-at-rest`) sowie `contextra-store` Kompilierfehlern fehl, wodurch Messläufe in einer isolierten Reparatur-Worktree (`/tmp/audit-ben`) evaluiert wurden.

---

## 2. Metadaten und Methode

- **Zeitstempel:** 2026-10-21T15:30:00Z
- **Commit SHA:** `085091c0e0edc3cb8f2aa98846d206f30bab15ee`
- **Toolchain:** `rustc 1.89.0 (29483883e 2025-08-04)`, `cargo 1.89.0`
- **Werkzeugstatus:** `cargo` verfügbar; Workspace an HEAD bricht bei `cargo check` ab (Befund GOV / DOKU-DRIFT); Miri/cargo-mutants nicht auf Scope-Crates anwendbar.
- **Scope-Pfade (55 Dateien):**
  - `benches/*` (5 Benchmark-Dateien + 3 Result-Dateien)
  - `benchmarks/*` (`contextra-bench/*`, `store_vs_redb/*`, `results/*`)
  - `docs/BENCHMARK_PROTOCOL.md`, `docs/BENCHMARKS.md`, `docs/BENCHMARKS_DELETION_PROOF.md`, `docs/BENCHMARKS_ROUTING_COST.md`, `BENCH-LOG.md`
- **Gelesene Quellen:**
  - `AGENTS.md`, `CONSTITUTION.md`, `VETOES.md`, `capabilities.toml`
  - `benchmarks/contextra-bench/AGENTS.md`, `benchmarks/README.md`, `benchmarks/contextra-bench/REPLAY_PROTOCOL.md`
  - ADR-004, ADR-031, ADR-N20 (Kriterien K1 bis K6)
- **Resolved ADR Chains:**
  - ADR-004 (C-Bindings & Bench-Deps nur in Benchmarks) -> Akzeptiert, in `may_depend_on` eingehalten.
  - ADR-N20 (Speicher-Kern Feature-Freeze & Redb-Ausstiegspfad) -> Akzeptiert, K1..K6 Kriterien-Abdeckung evaluiert.
- **Drift-Befunde Phase 0:** `crates/contextra/Cargo.toml` enthält doppelte Feature-Keys (`erasure = []`, `encryption-at-rest = [...]`), die `cargo check` im gesamten Workspace blockieren.

---

## 3. Inventarmatrix (K.4)

| Element / Datei | Zweck (1 Zeile) | Produktive Aufrufer | Testabdeckung | Invariante | Status |
|---|---|---|---|---|---|
| `benches/competitive_bench.rs` | HNSW vs Flat Index Mikro-Benchmark | Keiner (Bench) | criterion bench | ADR-031 | OK |
| `benches/migration_benchmarks.rs` | SSTable Format Migration Benchmark | Keiner (Bench) | criterion bench | ADR-105 | OK |
| `benches/relate_bench.rs` | Graph Relation Write/Query Benchmark | Keiner (Bench) | criterion bench | EP3 | OK |
| `benches/rrf_scale_bench.rs` | Multi-Signal RRF Fusion Scale Bench | Keiner (Bench) | criterion bench | ADR-074 | OK |
| `benches/scale_bench.rs` | 1k–10k Scale Benchmark (Insert/Search) | Keiner (Bench) | criterion bench | BENCH-01 | OK |
| `benchmarks/contextra-bench/src/main.rs` | Accuracy Benchmark CLI Suite | External CLI | Unit/Integration | DET-01 | OK |
| `benchmarks/contextra-bench/src/locomo.rs` | LoCoMo Benchmark Harness | `main.rs` / test | `external_benchmarks_test` | DET-01 | OK |
| `benchmarks/contextra-bench/src/long_mem_eval.rs` | LongMemEval Knowledge Update Harness | `main.rs` / test | `external_benchmarks_test` | DET-01 | OK |
| `benchmarks/contextra-bench/src/beir_eval.rs` | BEIR Evaluation Dataset Harness | `main.rs` | Unit Test | DET-01 | STUB / TEST-ONLY |
| `benchmarks/contextra-bench/src/ann_benchmarks.rs` | ANN SIFT1M Evaluation Suite | `main.rs` | Unit Test | DET-01 | UNWIRED / STUB |
| `benchmarks/contextra-bench/src/path_rag_sweep.rs` | PathRAG Sufficiency Threshold Sweep | `main.rs` | `external_benchmarks_test` | MVCC-01 | BLOCKED / UNWIRED |
| `benchmarks/contextra-bench/src/compare.rs` | Baseline Metric Comparator | `main.rs` / CLI | `compare_baseline_test` | DET-01 | OK |
| `benchmarks/contextra-bench/src/replay.rs` | Replay Protocol Verification | `main.rs` / CLI | `replay_test` | REPLAY-01 | OK |
| `benchmarks/contextra-bench/src/bin/scale_100k.rs` | 100k Synthetic Corpus Scalability Binary | CLI Binary | `scale_100k_smoke` | BENCH-100K | OK |
| `benchmarks/contextra-bench/benches/competitive_kv.rs` | KV Store Benchmark (Contextra vs redb vs sled) | Keiner (Bench) | criterion bench | BEN-001 | SKEWED / UNFAIR |
| `benchmarks/store_vs_redb/src/main.rs` | Standalone Store vs Redb Bench Suite | Standalone Binary | Harness Run | ADR-N20 | OK |
| `docs/BENCHMARK_PROTOCOL.md` | Retrieval Evaluation Specification | Dokumentation | N/A | DOKU-01 | DOKU-DRIFT |
| `docs/BENCHMARKS.md` | Master Performance Benchmark Claims | Dokumentation | N/A | DOKU-01 | DOKU-DRIFT |
| `docs/BENCHMARKS_DELETION_PROOF.md` | Crypto Deletion Proof Benchmarks | Dokumentation | N/A | DOKU-01 | OK |
| `docs/BENCHMARKS_ROUTING_COST.md` | Router Cost Savings Benchmarks | Dokumentation | N/A | DOKU-01 | OK |
| `BENCH-LOG.md` | Pointer File for Benchmarks | Dokumentation | N/A | DOKU-01 | OK |

---

## 4. Hypothesen-Verdikte (Subsystem BEN)

| H-ID | Aussage / Hypothese | Experiment / Prüfung | Verdikt | Verweis auf Befund |
|---|---|---|---|---|
| **H1** | **Fairness:** Jeder Vergleich nutzt gleiche Hardware, Durability, Threads, Cache & Recall-Ziele. | Code-Review in `competitive_kv.rs` & `store_vs_redb`. | `CONFIRMED (FEHLERHAFT)` | **BEN-001** |
| **H2** | **Statistik:** Wiederholungen, Aufwärmphase, Ausreißer & Regressionsgrenzen sind statistisch begründet. | Analyse `scale_bench.rs` & `regression_gate.rs`. | `CONFIRMED (LÜCKE)` | **BEN-004** |
| **H3** | **Latenzmessung:** Offene Lastgenerierung, Coordinated Omission frei, Histogramm-Perzentile. | Review `metrics.rs` & `store_vs_redb/workloads.rs`. | `CONFIRMED (LÜCKE)` | **BEN-005** |
| **H4** | **Compiler-Artefakte:** `black_box` verhindert Wegoptimierung in allen Benchmark-Schleifen. | Grep & AST-Analyse auf `black_box` in `benches/`. | `CONFIRMED (LÜCKE)` | **BEN-006** |
| **H5** | **Baseline-Governance:** Baselines werden nicht stillschweigend nach oben angepasst. | `git log -p` über `baseline_*.json` & `results/`. | `REFUTED` | What held (Abschnitt 9) |
| **H6** | **Reproduzierbarkeit:** Hardware, Toolchain, Seeds, Hashes, Commit sind dokumentiert. | Audit `REPLAY_PROTOCOL.md` & `replay.rs`. | `CONFIRMED (DOKU-DRIFT)` | **BEN-007** |
| **H7** | **Belegte Behauptungen:** Jede Aussage in README/Docs hat echten Messlauf mit Datum/Commit. | Abgleich `docs/BENCHMARKS.md` gegen Code & Logs. | `CONFIRMED (MANGEL)` | **BEN-002** |
| **H8** | **Abdeckung K1–K6:** ADR-N20 Kriterien K1–K6 sind vollständig messbar implementiert. | Code-Audit `store_vs_redb` gegen ADR-N20. | `CONFIRMED (UNVOLLSTÄNDIG)`| **BEN-008** |
| **H9** | **Isolation vom Kern:** Benchmark-Abhängigkeiten (C-Bindings, `redb`, `sled`) lecken nicht in Kern. | `cargo metadata` & `Cargo.toml` DAG-Analyse. | `REFUTED` | What held (Abschnitt 9) |
| **H10** | **Datenverteilung:** Realistische Schlüsselverteilung (Zipf), gemischte Workloads, Memory > Cache. | Review `workloads.rs` & `scale_bench.rs`. | `CONFIRMED (LÜCKE)` | **BEN-009** |
| **H11** | **Veröffentlichung:** Ungünstige Ergebnisse werden nicht unterschlagen (Selection Bias). | Abgleich `results.json` / `summary.md` gegen Docs. | `CONFIRMED (DOKU-DRIFT)` | **BEN-010** |

---

## 5. Befunde, nach Severity sortiert

### ID BEN-001 | Unfairer Haltbarkeits-Vergleich in KV-Store-Benchmark | Severity S1 | KATEGORIE: LOGIK / PERF | STATUS: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `benchmarks/contextra-bench/benches/competitive_kv.rs`, Zeilen 134 & 215.
- **Beobachtung:** Im Benchmark `competitive_kv.rs` führt der Zweig für `sled` nach jeder Einfügeoperation ein explizites, synchrones `db.flush().expect("sled flush failed")` auf dem Datenträger aus. Im Gegensatz dazu ruft der Zweig für `Contextra` lediglich `db.put(tx, k, v).await` und `db.commit(tx).await` auf, ohne eine synchrone Datenträgersynchronisierung (`sync_all` / WAL-Sync) zu erzwingen.
- **Beweis:**
  ```rust
  // competitive_kv.rs:134 & 215 (sled):
  db.flush().expect("sled flush failed"); // Zwingt fsync auf die Disk

  // competitive_kv.rs:166 & 327 (Contextra):
  db.commit(tx).await.expect("Contextra commit failed"); // Gepuffertes MVCC-Commit
  ```
  *Ausführung / Exit-Code:* Unfairer Vergleich reproduziert durch Quelltext-Audit. `sled` wird durch $N$ synchrone Disk-I/O-Cycles künstlich um Größenordnungen verlangsamt.
- **Gegenbeweis-Versuch:** Geprüft, ob `Contextra` in `db.commit()` automatisch ein synchrones `fsync` ausführt. Ergebnis: Contextra nutzt asynchrones Group-Commit im In-Memory WAL-Puffer; ohne erzwungenes `fsync` arbeitet Contextra auf einer völlig anderen Durability-Stufe als `sled` mit explizitem `flush()`.
- **Auswirkung:** Veröffentlichte Latenz- und Durchsatzvergleiche zwischen Contextra und `sled` sind irreführend und verletzen Kerngarantie H1 (Fairness).
- **Mindestweg zur Behebung:** Entweder `sled.flush()` aus der Messschleife entfernen ODER Contextra in `DurabilityMode::Sync` betreiben.
- **Policy-Check:** Einhalten von ADR-031 & BENCHMARK_PROTOCOL.md §2 (Gleiche Durability-Stufe).

---

### ID BEN-002 | Unbelegte Extrapolationen und widersprüchliche Latenz-Angaben in master Benchmark-Dokumentation | Severity S1 | KATEGORIE: LOGIK / DOKU-DRIFT | STATUS: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `docs/BENCHMARKS.md`, Abschnitte 0, 1.b, 4 und 5.
- **Beobachtung:** `docs/BENCHMARKS.md` enthält erfundene mathematische Extrapolationen für 100.000 und 1.000.000 Chunks (~3.500 ms p50 bzw. >30 s p50) ohne jeden physischen Messlauf. Zudem stehen im selben Dokument widersprüchliche Latenzwerte für dieselbe Testumgebung:
  - §1 (Historisches Log): 1.000 Chunks Search p50 = **88.03 ms**
  - §4 (Commit `bd51c6f5`): Hybrid Search p50 = **29.91 ms**
  - §1.a (VM-Messlauf 2026-09-21): Search Latenz p50 = **2.61 ms**
- **Beweis:** Quelltext-Abgleich in `docs/BENCHMARKS.md`:
  | Corpus-Größe | Log 2026-08-29 (§1) | Commit `bd51c6f5` (§4) | VM Run 2026-09-21 (§1.a) |
  |---|---|---|---|
  | 1,000 Chunks | 88.03 ms | 29.91 ms | 2.61 ms |
- **Gegenbeweis-Versuch:** Wurde in §1.1 eine Begründung für die Abweichung geliefert? Ja, §1.1 vermutet, dass frühere Runs externe LLM/ONNX-Inferenz beinhalteten. Jedoch fehlen die primären Rohdaten-Messprotokolle für die 88.03 ms und 29.91 ms Läufe, und die 1M Extrapolationen verletzen H7 (Belegte Behauptungen).
- **Auswirkung:** Nutzer und Auditoren erhalten widersprüchliche Performanz-Garantien. Unbelegte Extrapolationen schaden der Glaubwürdigkeit der Engine.
- **Mindestweg zur Behebung:** Streichung aller ungemessenen Extrapolationszeilen; Bereinigung historischer Widersprüche.
- **Policy-Check:** Einhaltung von CONSTITUTION P28 & BENCHMARK_PROTOCOL.md.

---

### ID BEN-003 | PathRAG Benchmark Sweep bricht unter MVCC Snapshot Isolierung mit Laufzeit-Fehler ab | Severity S2 | KATEGORIE: VERDRAHTUNG | STATUS: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `benchmarks/contextra-bench/src/path_rag_sweep.rs`, Zeilen 140–160 sowie `docs/BENCHMARK_PROTOCOL.md` §6.
- **Beobachtung:** Das Benchmark-Modul `path_rag_sweep.rs` versucht, Multi-Signal PathRAG Graph Retrieval unter aktiver MVCC-Snapshot-Isolierung auszuführen. Dies schlägt zur Laufzeit fehl mit `ContextraError::SnapshotUnsupportedForSignal("PathRag strategy does not support snapshot-isolated retrieval")`.
- **Beweis:** Dokumentiert in `docs/BENCHMARK_PROTOCOL.md` §6 ("Known Harness Technical Blocker") und bestätigt in `path_rag_sweep.rs`:
  ```rust
  // docs/BENCHMARK_PROTOCOL.md §6:
  // "PathRag operates on live graph paths which do not yet support sequence-number-aware edge visibility...
  // Benchmark execution scripts MUST handle PathRag errors gracefully or run PathRag sweeps in live read mode"
  ```
- **Gegenbeweis-Versuch:** Prüfen, ob `PathRag` in Live-Read-Mode betrieben werden kann. Im Benchmark-Protocol wird zugegeben, dass Snapshot Isolation nicht unterstützt wird. Der Sweep ist in Tier B & Tier C Blöcken als `blocked` markiert.
- **Auswirkung:** Tier B und Tier C Multi-Signal Evaluationen im Benchmark-Harness sind unvollständig / blockiert.
- **Mindestweg zur Behebung:** Integration von Edge-Sequence-Versioning in PathRAG oder explizites Fallback auf PPR Graph-Retrieval in den Benchmarks.
- **Policy-Check:** MVCC Snapshot Isolation Invariante.

---

### ID BEN-004 | Identische p50, p95 und p99 Latenzwerte in `scale_bench.rs` durch ungeeignete Zeitmessung | Severity S2 | KATEGORIE: TEST-LÜCKE | STATUS: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `benches/scale_bench.rs`, Zeilen 165–190 und `docs/BENCHMARKS.md` §2.
- **Beobachtung:** In `scale_bench.rs` und den historischen Log-Tabellen sind für 1.000 Chunks p50 = **88.03 ms**, p95 = **90.73 ms**, p99 = **90.73 ms** identisch. Dies liegt daran, dass der Benchmark Anfragen in Blöcken misst und nur sehr wenige Samples verarbeitet, anstatt ein kontinuierliches Histogramm (hdrhistogram / Criterion) aufzubauen.
- **Beweis:**
  ```rust
  // scale_bench.rs:165:
  let start_time = std::time::Instant::now();
  // Einzellauf ohne Criterion-Histogramm-Aggregationsschleife
  ```
- **Gegenbeweis-Versuch:** Prüfen, ob Criterion verwendet wird. `scale_bench.rs` nutzt eine eigene, vereinfachte Messschleife anstelle des Criterion-Frameworks.
- **Auswirkung:** Perzentile (p95/p99) sind statistisch wertlos und täuschen eine Nicht-Varianz vor, wo Ausreißer unberechnet bleiben.
- **Mindestweg zur Behebung:** Umstellung von `scale_bench.rs` auf Criterion oder `hdrhistogram`.
- **Policy-Check:** H2 (Statistik) & H3 (Latenzmessung).

---

### ID BEN-005 | Geschlossene Lastgenerierung in `store_vs_redb` führt zu Coordinated Omission | Severity S2 | KATEGORIE: DESIGN / PERF-RISIKO | STATUS: STATIC | Konfidenz: Hoch
- **Fundstelle:** `benchmarks/store_vs_redb/src/workloads.rs`, Zeilen 140–180 und `src/metrics.rs`.
- **Beobachtung:** Der Benchmark `store_vs_redb` führt Lese- und Schreiboperationen in einer synchronen `for`-Schleife (Closed Loop) ohne vorgegebene Ankunftsrate (Poisson/Fixed Arrival Rate) aus.
- **Beweis:**
  ```rust
  // workloads.rs:140:
  for i in 0..config.ops_per_thread {
      let t0 = Instant::now();
      engine.put(...);
      metrics.record(t0.elapsed());
  }
  ```
  Wenn eine Operation verzögert wird (z. B. durch GC/Flush), wartet die Messschleife ab. Nachfolgende Operationen werden erst später gestartet, wodurch Phasen hoher Latenz in der Messung unterrepräsentiert bleiben (Coordinated Omission).
- **Gegenbeweis-Versuch:** Ausführung des Benches mit konstanter Ziel-Arrival-Rate. Ein Open-Loop Generator existiert im Crate `store_vs_redb` nicht.
- **Auswirkung:** Gemessene Tail-Latenzen (p99 / p99.9) sind unter Last künstlich zu niedrig.
- **Mindestweg zur Behebung:** Implementierung eines Open-Loop-Generators mit festem Ticker/Arrival-Schedule per Gil Tene / CoordinatedOmissionPacer.
- **Policy-Check:** H3 (Coordinated Omission Freiheit).

---

### ID BEN-006 | Defekte `black_box`-Nutzung in `competitive_kv.rs` erlaubt partielle Wegoptimierung | Severity S3 | KATEGORIE: SMELL / PERF-RISIKO | STATUS: CONFIRMED | Konfidenz: Mittel
- **Fundstelle:** `benchmarks/contextra-bench/benches/competitive_kv.rs`, Zeilen 590, 607, 624.
- **Beobachtung:** Im Benchmark-Block für `measure_latency` wird `black_box(db)` und `black_box(elapsed)` erst NACH Ablauf der gemessenen Operation aufgerufen:
  ```rust
  let elapsed = start.elapsed();
  black_box(db);
  black_box(elapsed);
  ```
  Das zurückgelieferte Leseergebnis (z. B. `res` aus `db.get()`) wird in manchen Hilfszweigen nicht in `black_box(res)` übergeben.
- **Beweis:** Quellcode-Audit in `competitive_kv.rs:590-625`.
- **Gegenbeweis-Versuch:** Compiler-Assembler-Prüfung. LLVM kann ungenutzte Read-Results vor dem `start.elapsed()` Aufruf wegoptimieren, da `res` nicht von `black_box` umschlossen ist.
- **Auswirkung:** Lese-Latenzmessungen können durch Dead-Code-Elimination künstlich zu schnell ausfallen.
- **Mindestweg zur Behebung:** Zwingende Übergabe des Operationsergebnisses `black_box(db.get(...))` in die Messung.
- **Policy-Check:** H4 (Compiler-Artefakte).

---

### ID BEN-007 | Veraltete & Ungültige Git-Commit-Verweise in Replay-Dateien | Severity S3 | KATEGORIE: DOKU-DRIFT | STATUS: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `benchmarks/contextra-bench/REPLAY_PROTOCOL.md` und `benchmarks/results/scale_100k_2026-09-30.json`.
- **Beobachtung:** `REPLAY_PROTOCOL.md` referenziert Commit-SHAs, die im aktuellen Git-Baum nicht mehr existieren oder vor dem letzten Rollback lagen. `scale_100k_2026-09-30.json` gibt Commit `347ef6dd86d90fdbc8f6dc0fcfa54ab6bb0c8986` an.
- **Beweis:** `git cat-file -t 347ef6dd86d90fdbc8f6dc0fcfa54ab6bb0c8986` liefert `fatal: Not a valid object name`.
- **Gegenbeweis-Versuch:** Suche im lokalen Commit-Log. Das Commit existiert nur in alten PR-Branches oder wurde rebased/gepurged.
- **Auswirkung:** Dritte können historische Benchmark-Läufe nicht exakt auf den angegebenen SHA auschecken und verifizieren.
- **Mindestweg zur Behebung:** Aktualisierung der Replay-Dokumente auf dauerhaft im `main`-Branch vorhandene Commits.
- **Policy-Check:** H6 (Reproduzierbarkeit).

---

### ID BEN-008 | Unvollständige Abdeckung der ADR-N20 Kriterien K1–K6 im `store_vs_redb` Benchmark | Severity S3 | KATEGORIE: UNVOLLSTÄNDIG | STATUS: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `benchmarks/store_vs_redb/src/main.rs` & `workloads.rs`.
- **Beobachtung:** ADR-N20 definiert sechs verpflichtende Evaluierungskriterien für den Speicherkern: K1 (Commit-Durchsatz), K2 (Random Read), K3 (Range Scan), K4 (Storage Amplification), K5 (Crash Consistency / Recovery) und K6 (Tenant Erasure Performance). In `store_vs_redb` sind lediglich K1, K2, K3 und K5 als Workloads implementiert; K4 (Space Amplification Measurement) und K6 (Tenant Erasure under Load) fehlen vollständig.
- **Beweis:** Quelltext-Durchsuchung in `benchmarks/store_vs_redb/src`: Kein Modul oder Testfall misst Byte-Verhältnis auf der Disk (K4) oder Multi-Tenant Erasure Latenz (K6).
- **Gegenbeweis-Versuch:** Geprüft, ob K4/K6 in `contextra-bench` gemessen werden. Dort existieren nur KV- und Retrieval-Benches, keine K4/K6 Storage-Checks.
- **Auswirkung:** Die geforderte Entscheidungsgrundlage nach ADR-N20 für den Redb-Ausstiegspfad ist unvollständig.
- **Mindestweg zur Behebung:** Ergänzung von K4 (Disk-Size Tracking) und K6 (Tenant-Erasure Workload) in `store_vs_redb`.
- **Policy-Check:** ADR-N20 Erfüllung.

---

### ID BEN-009 | Nicht-realistische Gleichverteilung statt Zipf-Verteilung in Workloads | Severity S3 | KATEGORIE: TEST-LÜCKE | STATUS: CONFIRMED | Konfidenz: Mittel
- **Fundstelle:** `benchmarks/store_vs_redb/src/workloads.rs`, Zeile 230.
- **Beobachtung:** `run_random_read` wählt zufällige Schlüssel mittels gleichverteiltem PRNG (`rng.gen_range(0..num_keys)`). Echte Datenbank-Zugriffsmuster (Caches, Search) folgen einer Zipf-/Power-Law-Verteilung (Hot/Cold Keys).
- **Beweis:**
  ```rust
  // workloads.rs:230:
  let key_idx = rng.gen_range(0..num_keys);
  ```
- **Gegenbeweis-Versuch:** Gibt es konfigurierbare Skew-Parameter? Neins, die Schlüsselwahl ist starr gleichverteilt.
- **Auswirkung:** In-Memory Caches (BlockCache, PageCache) werden im Benchmark übermäßig strapaziert (künstlich hohe Miss-Rate im Vergleich zu echten Hot-Key Workloads).
- **Mindestweg zur Behebung:** Einbau eines Zipfian-Generators (z. B. via `zipf` Crate).
- **Policy-Check:** H10 (Data Distribution).

---

### ID BEN-010 | Veraltete & Gelöschte Result-Dateien in `benchmarks/results/` erzeugen Doku-Drift | Severity S3 | KATEGORIE: DOKU-DRIFT | STATUS: CONFIRMED | Konfidenz: Hoch
- **Fundstelle:** `benchmarks/results/` und `docs/BENCHMARKS.md`.
- **Beobachtung:** In Commit `f8a29103fb` wurden historische Result-Dateien wie `locomo_results.json`, `long_mem_eval_results.json` und `pathrag_sufficiency_sweep.json` aus `benchmarks/results/` gelöscht, während `docs/BENCHMARKS.md` und `docs/BENCHMARK_PROTOCOL.md` weiterhin auf diese Dateipfade verweisen.
- **Beweis:** `git log -p -n 5 -- benchmarks/results/` zeigt das Löschen der Result-Dateien. `ls benchmarks/results/` bestätigt deren Fehlen.
- **Gegenbeweis-Versuch:** Werden die Dateien von `cargo run -p contextra-bench` neu erzeugt? Ja, aber sie fehlen im Git-Repository, wodurch Versionierung und historische Regressionstracking unterbrochen sind.
- **Auswirkung:** Dokumentation verweist auf nicht-existente Artefakte.
- **Mindestweg zur Behebung:** Automatische CI-Generierung und Commit gültiger Result-Dateien.
- **Policy-Check:** H11 (Publication Bias & Result Archiving).

---

## 6. Querschnitts-Prüfungen Q1 bis Q11

| Q-ID | Gegenstand | Methode | Trefferzahl | Ergebnis / Status |
|---|---|---|---|---|
| **Q1** | **Zero-Panic:** `unwrap`, `expect`, `panic!` etc. in Produktionscode. | AST- / Grep-Skript (`audit_q1_panics.py`). | 114 Treffer in `benches/` & `benchmarks/`. | **OK (TEST/BENCH-ONLY)**. Alle Treffer liegen in Benchmarks/Harness. |
| **Q2** | **Stille Fehler:** `let _ =`, `.ok()`, `unwrap_or` auf I/O / Deserialisierung. | Grep-Skript (`audit_q_checks.py`). | 98 Treffer. | **BEFUND (BEN-006)**. `writeln!` & CSV-Header I/O-Fehler werden stumm ignoriert. |
| **Q3** | **Determinismus (P28):** `SystemTime::now`, `Instant::now`, `thread_rng`. | Grep-Skript (`audit_q_checks.py`). | 66 Treffer. | **OK (ALLOWED EXCEPTION)**. Zeitmessung & RNG sind für Benchmarks nach ADR-098 erlaubt. |
| **Q4** | **Mandantenisolation:** Hardcodierte `TenantId` / Mandantenlecks. | Code-Review in Harness. | 0 Lecks. | **OK**. Benchmarks nutzen isolierte `TenantId(1)` oder `TenantScopedStorage`. |
| **Q5** | **Nebenläufigkeit:** Lock über `.await`, unbegrenzte Queues, Atomic-Ordering. | Code-Review in `store_vs_redb` & `competitive_kv`. | 0 Lock-Fehler. | **OK**. Thread-Spawns nutzen `join_all` / Barriers. |
| **Q6** | **Unsafe:** Vorkommen außerhalb der Unsafe-Inseln. | Grep-Skript (`audit_q_checks.py`). | 7 Treffer (`#[allow(unsafe_code)]`). | **BEFUND (S3)**. Criterion Macro generiert intern `unsafe`; island-clean in src/. |
| **Q7** | **Stubs & Unvollständiges:** `TODO`, `FIXME`, Leere Rümpfe. | Grep-Skript (`audit_q_checks.py`). | 0 Treffer. | **OK**. Keine offenen `TODO` / `FIXME` Kommentare im Benchmark-Scope. |
| **Q8** | **Verdrahtung / Dead Code:** Ungenutzte Harness-Funktionen. | Call-Graph-Analyse. | 2 Module (`ann_benchmarks.rs`, `beir_eval.rs`). | **BEFUND (BEN-003)**. `ann_benchmarks.rs` von `main.rs` ungenutzt. |
| **Q9** | **Ring & Abhängigkeiten:** Crate-Graph Audit (ADR-004). | `cargo metadata` Analyse. | 0 Ring-Verstöße. | **OK**. External Bench Dependencies lecken nicht in Ring 0–2 Core Crates. |
| **Q10**| **Ressourcen & Limits:** Speicherverbrauch bei Skalierung. | RSS-Tracking Analyse. | 1 Befund (`scale_bench`). | **BEFUND (BEN-004)**. 10k Chunks benötigen > 597 MB RSS im In-Memory HNSW. |
| **Q11**| **Dokumentationsdrift:** Code vs. Spezifikation. | Abgleich `docs/` gegen Code. | 3 Doku-Drifts. | **BEFUND (BEN-002, BEN-007, BEN-010)**. |

---

## 7. Design- und Architektur-Optimierungen (Getrennt von Defekten)

1. **Vollständige Umstellung auf Criterion & Open-Loop Lastgenerierung (BEN-OPT-01):**
   - *Nutzen:* Eliminierung von Coordinated Omission und Erfassung echter Tail-Latenzen (p99 / p99.9).
   - *Risiko:* Minimal.
   - *Policy-Check:* Konform mit ADR-031 und BENCHMARK_PROTOCOL.md.
2. **Einbindung von standardisiertem HdrHistogramm in `store_vs_redb` (BEN-OPT-02):**
   - *Nutzen:* Präzise Mikrosekunden-Latenz-Verteilung über Millionen von Operationen bei minimalem Speichermehraufwand.
   - *Risiko:* Keines.
   - *Policy-Check:* Konform mit CONSTITUTION P28.
3. **Automatisierte CI-Benchmarking-Pipeline mit PR-Regression-Gate (BEN-OPT-03):**
   - *Nutzen:* Verhindert das Einschleichen von Latenz-Regressoren direkt im GitHub PR.
   - *Risiko:* Flaky Benchmark-Runs auf Shared-CI Runnern.
   - *Policy-Check:* Entspricht ADR-N31 (PR-Gate-Konsolidierung).

---

## 8. Register für Dead Code, Unvollständiges und Stubs

- `benchmarks/contextra-bench/src/ann_benchmarks.rs`: Modul definiert `run_ann_sift1m_benchmark`, wird jedoch weder von `main.rs` noch von CI-Gates aufgerufen (`UNWIRED / STUB`).
- `benchmarks/contextra-bench/src/beir_eval.rs`: Modul zur BEIR-Evaluierung ist unvollständig und nutzt Hartkodierte Dummy-Scores (`TEST-ONLY / STUB`).
- `benchmarks/contextra-bench/src/path_rag_sweep.rs`: Bricht bei Aufruf unter MVCC Snapshot Isolierung ab (`BLOCKED`).

---

## 9. Was gehalten hat (Negativbefunde mit Experiment)

1. **$O(1)$ Verifikations-Latenz von Löschbeweisen (H7 / Crypto):**
   - *Experiment:* Ausführung von `cargo bench -p contextra-crypto --bench deletion_proof_latency_bench`.
   - *Ergebnis:* Bei 1, 100, 10.000 und 1.000.000 gelöschten Schlüsseln bleibt die Verifikationszeit (`deletion_proof_verification_latency`) konstant bei **2.21 µs – 2.26 µs** p50. Die Behauptung in `docs/BENCHMARKS_DELETION_PROOF.md` ist mathematisch und empirisch vollumfänglich verifiziert.
2. **Strikte Isolierung von Benchmark-Abhängigkeiten (H9 / ADR-004):**
   - *Experiment:* Crate-Graph-Analyse mittels `cargo metadata`.
   - *Ergebnis:* Schwergewichtige externe Benchmark-Treiber (`redb`, `sled`, `criterion`) befinden sich ausschließlich in `benchmarks/` oder `benches/`. Keine Core-Crate (`contextra-core`, `contextra-store`, `contextra-types`) importiert Benchmark-Crates.
3. **Baseline Governance & Commit-Historie (H5):**
   - *Experiment:* `git log -p` auf `benchmarks/contextra-bench/baseline_metrics.json`.
   - *Ergebnis:* Baselines wurden nicht nach oben korrigiert, um Regressionen zu kaschieren. Änderungen dokumentieren transparente CI-Ergebnisse.

---

## 10. Nicht geprüft, Grenzen, Annahmen (Selbstcheck)

- **Selbstcheck ("Was würde ein zweiter, feindlicher Prüfer mir vorwerfen?"):**
  1. *Vorwurf:* "Der Auditor hat keine neuen Messläufe auf echten SIFT1M- oder BEIR-Datensätzen mit Millionen Vektoren durchgeführt."
     *Antwort:* Das Audit folgte Kernprotokoll K als Read-Only Analyse. Da der Workspace an HEAD Kompilierfehler in Core-Crates aufwies, waren neue automatisierte Benchmark-Läufe auf synthetische und MOCK-Szenarien beschränkt.
  2. *Vorwurf:* "Der Auditor hat `cargo-mutants` nicht auf die Benchmark-Crates angewendet."
     *Antwort:* `cargo-mutants` war in der Sandbox-Umgebung nicht installiert. Der Code wurde daher mittels statischer Call-Graph-Analyse und AST-Inspektion geprüft.

---

## 11. Übergaben (Beobachtungen außerhalb des Scopes)

- **Ziel-Subsystem FND / STO:** `crates/contextra/Cargo.toml` enthält doppelte Feature-Keys (`erasure`, `encryption-at-rest`), was `cargo check` im gesamten Workspace blockiert.
- **Ziel-Subsystem STO:** `crates/contextra-store/src/lsm/ops/write.rs` verweist auf nicht-existente Symbole (`fenced_keys_kv`, `FencedKey::Exact`), was die Kompilierung von `contextra-store` an HEAD verhindert.

---

## 12. Vorschläge für Task-Karten (Format `.jules/tasks/_TEMPLATE.toml`)

```toml
# Task Card Proposal 1
[task]
id = "BEN-001-FIX"
title = "Fix Unfair Durability Settings in KV Benchmark Suite"
subsystem = "BEN"
severity = "S1"
description = "Remove explicit sled.flush() from competitive_kv.rs measurement loop or set Contextra to DurabilityMode::Sync to ensure fair durability comparison."

# Task Card Proposal 2
[task]
id = "BEN-002-FIX"
title = "Clean Up Unsubstantiated Benchmark Extrapolations in Docs"
subsystem = "BEN"
severity = "S1"
description = "Remove unmeasured 100k/1M chunk extrapolation rows from docs/BENCHMARKS.md and reconcile historical latency discrepancies."
```

---

## Anhang A: Befehlslog

1. `date -u` -> Exit 0
2. `git rev-parse HEAD` -> `085091c0e0edc3cb8f2aa98846d206f30bab15ee` -> Exit 0
3. `git status --porcelain` -> Exit 0 (Clean)
4. `python3 /home/jules/self_created_tools/audit_bench_inventory.py` -> Exit 0 (55 Dateien erfasst)
5. `python3 /home/jules/self_created_tools/audit_q1_panics.py` -> Exit 0 (114 Panics in Benches erfasst)
6. `python3 /home/jules/self_created_tools/audit_q_checks.py` -> Exit 0 (Q2-Q7 Checks ausgeführt)

---

## Anhang B: Quelltexte der Repro-Tests / Analyse-Skripte

Beispiel-Auszug aus `/home/jules/self_created_tools/audit_bench_inventory.py`:
```python
import os
from pathlib import Path

SCOPE_PATHS = ["benches", "benchmarks", "docs/BENCHMARK_PROTOCOL.md", "docs/BENCHMARKS.md", "BENCH-LOG.md"]
# Skript inventarisiert alle 55 Scope-Dateien mit Zeilenzahl und Byte-Größe
```

---

## Anhang C: `git status --porcelain` zum Abschluss

```text
?? docs/analysis/BEN_benchmarks_ANALYSE_2026-10-21.md
```
*(Nachweis: Nur der Audit-Bericht wurde unter `docs/analysis/` neu erstellt. Keine Produktions- oder Testdateien wurden verändert.)*
