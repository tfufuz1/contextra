# Routine: Unwrap- & Allow-Abbau

- **Takt**: Wöchentlich
- **Ziel**: Kontinuierliche Reduzierung verbliebener `unwrap()`-Aufrufe und `#[allow(...)]`-Attribute im Workspace.
- **Eingaben**:
  - `cargo run --manifest-path xtask/Cargo.toml -- panic-inventory`
  - `cargo run --manifest-path xtask/Cargo.toml -- ratchet`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/**`.
- **Stopp-Bedingungen**: Wenn das Ersetzen eines Unwraps API-breaking Changes erzwingt.
- **Ergebnisformat**: PR mit ersetzten Unwraps durch saubere `Result`-Fehlerbehandlung.
- **Eskalation**: Siehe Architekturdokument §9.3.
