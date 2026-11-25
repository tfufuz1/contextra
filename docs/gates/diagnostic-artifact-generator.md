# Diagnostic Artifact Generator & Frontmatter Standard (§A.4)

## Zweck & Kontext
Gemäß `CONTEXTRA_SPEC_UPDATED.md` Teil A.4 (Diagnose-Artefakte und Commit-Bindung) müssen Testergebnisse, Lint-Reports, Audit-Dokumente und Marker-Tabellen im Dokumentkopf strikt definierte Metadaten-Felder tragen:
- `commit`: Volle Git-SHA des verifizierten `HEAD`-Commits (40 Hex-Zeichen)
- `generated_by`: Exakter aufgerufener Subbefehl inkl. Argumente
- `generated_at`: ISO-8601 UTC Zeitstempel
- `toolchain`: Ausgabe der aktiven `rustc --version`

Ein Artefakt mit `commit ≠ HEAD` gilt als veraltet und darf nicht als Begründung für Arbeit oder Nicht-Arbeit dienen. Artefakte werden stets mechanisch erzeugt und nie manuell editiert.

## Frontmatter-Format

Das Frontmatter wird im standardisierten YAML-Format am Beginn von Markdown-Diagnoseberichten eingefügt:

```yaml
---
commit: 1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b
generated_by: cargo xtask generate-diagnostics
generated_at: 2026-09-17T12:34:56Z
toolchain: rustc 1.80.0 (123456789 2024-07-25)
---
```

Alternativ kann das Header-Objekt über `render_json()` als deserialisierbares JSON gerendert werden.

## Aufruf des Diagnose-Generators

Das Subkommando `generate-diagnostics` führt registrierte Gates (`check-flatbuffers-drift`, `check-bandit-latency-budget`, `check-module-reachability`, `check-duplicate-symbols-cross-file`) aus und fasst die Ergebnisse zusammen.

### Befehl

```bash
cargo run -p xtask -- generate-diagnostics
```

### Ausgabe-Verzeichnis

Das Gesamtergebnis wird unter folgendem Pfad abgelegt:
`target/diagnostics/gate-report-<commit-kurz>.md`

Da das Verzeichnis `target/` in `.gitignore` ignoriert wird, werden erzeugte Artefakte nicht in die Versionsverwaltung eingecheckt.

## Fehlerbehandlung & Exit-Codes

Falls irgendeines der ausgeführten Gates fehlschlägt, beendet sich `cargo run -p xtask -- generate-diagnostics` mit einem Exit-Code ≠ 0. Dadurch lässt sich der Befehl direkt als CI-Sammel-Gate einsetzen.
