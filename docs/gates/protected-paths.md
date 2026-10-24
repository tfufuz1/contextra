# Gate: Protected Paths (`protected-paths`)

## Übersicht
Das Gate `protected-paths` stellt sicher, dass Änderungen an geschützten Pfaden (wie `.github/**`, `xtask/**`, `governance/**`, `capabilities.toml`, `AGENTS.md`) kontrolliert erfolgen und nicht unbefugt oder fehlerhaft freigegeben werden.

## Regeln & Invarianten
- Alle geänderten Dateien zwischen `--base` und `--head` werden analysiert (inkl. Umbenennungen, bei denen alter und neuer Pfad geprüft werden).
- Ist ein Pfad durch `governance/protected-paths.toml` geschützt, erfordert die Änderung eine gültige ADR-Ausnahme.
- **Fail-Closed:** Fehlen Basis-Commit, ADR-Datei oder ist das Repository flach geklont, wird die Ausführung mit Exit Code 2 blockiert.

## Ausnahmeregelung (Protected Change)
Eine Änderung an geschützten Pfaden ist **nur** zulässig, wenn:
1. Jeder Commit im Bereich (`base..head`) den Trailer `Protected-Change: ADR-NNN` enthält.
2. Die referenzierte ADR-Datei auf dem **Basis-Commit** bereits existiert (`git ls-tree <base> docs/decisions`).
3. Die ADR-Datei auf dem Basis-Commit den Status `accepted` (oder `final`) aufweist (`Status:` / `**Status**:`-Feld).
4. Die ADR-Datei **nicht** im selben PR/Diff (`git diff base...head`) neu angelegt oder modifiziert wurde.
5. Die Umgebungsvariable `PR_LABELS` das Label `protected-change` enthält.

## Exit Codes
- **0 (PASS):** Keine geschützten Pfade verändert ODER alle Änderungen sind durch eine valide ADR auf dem Basis-Commit sowie PR-Label legitimiert.
- **1 (FAIL):** Regelverstoß (z. B. geschützter Pfad verändert ohne Trailer, ADR nicht auf Basis-Commit, ADR im selben PR verändert, Status nicht accepted, PR-Label fehlt).
- **2 (ERROR):** Git-Laufzeitfehler oder Umgebungsfehler (z. B. flaches Repository / shallow clone, Basis-Commit nicht in Git auflösbar).

## Fehlermeldungen & Ursachen
- `"Flaches Repository erkannt (shallow repository). Bitte führe 'git fetch --unshallow' bzw. 'git fetch origin main' aus."` -> Git Repo in CI/Lokal ist shallow.
- `"Basis-Commit '<base>' konnte in Git nicht aufgelöst werden..."` -> `origin/main` fehlt lokal.
- `"ADR-Datei für 'ADR-NNN' nicht auf Basis-Commit '<base>' in docs/decisions/ gefunden."` -> ADR existiert nicht auf der Basis.
- `"ADR-Datei '<path>' wurde im selben PR angelegt oder verändert..."` -> Versuch der Selbst-Freigabe im PR.
- `"ADR 'ADR-NNN' auf Basis-Commit hat Status 'proposed', erforderlich ist 'accepted' oder 'final'."` -> Ungültiger ADR-Status.
- `"PR-Label 'protected-change' fehlt in PR_LABELS."` -> Label im Pull Request fehlt.

## Test-Fixtures (`xtask/tests/harness_protected_paths_test.rs`)
1. **ADR im selben PR angelegt:** Fails (Exit 1)
2. **ADR auf Basis, Status accepted:** Passes (Exit 0)
3. **ADR auf Basis, Status proposed:** Fails (Exit 1)
4. **ADR auf Basis, aber im PR verändert:** Fails (Exit 1)
5. **Trailer verweist auf nicht vorhandene ADR:** Fails (Exit 1)
6. **Kein geschützter Pfad berührt:** Passes (Exit 0) ohne Trailer
7. **Shallow Repository:** Fails with Exit 2 and unshallow instruction
