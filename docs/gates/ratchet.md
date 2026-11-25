# Gate: Ratchet (`ratchet`)

## Übersicht
Das `ratchet`-Tool verwaltet Qualitäts-Baselines in `governance/ratchet.toml` und `.github/unwrap_baseline.txt`. Es stellt Monotonie sicher (keine Verschlechterungen) und ermöglicht Auto-Tightening (Absenken von Baselines, wenn der Code verbessert wird).

## Befehle
- `ratchet check`: Vergleicht den aktuellen Zustand des Repos mit den Baselines. Aktueller Wert > Baseline => `fail`.
- `ratchet update`: Aktualisiert die Baselines in `governance/ratchet.toml`. Absenken ist stets erlaubt. Erhöhen erfordert `--adr ADR-NNN` und ein existierendes ADR. `--init` wird genutzt, um die initiale Baseline-Datei zu erzeugen.
- `ratchet show`: Zeigt die aktuellen Repository-Messwerte und Baselines an.
