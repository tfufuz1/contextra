# Routine: Unwrap- & Allow-Abbau

> **Status 30.10.2026 (Reduktion):** Diese Regel gilt weiter, soweit sie nicht auf entfernte Crates oder entfernte Routinen verweist. Solche Stellen sind veraltet. Maßgeblich: `AGENTS.md` und `docs/SPEC.md` (Fassung 4.0).

- **Takt**: Wöchentlich
- **Ziel**: Kontinuierliche Reduzierung verbliebener `unwrap()`-Aufrufe und `#[allow(...)]`-Attribute im Workspace.
- **Eingaben**:
  - `cargo clippy --workspace --all-targets --locked`
- **Harte Grenzen**: Max. 15 Dateien / 1 Crate pro PR. Scope-Globs: `crates/*/**`.
- **Stopp-Bedingungen**: Wenn das Ersetzen eines Unwraps API-breaking Changes erzwingt.
- **Ergebnisformat**: PR mit ersetzten Unwraps durch saubere `Result`-Fehlerbehandlung.
- **Eskalation**: Siehe Architekturdokument §9.3.
