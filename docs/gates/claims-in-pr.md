# Claims in PR Gate (`claims-in-pr`)

## Zweck
Überprüft, dass PR-Texte vollständig sind und dass im PR-Text gemachte Behauptungen (Claims) durch den tatsächlichen Git-Diff und Ausführungsergebnisse belegt sind.

## Regel-IDs
- **CP-001**: Alle sieben Pflichtabschnitte (`Ziel`, `Änderungen`, `Invarianten berührt`, `Verifikation`, `Out-of-scope Findings`, `Risiken/Rollback`, `ADR/Spec-Sync`) müssen vorhanden und nicht leer/Platzhalter sein.
- **CP-002**: Die Dateiliste unter `Änderungen` muss mengengleich mit dem tatsächlichen Diff sein (keine Phantom-Claims oder fehlende Dateien).
- **CP-003**: Behauptungen unter `Verifikation` (`<befehl>: <PASS|grün|bestanden>`) müssen durch Beleg-JSONs in `--results-dir` untermauert sein.
- **CP-004**: Formulierungen wie „Tests hinzugefügt“ verlangen mindestens ein neues `#[test]` oder eine neue Testdatei im Diff.
