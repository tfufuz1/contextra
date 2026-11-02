# Routine: Audit-Rotation

- **Takt:** wöchentlich, freitags 22:00 UTC (Workflow `scheduled-audit.yml`)
- **Ziel:** proaktive Prüfung wechselnder Kern-Crates nach [`AUDIT_INTAKE_PROTOCOL.md`](../AUDIT_INTAKE_PROTOCOL.md)
- **Eingaben:** `.github/workflows/scheduled-audit.yml`, `AUDIT_INTAKE_PROTOCOL.md`, SPEC-Abschnitte des geprüften Crates
- **Harte Grenzen:** höchstens 15 Dateien in einem Crate je PR. Der Auftragsdatei-Pfad ist im Workflow zu prüfen (siehe [`../SCHEDULED_AUDIT.md`](../SCHEDULED_AUDIT.md)).
- **Stopp:** Ein unklarer Befund erfordert Rückfrage vor jeder Codeänderung.
- **Ergebnis:** PR mit bestätigten Behebungen und aktualisierter Auftragsdatei.
- **Eskalation:** siehe [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).
