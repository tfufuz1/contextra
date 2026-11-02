# Geplante Audits und Mutationstests

Nicht normativ. Die Beschreibung stammt aus der Vorfassung. Vor einer Änderung ist die Workflow-Datei zu prüfen.

## Ablauf laut Vorfassung

Der Workflow `.github/workflows/scheduled-audit.yml` läuft freitags um 22:00 UTC und manuell per `workflow_dispatch`. Er besteht aus zwei Jobs:

1. **Kontext vorbereiten.** Er sammelt die Commits der letzten sieben Tage und legt einen Auftrag an. Der Auftrag verweist auf [`AUDIT_INTAKE_PROTOCOL.md`](AUDIT_INTAKE_PROTOCOL.md). Er öffnet einen PR gegen `main`. Die Prüfung umfasst Race Conditions, neue `unwrap()` und `expect()` außerhalb der Basis in `.github/unwrap_baseline.txt`, `let _ =` bei I/O und Verstöße gegen die Abhängigkeitsmatrix.
2. **Mutationstests rotieren.** Der Fokus wechselt wöchentlich zwischen Crates.

## Korrekturen zur Vorfassung

- **Pfad der Auftragsdatei.** Die Vorfassung nennt `.jules/pending-audit-task.md`. Das Verzeichnis `.jules/` existiert nicht; die Ablage ist `.agents/jules/`. Der Pfad im Workflow ist zu prüfen. Das ist ein offener Befund.
- **Fokus-Crates.** Die Vorfassung nennt `contextra-graph`, `contextra-agent` und `contextra-db`. Diese Crates sind nach SPEC D-24 und D-45 entfernt. Die Rotation muss vorhandene Crates verwenden, etwa `contextra-store`, `contextra-crypto`, `contextra-vector` und `contextra-text`. Ab Gate G-D kommt `contextra-erasure` hinzu.
- **Mutations-Workflow.** Der Workflow `mutation-testing.yml`, den der Job auslöst, existiert nicht. Die Mutationstests sind nach SPEC 15.1 Pflicht für `erasure`, Klassifikation, Matcher, Gate und Tresor. Ihr Workflow ist mit G-D einzurichten.

## Ergebnis

Ein Audit liefert einen PR mit Behebungen oder mit einem Bericht, der Befunde nach `AUDIT_INTAKE_PROTOCOL.md` einordnet. Unklare Befunde werden nicht umgesetzt.
