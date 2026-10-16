# AUDIT-BERICHT: Performance, Benchmarking & Regressions-Workflows

**Rolle:** S13 - Spezialist für unabhängiges Benchmarking und Performance-Regression
**Scope:** `benches/`, `benchmarks/`, `xtask-heavy/`, Benchmark-Dokumentation (`docs/BENCHMARKS.md`, `docs/BENCH-LOG.md`, `docs/BENCHMARK_PROTOCOL.md`) und Regressions-Workflows (`rules/benchmarking.md`).
**Commit:** `f9429808`
**Datum:** 2026-10-14
**Delta-Basis:** `docs/analysis/AUDIT_benchmarks_DEEP_2026-10-13.md` (Commit `1ee08ec2`, 2026-10-13)

---

## 1 Zusammenfassung & Ampel

| Prüfbereich | Status | Kurzbeschreibung |
|---|---|---|
| **1. Methodik & Last** | 🔴 ROT | Geschlossene Lastschleifen (Closed Loop) in allen Microbenchmarks; anfällig für Coordinated Omission; fehlende HdrHistogram-Nutzung in Core-Benches. |
| **2. Fairness & Systeme** | 🟡 GELB | Vergleiche in `competitive_kv.rs` und `competitive_bench.rs` konfigurieren Fsync/Durability ad-hoc uneinheitlich; Wettbewerber (RocksDB, faiss) fehlen. |
| **3. Regressionsgrenze** | 🟡 GELB | Regressions-Gate vergleicht skalar Punktwerte anstelle statistischer Konfidenzintervalle / Hypothesentests. Mismatch `--tolerance`/`--threshold` in `xtask-heavy/src/bench_gate.rs` durch PR #4913 behoben. |
| **4. Budgets & Abdeckung** | 🔴 ROT | Kompilation blockiert durch Syntaxfehler in `contextra-store`; kritische Pfade (Compaction, Recovery-Zeit, Purge-Dauer, HNSW-Aufbau) ungebenchet. |
| **5. Messfehler & Umgebung** | 🟡 GELB | `cargo xtask env-attest` attestiert nur Toolchain, aber keine CPU-Governor/Pinning/Frequency-Skalierung; `cargo-fuzz` fehlt in Sandbox. |
| **6. Schulden & Drift** | 🔴 ROT | Doku-Drift in `docs/BENCHMARKS.md` (veraltete Werten neben neuen VM-Daten); Benches kompilieren nicht wegen Ring-1 Kompilierfehler. |

---

## 2 Delta zum letzten Bericht (`docs/analysis/AUDIT_benchmarks_DEEP_2026-10-13.md`)

- **BEHOBEN (1):**
  - `FINDING-BENCH-01` (Argument-Mismatch `--tolerance` vs `--threshold` in `xtask-heavy/src/bench_gate.rs`): In PR #4913 wurde `"--threshold" | "--tolerance"` in `bench_gate.rs:32` ergänzt.
  - `check-bandit-latency-budget`: Veralteter Bandit-Router-Benchmark aus `xtask-heavy` entfernt (Prävention von Drift nach Router-Refactoring).
- **NEU (2):**
  - **`FINDING-S13-20261014-01` (P0):** Syntaxfehler `unclosed delimiter` in `crates/contextra-store/src/erasure_ledger/mod.rs:938` blockiert die Ausführung aller Benchmarks (`cargo check --benches` und `cargo xtask bench-gate` scheitern mit Exit Code 101).
  - **`FINDING-S13-20261014-02` (P2):** Doku-Drift in `crates/contextra-vector/AGENTS.md` und `crates/contextra-types/AGENTS.md` bezüglich geforderter aber nicht existierender Xtask-Befehle (aufgedeckt via `cargo xtask doc-truth`).
- **OFFEN / UNVERÄNDERT (5):**
  - `FINDING-BENCH-03` (Coordinated Omission in Microbenchmarks)
  - `FINDING-BENCH-04` (Regressions-Gate ohne Konfidenzintervalle)
  - `FINDING-BENCH-05` (Unvollständiges Environment-Attestment bzgl. CPU-Governor)
  - `FINDING-BENCH-06` (Doku-Drift in `docs/BENCHMARKS.md`)
  - `scale_bench` Transaction Staging Exceeded bei 10k Chunks.
- **VERSCHLECHTERT (0):** Keine Verschlechterungen im Scope `benches/` und `benchmarks/`.

---

## 3 Befunde

### FINDING-S13-20261014-01: Blockade aller Benchmarks durch Syntaxfehler in Ring 1 (`contextra-store`)
- **Schwere:** P0 (Absturz / Build-Blockade)
- **Datei:Zeile:** `crates/contextra-store/src/erasure_ledger/mod.rs:938`
- **Beleg:** `cargo check --benches` liefert: `error: this file contains an unclosed delimiter ... impl ErasureLedger {` (Exit Code 101).
- **Risiko:** Weder Microbenchmarks in `benches/` noch CI-Regressions-Gates (`cargo xtask bench-gate`) können ausgeführt oder validiert werden.
- **Empfehlung:** Behebung des Delimiter-Fehlers in `erasure_ledger/mod.rs` durch das Storage-Team.
- **Aufwand:** S

### FINDING-BENCH-03: Coordinated Omission in primären Microbenchmarks
- **Schwere:** P1 (Falsche Messergebnisse unter Last)
- **Datei:Zeile:** `benches/scale_bench.rs:180-210`, `benches/rrf_scale_bench.rs:80-120`
- **Beleg:** Latenzen werden in geschlossenen Schleifen (`b.iter(|| ... )`) gemessen. Bei GC-/Compaction-Spitzen blockiert der Client, wodurch Folgeanfragen verzögert und p99/p99.9-Latenzen fälschlicherweise zu niedrig ausgewiesen werden.
- **Risiko:** Vorhersagen zur Produktions-Latenz unter offener Last (Open Loop) sind nicht verlässlich.
- **Empfehlung:** Erzeugung offener Lasten mit fester Ankunftsrate (Poisson/Uniform) und Aufzeichnung via `hdrhistogram::Histogram`.
- **Aufwand:** M

### FINDING-BENCH-04: Regressions-Gate vergleicht rein Punktwerte ohne Konfidenzintervalle
- **Schwere:** P2 (Nichtdeterminismus / Flakiness im Gate)
- **Datei:Zeile:** `benchmarks/contextra-bench/src/compare.rs:115-160`
- **Beleg:** `compare_single_metric` prüft `(baseline - current) / baseline > threshold`. Statistisches Rauschen oder Varianzen zwischen Testläufen werden nicht berücksichtigt.
- **Risiko:** False Positives / False Negatives bei CI-Regressionsprüfungen.
- **Empfehlung:** Erweitere `CombinedMetrics` um Standardabweichung/Stichprobengröße und nutze Welch's t-Test für Signifikanzentscheidungen.
- **Aufwand:** M

### FINDING-BENCH-05: Hardware- und Kernel-Environment nicht im Attestment erfasst
- **Schwere:** P2 (Messfehler durch Host-Varianzen)
- **Datei:Zeile:** `xtask/src/env_attest.rs:1-120`
- **Beleg:** `cargo xtask env-attest` verifiziert Toolchain und `flatc`, erfasst aber weder CPU-Scaling Governor (`powersave` vs `performance`) noch NUMA/Core-Affinität.
- **Risiko:** Inkonsistente Testergebnisse zwischen lokalen Entwickler-VMs und CI-Runnern.
- **Empfehlung:** Erweiterung von `env-attest` zur Warnung bei aktiver CPU-Frequenzskalierung (`powersave`).
- **Aufwand:** S

### FINDING-BENCH-06: Doku-Drift in `docs/BENCHMARKS.md`
- **Schwere:** P2 (Doku-Drift)
- **Datei:Zeile:** `docs/BENCHMARKS.md:20-60`
- **Beleg:** Durchgestrichene alte Messwerte (88 ms) stehen direkt neben VM-Ergebnissen (2,61 ms), was Verwirrung bezüglich der aktuellen Leistungsdaten stifte.
- **Risiko:** Verwechselung von historischen und aktuellen Performance-Baselines.
- **Empfehlung:** Archivierung historischer Tabellen in `docs/archive/` und Reduktion von `BENCHMARKS.md` auf verifizierte Stände.
- **Aufwand:** S

---

## 4 Testlücken

1. **Compaction & Write-Amplication Benchmark:**
   - **Eingaben:** 100.000 kontinuierliche Updates/Deletes im LSM-Store.
   - **Ablauf:** Messung der Latenzausreißer (p99.9) während aktiver Background-Compaction.
   - **Erwartete Invariante:** Leselatenz p99 bleibt unter 20 ms während Compaction.
2. **Recovery-Zeit & WAL-Replay Benchmark:**
   - **Eingaben:** Unclean Shutdown nach 1.000.000 uncommitted WAL-Records.
   - **Ablauf:** Replay-Dauer beim Neustart (`LsmStorage::open`).
   - **Erwartete Invariante:** Startzeit skaliert linear mit uncommitted Segmenten und bleibt < 2,0 s.
3. **Physical Purge & Erasure Scope Benchmark:**
   - **Eingaben:** Löschbefehl über 10.000 Dokumente in einem Scope.
   - **Ablauf:** Messung der Ausführungsdauer von `purge_keys` und `OrxPurgeIntent` Statusübergängen.
   - **Erwartete Invariante:** Complete Physical Purge schließt in < 5,0 s ab.

---

## 5 Technische Schulden

1. **Inkonsistente Durability-Konfiguration in Wettbewerber-Benches:**
   - **Typ:** Fairness / Methodik
   - **Ort:** `benchmarks/contextra-bench/benches/competitive_kv.rs:40-120`
   - **Aufwand:** S
   - **Reihenfolge:** 1
2. **Duplizierter Datengenerator-Code:**
   - **Typ:** Duplikat
   - **Ort:** `benches/competitive_bench.rs` vs `benchmarks/contextra-bench/benches/competitive_kv.rs`
   - **Aufwand:** S
   - **Reihenfolge:** 2

---

## 6 Hypothesen

1. **Hypothese 1:** Der Staging-Buffer-Fail bei 10.000 Chunks in `scale_bench.rs` wird durch exzessives Cloning von Vectors in MVCC Read-View Invalidation verursacht.
   - *Klärender Test:* Profiling von `scale_bench` unter `flamegraph` / `hotspot-report` sobald Ring-1 wieder kompiliert.

---

## 7 Quellenkonflikte und Doku-Drift

- **Geforderte Scan-Befehle aus Rolleninstruktion vs `cargo xtask harness-list`:**
  - Die Rolleninstruktion fordert die Ausführung von `bench-compile`, `bench-gate`, `bench-trend`, `hotspot-report`, `check-recall-stability` und `reproducible-build-check`.
  - **Befund:** Bis auf `bench-gate` existieren diese Befehle derzeit **nicht** im `xtask`-Registry-Katalog (`xtask/registry.toml` / `harness-list`).
  - **Status:** Doku-Drift / Geplante Befehle.
- **`doc-truth` Befunde in AGENTS.md:**
  - `crates/contextra-vector/AGENTS.md` und `crates/contextra-types/AGENTS.md` referenzieren Kommandos wie `cargo xtask check-unsafe-islands` oder `cargo xtask check-agents-integrity`, die nicht direkt als Subcommand existieren.

---

## 8 Ausgeführte Befehle & Exit-Codes

| Befehl | Exit-Code | Grund / Ergebnis |
|---|---|---|
| `cargo xtask harness-list` | `0` | Identifikation registrierter Harness-Befehle |
| `cargo run -q --manifest-path xtask/Cargo.toml -- env-attest` | `1` | Toolchain-Attestierung (cargo-fuzz fehlt in Sandbox-System) |
| `cargo xtask doc-truth` | `1` | Doku-Truth Prüfung deckt veraltete Befehlsreferenzen auf |
| `cargo check --benches` | `101` | Fehlgeschlagen wegen Syntaxfehler in `contextra-store/src/erasure_ledger/mod.rs:938` |
| `git diff 1ee08ec2..HEAD -- stat ...` | `0` | Delta-Inspektion seit Vorgänger-Audit |

---

## 9 Nicht Geprüftes und Gründe

1. **Schwere Criterion Benchmarks (`scale_bench`, `competitive_bench`):**
   - **Grund:** Nicht ausführbar / kompilierbar aufgrund des Syntaxfehlers in Ring 1 (`contextra-store`).
2. **Reale ANN SIFT1M / BEIR Latencies via `external-benchmarks`:**
   - **Grund:** Erfordert Datensatz-Downloads via Internet (`bench-download`), was in der isolierten Sandbox ohne Internetzugang nicht möglich ist.
