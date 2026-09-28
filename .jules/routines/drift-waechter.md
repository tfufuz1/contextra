# Routine: Drift-Wächter

- **Takt**: Täglich
- **Ziel**: Früherkennung von Doku-, Typ- und Registrierungs-Drifts im Repository.
- **Eingaben**:
  - `cargo run --manifest-path xtask/Cargo.toml -- sync-docs --check`
  - `cargo run --manifest-path xtask/Cargo.toml -- check-agents-freshness`
  - `cargo run --manifest-path xtask/Cargo.toml -- check-jules-context-freshness`
  - `cargo run --manifest-path xtask/Cargo.toml -- check-flatbuffers-drift` <!-- harness:planned -->
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `docs/**`, `capabilities.toml`, `AGENTS.md`.
- **Stopp-Bedingungen**: Bei manuell geänderten generierten Dateien ohne Quellcode-Anpassung.
- **Ergebnisformat**: PR mit aktualisierten Doku-Artefakten oder GitHub Issue bei unklaren Quellcode-Drifts.
- **Eskalation**: Siehe Architekturdokument §9.3.
