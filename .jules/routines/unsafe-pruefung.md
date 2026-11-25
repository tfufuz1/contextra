# Routine: Unsafe-Prüfung

- **Takt**: Wöchentlich
- **Ziel**: Verifikation aller Unsafe-Inseln auf Gültigkeit der SAFETY-Kommentare und Miri-Sauberkeit.
- **Eingaben**:
  - `cargo run --manifest-path xtask/Cargo.toml -- check-unsafe-islands`
  - `cargo run --manifest-path xtask/Cargo.toml -- unsafe-audit`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/**`.
- **Stopp-Bedingungen**: Miri-Undefined-Behavior im Fremdcode ohne direkten Fix.
- **Ergebnisformat**: Audit-Report im PR-Text mit Miri-Verifikationsprotokoll.
- **Eskalation**: Siehe Architekturdokument §9.3.
