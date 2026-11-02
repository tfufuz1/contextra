# Routine: Benchmark-Trend

- **Takt:** wöchentlich
- **Ziel:** Erkennen von Regressionen bei Latenz und Durchsatz. Schwellen nach SPEC 15.2: Verschlechterung über 10 % gegenüber der Basis auf derselben Hardware blockiert (BM-WRITE, BM-GET, BM-ERASE-LAT).
- **Eingaben:** `cargo bench` (Mikrobenchmarks). Der Makro-Harness `ctx-bench` folgt mit WP-6.1.
- **Harte Grenzen:** höchstens 15 Dateien in einem Crate je PR. Scope: `crates/*/benches/**`, `benchmarks/**`.
- **Stopp:** Schwankungen durch VM-Rauschen sind keine Regression. Ohne Hardware-Protokoll ist ein Lauf ungültig (P9, SPEC 14.8).
- **Ergebnis:** Bericht als Markdown-Artefakt. Ein PR mit Korrektur nur, wenn die Ursache im Code liegt.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).
