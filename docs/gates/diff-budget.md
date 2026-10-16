# Diff Budget Gate (`diff-budget`)

## Zweck
Begrenzt die Größe von PRs, um gründliche Code Reviews zu gewährleisten und das Risiko von unbeabsichtigten Nebenwirkungen zu minimieren.

## Funktionsweise
- Prüft das Diff gegen Budget-Grenzen aus der Task-Karte (`budget = { files = ..., lines = ... }`) oder Standard-Schwellenwerte:
  - Max. 15 geänderte Dateien
  - Max. 600 geänderte Zeilen (Hinzugefügt + Gelöscht)
  - Max. 2 berührte Crates unter `crates/`
- Ausgenommen von den Zählungen sind: `Cargo.lock`, `docs/generated/**`, `*.snap`, und Dateien mit `@generated` in den ersten 5 Zeilen.
- Bei Überschreitung schlägt das Gate fehl (`fail`, Exit 1) und schlägt eine Aufteilung nach Crates vor.
