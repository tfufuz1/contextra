# Aufgabenkarten

Nicht normativ. Eine Aufgabenkarte beschreibt ein abgegrenztes Arbeitspaket für Jules. Sie setzt Umfang und Grenzen fest und verweist auf die SPEC.

## Wie die Dateien zusammenhängen

| Datei | Zweck |
|---|---|
| `BACKLOG.md` | Reihenfolge, Status und Abhängigkeiten aller Pakete. Maßgeblich für die Auswahl. |
| `_TEMPLATE.toml` | Vorlage für neue Karten. |
| `WP-x.y.toml` | Karte eines einzelnen Pakets. Wird angelegt, wenn das Paket aufgenommen wird. |

Eine Karte, die fehlt, ist kein Hindernis. Jules erstellt sie aus der Vorlage und der Beschreibung in SPEC 15.4.

## Pflichtfelder

| Feld | Inhalt |
|---|---|
| `id` | Kennung, gleich dem Arbeitspaket, z. B. `WP-3.3` |
| `title` | kurze Beschreibung |
| `wp` | Arbeitspaket aus SPEC 15.4 |
| `gate` | Gate aus SPEC 15.3 |
| `crate` | betroffene Crates |
| `goal` | gewünschte Wirkung, prüfbar |
| `non_goals` | ausdrücklich nicht zu ändern |
| `scope` | erlaubte Pfade (Glob) |
| `forbidden` | verbotene Pfade. Mindestens die Einträge aus `_TEMPLATE.toml`, zusätzlich alles aus `.agents/governance/protected-paths.toml` |
| `spec_ids` | betroffene SPEC-IDs |
| `acceptance` | Befehle, die bestehen müssen |
| `evidence_required` | was im PR belegt wird |

`[budget]` (`files`, `lines`, `iterations`) begrenzt eine Sitzung. Überschreitet die Arbeit das Budget, wird das Paket geteilt (siehe `SESSION_BOOTSTRAP.md`, Schritt 4).

## Verbindliche Prüfregel

Jede Karte nennt die Ausgangslage mit Datei und Zeile, zum Beispiel aus einem Befund. Der Agent prüft die Zahlen, bevor er sie verwendet. Befunde aus der SPEC sind Momentaufnahmen. Der Build und `grep` am aktuellen Stand gelten.

## Status-Ablauf

`todo` → `in_progress` → `done` oder `blocked`. Eine Änderung des Status gehört in denselben PR wie die Arbeit am Paket.
