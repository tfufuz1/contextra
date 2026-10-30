# Routine: Benchmark-Trend

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates, `cargo xtask` oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

- **Takt**: Wöchentlich
- **Ziel**: Erkennung von Performance-Degredationen und Regressionen in Latenz/Durchsatz.
- **Eingaben**:
  - `cargo run --manifest-path xtask/Cargo.toml -- bench-trend` <!-- harness:planned -->
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `benches/**`, `crates/*/**`.
- **Stopp-Bedingungen**: Benchmark-Schwankungen durch VM-Rauschen.
- **Ergebnisformat**: Benchmark-Bericht als Markdown-Artefakt oder PR mit Latenz-Fix.
- **Eskalation**: Siehe Architekturdokument §9.3.
