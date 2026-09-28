# Gate: Gate Weakening (`gate-weakening`)

## Übersicht
Das Gate `gate-weakening` analysiert `git diff -U0`, um das Abschwächen von CI-Pipelines, Rust-Lints, Testabdeckungen, Baselines oder Sicherheitsrichtlinien zu erkennen.

## Regelkatalog (IDs GW-001 bis GW-016)
- **Workflows:** Entfernen von `-D warnings`, Hinzufügen von `continue-on-error: true`, `|| true`, `--retries` > 0, Entfernen von `needs:`, `if: false`, Senken von Schwellenwerten (coverage, threshold, min, score; ID GW-015).
- **Rust Code:** Neue `#[allow]`, `#[ignore]`, Netto-Entfernen von `#[test]` oder `assert*!`, Entfernen von `#![forbid(unsafe_code)]` oder `#![deny(`.
- **Cargo & Lints:** Herabstufen von Lint-Leveln in `[workspace.lints]` / `[lints]`, Einfügen von Ausnahmen in `deny.toml`.
- **Baselines:** Erhöhung von `.github/unwrap_baseline.txt` oder Werten in `governance/ratchet.toml` (ID GW-016).

## Ausnahmeregelung
Erfordert den Trailer `Gate-Weakening: ADR-NNN` sowie eine existierende ADR-Datei in `docs/decisions/`.
