# Routine: Flake-Jagd

- **Takt**: Nächtlich
- **Ziel**: Aufspüren und Beheben nicht-deterministischer Test-Flakes.
- **Eingaben**:
  - `cargo run --manifest-path xtask/Cargo.toml -- flake-report --runs 10`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/tests/**`.
- **Stopp-Bedingungen**: Flake durch externe Umgebungseinflüsse außerhalb des Repos.
- **Ergebnisformat**: Flake-Issue oder PR mit Fix für Ursachen von Race-Conditions/Timeouts.
- **Eskalation**: Siehe Architekturdokument §9.3.
