# Routine: Dependency-Hygiene

- **Takt**: Wöchentlich
- **Ziel**: Prüfung von Lizenzen, Sicherheitsberichten und Supply-Chain-Audits neuer/aktualisierter Abhängigkeiten.
- **Eingaben**:
  - `cargo deny check licenses`
  - `cargo vet`
  - `cargo run --manifest-path xtask/Cargo.toml -- dependency-gate`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `Cargo.toml`, `Cargo.lock`, `supply-chain/**`, `deny.toml`.
- **Stopp-Bedingungen**: Unzulässige Lizenz ohne geschäftlich freigegebene Ausnahme.
- **Ergebnisformat**: PR mit aktualisierten Vet-Audits oder Dependency-Fixes.
- **Eskalation**: Siehe Architekturdokument §9.3.
