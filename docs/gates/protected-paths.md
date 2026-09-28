# Gate: Protected Paths (`protected-paths`)

## Übersicht
Das Gate `protected-paths` stellt sicher, dass Änderungen an geschützten Pfaden (wie `.github/**`, `xtask/**`, `governance/**`, `capabilities.toml`, `AGENTS.md`) kontrolliert erfolgen.

## Regeln & Invarianten
- Alle geänderten Dateien zwischen `--base` und `--head` werden analysiert (inkl. Umbenennungen, bei denen alter und neuer Pfad geprüft werden).
- Ist ein Pfad durch `governance/protected-paths.toml` geschützt, schlägt das Gate fehl.

## Ausnahmeregelung
Eine Änderung ist nur zulässig, wenn:
1. Jeder Commit im Diff den Trailer `Protected-Change: ADR-NNN` enthält.
2. In `docs/decisions/` ein entsprechendes ADR existiert.
3. Die Umgebungsvariable `PR_LABELS` das Label `protected-change` enthält.
