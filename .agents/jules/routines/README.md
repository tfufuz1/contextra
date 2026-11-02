# Routinen für wiederkehrende Aufgaben

Nicht normativ. Maßgeblich sind [`AGENTS.md`](../../../AGENTS.md) und [`docs/SPEC.md`](../../../docs/SPEC.md).

## Regeln

1. **Eine Quelle.** Wiederkehrende Aufgaben verweisen auf die Datei in diesem Verzeichnis. Eigene, abweichende Prompts sind nicht erlaubt.
2. **PR-Integrität.** Änderungen an Routinen erfolgen als gewöhnlicher Pull Request, nie als Push auf `main`.
3. **Aufgaben über Task-Karten.** Automatisch erzeugte Aufgaben sind nach der Vorfassung deaktiviert oder auf explizite Labels mit Task-Karten beschränkt (siehe [`../ISSUE_AUTOMATION_POLICY.md`](../ISSUE_AUTOMATION_POLICY.md)). Die Einstellung im Jules-Interface ist nicht geprüft.
4. **Harte Grenzen.** Jede Routine nennt Grenzen (Dateien, Crates, Scope). Die Vorfassung setzte 15 Dateien und ein Crate pro PR.
5. **Eskalation.** Bei unklaren Befunden wird nicht geändert, sondern gemeldet. Die Vorfassung verwies auf ein Architekturdokument §9.3, das nicht mehr existiert; gültig ist die Eskalation in [`../HARNESS_QUICKSTART.md`](../HARNESS_QUICKSTART.md).

## Routinen

| Datei | Takt | Zweck |
|---|---|---|
| `audit-rotation.md` | wöchentlich | Audit wechselnder Kern-Crates |
| `bench-trend.md` | wöchentlich | Regressionen bei Latenz und Durchsatz |
| `chaos-fuzz-langlauf.md` | nächtlich | Chaos-Matrix und Fuzz-Läufe |
| `dependency-hygiene.md` | wöchentlich | Lizenzen und Abhängigkeitsprüfung |
| `drift-waechter.md` | täglich | Abweichungen zwischen Dokumenten, Code und SPEC |
| `flake-jagd.md` | nächtlich | nicht deterministische Tests |
| `unsafe-pruefung.md` | wöchentlich | Unsafe-Inseln und ihre Sicherheitskommentare |
| `unwrap-allow-abbau.md` | wöchentlich | Abbau von `unwrap` und `allow` |
