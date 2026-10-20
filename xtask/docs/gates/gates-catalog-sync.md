# Gate: Gate Catalog Sync (`gates-catalog-sync`)

Das Gate `gates-catalog-sync` stellt sicher, dass der zentrale Gate-Katalog `xtask/config/governance/gates.toml` vollständig synchron mit den lokalen Ausführungsphasen (`.jules/harness-phases.toml`) und den CI-Verdicts (`xtask/config/governance/verdict-required.toml`) bleibt.

## Funktionsweise
1. Liest `xtask/config/governance/gates.toml` als Single Source of Truth für Gate-Metadaten.
2. Gleicht `required_local` mit `.jules/harness-phases.toml` ab.
3. Gleicht `required_ci` mit `xtask/config/governance/verdict-required.toml` ab.
4. Prüft im Hinweis-Modus Aufrufe in `justfile` und `.github/workflows/`.

## Ausführung
```bash
cargo xtask gates-catalog-sync [--json]
```
