# Gate: Jules Dispatch (`jules-dispatch`)

## Zweck
`jules-dispatch` steuert das sichere Absenden von Task-Karten an die Jules REST API.

## Befehl
`cargo xtask jules-dispatch --card <DATEI> [--send]`

## Invarianten
- Ohne `--send` immer Trockenlauf (Payload-Anzeige).
- `requirePlanApproval: true` wird bedingungslos erzwungen.
- `automationMode` darf niemals `AUTO_CREATE_PR` sein.
- `JULES_API_KEY` wird ausschließlich aus Umgebungsvariablen gelesen und niemals geloggt oder in Dateien geschrieben.
