# Governance Rule: Dependency Hygiene & Supply Chain Trust (`rules/dependencies.md`)

## Gilt für
- `Cargo.toml` (Workspace Root)
- `deny.toml`
- `supply-chain/`

## Pflichtregeln
1. **Verbot unangemeldeter Crates**: Es dürfen keine neuen externen Dependencies ohne vorheriges Security-Review und `cargo-vet` Auditierung hinzugefügt werden (Quelle: `supply-chain/config.toml`, `docs/supply-chain-trust.md`).
2. **Strict Ring Dependency Limits**: Abhängigkeiten dürfen nur von höheren Ringen zu niedrigeren Ringen verlaufen (z.B. Ring 3 -> Ring 0/1/2). Zirkuläre Abhängigkeiten oder Upward-Dependencies sind untersagt (Quelle: `capabilities.toml`, `xtask/src/check_ring_layering.rs`).
3. **Preventing Tokio Masking in Ring 0**: Die `wrappers`-Allowlist in `deny.toml` ist streng reglementiert, um zu verhindern, dass Ring-0/1-Crates transitive `tokio`-Dependencies verbergen (Quelle: `deny.toml`).

## Häufige Fehler
- Hinzufügen von Hilfs-Crates in Ring-0/1 ohne Überprüfung der transitiven Abhängigkeiten.
- Umgehung von `cargo deny` Lints bei Lizenzen oder Security Advisories.

## Verweise
- `deny.toml`
- `capabilities.toml`
- `docs/supply-chain-trust.md`
