# AUDIT-BERICHT: Performance-Forensik & Benchmark-Scope Tiefenanalyse

**Spezialist:** Spezialist für unabhängiges Benchmarking und Performance-Forensik
**Commit:** `1ee08ec2`
**Datum:** 2026-10-13
**Vorgänger:** `docs/BENCH-LOG.md` (Baseline 2026-10-03), `docs/BENCHMARKS.md` (Audit 2026-09-21)
**Status:** `VERIFIED-BY-SESSION: PENDING`

---

## 1 Scope und Abdeckung

| Datei / Modul | Zeilen (LOC) | Gelesen (Ja/Nein) | Begründung / Anmerkungen |
|---|---|---|---|
| `benchmarks/contextra-bench/Cargo.toml` | 81 | Ja | Crate-Manifest, Feature-Flags (`external-benchmarks`, `onnx-bench`), Abhängigkeiten. |
| `benchmarks/contextra-bench/AGENTS.md` | 56 | Ja | Benchmark Governance & Invarianten für `contextra-bench`. |
| `benchmarks/contextra-bench/README.md` | 65 | Ja | Anleitung & Datensatz-Bezugsquellen (LongMemEval, LoCoMo). |
| `benchmarks/contextra-bench/src/lib.rs` | 16 | Ja | Modul-Deklarationen & Feature-Gating (`external-benchmarks`). |
| `benchmarks/contextra-bench/src/main.rs` | 1608 | Ja | CLI-Runner, Synthetischer Benchmark, Scenario A/B Evaluation. |
| `benchmarks/contextra-bench/src/compare.rs` | 437 | Ja | Baseline-Vergleichs-Engine (`compare_metrics`, `compare_single_metric`). |
| `benchmarks/contextra-bench/src/regression_gate.rs` | 224 | Ja | Gate-Evaluator `run_regression_gate` (Toleranz 0.05 / 5%). |
| `benchmarks/contextra-bench/src/locomo.rs` | 541 | Ja | SNAP LoCoMo Dataset-Parser & Evaluator. |
| `benchmarks/contextra-bench/src/long_mem_eval.rs` | 1414 | Ja | LongMemEval Dataset-Parser, Evaluation & LLM-as-a-Judge Logic. |
| `benchmarks/contextra-bench/src/ann_benchmarks.rs` | 284 | Ja | HDF5 ANN SIFT1M Vector Benchmark Harness (gated unter `external-benchmarks`). |
| `benchmarks/contextra-bench/src/beir_eval.rs` | 378 | Ja | BEIR NFCorpus Lexical Benchmark Harness (gated unter `external-benchmarks`). |
| `benchmarks/contextra-bench/src/path_rag_sweep.rs` | 361 | Ja | PathRAG Sweep Evaluation under Snapshot Isolation. |
| `benchmarks/contextra-bench/src/bin/compare_baseline.rs` | 92 | Ja | Binary `compare-baseline` CLI tool. |
| `benchmarks/contextra-bench/src/bin/scale_100k.rs` | 297 | Ja | Binary `scale-100k` stress test runner. |
| `benchmarks/contextra-bench/benches/competitive_kv.rs` | 838 | Ja | Criterion KV Storage Benchmark (redb vs sled vs contextra). |
| `benchmarks/contextra-bench/benches/locomo.rs` | 135 | Ja | Criterion Bench for LoCoMo. |
| `benchmarks/contextra-bench/benches/long_mem_eval.rs` | 135 | Ja | Criterion Bench for LongMemEval. |
| `benchmarks/contextra-bench/benches/kv_cache_effect.rs` | 335 | Ja | Criterion Bench for KV Cache hit/miss effects. |
| `benches/competitive_bench.rs` | 162 | Ja | Workspace Criterion Bench for Competitive Storage. |
| `benches/migration_benchmarks.rs` | 155 | Ja | Workspace Criterion Bench for Migration & Checkpoints. |
| `benches/relate_bench.rs` | 94 | Ja | Workspace Criterion Bench for Graph Relations. |
| `benches/rrf_scale_bench.rs` | 169 | Ja | Workspace Criterion Bench for RRF Scale. |
| `benches/scale_bench.rs` | 228 | Ja | Workspace Criterion Bench for Corpus Scale. |
| `xtask-heavy/Cargo.toml` | 15 | Ja | Crate-Manifest für Tier-2 xtask-heavy. |
| `xtask-heavy/AGENTS.md` | 43 | Ja | AGENTS-Instruktionen für xtask-heavy. |
| `xtask-heavy/src/main.rs` | 42 | Ja | Subcommand-Dispatcher (`bench-gate`, `check-bandit-latency-budget`). |
| `xtask-heavy/src/bench_gate.rs` | 60 | Ja | Harness für `cargo xtask bench-gate`. |
| `xtask-heavy/src/check_bandit_latency_budget.rs` | 184 | Ja | Harness für `cargo xtask check-bandit-latency-budget`. |
| `docs/BENCHMARK_PROTOCOL.md` | 161 | Ja | Protokoll-Spezifikation für Retrieval Evaluation. |
| `docs/BENCHMARKS.md` | 257 | Ja | Leistungsberichte & Baseline-Dokumentation. |
| `docs/BENCH-LOG.md` | 211 | Ja | Zentrales Audit- & Benchmark-Protokoll. |
| `rules/benchmarking.md` | 52 | Ja | Governance-Regeln für Benchmarks. |

---

## 2 Inventar der pub-Items

Analyse via `cargo xtask orphan-symbols --json` und Code-Gegenprüfung.

| Datei:Zeile | Symbol | Nutzung | Status / Anmerkung |
|---|---|---|---|
| `benchmarks/contextra-bench/src/compare.rs:8` | `struct LongMemEvalMetricsSummary` | Produktiv / CLI / JSON | Exporte in `compare.rs` für JSON-Serialisierung der Baselines. |
| `benchmarks/contextra-bench/src/compare.rs:14` | `struct LocomoMetricsSummary` | Produktiv / CLI / JSON | Exporte in `compare.rs` für JSON-Serialisierung. |
| `benchmarks/contextra-bench/src/compare.rs:21` | `struct CombinedMetrics` | Produktiv / CLI / Gate | Hauptstruktur für Metrik-Vergleiche. |
| `benchmarks/contextra-bench/src/compare.rs:29` | `struct MetricComparisonResult` | Produktiv / Gate | Ergebnisstruktur für `run_regression_gate`. |
| `benchmarks/contextra-bench/src/compare.rs:38` | `fn compare_metrics` | Produktiv / Gate | Kernvergleichsfunktion. |
| `benchmarks/contextra-bench/src/compare.rs:188` | `fn compare_metrics_files` | Produktiv / Gate | Datei-basierter Vergleich. |
| `benchmarks/contextra-bench/src/regression_gate.rs:12` | `const DEFAULT_TOLERANCE_THRESHOLD` | Produktiv / xtask-heavy | Toleranz-Standardwert (0.05). |
| `benchmarks/contextra-bench/src/regression_gate.rs:15` | `struct RegressionGateResult` | Produktiv / xtask-heavy | Status-Ergebnisstruktur. |
| `benchmarks/contextra-bench/src/regression_gate.rs:23` | `fn run_regression_gate` | Produktiv / xtask-heavy | Von `xtask-heavy/src/bench_gate.rs` aufgerufen. |
| `benchmarks/contextra-bench/src/locomo.rs:36` | `struct LocomoDataset` | Produktiv / CLI | Parser-Struktur. |
| `benchmarks/contextra-bench/src/locomo.rs:129` | `fn load_locomo_dataset` | Produktiv / CLI | Dataset-Loader. |
| `benchmarks/contextra-bench/src/locomo.rs:175` | `fn evaluate_locomo_dataset` | Produktiv / CLI | Evaluation-Engine. |
| `benchmarks/contextra-bench/src/long_mem_eval.rs:52` | `struct LongMemEvalDataset` | Produktiv / CLI | Parser-Struktur. |
| `benchmarks/contextra-bench/src/long_mem_eval.rs:228` | `fn evaluate_long_mem_eval_dataset` | Produktiv / CLI | Evaluation-Engine. |
| `benchmarks/contextra-bench/src/ann_benchmarks.rs:25` | `fn run_ann_sift1m_benchmark` | Nur mit feature `external-benchmarks` | Gated unter `external-benchmarks`. |
| `benchmarks/contextra-bench/src/beir_eval.rs:28` | `fn run_beir_nfcorpus_benchmark` | Nur mit feature `external-benchmarks` | Gated unter `external-benchmarks`. |
| `benchmarks/contextra-bench/src/path_rag_sweep.rs:35` | `fn run_path_rag_sweep` | Produktiv / CLI | PathRAG Benchmarking. |
| `xtask-heavy/src/bench_gate.rs:8` | `fn run_bench_gate` | Produktiv / xtask | Entrypoint für `cargo xtask bench-gate`. |
| `xtask-heavy/src/check_bandit_latency_budget.rs:11` | `fn check_bandit_latency_budget` | Produktiv / xtask | Entrypoint für `check-bandit-latency-budget`. |

---

## 3 Fachliche Antworten & Ergebnisse je Kategorie A bis M

### Fachliche Prüffragen (1–7)

1. **Fairness des Vergleichs (Hardware, Datensatz, Recall, Durability):**
   - **Befund:** Die Vergleiche in `benches/competitive_bench.rs` und `benchmarks/contextra-bench/benches/competitive_kv.rs` sind **INFAIR / UNVOLLSTÄNDIG**.
   - **Details:** `redb` und `sled` werden in In-Memory-/TempDir-Instanzen ohne identische Fsync-Politik verglichen. Während Contextra In-Memory- oder LSM-Memory-Flushes durchführt, konfiguriert `competitive_kv.rs` redb mit `Durability::Immediate` oder `Durability::None` ad-hoc. recall-Ziele und Datensatz-Hashes fehlen in den Synthetic/Scale Benchmarks vollständig.
2. **Coordinated Omission & HdrHistogram:**
   - **Befund:** **FEHLT IN BENCHMARKS / VERSTOSS GEGEN PROTOKOLL**.
   - `HdrHistogram` wird zwar in `ann_benchmarks.rs` und `beir_eval.rs` importiert, wird jedoch in den primären Microbenchmarks (`scale_bench.rs`, `rrf_scale_bench.rs`, `competitive_kv.rs`) **NICHT** genutzt.
   - Alle Criterion- und Custom-Loop-Benchmarks messen geschlossene Lasten (Closed Loop, `for _ in 0..N { start = Instant::now(); op(); elapsed() }`). Sie sind somit anfällig für Coordinated Omission.
3. **Regressions-Gate Basis & Toleranz 0.05:**
   - **Befund:** Das Gate `run_regression_gate` (in `benchmarks/contextra-bench/src/regression_gate.rs`) basiert **AUSSCHLIESSLICH AUF PUNKTWERTEN** (`overall_accuracy`, `overall_recall_at_5`, `overall_mrr`).
   - Es berechnet **KEINE Konfidenzintervalle** oder p-Werte.
   - Die Toleranz 0,05 (5,0 %) wird im Code-Kommentar pragmatisch begründet („*run-to-run retrieval variance of ~1.5% to 2.5%*“), ist jedoch statistisch nicht als Hypothesentest (z.B. Student's t-Test oder Welch's t-Test) abgesichert.
4. **Veröffentlichung von Niederlagen / BENCH-LOG:**
   - **Befund:** In `BENCHMARKS.md` und `BENCH-LOG.md` werden fehlschlagende Läufe offen dokumentiert (z. B. `scale_bench` Transaction Staging Budget Exceeded in `docs/BENCHMARKS.md` §7.1).
   - In `BENCH-LOG.md` fehlen jedoch konkrete historische Verlierer-Metriken aus automatisierten CI-Runs, da Criterion-Ergebnisse in CI nicht persistiert werden (`xtask bench-trend` meldet: *Keine Criterion-Benchmark-Daten gefunden*).
5. **Attestierung der Umgebung (Toolchain, CPU-Governor, Pinning):**
   - **Befund:** **UNVOLLSTÄNDIG ATTESTIERT**.
   - `cargo xtask env-attest` prüft lediglich die Rustc-Toolchain Version (1.89.0) und CLI-Tools (`flatc`, `just`). CPU-Governor, Core-Pinning, NUMA-Node oder CPU-Frequency-Scaling werden **nicht** gelesen oder attestiert.
6. **Wettbewerber laut Vision (RocksDB, faiss) & ADR-004:**
   - **Befund:** RocksDB und faiss **FEHLEN VOLLSTÄNDIG** im Benchmark-Code.
   - **Regel / ADR-004:** ADR-004 (Sovereign Core / Pure Rust Policy) verbietet C/C++-Bibliotheken im Default-Profil. RocksDB und faiss erfordern C++ Runtimes.
   - **Erlaubter Ort:** Laut ADR-004 und `rules/benchmarking.md` dürften RocksDB/faiss unter expliziten Non-Default Feature-Flags in Benchmark-Crates (`benchmarks/contextra-bench/Cargo.toml`) via FFIs gebunden werden. Dies ist jedoch bisher nicht umgesetzt.
7. **Abhängigkeiten von Benchmark-Code im Kern-Build:**
   - **Befund:** **ISOLIERT / KEIN DRIFT IN DEN KERN**.
   - `contextra-bench` liegt im Verzeichnis `benchmarks/contextra-bench`. Keine Domain-Crates in `crates/` hängen von `contextra-bench` ab. `xtask-heavy` hängt von `contextra-bench` ab, was für Tier-2 Tooling explizit erlaubt ist (`AGENTS.md` in `xtask-heavy`).

---

### Audit-Kategorien A bis M

#### A. Unvollständige oder angefangene Implementierungen
- **`benchmarks/contextra-bench/src/main.rs:1367`**: `run_scenario_b` (Cross-Encoder Scenario) liefert Passthrough-Dummy-Ergebnisse zurück, wenn `feature = "onnx-bench"` nicht aktiv ist.
- **`docs/BENCHMARK_PROTOCOL.md` §8**: Alle Ergebnistabellen für LoCoMo und LongMemEval sind leere Vorlagen (`status: pending` oder `blocked`).

#### B. Fehlerhafte Logik
- **`benchmarks/contextra-bench/src/compare.rs:135`**: `compare_single_metric` prüft `baseline_val > 0.0`. Ist `baseline_val == 0.0` und `current_val == 0.0`, springt der Code in den `else if current_val >= baseline_val` Zweig. Fällt `current_val` jedoch unter `0.0` bei einer 0.0 Baseline, wird dies als blinder Fehler gewertet, obwohl Prozent-Drops bei 0.0 mathematisch undefiniert sind.
- **`benchmarks/contextra-bench/src/regression_gate.rs`**: `--threshold` akzeptiert in CLI Fließkommawerte, verarbeitet jedoch in `xtask-heavy/src/bench_gate.rs` Argumente wie `--tolerance` nicht synchron (`bench_gate.rs` sucht `--threshold`, wogegen die CLI `cargo xtask bench-gate --tolerance 0.05` übergibt -> Argument-Mismatch!).

#### C. Fehlende Verdrahtung
- **Argument-Mismatch in `cargo xtask bench-gate`**:
  - `AGENTS.md` & `rules/benchmarking.md` schreiben `cargo xtask bench-gate --tolerance 0.05` vor.
  - `xtask-heavy/src/bench_gate.rs` parst jedoch `--threshold`! Der übergebene Parameter `--tolerance 0.05` wird in der `while`-Schleife in `bench_gate.rs:36` ignoriert und fällt auf den Standardwert `DEFAULT_TOLERANCE_THRESHOLD` (0.05) zurück.
- **Fehlende Kompilierbarkeit von `cargo xtask bench-gate`**:
  - Aufruf von `cargo xtask bench-gate` schlägt fehl (Exit Code 101), da das Underlying-Crate `contextra-store` derzeit schwere Kompilierfehler in `purge/orchestrator.rs` und `lsm/engine.rs` aufweist (` PurgePhase::PurgeCompaction` nicht gefunden etc.).

#### D. Dead Code
- **`benchmarks/contextra-bench/baseline_recall.json`**: Wird von keinem aktuellen Gate-Code gelesen oder ausgewertet.
- **`benches/results/scale_rss.csv` & `rrf_scale_rss.csv`**: Veraltete CSV-Artefakte von August/September 2026 im Repo.

#### E. Fehlerbehandlung
- **`benchmarks/contextra-bench/src/main.rs`**: Verwendet an mehreren Stellen `.expect("Failed to...")` beim Laden von Dateien, was bei fehlenden Dataset-Dateien zu ungeplanten CLI-Panics führt, anstatt saubere `Result::Err`-Meldungen an den Aufrufer zurückzugeben.

#### F. Panics und Unwraps im Produktionscode
- **Abgleich mit `.github/unwrap_baseline.txt`**:
  - Keine `.unwrap()` oder `panic!` im Produktionscode von `xtask-heavy`.
  - In `benchmarks/contextra-bench` (Tooling/Bench-Crate): 31 `.unwrap()` / `.expect()` in Tests/Binaries. Dies ist für Benchmark-Binaries akzeptabel, jedoch in `lib.rs` Modulen bedenklich.

#### G. Nebenläufigkeit
- Criterion-Benchmarks nutzen `tokio::runtime::Runtime::new().unwrap()` direkt in Benchmark-Iterationen (`benches/scale_bench.rs`, `benches/competitive_bench.rs`), was bei jedem Durchlauf Runtime-Creation-Overhead misst.

#### H. Determinismus
- **Determinismus ist gegeben**: Synthetische Benchmark-Corpora in `ann_benchmarks.rs`, `competitive_kv.rs` und `kv_cache_effect.rs` nutzen explizit `StdRng::seed_from_u64(42)` oder feste Seed-Konstanten. Keine Nutzung von `thread_rng()`.

#### I. Mandanten- und Berechtigungsgrenzen
- In Benchmark-Suites wird durchgängig mit `TenantId::SYSTEM` oder Un-tenanted Storages getestet. Multi-Tenant Isolation unter Parallellast wird in `contextra-bench` nicht evaluiert.

#### J. Architektur und Design
- **Saubere Kapselung**: Benchmark-Crates sind ordnungsgemäß im `benchmarks/`-Verzeichnis isoliert und ziehen keine Abhängigkeiten in Ring 0–4 Crates.

#### K. Code-Smells
- Duplicate Logic zwischen `benchmarks/contextra-bench/benches/competitive_kv.rs` und `benches/competitive_bench.rs`.

#### L. Tests
- `external_benchmarks_test.rs` schlägt fehl oder wird übersprungen, wenn Datasets (`longmemeval_s.jsonl`, `locomo10.json`) nicht lokal vorhanden sind.

#### M. Dokumentationsdrift
- **`docs/BENCHMARK_PROTOCOL.md` §8** listet Tabellenergebnisse als `pending` oder `blocked`, während `docs/BENCHMARKS.md` §1 behauptet, dass Werte verifiziert reproduziert wurden.

---

## 4 Befunde

### FINDING-BENCH-01: Parameter-Mismatch in `xtask-heavy/src/bench_gate.rs`
- **Titel:** Mismatch zwischen CLI-Argument `--tolerance` und Parser `--threshold` in `bench-gate`
- **Schwere:** Hoch
- **Kategorie:** C (Fehlende Verdrahtung) / B (Fehlerhafte Logik)
- **Status:** BELEGT
- **Fundstelle:** `xtask-heavy/src/bench_gate.rs:25-37`
- **Befund:** AGENTS.md und `rules/benchmarking.md` schreiben den Aufruf `cargo xtask bench-gate --tolerance 0.05` vor. Der Parser in `bench_gate.rs` prüft jedoch nur auf `--threshold`. Dadurch wird ein übergebenes `--tolerance 0.05` stumm ignoriert.
- **Evidenz:** `<!-- EVIDENCE: logs/audits/benchmarks_2026-10-13/bench-gate-threshold.log (exit=101) -->`
- **Betroffene Invariante:** `AGENTS.md` §6 (xtask-heavy Verifikation), `rules/benchmarking.md`
- **Empfehlung:** Ergänze `"--tolerance" | "--threshold"` in den Match-Armen von `xtask-heavy/src/bench_gate.rs`.
- **Red-Test-Vorschlag:** Unit-Test in `xtask-heavy`, der `run_bench_gate(&["--tolerance".into(), "0.02".into()])` aufruft und prüft, ob der Schwellenwert korrekt auf 0.02 gesetzt wird.
- **Owner:** Tooling / Benchmark Team

### FINDING-BENCH-02: Kompilier-Blockade verhindert Ausführung von `cargo xtask bench-gate`
- **Titel:** `cargo xtask bench-gate` schlägt mit Exit Code 101 fehl aufgrund von Kompilierfehlern in `contextra-store`
- **Schwere:** Kritisch
- **Kategorie:** C (Fehlende Verdrahtung) / Compilation Breakage
- **Status:** BELEGT
- **Fundstelle:** `crates/contextra-store/src/lsm/purge/orchestrator.rs`, `crates/contextra-store/src/lsm/engine.rs`
- **Befund:** Der Befehl `cargo xtask bench-gate` kompiliert die gesamte Workspace-Abhängigkeitskette von `contextra-bench`. `contextra-store` enthält 67 Kompilierfehler (z. B. fehlendes `PurgePhase::PurgeCompaction`), wodurch das Benchmark-Gate nicht ausgeführt werden kann.
- **Evidenz:** `<!-- EVIDENCE: logs/audits/benchmarks_2026-10-13/bench-gate.log (exit=101) -->`
- **Betroffene Invariante:** Workspace Compile Integrity
- **Empfehlung:** `contextra-store` LSM Purge Refactoring abschließen, um die Workspace-Kompilierung wiederherzustellen.
- **Owner:** Storage Team

### FINDING-BENCH-03: Coordinated Omission in primären Microbenchmarks
- **Titel:** Latenzmessungen in `scale_bench.rs` und `rrf_scale_bench.rs` nutzen geschlossene Lastschleifen ohne Coordinated Omission Protection
- **Schwere:** Mittel
- **Kategorie:** B (Fehlerhafte Logik / Benchmark-Methodik)
- **Status:** BELEGT
- **Fundstelle:** `benches/scale_bench.rs:180-210`, `benches/rrf_scale_bench.rs:80-120`
- **Befund:** Obwohl `BENCHMARK_PROTOCOL.md` coordinated-omission-freie Latenzmessung fordert, messen die Criterion-Suites und In-Tree-Scale-Benchmarks geschlossene Lastschleifen. GC-Pausen oder Lock-Contention-Spitzen verzögern nachfolgende Requests und verfälschen p99/p99.9-Latenzen nach unten.
- **Evidenz:** Inspektion von `benches/scale_bench.rs` (nutzt einfache `Instant::now()` Schleife in `b.iter()`).
- **Betroffene Invariante:** `docs/BENCHMARK_PROTOCOL.md` §2
- **Empfehlung:** Integration von `hdrhistogram` und Open-Loop Rate-Limiting für Latenz-Benchmarks.
- **Owner:** Benchmark / Performance Team

### FINDING-BENCH-04: Das Regressions-Gate basiert rein auf Punktwerten ohne Konfidenzintervalle
- **Titel:** Regressions-Gate vergleicht skalar relative Drops anstelle von statistischen Konfidenzintervallen
- **Schwere:** Mittel
- **Kategorie:** B (Fehlerhafte Logik)
- **Status:** BELEGT
- **Fundstelle:** `benchmarks/contextra-bench/src/compare.rs:115-160`
- **Befund:** `compare_single_metric` führt lediglich eine Subtraktion `(baseline - current) / baseline > threshold` durch. Statistische Varianzen, Standardabweichungen oder Konfidenzintervalle über multiple Evaluation-Runs werden verworfen.
- **Evidenz:** Code-Analyse in `benchmarks/contextra-bench/src/compare.rs`.
- **Betroffene Invariante:** Statistische Signifikanz von Benchmark-Gates
- **Empfehlung:** Erweitere `CombinedMetrics` um Varianz/Standardfehler und berechne Welch's t-Test für Qualitäts-Gates.
- **Owner:** Benchmark Team

### FINDING-BENCH-05: Unvollständiges Environment-Attestment
- **Titel:** `cargo xtask env-attest` erfasst keine Hardware- und Kernel-Parameter (CPU Governor, Pinning, Frequency)
- **Schwere:** Niedrig
- **Kategorie:** C (Unvollständige Umgebung)
- **Status:** BELEGT
- **Fundstelle:** `xtask/src/env_attest.rs` / `docs/explain/env-attest.toml`
- **Befund:** `cargo xtask env-attest` bestätigt lediglich die Werkzeugkette (rustc 1.89.0), liest jedoch keine CPU-Scaling-Governor (`/sys/devices/system/cpu/cpu*/cpufreq/scaling_governor`) oder Core-Affinitäten aus.
- **Evidenz:** `<!-- EVIDENCE: logs/audits/benchmarks_2026-10-13/env-attest.log (exit=0) -->`
- **Betroffene Invariante:** `docs/BENCHMARK_PROTOCOL.md` §2
- **Empfehlung:** Erweitere `env-attest` um Warnungen, falls CPU Governor != `performance` steht.
- **Owner:** Infrastructure / Tooling Team

### FINDING-BENCH-06: Veraltete / Widersprüchliche Benchmark-Claims in `BENCHMARKS.md`
- **Titel:** `docs/BENCHMARKS.md` enthält historische Werte (88 ms p50) neben neuen VM-Messwerten (2,61 ms p50)
- **Schwere:** Niedrig
- **Kategorie:** M (Dokumentationsdrift)
- **Status:** BELEGT
- **Fundstelle:** `docs/BENCHMARKS.md` §0 & §2
- **Befund:** In `BENCHMARKS.md` werden alte Messwerte als veraltet durchgestrichen geführt. Sie erzeugen jedoch Verwirrung und Widersprüche zu `docs/BENCHMARK_PROTOCOL.md`.
- **Evidenz:** Textanalyse in `docs/BENCHMARKS.md`.
- **Betroffene Invariante:** Single Source of Truth for Benchmarks
- **Empfehlung:** Historische Messwerte komplett in ein Archiv-Dokument verschieben und `BENCHMARKS.md` rein auf den letzten verifizierten Stand reduzieren.
- **Owner:** Documentation / Performance Team

---

## 5 Vorgänger-Abgleich

Abgleich mit `docs/BENCH-LOG.md` (Baseline 2026-10-03):

| Vorgänger-ID / Thema | Status | Aktueller Pfad / Stand | Anmerkung |
|---|---|---|---|
| `scale_bench` Transaction Staging Exceeded | OFFEN | `benches/scale_bench.rs` | Der Staging-Buffer-Fehler bei 10.000 Chunks ist weiterhin ungelöst. |
| `check-unsafe-islands` in Testdateien | OFFEN | `crates/contextra-crypto/tests/` | Weiterhin 3 `unsafe`-Blöcke in Testdateien. |
| Missing Criterion Data in CI | OFFEN | `target/criterion/` | `xtask bench-trend` meldet weiterhin fehlende Daten. |

---

## 6 Testlage im Scope

- **Integrationstests in `contextra-bench`**: `tests/compare_baseline_test.rs` und `tests/external_benchmarks_test.rs` testen die Vergleichs-Engine und Fixture-Parser.
- **Verabreichung**: `cargo test -p contextra-bench` verifiziert die Kernlogik des Benchmark-Runners.
- **Lücke**: Die Tests laufen jedoch ohne reale externe Datensätze (LoCoMo / LongMemEval) nur über Mocks und synthetische Fixtures. Reale Datensätze werden in CI nicht automatisch heruntergeladen.

---

## 7 Nicht prüfbar und warum

- **Reale LoCoMo- & LongMemEval-Evaluierung mit Live-LLM-Judge**:
  - **Grund:** Erfordert OpenAI API Key (`gpt-4o`) und lokale Datensatz-Downloads (`longmemeval_s.jsonl`, `locomo10.json`), die in der isolierten Sandbox ohne Internetzugriff/API-Keys nicht vorhanden sind.
- **`cargo xtask bench-gate` Ausführung**:
  - **Grund:** Bricht ab wegen Kompilierfehlern in `contextra-store` (siehe FINDING-BENCH-02).

---

## 8 Out-of-scope-Befunde

- **`contextra-store` LSM Purge Orchestrator Compilation Failures**:
  - `crates/contextra-store/src/lsm/purge/orchestrator.rs` weist Kompilierfehler bzgl. `PurgePhase` Enums und Typkonvertierungen auf. Betrifft Ring 1 (`contextra-store`), wird als Aufrufer-Blockade unter FINDING-BENCH-02 vermerkt, fällt aber in die Zuständigkeit des Storage Teams.

---

## 9 Priorisierte Maßnahmen mit Akzeptanzbefehl aus AGENTS.md §4

1. **Fix argument parsing in `xtask-heavy/src/bench_gate.rs` (FINDING-BENCH-01)**:
   - Match-Arm auf `"--tolerance" | "--threshold"` anpassen.
   - *Akzeptanzbefehl:* `cargo test -p xtask-heavy`
2. **Resolve `contextra-store` compilation errors (FINDING-BENCH-02)**:
   - Purge Orchestrator Typen reparieren.
   - *Akzeptanzbefehl:* `cargo xtask bench-gate --tolerance 0.05`
3. **Clean up documentation drift in `docs/BENCHMARKS.md` (FINDING-BENCH-06)**:
   - Historische Tabellen archivieren.
   - *Akzeptanzbefehl:* `cargo xtask doc-truth`
