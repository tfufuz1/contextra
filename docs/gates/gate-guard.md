# Aggregate Gate Guard (`gate-guard`)

## Zweck
Fasst die sicherheits- und umfangsrelevanten Qualitäts-Gates (`scope-guard`, `protected-paths`, `gate-weakening`, `diff-budget`) zu einer einheitlichen Ausführungseinheit zusammen.

## Bestandteile
1. `scope-guard`: Prüft Einhaltung von Scope und Forbidden-Pfaden aus der Task-Karte.
2. `protected-paths`: Erzwingt Schutzregeln für geschützte Systempfade.
3. `gate-weakening`: Erkennt versuchte Gate- und Lint-Abschwächungen im Diff.
4. `diff-budget`: Überprüft Diff-Größe gegen Task-Karten- und System-Budgets.

## CLI-Syntax & Trigger
```bash
cargo xtask gate-guard [--root <dir>] [--base <rev>] [--head <rev>] [--card <file>] [--changed|--all] [--json]
```
- Flaggensemantik: `--changed` (Standard; merge-base mit origin/main) | `--all` | `--json`.
- Exit-Codes: `0` bei allen Bestandteilen OK/SKIP, `1` bei jedem FAIL.
