# Routine: Chaos- & Fuzz-Langlauf

- **Takt**: Nächtlich (02:00 UTC)
- **Ziel**: Automatisierter Ausführungslauf der Chaos-Matrix und Fuzz-Smoke-Tests.
- **Eingaben**:
  - `.github/workflows/chaos.yml`
  - `cargo run --manifest-path xtask/Cargo.toml -- fuzz-smoke`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/**`.
- **Stopp-Bedingungen**: Chaos-Crash durch Hardware/OS-Simulationseinschränkung.
- **Ergebnisformat**: Issue bei Fuzz/Chaos-Fund mit Repro-Artifacts.
- **Eskalation**: Siehe Architekturdokument §9.3.
