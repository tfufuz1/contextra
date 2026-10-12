# Governance Rule: Benchmarking & Performance Regression Standards (`rules/benchmarking.md`)

## Gilt für
- `benchmarks/`
- `benchmarks/contextra-bench`
- Performance-sensitive Crate-Module (`contextra-store`, `contextra-vector`, `contextra-rank`, `contextra-db`)

## Pflichtregeln
1. **Automatisierter Datensatz-Download**: Vor der Ausführung schwerer Benchmarks (ANN SIFT1M, BEIR nfcorpus, ONNX Test-Modelle) müssen Datensätze via `bench-download` geladen werden (Quelle: `xtask/src/bench_download.rs`).
2. **Kein manuelles Manipulieren von Baselines**: Jules darf NIE `baseline_metrics.json` oder gespeicherte Criterion-Referenzen manuell editieren oder fälschen, um ein fehlschlagendes Benchmark-Gate zu "ergrünen".
3. **Deterministische Messungen**: Alle synthetischen Test-Corpora müssen feste Random-Seeds nutzen (`INV-BENCH-DETERMINISM`).
4. **Belegpflicht im PR-Body**: Performance-Ergebnisse müssen im PR-Body im Abschnitt `## Verifikation` mit vorher/nachher Messwerten exakt nachgewiesen werden.
5. **Entwickler-VM Grenzwerte**: Lokale Benchmark-Läufe in der CI/Sandbox müssen Zeit- und Arbeitsspeichergrenzen beachten. Exzessive Swaps oder Netzwerk-Downloads außerhalb von `bench-download` sind unzulässig.

## Datensätze & Downloads

| Datensatz | Typ / Quelle | Zielpfad |
|---|---|---|
| `AnnSift1m` | SIFT-128 Euclidean (`ann-benchmarks.com`) | `benchmarks/data/sift-128-euclidean.hdf5` |
| `Beir` | NFCorpus (`public.ukp.informatik.tu-darmstadt.de`) | `benchmarks/data/nfcorpus.zip` |
| `OnnxTestModel` | Tiny BertModel + Tokenizer (`huggingface.co`) | `crates/contextra-infer-onnx/tests/fixtures/` |

## Befehle

```bash
# Benchmark-Datensätze herunterladen
cargo run -q --manifest-path xtask/Cargo.toml -- bench-download --dataset all

# Benchmark-Regressionen prüfen (gegen Baseline mit ±5% Toleranz)
cargo run -q --manifest-path xtask-heavy/Cargo.toml -- bench-gate --threshold 0.05

# Benchmark-Delta analysieren via just
just bench-delta

# Baseline offiziell aktualisieren (nur nach freigegebenem Performance-Architektur-Review)
just bench-set-baseline
```

## Verbotenes Verhalten
- Manuelles Anpassen von Schwellenwerten oder `baseline_metrics.json`, um fehlschlagende Prüfungen zu verschleiern.
- Nutzung nicht-deterministischer Zufallsdaten ohne festen Seed in Benchmark-Schleifen.
- Ignorieren von `NaN` oder `Inf` Werten in Metrik-Vergleichen (`INV-BENCH-NO-NAN`).

## Häufige Fehlerbilder
- `BENCH-GATE ERROR`: Leistungsabfall überschreitet die zugelassene Toleranzschwelle (Standard: 5.0%).
- `curl failed downloading`: Netzwerkfehler oder geänderte Upstream-URL beim Herunterladen von Test-Datensätzen.

## Verweise
- `benchmarks/README.md`
- `benchmarks/contextra-bench/AGENTS.md`
- `xtask/src/bench_download.rs`
- `xtask-heavy/src/bench_gate.rs`
