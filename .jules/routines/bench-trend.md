# Routine: Benchmark-Trend

- **Takt**: Wöchentlich
- **Ziel**: Erkennung von Performance-Degredationen und Regressionen in Latenz/Durchsatz.
- **Eingaben**:
  - `cargo run --manifest-path xtask/Cargo.toml -- bench-trend`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `benches/**`, `crates/*/**`.
- **Stopp-Bedingungen**: Benchmark-Schwankungen durch VM-Rauschen.
- **Ergebnisformat**: Benchmark-Bericht als Markdown-Artefakt oder PR mit Latenz-Fix.
- **Eskalation**: Siehe Architekturdokument §9.3.
