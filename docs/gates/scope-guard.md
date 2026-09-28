# Scope Guard Gate (`scope-guard`)

## Zweck
Stellt sicher, dass alle im Diff geänderten Dateien innerhalb des in der Task-Karte definierten `scope` liegen und keine `forbidden` Pfade berühren.

## Funktionsweise
- Liest die Task-Karte (`.jules/tasks/<id>.toml`) aus `--card`, Commit-Trailer `Task-Card: <id>` oder PR-Body (`PR_BODY` / `--body-file`).
- Modus `--card-mode required` (Default): Ohne Task-Karte schlägt das Gate mit Exit-Code 2 (Error) fehl.
- Modus `--card-mode optional`: Ohne Task-Karte wird `not_applicable` mit Warnung zurückgegeben.
- Vergleicht `git diff --name-status -z --find-renames base...head` mit `scope` (muss matchen) und `forbidden` (darf nicht matchen).
