# Contextra – Ablauf einer Jules-Sitzung

Nicht normativ. Maßgeblich: [`docs/SPEC.md`](../../docs/SPEC.md) . Repository: `tfufuz1/contextra`.

Die Sitzung läuft ohne Unterbrechung, bis die Arbeitsliste abgearbeitet oder ein Stopp nach `PREAMBLE.md`, Abschnitt B, eingetreten ist. Sobald der Implementierungsplan erstellt ist, führt Jules alle Schritte pausenlos und selbstständig aus. Es erfolgen keine Rückfragen an den Nutzer, keine Unterbrechungen zur Einholung von Zwischenbestätigungen und keine Pausen. Die Arbeitsliste im Repository ist die Quelle der Wahrheit. GitHub-Issues sind eine optionale Ergänzung und keine Voraussetzung.

## Schritt 0 – Umgebung prüfen (1 Minute)

```bash
cargo --version && rustc --version      # Pin laut rust-toolchain.toml lesen, nicht annehmen
just --version                          # optional, Ausführung der Befehle geht auch ohne
```

Fehlt die Toolchain: `bash .agents/jules/setup/environment_script.sh`. Das Skript ist idempotent. Einzelne optionale Werkzeuge dürfen fehlen, die Sitzung läuft weiter.

## Schritt 1 – Kontext laden

In dieser Reihenfolge, jeweils nur die nötigen Abschnitte:

1. `AGENTS.md` (Ist-Zustand, harte Regeln).
2. `.agents/jules/PREAMBLE.md` (Entscheidungsregeln, Stopp-Katalog, Prüfbefehle).
3. `.agents/jules/tasks/BACKLOG.md` (Reihenfolge und Status der Arbeitspakete).
4. `docs/SPEC.md` Abschnitt 15.3 (Gates) und den Abschnitt 15.4 zum gewählten `WP-x.y`, dazu die dort verlinkten Befehle, IDs und Abschnitte.
5. `crates/<crate>/AGENTS.md` des betroffenen Crates (Fachprofil, Anker, Abnahme) und bei Querschnittsarbeit `.agents/experts/`.
6. `.agents/jules/context/SESSION_START.md`, falls vorhanden: aktuelle CI-Lage und offene PRs. Die Datei entsteht durch `refresh-context.sh` und ist nicht versioniert.

## Schritt 2 – Paket wählen

- Wähle das erste Paket in `BACKLOG.md` mit Status `todo`, dessen Vorgänger alle `done` sind.
- Ist `main` rot, weil ein Befehl aus dem Paket betroffen ist, gehört die Ursache zum Paket. Ist `main` rot aus anderem Grund, notiere das und arbeite trotzdem am Paket, sofern es nicht von dem Fehler abhängt.
- Setze den Status auf `in_progress` und committe das gemeinsam mit der ersten Änderung des Pakets.

## Schritt 3 – Rot

- Lies die Karte `tasks/WP-x.y.toml`. Fehlt sie, erstelle sie aus `_TEMPLATE.toml` und der Beschreibung in SPEC 15.4.
- Schreibe einen fehlschlagenden Test mit SPEC-ID nach `PREAMBLE.md`, Abschnitt D. Führe ihn aus und halte die Fehlerausgabe im PR fest.
- Ist das Paket ein Build-Paket (z. B. WP-0.3), ist das „Rot“ die Ausgabe von `cargo check --workspace --all-targets --locked`.

## Schritt 4 – Grün

- Minimale Änderung, die nur die Karte und das Paket betrifft.
- Keine Änderung an Gates, Lints, Baselines, Toolchain-Pins, `governance/` oder geschützten Pfaden (siehe `PREAMBLE.md`, Abschnitt B, S3).
- Hält eine Änderung das Budget der Karte nicht ein, teile das Paket in zwei Teilpakete `WP-x.ya` und `WP-x.yb` und trage sie in die Arbeitsliste ein. Beides ist eine Entscheidung, die du selbst triffst.

## Schritt 5 – Prüfen

Die Befehle stehen in `PREAMBLE.md`, Abschnitt C, Punkt 7. Ergebnisse notieren. Bei Zeitüberschreitung eines Gesamtlaufs die betroffenen Crates einzeln prüfen und den Rest im PR als offen nennen.

## Schritt 6 – Arbeitsliste aktualisieren

- Status des Pakets auf `done` oder `blocked` setzen. Bei `blocked` die Ursache und den Verweis auf den Fehler in der Spalte „Notiz“ schreiben.
- Neu gefundene Pakete (z. B. Teilpakete oder Altbestand aus F-Befunden) als neue Zeilen mit Status `todo` eintragen.

## Schritt 7 – Pull Request

1. `git fetch origin && git rebase origin/<Hauptzweig>`. Bei Konflikten: lösen und Schritt 5 wiederholen.
2. Commit im Format `WP-x.y: <Kurzbeschreibung>`.
3. PR-Beschreibung mit den Abschnitten aus `PREAMBLE.md`, Abschnitt E.
4. Einreichen.

## Schritt 8 – Weitermachen

Mit dem nächsten Paket in `BACKLOG.md` (zurück zu Schritt 2), bis S4 eintritt. Jedes Paket erhält einen eigenen Branch und PR.

## Schutzregeln für Sitzungsdaten

- Setze `git reset --hard` und `git clean` nicht ein, um den Arbeitsbaum zu säubern. Sie verwerfen eigene, noch nicht committete Arbeit. Committe oder stashe stattdessen.
- Generierte Dateien unter `.agents/jules/context/` sind nicht versioniert (`.gitignore`). Verändere sie nicht.
