# AGENTS.md — contextra-bench
> Tooling · experimental · Quelle: capabilities.toml · Spec: Anhang C

## 1. Zweck
Eigenständiges, reproduzierbares Benchmark-Harness zur Messung von Retrieval-Genauigkeit und Performance.
Evaluiert Recall@K, MRR, RRF-Fusion und Reranking über LoCoMo, BEIR und synthetische Datensätze.
Bietet Regressions-Gates und Baseline-Vergleiche für CI-Sicherheitsprüfungen (`compare_baseline`).

## 2. Modul-Karte

| Datei / Verzeichnis | Verantwortung |
|---|---|
| `lib.rs` | Öffentliche Benchmark-Module und Re-Exports |
| `main.rs` | CLI-Einstiegspunkt für Benchmark-Ausführungen |
| `regression_gate.rs` | Auswertung von Qualitätsabfällen gegen gespeicherte Baselines |
| `compare.rs` | Vergleichs-Logik für Metrik-Abweichungen (`compare_metrics`) |
| `locomo.rs` | Benchmark-Evaluator für den LoCoMo-Datensatz |
| `beir_eval.rs` | BEIR Benchmark-Evaluator |
| `ann_benchmarks.rs` | Evaluierung von Vektor-Suchindizes (HNSW, Quantisierung) |
| `long_mem_eval.rs` | Langzeit-Gedächtnis-Evaluierung |
| `path_rag_sweep.rs` | RAG-Sweep-Evaluierung über Graphpfade |
| `bin/compare_baseline.rs`, `bin/scale_100k.rs` | Executables für Baseline-Vergleiche und 100k-Skalierungstests |
| `benches/` | Criterion-Benchmarks (`competitive_kv.rs`, `locomo.rs`, `long_mem_eval.rs`) |

## 3. Invarianten

- **INV-BENCH-DETERMINISM**: Alle Benchmark-Läufe auf synthetischen Corpora nutzen deterministische Ground-Truth-Datensätze.
  *Prüfung*: `cargo test -p contextra-bench --lib`
- **INV-BENCH-NO-NAN**: Metrik-Vergleiche prüfen strikt auf `NaN` / `Inf` Abweichungen und schlagen fehl, wenn ungültige Werte auftreten.
  *Prüfung*: `cargo test -p contextra-bench --lib`
- **INV-FORBID-UNSAFE**: Strikte Einhaltung von `#![forbid(unsafe_code)]`.
  *Prüfung*: `cargo check -p contextra-bench`

## 4. Verboten / Anti-Patterns

- **Keine ungeprüften Regressions-Toleranzen**: Schwellenwerte für Metrik-Abfälle dürfen nicht ohne CI-Benachrichtigung angepasst werden.
- **Keine nicht-reproduzierbaren Zufallsdaten**: Zufallsgeneratoren in Benchmarks müssen mit festen Seeds initialisiert werden.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Benchmark-Ausführungen nutzen synchrone oder asynchrone Multi-Thread-Konfigurationen je nach Evaluator.
- Keine gegenseitigen Blockaden zwischen Datensatz-Loader und Messschleife.

## 6. Verifikation

```bash
cargo test -p contextra-bench
cargo check -p contextra-bench
cargo xtask check-agents-integrity
cargo xtask bench-gate --tolerance 0.05
```
- Dokumentation und Historie siehe `docs/BENCHMARKS.md` und `docs/BENCH-LOG.md`.

## 7. Bekannte Lücken / SOLL

- Kein Eintrag `[crates.contextra-bench]` in `capabilities.toml` vorhanden (als Tooling-Verzeichnis geführt).
