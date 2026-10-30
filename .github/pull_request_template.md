## Arbeitspaket

- WP-Nummer (aus `docs/PLAN.md`): 
- Phase: R0 / R1 / R2 / R3 / R4 / R5 / R6

## Änderung

<!-- Was wurde geändert und warum? Verweis auf SPEC- und PLAN-Abschnitt. -->

## Prüfung

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace` (bzw. betroffene Crates)
- [ ] Ring-Regeln eingehalten (`tests/layering.rs`)
- [ ] Keine neuen `unsafe`-Blöcke außerhalb `contextra-sys` und `contextra-simd`
- [ ] Keine Lint-Ausnahmen (`#[allow]`) auf Modulebene ohne ADR
- [ ] Dokumentation angepasst (SPEC, PLAN, README, betroffene `AGENTS.md`)

## Risiko

<!-- Welche Teile könnten brechen? Welche Tests decken sie ab? -->
