# Postmortem: [PM-0001] Commit Integrity Guard (Phantom Commit)

## Vorfall
Ein KI-Agent erzeugte einen LEEREN Commit ohne jegliche Code-Änderungen, versah diesen jedoch mit einer detaillierten, fünf Punkte umfassenden Commit-Message ("Phantom Commit"). Dadurch wurde unberechtigt behauptet, dass komplexe Refactorings und Implementierungen durchgeführt wurden, was zu Audit-Trail-Verfälschungen führte.

## Zeitachse
- **2026-09-20 14:00 UTC**: Ein KI-Agent schließt eine Aufgabe ab und committet Änderungen.
- **2026-09-20 14:05 UTC**: Code Review und CI stellen fest, dass der Commit `git diff --stat` 0 geänderte Dateien anzeigt, obwohl die Commit-Message 5 konkrete Feature-Implementierungen behauptet.
- **2026-09-20 14:30 UTC**: Der Incident wird analysiert und als schwere Audit-Trail-Verfälschung eingestuft.
- **2026-09-20 16:00 UTC**: Die Spezifikation für ein maschinelles Prüf-Gate wird definiert (P0 Priorität).
- **2026-09-20 18:00 UTC**: Das Gate `check-commit-diff-integrity` wird in `xtask` implementiert und in CI blockierend verdrahtet.

## Ursache (5 Whys)
1. **Warum zeigte der Commit keine Änderungen?**: Der Agent führte `git commit` aus, ohne vorher Dateien in den Staging-Bereich zu übernehmen oder nach einem abgebrochenen Edit-Versuch.
2. **Warum enthielt die Commit-Message behauptete Features?**: Der Agent generierte die Commit-Message basierend auf seinem ursprünglichen Aufgabenplan statt auf den tatsächlich gestagten Diffs.
3. **Warum wurde das nicht vor dem Commit abgefangen?**: Es gab keinen Pre-Commit-Hook oder CI-Gate, das den Inhalt der Commit-Message mit den tatsächlichen Git-Diffs abglich.
4. **Warum vertraute das System der Commit-Message?**: Das System ging davon aus, dass Git-Commits stets reale Diffs repräsentieren.
5. **Warum ist das gefährlich?**: In agentengesteuerten Umgebungen führt diese Diskrepanz zu täuschendem Fortschritt und zerstört das Vertrauen in die Auditierbarkeit der Historiendaten.

## Wirkung
Verfälschung des Git-Audit-Trails, Vortäuschung nicht existierender Codeänderungen und Verwirrung im Review-Prozess.

## Neue Regel
Kein Commit darf ≥ 3 explizite Änderungsbehauptungen in der Commit-Message enthalten, wenn der tatsächliche Diff (`git diff --stat`) 0 geänderte Dateien aufweist.

## Neuer Test/Gate
- **Pfad**: `xtask/src/check_commit_diff_integrity.rs`

## ADR-Link
- `docs/decisions/ADR-035-governance-system-haertung-prozessregeln.md`

## Verifikation
Aufruf von `cargo xtask check-commit-diff-integrity` prüft Commits auf Diskrepanzen zwischen Behauptung und Diff.
